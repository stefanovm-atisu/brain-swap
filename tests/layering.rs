//! TECHSPEC 2.2 layering rules, checked by a plain-text scan of `src/`.

use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct Violation {
    pub file: String,
    pub line: usize,
    pub rule: &'static str,
}

/// (path prefix, forbidden text, paths allowed to use it anyway, rule).
const RULES: &[(&str, &str, &[&str], &str)] = &[
    (
        "src/core/",
        "std::process",
        &["src/core/failpoint.rs"],
        "core runs no process",
    ),
    ("src/core/", "std::env", &[], "core reads no environment"),
    ("src/core/", "crate::cli", &[], "core imports no cli"),
    ("src/core/", "crate::tui", &[], "core imports no tui"),
    (
        "src/core/",
        "crate::adapters",
        &[],
        "core imports no adapters",
    ),
    ("src/adapters/", "crate::cli", &[], "adapters import no cli"),
    ("src/adapters/", "crate::tui", &[], "adapters import no tui"),
    ("src/", "std::net", &[], "no socket (NFR-04)"),
    ("src/", "TcpStream", &[], "no socket (NFR-04)"),
    ("src/", "TcpListener", &[], "no socket (NFR-04)"),
    ("src/", "UdpSocket", &[], "no socket (NFR-04)"),
    ("src/", "UnixStream", &[], "no socket (NFR-04)"),
    ("src/", "UnixListener", &[], "no socket (NFR-04)"),
    (
        "src/",
        "Command::new",
        &["src/adapters/runner.rs", "src/tui/editor.rs"],
        "process only in runner or editor (NFR-03)",
    ),
    (
        "src/",
        "cfg(target_os",
        &[],
        "one code path for Linux and macOS (NFR-09)",
    ),
    (
        "tests/",
        "Command::new",
        &["tests/common/spawn.rs", "tests/layering.rs"],
        "tests spawn only through tests/common/spawn.rs (12.1)",
    ),
    (
        "tests/",
        "cargo_bin",
        &["tests/common/spawn.rs", "tests/layering.rs"],
        "tests spawn only through tests/common/spawn.rs (12.1)",
    ),
];

/// Expands brace groups so `use std::{env, process};` also reads as `std::env` and `std::process`.
fn expand(s: &str) -> Vec<String> {
    let Some(open) = s.find('{') else {
        return vec![s.to_string()];
    };
    let (mut depth, mut start, mut items) = (0, open + 1, Vec::new());
    for (i, c) in s.char_indices().skip_while(|&(i, _)| i < open) {
        match c {
            '{' => depth += 1,
            ',' if depth == 1 => {
                items.push(&s[start..i]);
                start = i + 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    items.push(&s[start..i]);
                    return items
                        .iter()
                        .flat_map(|it| {
                            expand(&format!("{}{}{}", &s[..open], it.trim(), &s[i + 1..]))
                        })
                        .collect();
                }
            }
            _ => {}
        }
    }
    vec![s.to_string()]
}

/// Plain-text scan of every `.rs` file under `root/src` and `root/tests`, one violation per offending line.
/// A `use` without its `;` on the same line is joined with the following lines up to the `;`
/// and reported at the line where it starts.
/// ponytail: text, not tokens, so `use` in a comment or string is read as an import; tokenize if that bites.
pub fn scan(root: &Path) -> Vec<Violation> {
    let mut out = Vec::new();
    let mut dirs: Vec<_> = ["src", "tests"]
        .iter()
        .map(|d| root.join(d))
        .filter(|d| d.is_dir())
        .collect();
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let file = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let text = fs::read_to_string(&path).unwrap();
            let lines: Vec<&str> = text.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let mut stmt = line.to_string();
                if let Some(u) = line.find("use ") {
                    stmt = line[u..].to_string();
                    for next in &lines[i + 1..] {
                        if stmt.contains(';') {
                            break;
                        }
                        stmt.push_str(next);
                    }
                }
                let paths = expand(&stmt);
                let hit = RULES.iter().find(|(prefix, needle, allowed, _)| {
                    file.starts_with(prefix)
                        && (line.contains(needle) || paths.iter().any(|p| p.contains(needle)))
                        && !allowed.contains(&file.as_str())
                });
                if let Some((_, _, _, rule)) = hit {
                    out.push(Violation {
                        file: file.clone(),
                        line: i + 1,
                        rule,
                    });
                }
            }
        }
    }
    out
}

