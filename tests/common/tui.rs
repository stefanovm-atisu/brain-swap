//! `TuiFixture` and card writers for the TUI tests.

use super::env::core_env;
use brain_swap::core::board::load;
use brain_swap::core::env::Env;
use brain_swap::core::keys::{Key, KeyMap, parse_binding};
use brain_swap::core::model::Board;
use brain_swap::tui::app::{App, Effect, Input, update};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs;
use std::path::{Path, PathBuf};

pub const NOW: &str = "2026-10-08T11:52:00+03:00";

pub struct TuiFixture {
    _tmp: tempfile::TempDir,
    pub home: PathBuf,
    pub board_dir: PathBuf,
    pub env: Env,
}

impl TuiFixture {
    /// A temp home, `tests/fixtures/boards/<fixture>/` as board `work`, the default config written.
    pub fn new(fixture: Option<&str>) -> TuiFixture {
        let fx = TuiFixture::without_config(fixture);
        brain_swap::core::config::load(&fx.env).unwrap();
        fx
    }

    /// Like `new`, but no config file exists (first run).
    pub fn without_config(fixture: Option<&str>) -> TuiFixture {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("home");
        let board_dir = home.join(".local/share/brain-swap/work");
        fs::create_dir_all(&home).unwrap();
        if let Some(f) = fixture {
            fs::create_dir_all(&board_dir).unwrap();
            let from = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/boards")
                .join(f);
            for e in fs::read_dir(from).unwrap() {
                let e = e.unwrap();
                fs::copy(e.path(), board_dir.join(e.file_name())).unwrap();
            }
        }
        let env = core_env(&home, NOW);
        TuiFixture {
            _tmp: tmp,
            home,
            board_dir,
            env,
        }
    }

    pub fn board(&self) -> Board {
        load("work", &self.board_dir, &self.env).unwrap()
    }

    pub fn app(&self) -> App {
        App::new(self.board(), KeyMap::defaults(), self.env.now.clone())
    }
}

/// A canonical card `card <id>` with one note per `(stamp, next)`.
pub fn write_card(dir: &Path, id: &str, column: &str, created: &str, notes: &[(&str, &str)]) {
    let mut text = format!(
        "---\ncolumn: {column}\ntemplate: Feature\ncreated: {created}\n---\n# card {id}\n\n## Timeline\n"
    );
    for (at, next) in notes {
        text += &format!(
            "\n### {at}\n- Doing: doing {id}\n- Next: {next}\n- Watch out: nothing\n<!-- where: cwd=/w -->\n"
        );
    }
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join(format!("{id}.md")), text).unwrap();
}

/// The press event crossterm reports for a 5.3 binding; an upper-case `Char` carries SHIFT.
pub fn key(spec: &str) -> KeyEvent {
    let b = parse_binding(spec).unwrap();
    let code = match b.key {
        Key::Char(c) => KeyCode::Char(c),
        Key::Space => KeyCode::Char(' '),
        Key::Enter => KeyCode::Enter,
        Key::Esc => KeyCode::Esc,
        Key::Tab => KeyCode::Tab,
        Key::BackTab => KeyCode::BackTab,
        Key::Backspace => KeyCode::Backspace,
        Key::Delete => KeyCode::Delete,
        Key::Up => KeyCode::Up,
        Key::Down => KeyCode::Down,
        Key::Left => KeyCode::Left,
        Key::Right => KeyCode::Right,
        Key::Home => KeyCode::Home,
        Key::End => KeyCode::End,
        Key::PageUp => KeyCode::PageUp,
        Key::PageDown => KeyCode::PageDown,
        Key::F(n) => KeyCode::F(n),
    };
    let mut m = KeyModifiers::NONE;
    for (on, flag) in [
        (b.ctrl, KeyModifiers::CONTROL),
        (b.alt, KeyModifiers::ALT),
        (
            b.shift || matches!(b.key, Key::Char(c) if c.is_uppercase()),
            KeyModifiers::SHIFT,
        ),
    ] {
        if on {
            m |= flag;
        }
    }
    KeyEvent::new(code, m)
}

/// All effects of the presses, in order.
pub fn press(app: &mut App, keys: &[&str]) -> Vec<Effect> {
    keys.iter()
        .flat_map(|k| update(app, Input::Key(key(k))))
        .collect()
}
