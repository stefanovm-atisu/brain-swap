//! Message and ConfigError: the text, one row per line, wrapped to the width (TECHSPEC 7.2, A-E4-15).

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Wrap};

pub fn draw(frame: &mut Frame, text: &str, area: Rect) {
    let rows: Vec<String> = text.lines().map(|l| format!(" {l}")).collect();
    frame.render_widget(
        Paragraph::new(rows.join("\n")).wrap(Wrap { trim: false }),
        area,
    );
}