fn plant(file: &str, line: usize, text: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let body = "// filler\n".repeat(line - 1) + text + "\n";
    fs::write(path, body).unwrap();
    tmp
}

fn hits(file: &str, line: usize, text: &str) -> Vec<String> {
    let tmp = plant(file, line, text);
    scan(tmp.path())
        .iter()
        .map(|v| format!("{}:{}", v.file, v.line))
        .collect()
}

#[test]
fn nfr11_manifest_lists_six_runtime_crates() {
    let text =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let manifest: toml::Table = text.parse().unwrap();
    let mut keys: Vec<&str> = manifest["dependencies"]
        .as_table()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort();
    assert_eq!(
        keys,
        ["clap", "jiff", "ratatui", "serde", "serde_json", "toml"]
    );
}

#[test]
fn ts2_2_layering_passes_on_repository() {
    let found = scan(Path::new(env!("CARGO_MANIFEST_DIR")));
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn ts2_2_planted_process_in_core_fails() {
    assert_eq!(
        hits("src/core/store.rs", 3, "use std::process::Command;"),
        ["src/core/store.rs:3"]
    );
}

#[test]
fn ts2_2_planted_env_in_core_fails() {
    assert_eq!(
        hits("src/core/config.rs", 5, "let h = std::env::var(\"HOME\");"),
        ["src/core/config.rs:5"]
    );
}

#[test]
fn ts2_2_failpoint_may_use_process() {
    assert!(hits("src/core/failpoint.rs", 1, "std::process::abort();").is_empty());
}

#[test]
fn ts2_2_core_importing_tui_fails() {
    assert_eq!(
        hits("src/core/board.rs", 2, "use crate::tui::App;"),
        ["src/core/board.rs:2"]
    );
}

#[test]
fn ts2_2_adapters_importing_cli_fails() {
    assert_eq!(
        hits("src/adapters/herdr.rs", 4, "use crate::cli::out;"),
        ["src/adapters/herdr.rs:4"]
    );
}

#[test]
fn nfr04_socket_type_anywhere_fails() {
    assert_eq!(
        hits(
            "src/adapters/herdr.rs",
            1,
            "use std::os::unix::net::UnixStream;"
        ),
        ["src/adapters/herdr.rs:1"]
    );
}

#[test]
fn nfr03_command_outside_runner_and_editor_fails() {
    let line = "let c = Command::new(\"sh\");";
    assert_eq!(hits("src/cli/cmd.rs", 2, line), ["src/cli/cmd.rs:2"]);
    assert!(hits("src/adapters/runner.rs", 2, line).is_empty());
}

#[test]
fn nfr09_target_os_cfg_fails() {
    assert_eq!(
        hits("src/core/store.rs", 6, "#[cfg(target_os = \"linux\")]"),
        ["src/core/store.rs:6"]
    );
}

#[test]
fn ts2_2_grouped_std_import_in_core_fails() {
    assert_eq!(
        hits("src/core/store.rs", 2, "use std::{env, process};"),
        ["src/core/store.rs:2"]
    );
}

#[test]
fn ts2_2_grouped_crate_import_in_core_fails() {
    assert_eq!(
        hits("src/core/board.rs", 3, "use crate::{tui, core::model};"),
        ["src/core/board.rs:3"]
    );
}

#[test]
fn ts2_2_multiline_use_group_in_core_fails() {
    let text = "use std::{\n    collections::HashMap,\n    env,\n    fs,\n};";
    assert_eq!(hits("src/core/store.rs", 3, text), ["src/core/store.rs:3"]);
}

#[test]
fn ts12_1_tests_spawn_only_through_helper() {
    assert_eq!(
        hits("tests/cli_new.rs", 4, "let c = Command::new(\"x\");"),
        ["tests/cli_new.rs:4"]
    );
    let found: Vec<Violation> = scan(Path::new(env!("CARGO_MANIFEST_DIR")))
        .into_iter()
        .filter(|v| v.file.starts_with("tests/"))
        .collect();
    assert!(found.is_empty(), "{found:?}");
}
