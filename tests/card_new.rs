//! TECHSPEC 4.8 R4 and 5.4: rendering a new card file.

use std::path::Path;

use brain_swap::core::card_file::{parse, render_new};
use brain_swap::core::model::{CardId, NewCard};
use brain_swap::core::time::{format_stamp, parse_stamp};
use jiff::tz::TimeZone;

fn input(body: &str) -> NewCard {
    NewCard {
        title: "migrate invoices to v13".into(),
        template: "Feature".into(),
        column: "Todo".into(),
        created: parse_stamp("2026-10-08T10:02:11+03:00", &TimeZone::UTC).unwrap(),
        body: body.into(),
    }
}

fn text(card: &NewCard) -> (String, Vec<String>) {
    let (bytes, warnings) = render_new(card);
    (String::from_utf8(bytes).unwrap(), warnings)
}

const BODY: &str = "## Goal\n\n## Acceptance\n\n## Context\n";

#[test]
fn r4_new_card_parses_back_to_rendered_values() {
    let new = input(BODY);
    let (bytes, _) = render_new(&new);
    let id = CardId::parse("W-12").unwrap();
    let c = parse(id, Path::new("W-12.md"), &bytes, "Done", &TimeZone::UTC).card;
    assert_eq!(c.title, new.title);
    assert_eq!(c.column, new.column);
    assert_eq!(c.template.as_deref(), Some("Feature"));
    assert_eq!(
        c.created.as_ref().map(format_stamp),
        Some(format_stamp(&new.created))
    );
    assert_eq!(c.body.trim(), new.body.trim());
    assert!(c.notes.is_empty());
}

#[test]
fn r4_new_card_bytes_snapshot() {
    insta::assert_snapshot!("new_card", text(&input(BODY)).0);
}

#[test]
fn ts6_3_timeline_heading_in_body_dropped_with_warning() {
    let (t, warnings) = text(&input("## Goal\n## Timeline\nx\n"));
    let lines: Vec<&str> = t.lines().collect();
    assert_eq!(lines.iter().filter(|l| **l == "## Timeline").count(), 1);
    assert_eq!(lines.last(), Some(&"## Timeline"));
    assert!(lines.contains(&"x"));
    assert_eq!(warnings, vec!["## Timeline in body dropped".to_string()]);
}

#[test]
fn r4_empty_body_renders_title_then_timeline() {
    let (t, _) = text(&input(""));
    let after = t.splitn(3, "---\n").nth(2).unwrap();
    assert_eq!(after, "# migrate invoices to v13\n\n## Timeline\n");
}

#[test]
fn nfr06_new_card_is_utf8_markdown_with_one_title() {
    let (bytes, _) = render_new(&input(BODY));
    let t = String::from_utf8(bytes).unwrap();
    assert_eq!(t.lines().filter(|l| l.starts_with("# ")).count(), 1);
    assert!(!t.contains("W-12"));
}

#[test]
fn r4_unclosed_fence_in_body_is_closed_before_timeline() {
    let (bytes, _) = render_new(&input("## Repro\n\n```sh\ncargo test billing\n"));
    let id = CardId::parse("W-12").unwrap();
    let parsed = parse(id, Path::new("W-12.md"), &bytes, "Todo", &TimeZone::UTC);
    let text = String::from_utf8(bytes).unwrap();
    assert!(!parsed.card.body.contains("## Timeline"), "{text}");
    assert!(text.ends_with("```\n\n## Timeline\n"), "{text}");
}

#[test]
fn ts6_3_fenced_timeline_line_in_body_is_kept() {
    let body = "## Repro\n\n```md\n# card\n\n## Timeline\n```\n";
    let (bytes, warnings) = render_new(&input(body));
    let id = CardId::parse("W-12").unwrap();
    let c = parse(id, Path::new("W-12.md"), &bytes, "Todo", &TimeZone::UTC).card;
    let text = String::from_utf8(bytes).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(c.body.trim(), body.trim(), "{text}");
    assert!(text.ends_with("```\n\n## Timeline\n"), "{text}");
}
