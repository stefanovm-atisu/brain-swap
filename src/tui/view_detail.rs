//! The detail view of one card on one note (TECHSPEC 7.2, A-E4-11).

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::{Paragraph, Wrap};

use super::app::{App, Mode, body_lines};
use crate::core::keys::{Action, KeyMap};
use crate::core::model::{Note, Place};
use crate::core::time::{Age, Stamp, age, format_stamp};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let Mode::Detail { card, note, scroll } = app.mode else {
        return;
    };
    let Some(card) = app.board.card(card) else {
        return;
    };
    let mut lines = vec![format!(
        " {} {} ({}, {})",
        card.id, card.title, card.column, app.board.name
    )];
    match card.notes.get(note) {
        Some(n) => {
            lines.extend(note_lines(n, note, card.notes.len(), &app.now));
            lines.push(where_line(&n.place));
        }
        None => lines.push(" no note yet".to_string()),
    }
    let width = usize::from(area.width);
    lines.push("-".repeat(width));
    // Each body line fits the width, so the wrap below never splits it (A-E4-11).
    lines.extend(
        body_lines(&card.body)
            .into_iter()
            .skip(usize::from(scroll))
            .map(|l| {
                let mut used = 0;
                format!(" {l}")
                    .chars()
                    .take_while(|c| {
                        used += Span::raw(c.to_string()).width();
                        used <= width
                    })
                    .collect()
            }),
    );
    let Some(hint_row) = area.height.checked_sub(1) else {
        return;
    };
    let text = lines.join("\n");
    let screen = Rect {
        height: hint_row,
        ..area
    };
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), screen);
    let hint = Rect {
        y: area.y + hint_row,
        height: 1,
        ..area
    };
    frame.render_widget(Paragraph::new(hint_line(&app.keys)), hint);
}

/// Each Detail action's first binding with its word (TECHSPEC 7.2).
pub fn hint_line(keys: &KeyMap) -> String {
    let hints = [
        (Action::Down, "older"),
        (Action::Up, "newer"),
        (Action::ScrollDown, "down"),
        (Action::ScrollUp, "up"),
        (Action::Jump, "jump"),
        (Action::Edit, "edit"),
        (Action::Help, "help"),
        (Action::Quit, "back"),
    ];
    let line: String = hints
        .iter()
        .filter_map(|&(a, word)| keys.keys(a).first().map(|k| format!(" {k} {word} ")))
        .collect();
    line.trim_end().to_string()
}

/// The `note n/N` line, then the three parts with continuation lines indented to the text.
pub fn note_lines(note: &Note, index: usize, count: usize, now: &Stamp) -> Vec<String> {
    let ago = match age(note.at.as_ref(), now) {
        Age::Now => "now".to_string(),
        a => format!("{a} ago"),
    };
    // A bad stamp shows the heading text after `### ` (TECHSPEC 4.7).
    let stamp = note.at.as_ref().map_or_else(
        || note.heading.get(4..).unwrap_or_default().trim().to_string(),
        format_stamp,
    );
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
