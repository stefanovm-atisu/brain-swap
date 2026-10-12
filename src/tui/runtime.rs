//! The TUI loop (TECHSPEC 7.1, 7.5): draws, polls keys, runs effects; terminal, input and
//! clock are injected so tests need no tty (12.1).

use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::time::Duration;

use jiff::tz::TimeZone;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};

use super::Prepared;
use super::app::{App, Effect, Input, Mode, update};
use super::view_frame;
use crate::core::env::Env;
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
}

pub struct Runtime<B: Backend> {
    pub terminal: Terminal<B>,
    app: App,
    parts: Parts,
}

impl<B: Backend> Runtime<B> {
    pub fn new(_env: Env, prepared: Prepared, terminal: Terminal<B>, parts: Parts) -> Runtime<B> {
        Runtime {
            terminal,
            app: prepared.app,
            parts,
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
            if effect == Effect::Quit {
                return Some(matches!(self.app.mode, Mode::ConfigError(_)) as i32);
            }
        }
        None
    }

    /// Enters the terminal, loops until an exit code and leaves it on every way out,
    /// a panic included; a terminal I/O error exits 1 (A-E4-04).
    pub fn run(mut self) -> i32 {
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
        }
    }
}
