//! `TuiFixture` and card writers for the TUI tests.

use super::env::core_env;
use brain_swap::core::board::load;
use brain_swap::core::env::Env;
use brain_swap::core::keys::{Key, KeyMap, parse_binding};
use brain_swap::core::model::Board;
use brain_swap::core::time::Stamp;
use brain_swap::tui::app::{App, Effect, Input, update};
use brain_swap::tui::editor::Launcher;
use brain_swap::tui::runtime::{Clock, Events, Parts, Runtime, TerminalGuard};
use brain_swap::tui::view_frame::draw;
use brain_swap::tui::{Prepared, TuiStart, prepare};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::rc::Rc;
use std::time::Duration;

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
        App::new(
            self.board(),
            KeyMap::defaults(),
            self.env.now.clone(),
            self.home.clone(),
        )
    }

    /// A runtime over `tui::prepare` of the default board on a `TestBackend`, with fakes and a
    /// launcher that fails the test when called.
    pub fn runtime(
        &self,
        width: u16,
        height: u16,
    ) -> (Runtime<TestBackend>, FakeGuard, FakeEvents) {
        let mut env = self.env.clone();
        let start = TuiStart {
            config: brain_swap::core::config::load(&env),
            board: None,
        };
        let prepared = prepare(&mut env, start).unwrap();
        fake_runtime(&env, prepared, width, height)
    }
}

/// A runtime over `prepared` with a `FakeGuard`, an empty `FakeEvents` and a `FakeClock` at `env.now`.
pub fn fake_runtime(
    env: &Env,
    prepared: Prepared,
    width: u16,
    height: u16,
) -> (Runtime<TestBackend>, FakeGuard, FakeEvents) {
    fake_runtime_with(env, prepared, width, height, Box::new(NoLauncher))
}

/// Like `fake_runtime`, with `launcher` as the editor launcher.
pub fn fake_runtime_with(
    env: &Env,
    prepared: Prepared,
    width: u16,
    height: u16,
    launcher: Box<dyn Launcher>,
) -> (Runtime<TestBackend>, FakeGuard, FakeEvents) {
    let guard = FakeGuard::default();
    let clock = FakeClock(Rc::new(RefCell::new(env.now.clone())));
    let events = FakeEvents {
        queue: Rc::default(),
        timeouts: Rc::default(),
        clock: clock.clone(),
    };
    let parts = Parts {
        guard: Box::new(guard.clone()),
        events: Box::new(events.clone()),
        clock: Box::new(clock),
        launcher,
    };
    let terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    (
        Runtime::new(env.clone(), prepared, terminal, parts),
        guard,
        events,
    )
}

/// A shared log of `enter` and `leave`.
#[derive(Clone, Default)]
pub struct FakeGuard(pub Rc<RefCell<Vec<&'static str>>>);

impl TerminalGuard for FakeGuard {
    fn enter(&mut self) -> io::Result<()> {
        self.0.borrow_mut().push("enter");
        Ok(())
    }
    fn leave(&mut self) -> io::Result<()> {
        self.0.borrow_mut().push("leave");
        Ok(())
    }
}

/// Fails the test when an editor is launched.
pub struct NoLauncher;

impl Launcher for NoLauncher {
    fn launch(&mut self, argv: &[String]) -> io::Result<ExitStatus> {
        panic!("unexpected editor launch: {argv:?}")
    }
}

/// One launch: the argv and the copy's bytes at that moment.
pub type Launch = (Vec<String>, Vec<u8>);

/// Records each argv with the copy's bytes at launch, runs `on_launch` on the copy (the last
/// argument), then exits with `code`, or fails to launch when `code` is `None`.
#[derive(Clone)]
pub struct FakeLauncher {
    pub calls: Rc<RefCell<Vec<Launch>>>,
    pub on_launch: Rc<dyn Fn(&Path)>,
    pub code: Option<i32>,
}

impl FakeLauncher {
    pub fn new(code: Option<i32>, on_launch: impl Fn(&Path) + 'static) -> FakeLauncher {
        FakeLauncher {
            calls: Rc::default(),
            on_launch: Rc::new(on_launch),
            code,
        }
    }
}

impl Launcher for FakeLauncher {
    fn launch(&mut self, argv: &[String]) -> io::Result<ExitStatus> {
        let copy = Path::new(argv.last().expect("argv without a copy path"));
        let bytes = fs::read(copy).unwrap();
        self.calls.borrow_mut().push((argv.to_vec(), bytes));
        (self.on_launch)(copy);
        match self.code {
            Some(c) => Ok(ExitStatus::from_raw(c << 8)),
            None => Err(io::ErrorKind::NotFound.into()),
        }
    }
}

#[derive(Clone)]
pub struct FakeClock(pub Rc<RefCell<Stamp>>);

impl Clock for FakeClock {
    fn now(&self) -> Stamp {
        self.0.borrow().clone()
    }
}

/// One scripted answer of `FakeEvents::poll`.
pub enum Ev {
    Key(&'static str),
    Timeout,
    Error,
    Panic,
}

/// Answers `poll` from `queue`, records each timeout; a `Timeout` advances the clock by it.
#[derive(Clone)]
pub struct FakeEvents {
    pub queue: Rc<RefCell<VecDeque<Ev>>>,
    pub timeouts: Rc<RefCell<Vec<Duration>>>,
    pub clock: FakeClock,
}

impl FakeEvents {
    pub fn push(&self, events: impl IntoIterator<Item = Ev>) {
        self.queue.borrow_mut().extend(events);
    }
}

impl Events for FakeEvents {
    fn poll(&mut self, timeout: Duration) -> io::Result<Option<KeyEvent>> {
        self.timeouts.borrow_mut().push(timeout);
        match self
            .queue
            .borrow_mut()
            .pop_front()
            .expect("FakeEvents exhausted")
        {
            Ev::Key(k) => Ok(Some(key(k))),
            Ev::Timeout => {
                let mut now = self.clock.0.borrow_mut();
                *now = now.checked_add(timeout).unwrap();
                Ok(None)
            }
            Ev::Error => Err(io::Error::other("fake event error")),
            Ev::Panic => panic!("fake event panic"),
        }
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

/// One `TestBackend` frame of `view_frame::draw`.
pub fn render(app: &App, width: u16, height: u16) -> Buffer {
    let mut t = Terminal::new(TestBackend::new(width, height)).unwrap();
    t.draw(|f| draw(f, app)).unwrap();
    t.backend().buffer().clone()
}

/// The buffer's rows, trailing blanks trimmed.
pub fn rows(buf: &Buffer) -> Vec<String> {
    let w = buf.area.width as usize;
    let cells: Vec<&str> = buf.content.iter().map(|c| c.symbol()).collect();
    cells
        .chunks(w)
        .map(|r| r.concat().trim_end().to_string())
        .collect()
}

/// Every cell has fg and bg `Color::Reset` (NFR-10); every TUI snapshot test calls it.
pub fn assert_no_colour(buf: &Buffer) {
    for (i, c) in buf.content.iter().enumerate() {
        assert_eq!((c.fg, c.bg), (Color::Reset, Color::Reset), "cell {i}");
    }
}
