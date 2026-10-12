//! TECHSPEC 4.4 writer and 4.8 R3, R5: rendering and appending a switch note.

use std::path::{Path, PathBuf};

use brain_swap::core::card_file::{Appended, append_note, parse, render_note};
use brain_swap::core::error::Error;
use brain_swap::core::model::{Card, CardId, HerdrPlace, NewNote, Place};
use brain_swap::core::time::{format_stamp, parse_stamp};
use jiff::tz::TimeZone;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cards");

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(Path::new(FIXTURES).join(name)).unwrap()
}

fn card(bytes: &[u8]) -> Card {
    let id = CardId::parse("W-12").unwrap();
    parse(id, Path::new("W-12.md"), bytes, "Todo", &TimeZone::UTC).card
}

/// The values of the first note of TECHSPEC 4.9.
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

fn append(name: &str, bytes: &[u8]) -> Appended {
    append_note(name, bytes, &note()).unwrap()
}

fn outside(a: &Appended) -> Vec<u8> {
    [&a.bytes[..a.inserted.start], &a.bytes[a.inserted.end..]].concat()
}

fn block() -> String {
    render_note(&note(), "\n").0
}

#[test]
fn fr16_append_only_adds_lines_inside_timeline() {
    let input = fixture("w12_4_9.md");
    let a = append("w12_4_9.md", &input);
    assert_eq!(outside(&a), input);
    let (before, after) = (card(&input), card(&a.bytes));
    assert_eq!(after.notes.len(), 3);
    assert_eq!(after.notes[..2], before.notes[..]);
    assert_eq!(
        (&after.column, &after.title, &after.body),
        (&before.column, &before.title, &before.body)
    );
}

#[test]
fn fr14_written_note_parses_back() {
    let n = note();
    let a = append("w11_4_9.md", &fixture("w11_4_9.md"));
    let last = card(&a.bytes).notes.pop().unwrap();
    assert_eq!(
        last.at.as_ref().map(format_stamp),
        Some(format_stamp(&n.at))
    );
    assert!(last.auto);
    assert_eq!(
        (last.doing, last.next, last.watch_out),
        (n.doing, n.next, n.watch_out)
    );
    assert_eq!(
        last.place,
        Place {
            cwd: Some(n.cwd),
            herdr: n.herdr,
            session: n.session
        }
    );
    // The block for the 4.9 first note is the 4.9 text byte for byte.
    let text = String::from_utf8(fixture("w12_4_9.md")).unwrap();
    assert!(text.contains(&block()), "{}", block());
    insta::assert_snapshot!("note_4_9", block());
}

#[test]
fn ts4_4_writer_emits_all_three_parts_even_empty() {
    let n = NewNote {
        next: String::new(),
        ..note()
    };
    let (b, _) = render_note(&n, "\n");
    assert!(b.lines().any(|l| l == "- Next:"), "{b}");
}

#[test]
fn ts4_4_multiline_part_uses_two_space_continuation() {
    let n = NewNote {
        doing: "a\nb".into(),
        ..note()
    };
    let (b, _) = render_note(&n, "\n");
    assert!(b.contains("- Doing: a\n  b\n"), "{b}");
}

#[test]
fn ts4_4_place_values_with_space_quote_backslash_are_quoted() {
    let cwd = PathBuf::from(r#"/a b/"c"\d"#);
    let n = NewNote {
        cwd: cwd.clone(),
        ..note()
    };
    let (b, _) = render_note(&n, "\n");
    let place = b.lines().find(|l| l.starts_with("<!-- where: ")).unwrap();
    assert!(place.contains(r#"cwd="/a b/\"c\"\\d""#), "{place}");
    let a = append_note("w11_4_9.md", &fixture("w11_4_9.md"), &n).unwrap();
    assert_eq!(card(&a.bytes).notes[0].place.cwd, Some(cwd));
}

#[test]
fn ts4_4_place_value_with_comment_end_is_dropped() {
    let n = NewNote {
        session: Some("x-->y".into()),
        ..note()
    };
    let (b, warnings) = render_note(&n, "\n");
    let place = b.lines().find(|l| l.starts_with("<!-- where: ")).unwrap();
    assert!(!place.contains("session="), "{place}");
    assert_eq!(
        warnings,
        vec!["place session contains '-->', dropped".to_string()]
    );
}

#[test]
fn fr18_all_empty_parts_rejected() {
    let n = NewNote {
        doing: String::new(),
        next: " ".into(),
        watch_out: "\n".into(),
        ..note()
    };
    let r = append_note("w12_4_9.md", &fixture("w12_4_9.md"), &n);
    assert!(
        matches!(&r, Err(Error::InvalidInput(m)) if m == "empty note"),
        "{r:?}"
    );
}

#[test]
fn r3_note_goes_before_post_section() {
    let input = fixture("post_section.md");
    let a = append("post_section.md", &input);
    assert!(a.bytes[..a.inserted.start].ends_with(b"- Watch out: f\n"));
    let text = String::from_utf8(input.clone()).unwrap();
    let post = text.find("\n# Appendix").unwrap();
    assert_eq!(&a.bytes[a.inserted.end..], &input[post..]);
    assert_eq!(&a.bytes[a.inserted.clone()], block().as_bytes());
}

#[test]
fn r3_missing_timeline_is_appended_at_end() {
    let input = fixture("no_timeline.md");
    let a = append("no_timeline.md", &input);
    let want = [&input[..], b"\n## Timeline\n", block().as_bytes()].concat();
    assert_eq!(a.bytes, want);
}

#[test]
fn r5_missing_final_newline_added_before_block() {
    let input = fixture("no_final_newline.md");
    let a = append("no_final_newline.md", &input);
    let want = [&input[..], b"\n", block().as_bytes()].concat();
    assert_eq!(a.bytes, want);
}

#[test]
fn r5_block_uses_crlf_in_crlf_card() {
    let lf = String::from_utf8(fixture("w12_4_9.md")).unwrap();
    let input = lf.replace('\n', "\r\n").into_bytes();
    let a = append("w12_4_9.md", &input);
    let inserted = std::str::from_utf8(&a.bytes[a.inserted.clone()]).unwrap();
    assert!(!inserted.is_empty());
    for l in inserted.split_inclusive('\n') {
        assert!(l.ends_with("\r\n"), "{l:?}");
    }
    assert_eq!(outside(&a), input);
}

#[test]
fn ts4_7_append_to_read_only_card_is_unreadable() {
    for name in [
        "invalid_frontmatter.md",
        "non_utf8.md",
        "conflict_markers.md",
    ] {
        let r = append_note(name, &fixture(name), &note());
        assert!(matches!(r, Err(Error::Unreadable(_))), "{name}: {r:?}");
    }
}

#[test]
fn nfr07_bytes_around_block_identical_with_unknown_keys() {
    let input = fixture("unknown_keys.md");
    let a = append("unknown_keys.md", &input);
    assert_eq!(outside(&a), input);
    assert_eq!(card(&a.bytes).notes.len(), 1);
}
