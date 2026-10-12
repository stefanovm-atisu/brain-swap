//! The board screen of TECHSPEC 4.9: header, columns and cards (7.2, 7.3).

use std::ops::Range;

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Paragraph;

use super::app::App;
use crate::core::keys::Action;
use crate::core::model::Card;
use crate::core::time::{Age, Stamp, age};

/// Narrowest column before the board shows a window of columns (A-E4-09).
pub const MIN_COLUMN_WIDTH: u16 = 20;

/// Terminal rows below which the preview panel is hidden (7.3, A-19).
pub const PANEL_MIN_ROWS: u16 = 20;

/// The columns drawn at `width`, a window around `focused` when not all fit (A-E4-09).
pub fn visible_columns(count: usize, focused: usize, width: u16) -> Range<usize> {
    let fit = ((width.saturating_sub(1) / MIN_COLUMN_WIDTH).max(1) as usize).min(count);
    let start = focused.saturating_sub(fit / 2).min(count - fit);
    start..start + fit
}

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

/// The preview panel's rows after the separator, without the margin (FR-22, A-E4-10).
pub fn panel_lines(card: Option<&Card>, now: &Stamp) -> Vec<String> {
    let Some(card) = card else {
        return Vec::new();
    };
    let Some(n) = card.latest_note() else {
        return vec![format!("{} no note yet", card.id)];
    };
    let mut head = format!("{} latest: {}", card.id, age_ago(n.at.as_ref(), now));
    if n.auto {
        head += ", auto";
    }
    if let Some(h) = &n.place.herdr {
        head += &format!(", pane {}", h.pane);
    }
    if let Some(cwd) = &n.place.cwd {
        head += &format!(", {}", cwd.display());
    }
    let first = |s: &str| s.lines().next().unwrap_or_default().to_string();
    vec![
        head,
        format!("Doing:      {}", first(&n.doing)),
        format!("Next:       {}", first(&n.next)),
        format!("Watch out:  {}", first(&n.watch_out)),
    ]
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
    let shown = visible_columns(columns.len(), app.sel.column, area.width);
    let w = area.width.saturating_sub(1) / shown.len().max(1) as u16;
    let text = w.saturating_sub(2);
    // The panel takes the separator and four rows above the status line (7.3).
    let panel = 5 * u16::from(frame.area().height >= PANEL_MIN_ROWS);
    let slots = area.height.saturating_sub(2 + panel) as usize / 2;
    for (k, i) in shown.enumerate() {
        let (x, column) = (area.x + 1 + k as u16 * w, &columns[i]);
        let cards = board.cards_in(i);
        let mut title = format!("{} ({})", column.name, cards.len());
        let mut style = Style::default();
        if i == app.sel.column {
            title = format!("> {title}");
            style = style.add_modifier(Modifier::BOLD);
        }
        frame.render_widget(Paragraph::new(title).style(style), row(x, area.y + 1, text));
        // The focused column scrolls just enough to show its selection, the others show their top (A-E4-06).
        let sel = cards.iter().position(|c| Some(c.id) == app.sel.card);
        let first = match sel {
            Some(r) if i == app.sel.column => (r + 1).saturating_sub(slots),
            _ => 0,
        };
        for (j, card) in cards.iter().skip(first).take(slots).enumerate() {
            let y = area.y + 2 + 2 * j as u16;
            let mut style = Style::default();
            if app.sel.card == Some(card.id) {
                style = style.add_modifier(Modifier::REVERSED);
            }
            let (a, b) = card_lines(card, &app.now);
            frame.render_widget(Paragraph::new(a).style(style), row(x, y, text));
            frame.render_widget(Paragraph::new(b).style(style), row(x, y + 1, text));
        }
    }
    if panel == 0 {
        return;
    }
    let y = area.bottom() - panel;
    let line = "-".repeat(area.width.saturating_sub(2).into());
    frame.render_widget(Paragraph::new(line), row(area.x + 1, y, area.width - 1));
    let card = app.sel.card.and_then(|id| board.card(id));
    for (k, l) in panel_lines(card, &app.now).into_iter().enumerate() {
        let r = row(area.x + 1, y + 1 + k as u16, area.width.saturating_sub(1));
        frame.render_widget(Paragraph::new(l), r);
    }
}
