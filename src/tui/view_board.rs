//! The board screen of TECHSPEC 4.9: header, columns and cards (7.2, 7.3).

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Paragraph;

use super::app::App;
use crate::core::keys::Action;
use crate::core::model::Card;
use crate::core::time::{Age, Stamp, age};

/// `<ID> <title>` and `  <age>  <preview>`, uncut (FR-20, A-E4-10).
pub fn card_lines(card: &Card, now: &Stamp) -> (String, String) {
    let second = match card.latest_note() {
        None => "  no note".to_string(),
        Some(n) => {
            let age = age(n.at.as_ref(), now);
            let mut lines = n.next.lines().chain(n.doing.lines()).map(str::trim);
            match lines.find(|l| !l.is_empty()) {
                Some(p) => format!("  {age}  {p}"),
                None => format!("  {age}"),
            }
        }
    };
    (format!("{} {}", card.id, card.title), second)
}

/// `now`, else `<age> ago` (the panel's form, 7.3).
pub fn age_ago(at: Option<&Stamp>, now: &Stamp) -> String {
    match age(at, now) {
        Age::Now => "now".into(),
        a => format!("{a} ago"),
    }
}

/// Columns split the width after a one-character margin, each ending in a two-character gap (A-E4-09).
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let row = |x: u16, y: u16, width: u16| Rect {
        x,
        y,
        width,
        height: 1,
    };
    let board = &app.board;
    let path = match board.path.strip_prefix(&app.home) {
        Ok(rel) => format!("~/{}", rel.display()),
        Err(_) => board.path.display().to_string(),
    };
    let help = app.keys.keys(Action::Help).first();
    let help = help.map(|k| format!("{k} help")).unwrap_or_default();
    let header = row(area.x, area.y, area.width.saturating_sub(1));
    frame.render_widget(Paragraph::new(format!(" {}  {path}", board.name)), header);
    frame.render_widget(Paragraph::new(help).alignment(Alignment::Right), header);

    let columns = board.column_list();
    let w = area.width.saturating_sub(1) / columns.len().max(1) as u16;
    let text = w.saturating_sub(2);
    for (i, column) in columns.iter().enumerate() {
        let x = area.x + 1 + i as u16 * w;
        let cards = board.cards_in(i);
        let mut title = format!("{} ({})", column.name, cards.len());
        let mut style = Style::default();
        if i == app.sel.column {
            title = format!("> {title}");
            style = style.add_modifier(Modifier::BOLD);
        }
        frame.render_widget(Paragraph::new(title).style(style), row(x, area.y + 1, text));
        for (j, card) in cards.iter().enumerate() {
            let y = area.y + 2 + 2 * j as u16;
            if y + 1 >= area.bottom() {
                break;
            }
            let mut style = Style::default();
            if app.sel.card == Some(card.id) {
                style = style.add_modifier(Modifier::REVERSED);
            }
            let (a, b) = card_lines(card, &app.now);
            frame.render_widget(Paragraph::new(a).style(style), row(x, y, text));
            frame.render_widget(Paragraph::new(b).style(style), row(x, y + 1, text));
        }
    }
}
