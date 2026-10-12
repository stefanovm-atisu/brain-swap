//! The frame: the mode's screen above, the status line on the last row (TECHSPEC 7.2).

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use super::app::{App, Mode};
use super::{view_board, view_detail, view_help, view_message, view_pick};

/// Mode screens fill the rows above as later tasks add them.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let Some(row) = area.height.checked_sub(1) else {
        return;
    };
    let line = Rect {
        y: area.y + row,
        height: 1,
        ..area
    };
    let status = app.status.as_deref().map(|s| format!(" {s}"));
    frame.render_widget(Paragraph::new(status.unwrap_or_default()), line);
    let screen = Rect {
        height: row,
        ..area
    };
    match &app.mode {
        Mode::Board => view_board::draw(frame, screen, app),
        Mode::Detail { .. } => view_detail::draw(frame, screen, app),
        Mode::TemplatePick { .. } => view_pick::draw_templates(frame, screen, app),
        Mode::Help { .. } => view_help::draw(frame, app, screen),
        Mode::Message { text, .. } | Mode::ConfigError(text) => {
            view_message::draw(frame, text, screen)
        }
        _ => {}
    }
}
