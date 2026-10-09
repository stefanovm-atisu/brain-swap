//! TECHSPEC 4.2 and 4.8: setting a frontmatter key is a byte splice.

mod common;

use brain_swap::core::error::Error;
use brain_swap::core::frontmatter::{line_ending, set};
use common::bytes::assert_only_lines_changed;

fn set_ok(before: &str, key: &str, value: &str) -> String {
    String::from_utf8(set(before.as_bytes(), key, value).unwrap()).unwrap()
}

#[test]
fn r2_set_replaces_existing_line_only() {
    let before = "---\ncolumn: Todo\ntemplate: Feature\n---\nbody\n";
    let after = set_ok(before, "column", "Doing");
    assert_eq!(after, "---\ncolumn: Doing\ntemplate: Feature\n---\nbody\n");
    assert_only_lines_changed(before.as_bytes(), after.as_bytes(), &["column: Doing\n"]);
}

#[test]
fn r2_set_keeps_key_spelling() {
    let before = "---\nColumn: Todo\n---\n";
    let after = set_ok(before, "column", "Doing");
    assert_eq!(after, "---\nColumn: Doing\n---\n");
    assert_only_lines_changed(before.as_bytes(), after.as_bytes(), &["Column: Doing\n"]);
}

#[test]
fn r2_set_inserts_before_closing_line() {
    let before = "---\ntemplate: Feature\n---\nbody\n";
    let after = set_ok(before, "column", "Doing");
    assert_eq!(after, "---\ntemplate: Feature\ncolumn: Doing\n---\nbody\n");
    assert_only_lines_changed(before.as_bytes(), after.as_bytes(), &["column: Doing\n"]);
}

#[test]
fn r2_set_prepends_three_line_block() {
    let before = "# t\nbody\n";
    let after = set_ok(before, "column", "Doing");
    assert_eq!(after, "---\ncolumn: Doing\n---\n# t\nbody\n");
    assert_only_lines_changed(
        before.as_bytes(),
        after.as_bytes(),
        &["---\n", "column: Doing\n", "---\n"],
    );
}

#[test]
fn r5_inserted_line_uses_crlf_in_crlf_file() {
    let before = "---\r\ntemplate: Feature\r\n---\r\nbody\r\n";
    let after = set_ok(before, "column", "Doing");
    assert_eq!(
        after,
        "---\r\ntemplate: Feature\r\ncolumn: Doing\r\n---\r\nbody\r\n"
    );
    assert_only_lines_changed(before.as_bytes(), after.as_bytes(), &["column: Doing\r\n"]);
}

#[test]
fn r5_prepended_block_uses_dominant_line_ending() {
    let before = "# t\r\nbody\r\n";
    let after = set_ok(before, "column", "Doing");
    assert_eq!(after, "---\r\ncolumn: Doing\r\n---\r\n# t\r\nbody\r\n");
    assert_only_lines_changed(
        before.as_bytes(),
        after.as_bytes(),
        &["---\r\n", "column: Doing\r\n", "---\r\n"],
    );
}

#[test]
fn nfr07_unknown_keys_comments_and_continuations_survive() {
    let before = "---\nowner: me\n# c\ncolumn: Todo\ntags:\n  - x\n---\nbody\n";
    let after = set_ok(before, "column", "Done");
    assert_eq!(
        after,
        "---\nowner: me\n# c\ncolumn: Done\ntags:\n  - x\n---\nbody\n"
    );
    assert_only_lines_changed(before.as_bytes(), after.as_bytes(), &["column: Done\n"]);
}

#[test]
fn ts4_2_set_on_invalid_block_is_unreadable() {
    match set(b"---\ncolumn Doing\n---\n", "column", "Done") {
        Err(Error::Unreadable(m)) => assert!(m.starts_with("2: "), "{m}"),
        other => panic!("expected Unreadable, got {other:?}"),
    }
}

#[test]
fn ts4_2_line_ending_tie_is_lf() {
    assert_eq!(line_ending(b"a\r\nb\n"), "\n");
    assert_eq!(line_ending(b"no line end"), "\n");
}
