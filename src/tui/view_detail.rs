//! The detail view of one card on one note (TECHSPEC 7.2, A-E4-11).

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Wrap};

use super::app::{App, Mode};
use crate::core::model::{Note, Place};
use crate::core::time::{Age, Stamp, age, format_stamp};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let Mode::Detail { card, note, .. } = app.mode else {
        return;
    };
    let Some(card) = app.board.card(card) else {
        return;
    };
    let mut lines = vec![format!(
        " {} {} ({}, {})",
        card.id, card.title, card.column, app.board.name
    )];
    if let Some(n) = card.notes.get(note) {
        lines.extend(note_lines(n, note, card.notes.len(), &app.now));
        lines.push(where_line(&n.place));
    }
    let text = lines.join("\n");
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), area);
}

/// The `note n/N` line, then the three parts with continuation lines indented to the text.
pub fn note_lines(note: &Note, index: usize, count: usize, now: &Stamp) -> Vec<String> {
    let ago = match age(note.at.as_ref(), now) {
        Age::Now => "now".to_string(),
        a => format!("{a} ago"),
    };
    let stamp = note.at.as_ref().map(format_stamp).unwrap_or_default();
    let auto = if note.auto { ", auto" } else { "" };
    let mut lines = vec![format!(" note {}/{count}, {stamp}, {ago}{auto}", index + 1)];
    for (label, text) in [
        ("Doing:", &note.doing),
        ("Next:", &note.next),
        ("Watch out:", &note.watch_out),
    ] {
        let indented = text.replace('\n', &format!("\n{:13}", ""));
        lines.extend(format!(" {label:<12}{indented}").lines().map(String::from));
    }
    lines
}

pub fn where_line(place: &Place) -> String {
    match (&place.cwd, &place.herdr) {
        (Some(cwd), Some(h)) => format!(" Where: {} (pane {})", cwd.display(), h.pane),
        (Some(cwd), None) => format!(" Where: {}", cwd.display()),
        _ => " Where: none".to_string(),
    }
}
