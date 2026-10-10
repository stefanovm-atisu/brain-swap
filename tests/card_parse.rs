//! TECHSPEC 4.4 and 4.9: reading a card file into the model.

use std::path::Path;

use brain_swap::core::card_file::{note_blocks, parse};
use brain_swap::core::model::{Card, CardId};
use brain_swap::core::time::format_stamp;
use jiff::tz::TimeZone;

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cards")
        .join(name);
    String::from_utf8(std::fs::read(path).unwrap()).unwrap()
}

fn card(text: &str) -> Card {
    let id = CardId::parse("W-12").unwrap();
    parse(
        id,
        Path::new("W-12.md"),
        text.as_bytes(),
        "Todo",
        &TimeZone::UTC,
    )
    .card
}

#[test]
fn fr07_parse_reads_title_column_template_created() {
    let c = card(&fixture("w12_4_9.md"));
    assert_eq!(c.title, "migrate invoices to v13");
    assert_eq!(c.column, "Doing");
    assert_eq!(c.template.as_deref(), Some("Feature"));
    assert_eq!(
        format_stamp(&c.created.unwrap()),
        "2026-10-08T10:02:11+03:00"
    );
}

#[test]
fn fr07_body_lies_between_title_and_timeline() {
    let body = card(&fixture("w12_4_9.md")).body;
    assert!(body.contains("## Goal"));
    assert!(body.contains("## Acceptance"));
    assert!(body.contains("The DBA applies the procedure on staging; we hand over the SQL only."));
    assert!(!body.contains("# migrate invoices to v13"));
    assert!(!body.contains("## Timeline"));
}

#[test]
fn fr14_note_has_parts_stamp_auto_and_place() {
    let n = &card(&fixture("w12_4_9.md")).notes[0];
    assert_eq!(
        format_stamp(n.at.as_ref().unwrap()),
        "2026-10-08T10:31:05+03:00"
    );
    assert!(n.auto);
    assert_eq!(
        n.doing,
        "switched InvoiceRepository to v13, unit tests green"
    );
    assert_eq!(n.next, "rerun the migration test against the staging copy");
    assert_eq!(
        n.watch_out,
        "v13 needs a 10 s timeout entry, the default is 2 s"
    );
    assert_eq!(
        n.place.cwd.as_deref(),
        Some(Path::new("/home/smilen/Work/ati.billing"))
    );
    let h = n.place.herdr.as_ref().unwrap();
    assert_eq!(h.pane, "w4V:p9");
    assert_eq!(h.tab.as_deref(), Some("w4V:t1"));
    assert_eq!(h.workspace.as_deref(), Some("w4V"));
    assert_eq!(
        n.place.session.as_deref(),
        Some("55583127-a22a-49bc-a803-e777c377a595")
    );
}

#[test]
fn fr14_note_without_auto_marker() {
    let n = &card(&fixture("w12_4_9.md")).notes[1];
    assert!(!n.auto);
    assert_eq!(n.heading, "### 2026-10-08T11:12:40+03:00");
}

#[test]
fn fr17_notes_keep_file_order_whatever_stamp() {
    let notes = card(&fixture("order_by_file.md")).notes;
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[1].heading, "### 2026-10-07T12:00:00+03:00");
    assert!(notes[1].at < notes[0].at);
}

#[test]
fn fr01_id_comes_from_file_name_not_content() {
    let text = fixture("w12_4_9.md").replace("# migrate invoices to v13", "# W-7 migrate");
    assert_eq!(card(&text).id, CardId::parse("W-12").unwrap());
}

#[test]
fn ts4_4_place_with_cwd_only() {
    let c = card(&fixture("w9_4_9.md"));
    let p = &c.notes[0].place;
    assert_eq!(
        p.cwd.as_deref(),
        Some(Path::new("/home/smilen/Work/private/brain-swap"))
    );
    assert_eq!(p.herdr, None);
    assert_eq!(p.session, None);
}

#[test]
fn ts4_4_continuation_joins_part() {
    let c = card("# t\n## Timeline\n### 2026-10-08T10:31:05+03:00\n- Next: a\n  b\n");
    assert_eq!(c.notes[0].next, "a\nb");
}

#[test]
fn ts4_4_missing_column_uses_default() {
    let text = fixture("w12_4_9.md").replace("column: Doing\n", "");
    assert_eq!(card(&text).column, "Todo");
}

#[test]
fn ts4_4_title_skips_fenced_heading() {
    let c = card("---\ncolumn: Doing\n---\n```\n# fenced\n```\n# real\nbody\n");
    assert_eq!(c.title, "real");
}

#[test]
fn ts4_4_untitled_without_title_line() {
    let c = card("---\ncolumn: Doing\n---\nsome body\n## Goal\n");
    assert_eq!(c.title, "(untitled)");
}

#[test]
fn ts4_4_empty_timeline_has_no_notes() {
    let c = card(&fixture("w11_4_9.md"));
    assert_eq!(c.title, "invoice PDF shows the wrong VAT");
    assert!(c.notes.is_empty());
}

#[test]
fn ts7_6_note_blocks_span_heading_to_place_line() {
    let text = fixture("w12_4_9.md");
    let blocks = note_blocks(&text);
    assert_eq!(blocks.len(), 2);
    let first = &text[blocks[0].clone()];
    assert!(first.starts_with("### 2026-10-08T10:31:05+03:00 auto"));
    assert!(first.ends_with("-->"));
}

#[test]
fn ts4_9_fixtures_parse_to_snapshot() {
    for (id, name) in [("W-12", "w12_4_9"), ("W-9", "w9_4_9"), ("W-11", "w11_4_9")] {
        let text = fixture(&format!("{name}.md"));
        let path = format!("{id}.md");
        let id = CardId::parse(id).unwrap();
        let card = parse(
            id,
            Path::new(&path),
            text.as_bytes(),
            "Todo",
            &TimeZone::UTC,
        )
        .card;
        insta::assert_debug_snapshot!(name, card);
    }
}
