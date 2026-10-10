//! TECHSPEC 5.2 and 6.2: parsing `config.toml` and looking up boards.

use std::path::{Path, PathBuf};

use brain_swap::core::config::{Config, parse};
use brain_swap::core::error::Error;

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
