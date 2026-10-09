//! TECHSPEC 12.1 docs: no U+2014 or U+2013 in any repository markdown file.

use std::fs;
use std::path::{Path, PathBuf};

/// 1-based numbers of the lines holding an em or en dash.
pub fn find_dashes(text: &str) -> Vec<usize> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| l.contains(['\u{2014}', '\u{2013}']))
        .map(|(i, _)| i + 1)
        .collect()
}

/// Every `*.md` under `root`, skipping `target/` and dot directories, read lossily;
/// paths relative to `root`.
pub fn scan(root: &Path) -> Vec<(PathBuf, usize)> {
    let mut out = Vec::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy();
            if path.is_dir() {
                if !name.starts_with('.') && name != "target" {
                    dirs.push(path);
                }
            } else if name.ends_with(".md") {
                let text = String::from_utf8_lossy(&fs::read(&path).unwrap()).into_owned();
                let rel = path.strip_prefix(root).unwrap();
                out.extend(
                    find_dashes(&text)
                        .into_iter()
                        .map(|l| (rel.to_path_buf(), l)),
                );
            }
        }
    }
    out
}

fn tree(files: &[(&str, &[u8])]) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    for (name, body) in files {
        let path = tmp.path().join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }
    tmp
}

#[test]
fn ts12_1_find_dashes_reports_em_dash_line() {
    assert_eq!(find_dashes("a\nb\u{2014}c\n"), [2]);
}

#[test]
fn ts12_1_find_dashes_reports_en_dash_line() {
    assert_eq!(find_dashes("a\u{2013}b\n"), [1]);
}

#[test]
fn ts12_1_hyphen_is_allowed() {
    assert!(find_dashes("a - b").is_empty());
}

#[test]
fn ts12_1_repository_markdown_has_no_dashes() {
    let found: Vec<String> = scan(Path::new(env!("CARGO_MANIFEST_DIR")))
        .iter()
        .map(|(p, l)| format!("{}:{l}", p.display()))
        .collect();
    assert!(found.is_empty(), "em or en dash at {found:?}");
}

#[test]
fn ts12_1_non_utf8_markdown_is_scanned_lossily() {
    let mut body = b"a\xff\n".to_vec();
    body.extend_from_slice("b\u{2014}\n".as_bytes());
    let tmp = tree(&[("x.md", &body)]);
    assert_eq!(scan(tmp.path()), [(PathBuf::from("x.md"), 2)]);
}

#[test]
fn ts12_1_dot_directories_are_skipped() {
    let tmp = tree(&[(".serena/m.md", "\u{2014}\n".as_bytes())]);
    assert!(scan(tmp.path()).is_empty());
}
