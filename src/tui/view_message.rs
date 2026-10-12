//! Message: its text, one row per line (TECHSPEC 7.2, A-E4-15).

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame, text: &str, area: Rect) {
    let rows: Vec<String> = text.lines().map(|l| format!(" {l}")).collect();
    frame.render_widget(Paragraph::new(rows.join("\n")), area);
}
