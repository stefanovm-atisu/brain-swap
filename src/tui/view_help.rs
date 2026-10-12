//! Help: every action with its current keys, then all warnings (TECHSPEC 7.2, 11, FR-32).

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use super::app::App;
use crate::core::keys::Action;

/// Rows beyond `area` are cut (A-E4-14).
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let mut rows = vec![" help".to_string()];
    for a in Action::all() {
        let keys: Vec<String> = app.keys.keys(a).iter().map(|k| k.to_string()).collect();
        rows.push(format!(" {:<14}{}", a.config_name(), keys.join(", ")));
    }
    let warnings: Vec<&String> = app.warnings.iter().chain(&app.board.warnings).collect();
    if !warnings.is_empty() {
        rows.push(String::new());
        rows.push(" warnings:".into());
        rows.extend(warnings.iter().map(|w| format!(" {w}")));
    }
    frame.render_widget(Paragraph::new(rows.join("\n")), area);
}
