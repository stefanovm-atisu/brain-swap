//! TECHSPEC 5.2 and 6.2: parsing `config.toml` and looking up boards.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use brain_swap::core::config::{Config, default_text as text_for, load, parse, write_default};
use brain_swap::core::error::Error;
use common::env::core_env;

const NOW: &str = "2026-10-08T11:52:00+03:00";

fn default_text() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/config/default.toml");
    std::fs::read_to_string(path).unwrap()
}

fn cfg(text: &str) -> Config {
    parse(
        text,
        Path::new("/h/.config/brain-swap/config.toml"),
        Path::new("/h"),
    )
    .unwrap()
    .0
}

fn boards(c: &Config) -> Vec<(&str, &Path)> {
    c.boards
        .iter()
        .map(|(n, p)| (n.as_str(), p.as_path()))
        .collect()
}

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn fr42_parses_default_file() {
    let c = cfg(&default_text());
    assert_eq!(c.default_board, "work");
    assert_eq!(c.editor, "");
    let base = PathBuf::from("/h/.local/share/brain-swap");
    assert_eq!(
        boards(&c),
        [
            ("work", base.join("work").as_path()),
            ("home", base.join("home").as_path()),
            ("personal", base.join("personal").as_path()),
        ]
    );
    assert!(c.keys.contains(&("up".to_string(), strings(&["k", "up"]))));
}

#[test]
fn fr02_boards_keep_file_order() {
    let c = cfg("default_board = \"mid\"\n[boards]\nzeta = \"/z\"\nalpha = \"/a\"\nmid = \"/m\"\n");
    let names: Vec<&str> = c.boards.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["zeta", "alpha", "mid"]);
}

#[test]
fn fr02_default_board_defaults_to_work() {
    let c = cfg("[boards]\nwork = \"/b/w\"\n");
    assert_eq!(c.default_board, "work");
}

#[test]
fn ts5_2_tilde_path_expands_to_home() {
    let c = cfg("[boards]\nwork = \"~/b\"\nhome = \"/abs/h\"\n");
    assert_eq!(
        boards(&c),
        [("work", Path::new("/h/b")), ("home", Path::new("/abs/h"))]
    );
}

#[test]
fn ts6_2_board_lookup_ignores_case() {
    let c = cfg(&default_text());
    assert_eq!(
        c.board("WORK").unwrap(),
        ("work", Path::new("/h/.local/share/brain-swap/work"))
    );
}

#[test]
fn ts6_2_unknown_board_lists_names() {
    let c = cfg(&default_text());
    let e = c.board("work2").unwrap_err();
    assert_eq!(
        e,
        Error::NotFound("unknown board 'work2' (work, home, personal)".to_string())
    );
    assert_eq!(e.exit_code(), 3);
}

#[test]
fn fr42_key_value_is_string_or_list() {
    let c = cfg("[boards]\nwork = \"/w\"\n[keys]\nopen = \"o\"\nquit = [\"q\", \"esc\"]\n");
    assert_eq!(
        c.keys,
        [
            ("open".to_string(), strings(&["o"])),
            ("quit".to_string(), strings(&["q", "esc"])),
        ]
    );
}

fn err(text: &str) -> Error {
    parse(
        text,
        Path::new("/h/.config/brain-swap/config.toml"),
        Path::new("/h"),
    )
    .unwrap_err()
}

fn at(e: &Error) -> (usize, usize, &str) {
    match e {
        Error::Config { line, col, msg, .. } => (*line, *col, msg.as_str()),
        e => panic!("not a config error: {e:?}"),
    }
}

const BOARDS: &str = "[boards]\nwork = \"/w\"\nhome = \"/h\"\n";

fn error_cases() -> Vec<Error> {
    let mut v = vec![
        err("editor = \"\"\n\n\ndefault_board \"work\"\n"),
        err(&format!("default_board = \"x\"\n{BOARDS}")),
        err("default_board = \"work\"\n\n[boards]\n"),
        err("default_board = \"work\"\n"),
    ];
    for t in [
        "[boards]\nWork = \"/b\"\n",
        "[boards]\n1st = \"/b\"\n",
        "editor = 3\n[boards]\nwork = \"/w\"\n",
        "default_board = true\n[boards]\nwork = \"/w\"\n",
        "[boards]\nwork = \"/w\"\n[keys]\nup = 5\n",
        "[boards]\nwork = \"/w\"\n[keys]\nup = [1]\n",
        "[boards]\nwork = \"boards/work\"\n",
        "[boards]\nwork = \"~user/x\"\n",
    ] {
        v.push(err(t));
    }
    v
}

#[test]
fn fr45_syntax_error_names_file_line_col() {
    let e = err("editor = \"\"\n\n\ndefault_board \"work\"\n");
    let (line, col, _) = at(&e);
    assert_eq!(line, 4);
    assert_eq!(col, 15);
    assert!(
        e.to_string()
            .starts_with("config /h/.config/brain-swap/config.toml:4:"),
        "{e}"
    );
}

#[test]
fn ts5_2_default_board_not_in_boards_is_config_error() {
    let e = err(&format!("default_board = \"x\"\n{BOARDS}"));
    assert_eq!(at(&e), (1, 1, "default_board 'x' is not in [boards]"));
}

#[test]
fn ts5_2_empty_boards_is_config_error() {
    let e = err("default_board = \"work\"\n\n[boards]\n");
    let (line, _, msg) = at(&e);
    assert_eq!((line, msg), (3, "[boards] is empty"));
}

#[test]
fn ts5_2_missing_boards_is_config_error() {
    let e = err("default_board = \"work\"\n");
    assert_eq!(at(&e), (1, 1, "[boards] is empty"));
}

