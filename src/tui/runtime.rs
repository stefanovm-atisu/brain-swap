//! The TUI loop (TECHSPEC 7.1, 7.5): draws, polls keys, runs effects; terminal, input and
//! clock are injected so tests need no tty (12.1).

use std::fs;
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::path::Path;
use std::time::{Duration, SystemTime};

use jiff::SignedDuration;
use jiff::tz::TimeZone;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};

use super::Prepared;
use super::app::{App, Effect, Input, Mode, Outcome, update};
use super::editor::{Launcher, edit_card};
use super::view_frame;
use crate::core::board;
use crate::core::env::Env;
use crate::core::log;
use crate::core::model::CardId;
use crate::core::store::move_card;
use crate::core::time::Stamp;

pub trait TerminalGuard {
    fn enter(&mut self) -> io::Result<()>;
    fn leave(&mut self) -> io::Result<()>;
}

pub trait Events {
    fn poll(&mut self, timeout: Duration) -> io::Result<Option<KeyEvent>>;
}

pub trait Clock {
    fn now(&self) -> Stamp;
}

/// Raw mode and the alternate screen; the only code that enters them (7.5).
pub struct CrosstermGuard;

impl TerminalGuard for CrosstermGuard {
    fn enter(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)
    }
    fn leave(&mut self) -> io::Result<()> {
        let screen = execute!(io::stdout(), LeaveAlternateScreen);
        terminal::disable_raw_mode().and(screen)
    }
}

/// Key presses; resize and other events give `None`, so the next loop redraws.
pub struct CrosstermEvents;

impl Events for CrosstermEvents {
    fn poll(&mut self, timeout: Duration) -> io::Result<Option<KeyEvent>> {
        if !event::poll(timeout)? {
            return Ok(None);
        }
        Ok(match event::read()? {
            Event::Key(k) if k.kind == KeyEventKind::Press => Some(k),
            _ => None,
        })
    }
}

/// The system time in `tz` (`env.tz`).
pub struct SystemClock {
    pub tz: TimeZone,
}

impl Clock for SystemClock {
    fn now(&self) -> Stamp {
        jiff::Zoned::now().with_time_zone(self.tz.clone())
    }
}

pub struct Parts {
    pub guard: Box<dyn TerminalGuard>,
    pub events: Box<dyn Events>,
    pub clock: Box<dyn Clock>,
    pub launcher: Box<dyn Launcher>,
}

/// The sorted `(name, mtime, length)` of `board.md` and the card files of one letter (7.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint(Vec<(String, Option<SystemTime>, u64)>);

/// Lists `dir` and reads metadata only; an unlistable or missing folder gives an empty one.
pub fn fingerprint(dir: &Path, letter: char) -> Fingerprint {
    let mut v: Vec<_> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let card = CardId::from_file_name(&name).is_some_and(|id| id.letter == letter);
            let m = fs::metadata(e.path()).ok()?;
            (card || name == "board.md").then(|| (name, m.modified().ok(), m.len()))
        })
        .collect();
    v.sort();
    Fingerprint(v)
}

pub struct Runtime<B: Backend> {
    pub terminal: Terminal<B>,
    app: App,
    parts: Parts,
    env: Env,
    editor: String,
    print: Fingerprint,
    scans: u64,
    last_scan: Stamp,
}

