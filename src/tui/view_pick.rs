//! The template and board pickers (TECHSPEC 7.2, A-E4-17).

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::app::{App, Mode};

/// The title, then ` <hotkey>  <name>` per template; the selected row starts `>`, reversed.
pub fn draw_templates(frame: &mut Frame, area: Rect, app: &App) {
    let sel = match app.mode {
        Mode::TemplatePick { sel } => sel,
        _ => 0,
    };
    let column = app
        .board
        .column_list()
        .get(app.sel.column)
        .map(|c| c.name.clone())
        .unwrap_or_default();
    let mut lines = vec![Line::from(format!(
        " new card in {column}: pick a template"
    ))];
    for (i, (name, key)) in app.templates.iter().enumerate() {
        let (mark, style) = if i == sel {
            ('>', Style::default().add_modifier(Modifier::REVERSED))
        } else {
            (' ', Style::default())
        };
        let key = key.unwrap_or(' ');
        lines.push(Line::styled(format!("{mark}{key}  {name}"), style));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

/// ` switch board`, then ` <name>  <path>` per board with HOME as `~`; the selected row starts `>`, reversed.
pub fn draw_boards(frame: &mut Frame, area: Rect, app: &App) {
    let sel = match app.mode {
        Mode::BoardPick { sel } => sel,
        _ => 0,
    };
    let mut lines = vec![Line::from(" switch board")];
    for (i, (name, path)) in app.boards.iter().enumerate() {
        let (mark, style) = if i == sel {
            ('>', Style::default().add_modifier(Modifier::REVERSED))
        } else {
            (' ', Style::default())
        };
        let path = match path.strip_prefix(&app.home) {
            Ok(rel) => format!("~/{}", rel.display()),
            Err(_) => path.display().to_string(),
        };
        lines.push(Line::styled(format!("{mark}{name}  {path}"), style));
    }
    frame.render_widget(Paragraph::new(lines), area);
}