#[test]
fn ts5_2_bad_board_name_is_config_error() {
    for (name, col) in [("Work", 1), ("1st", 3)] {
        let e = err(&format!(
            "default_board = \"ok\"\n[boards]\nok = \"/o\"\n{}{name} = \"/b\"\n",
            " ".repeat(col - 1)
        ));
        let msg = format!("board name '{name}' must match [a-z][a-z0-9_-]*");
        assert_eq!(at(&e), (4, col, msg.as_str()));
    }
}

#[test]
fn ts5_2_wrong_type_is_config_error() {
    for (text, line) in [
        ("editor = 3\n[boards]\nwork = \"/w\"\n", 1),
        ("[boards]\nwork = \"/w\"\n\ndefault_board = true\n", 4),
        ("[boards]\nwork = \"/w\"\n[keys]\nup = 5\n", 4),
        ("[boards]\nwork = \"/w\"\n[keys]\nup = [1]\n", 4),
    ] {
        let e = err(text);
        let (l, c, msg) = at(&e);
        assert_eq!((l, c), (line, 1), "{text}");
        assert!(msg.contains("expected"), "{msg}");
    }
}

#[test]
fn ts5_2_relative_board_path_is_config_error() {
    for p in ["boards/work", "~user/x"] {
        let e = err(&format!("[boards]\nwork = \"{p}\"\n"));
        let (_, _, msg) = at(&e);
        assert_eq!(msg, "board work: path must be absolute or start with ~/");
    }
}

#[test]
fn ts5_2_unknown_top_level_key_warns() {
    let text = format!("colour = \"red\"\n{}", default_text());
    let (_, warnings) = parse(
        &text,
        Path::new("/h/.config/brain-swap/config.toml"),
        Path::new("/h"),
    )
    .unwrap();
    assert_eq!(warnings, ["config: unknown key 'colour'"]);
}

#[test]
fn fr45_config_errors_exit_1() {
    let cases = error_cases();
    assert_eq!(cases.len(), 12);
    for e in cases {
        assert!(matches!(e, Error::Config { .. }), "{e:?}");
        assert_eq!((e.exit_code(), e.code()), (1, "config"));
    }
}

#[test]
fn fr66_missing_config_is_written_and_reported() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let loaded = load(&env).unwrap();
    assert!(env.config_file().is_file());
    assert_eq!(loaded.created, Some(env.config_file()));
    assert_eq!(loaded.config.default_board, "work");
}

#[test]
fn fr66_default_text_equals_5_2_under_default_xdg() {
    let env = core_env(Path::new("/h"), NOW);
    assert_eq!(text_for(&env), default_text());
}

#[test]
fn fr66_existing_config_is_not_rewritten() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    fs::create_dir_all(env.config_file().parent().unwrap()).unwrap();
    let custom = "default_board = \"a\"\n[boards]\na = \"/a\"\n";
    fs::write(env.config_file(), custom).unwrap();
    let loaded = load(&env).unwrap();
    assert_eq!(loaded.created, None);
    assert_eq!(fs::read_to_string(env.config_file()).unwrap(), custom);
    let broken = "default_board = \n";
    fs::write(env.config_file(), broken).unwrap();
    assert!(matches!(load(&env), Err(Error::Config { .. })));
    assert_eq!(fs::read_to_string(env.config_file()).unwrap(), broken);
}

#[test]
fn fr66_create_new_never_clobbers() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    fs::create_dir_all(env.config_file().parent().unwrap()).unwrap();
    let custom = "default_board = \"a\"\n[boards]\na = \"/a\"\n";
    fs::write(env.config_file(), custom).unwrap();
    let loaded = write_default(&env).unwrap();
    assert_eq!(loaded.created, None);
    assert_eq!(loaded.config.default_board, "a");
    assert_eq!(fs::read_to_string(env.config_file()).unwrap(), custom);
}

#[test]
fn fr66_first_run_creates_no_board_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    load(&env).unwrap();
    assert!(!env.data_dir().join("work").exists());
}

#[test]
fn ts5_1_custom_data_home_written_as_absolute() {
    let mut env = core_env(Path::new("/h"), NOW);
    env.data_home = PathBuf::from("/x/d");
    assert!(
        text_for(&env).contains("\nwork = \"/x/d/brain-swap/work\"\n"),
        "{}",
        text_for(&env)
    );
}

fn config_dir_entries(env: &brain_swap::core::env::Env) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(env.config_file().parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

#[test]
fn fr66_first_run_leaves_no_temp_file() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    write_default(&env).unwrap();
    assert_eq!(config_dir_entries(&env), ["config.toml"]);
    assert_eq!(
        fs::read_to_string(env.config_file()).unwrap(),
        text_for(&env)
    );
}

#[test]
fn fr66_stale_temp_does_not_block_first_run() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let dir = env.config_file().parent().unwrap().to_path_buf();
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!(".config.toml.bs-tmp-{}", env.pid)), "").unwrap();
    let loaded = load(&env).unwrap();
    assert_eq!(loaded.created, Some(env.config_file()));
    assert_eq!(
        fs::read_to_string(env.config_file()).unwrap(),
        text_for(&env)
    );
    assert_eq!(config_dir_entries(&env), ["config.toml"]);
}

#[test]
fn fr45_invalid_utf8_is_config_error_at_its_line() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    fs::create_dir_all(env.config_file().parent().unwrap()).unwrap();
    let mut bytes = b"[boards]\nwork = \"/w\"\n# caf".to_vec();
    bytes.extend_from_slice(b"\xe9\n");
    fs::write(env.config_file(), bytes).unwrap();
    let e = load(&env).unwrap_err();
    let (line, col, _) = at(&e);
    assert_eq!((line, col), (3, 6));
}