impl<B: Backend> Runtime<B> {
    pub fn new(env: Env, prepared: Prepared, terminal: Terminal<B>, parts: Parts) -> Runtime<B> {
        let board = &prepared.app.board;
        Runtime {
            terminal,
            print: fingerprint(&board.path, board.meta.letter),
            last_scan: parts.clock.now(),
            scans: 0,
            app: prepared.app,
            parts,
            env,
            editor: prepared.config.map(|c| c.editor).unwrap_or_default(),
        }
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn draw(&mut self) -> io::Result<()> {
        let app = &self.app;
        self.terminal
            .draw(|f| view_frame::draw(f, app))
            .map(|_| ())
            .map_err(|e| io::Error::other(e.to_string()))
    }

    /// Runs `update`, then its effects in order; the exit code once quitting (A-E4-04).
    pub fn dispatch(&mut self, input: Input) -> Option<i32> {
        for effect in update(&mut self.app, input) {
            match effect {
                Effect::Quit => return Some(matches!(self.app.mode, Mode::ConfigError(_)) as i32),
                Effect::Move(id, column) => {
                    let b = &self.app.board;
                    let outcome = match move_card(&self.env, b, id, &column) {
                        Ok(_) => {
                            // Reload before `Done` (A-E4-05); a failed load keeps the board.
                            if let Ok(new) = board::load(&b.name, &b.path, &self.env) {
                                self.dispatch(Input::Reloaded(new));
                            }
                            Outcome::Moved(id)
                        }
                        Err(e) => Outcome::Failed(e.to_string()),
                    };
                    self.dispatch(Input::Done(outcome));
                }
                Effect::Edit(id) => {
                    let b = &self.app.board;
                    let outcome = edit_card(
                        &self.env,
                        b,
                        id,
                        &self.editor,
                        &mut *self.parts.launcher,
                        &mut *self.parts.guard,
                    );
                    // After `guard.enter()`: the editor drew over the screen, repaint it fully (7.5).
                    let _ = self.terminal.clear();
                    // Reload before `Done` (A-E4-05); a failed load keeps the board.
                    if matches!(outcome, Outcome::Edited(_))
                        && let Ok(b) = board::load(&b.name, &b.path, &self.env)
                    {
                        self.dispatch(Input::Reloaded(b));
                    }
                    self.dispatch(Input::Done(outcome));
                }
                // A failed load keeps the current board (A-E4-18); the new folder is the baseline.
                Effect::LoadBoard(name) => {
                    let Some((_, dir)) = self.app.boards.iter().find(|b| b.0 == name) else {
                        continue;
                    };
                    let input = match board::load(&name, dir, &self.env) {
                        Ok(b) => {
                            self.print = fingerprint(&b.path, b.meta.letter);
                            Input::Reloaded(b)
                        }
                        Err(e) => Input::Done(Outcome::Failed(e.to_string())),
                    };
                    self.dispatch(input);
                }
                _ => {}
            }
        }
        None
    }

    /// One scan of the board folder; a change reloads the board (7.4). Then `Tick`.
    pub fn tick(&mut self) -> Option<i32> {
        self.scans += 1;
        self.last_scan = self.parts.clock.now();
        let b = &self.app.board;
        let print = fingerprint(&b.path, b.meta.letter);
        if print != self.print {
            self.print = print;
            let input = match board::load(&b.name, &b.path, &self.env) {
                Ok(board) => {
                    log::event(&self.env, "reload", &board.name);
                    Input::Reloaded(board)
                }
                Err(e) => Input::Done(Outcome::Failed(e.to_string())),
            };
            if let Some(code) = self.dispatch(input) {
                return Some(code);
            }
        }
        self.dispatch(Input::Tick(self.last_scan.clone()))
    }

    /// Scans made by `tick`, the baseline of `new` not counted.
    pub fn scan_count(&self) -> u64 {
        self.scans
    }

    /// Enters the terminal, loops until an exit code and leaves it on every way out,
    /// a panic included; a terminal I/O error exits 1 (A-E4-04).
    pub fn run(&mut self) -> i32 {
        let result = catch_unwind(AssertUnwindSafe(|| self.lp()));
        let _ = self.parts.guard.leave();
        match result {
            Ok(Ok(code)) => code,
            Ok(Err(e)) => {
                eprintln!("brain-swap: terminal: {e}");
                1
            }
            Err(panic) => resume_unwind(panic),
        }
    }

    fn lp(&mut self) -> io::Result<i32> {
        self.parts.guard.enter()?;
        loop {
            self.draw()?;
            if let Some(key) = self.parts.events.poll(Duration::from_millis(250))?
                && let Some(code) = self.dispatch(Input::Key(key))
            {
                return Ok(code);
            }
            let due = self.parts.clock.now().duration_since(&self.last_scan);
            if due >= SignedDuration::from_secs(1)
                && let Some(code) = self.tick()
            {
                return Ok(code);
            }
        }
    }
}
