//! TECHSPEC 4.8 R1, R2, R3 and 12.1 round trip: every fixture through both splices.

mod common;

use std::path::Path;

use brain_swap::core::card_file::{Appended, append_note, parse, set_column};
use brain_swap::core::error::Error;
use brain_swap::core::model::{CardId, HerdrPlace, NewNote};
use brain_swap::core::time::parse_stamp;
use common::bytes::assert_only_lines_changed;
use jiff::tz::TimeZone;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cards");

/// The fixtures 4.7 makes read only; every other fixture is writable.
const READ_ONLY: [&str; 3] = [
    "conflict_markers.md",
    "invalid_frontmatter.md",
    "non_utf8.md",
];

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(Path::new(FIXTURES).join(name)).unwrap()
}

/// Every fixture file name, from the directory listing.
fn fixtures() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(FIXTURES)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert!(!names.is_empty());
    names
}

fn writable() -> Vec<String> {
    fixtures()
        .into_iter()
        .filter(|n| !READ_ONLY.contains(&n.as_str()))
        .collect()
}

fn note() -> NewNote {
    NewNote {
        at: parse_stamp("2026-10-08T10:31:05+03:00", &TimeZone::UTC).unwrap(),
        auto: true,
        doing: "switched InvoiceRepository to v13, unit tests green".into(),
        next: "rerun the migration test against the staging copy".into(),
        watch_out: "v13 needs a 10 s timeout entry, the default is 2 s".into(),
        cwd: "/home/smilen/Work/ati.billing".into(),
        herdr: Some(HerdrPlace {
            pane: "w4V:p9".into(),
            tab: Some("w4V:t1".into()),
            workspace: Some("w4V".into()),
        }),
        session: Some("55583127-a22a-49bc-a803-e777c377a595".into()),
    }
}

fn append(name: &str) -> Appended {
    append_note(name, &fixture(name), &note()).unwrap()
}

fn column(bytes: &[u8]) -> String {
    let id = CardId::parse("W-12").unwrap();
    parse(id, Path::new("W-12.md"), bytes, "Todo", &TimeZone::UTC)
        .card
        .column
}

fn stem(name: &str) -> &str {
    name.trim_end_matches(".md")
}

/// R2: `after` is `before` with one line replaced or inserted, that line reading
/// `column: Doing` (key spelling kept), or with the three-line frontmatter prepended.
fn r2_holds(before: &[u8], after: &[u8]) -> bool {
    let b: Vec<&[u8]> = before.split_inclusive(|&c| c == b'\n').collect();
    let a: Vec<&[u8]> = after.split_inclusive(|&c| c == b'\n').collect();
    let trim = |l: &[u8]| String::from_utf8_lossy(l.trim_ascii_end()).into_owned();
    if a.len() == b.len() + 3 && a[3..] == b[..] {
        return a[..3].iter().map(|l| trim(l)).collect::<Vec<_>>()
            == ["---", "column: Doing", "---"];
    }
    (0..a.len()).any(|i| {
        let doing = trim(a[i])
            .split_once(": ")
            .is_some_and(|(k, v)| k.eq_ignore_ascii_case("column") && v == "Doing");
        if !doing {
            return false;
        }
        let mut rest = a.clone();
        rest.remove(i);
        let replaced = a.len() == b.len()
            && trim(b[i]).to_ascii_lowercase().starts_with("column:")
            && a.iter()
                .zip(&b)
                .enumerate()
                .all(|(j, (x, y))| j == i || x == y);
        rest == b || replaced
    })
}

#[test]
fn r2_set_column_changes_one_line() {
    let before = fixture("w12_4_9.md");
    let after = set_column("w12_4_9.md", &before, "Done").unwrap();
    assert_only_lines_changed(&before, &after, &["column: Done\n"]);
}

#[test]
fn r2_set_column_adds_frontmatter_to_card_without_one() {
    let before = fixture("no_frontmatter.md");
    let after = set_column("no_frontmatter.md", &before, "Doing").unwrap();
    assert_eq!(after, [&b"---\ncolumn: Doing\n---\n"[..], &before].concat());
}

#[test]
fn r2_set_column_on_read_only_card_is_unreadable() {
    let r = set_column(
        "conflict_markers.md",
        &fixture("conflict_markers.md"),
        "Done",
    );
    assert!(matches!(r, Err(Error::Unreadable(_))), "{r:?}");
}

#[test]
fn r1_every_fixture_round_trips_append_byte_for_byte() {
    for name in fixtures() {
        let input = fixture(&name);
        let r = append_note(&name, &input, &note());
        if READ_ONLY.contains(&name.as_str()) {
            assert!(matches!(r, Err(Error::Unreadable(_))), "{name}: {r:?}");
            continue;
        }
        let a = r.unwrap();
        let outside = [&a.bytes[..a.inserted.start], &a.bytes[a.inserted.end..]].concat();
        assert_eq!(outside, input, "{name}");
    }
}

#[test]
fn r1_every_fixture_round_trips_set_column_byte_for_byte() {
    for name in fixtures() {
        let input = fixture(&name);
        let r = set_column(&name, &input, "Doing");
        if READ_ONLY.contains(&name.as_str()) {
            assert!(matches!(r, Err(Error::Unreadable(_))), "{name}: {r:?}");
            continue;
        }
        let out = r.unwrap();
        assert!(r2_holds(&input, &out), "{name}");
        insta::assert_snapshot!(
            format!("set_column_{}", stem(&name)),
            String::from_utf8_lossy(&out)
        );
    }
}

#[test]
fn r3_append_snapshot_per_fixture() {
    for name in writable() {
        let a = append(&name);
        insta::assert_snapshot!(
            format!("append_{}", stem(&name)),
            String::from_utf8_lossy(&a.bytes)
        );
    }
}

#[test]
fn fr16_append_never_changes_column() {
    for name in writable() {
        let input = fixture(&name);
        assert_eq!(column(&append(&name).bytes), column(&input), "{name}");
    }
}
