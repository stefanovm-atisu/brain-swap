//! The frame: the mode's screen above, the status line on the last row (TECHSPEC 7.2).

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use super::app::App;

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
}
