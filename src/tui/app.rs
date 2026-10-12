//! TUI state machine (TECHSPEC 7.1): pure, no I/O.

use std::path::PathBuf;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::core::keys::{Action, Key, KeyContext, KeyMap, KeyPress};
use crate::core::model::{Board, BoardMeta, Card, CardId};
use crate::core::time::Stamp;

#[derive(Debug, Clone)]
pub struct App {
    pub mode: Mode,
    pub board: Board,
    pub sel: Selection,
    pub keys: KeyMap,
    pub status: Option<String>,
    pub now: Stamp,
    pub warnings: Vec<String>,
    /// Shown as `~` in the board header (A-E4-09).
    pub home: PathBuf,
}

/// `column` indexes `Board::column_list()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub column: usize,
    pub card: Option<CardId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Board,
    Detail {
        card: CardId,
        note: usize,
        scroll: u16,
    },
    TemplatePick {
        sel: usize,
    },
    TitleInput {
        template: String,
        buf: String,
    },
    BoardPick {
        sel: usize,
    },
    Message {
        text: String,
        back: Box<Mode>,
    },
    Help {
        back: Box<Mode>,
    },
    ConfigError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Key(KeyEvent),
    Tick(Stamp),
    Reloaded(Board),
    Done(Outcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Moved(CardId),
    Created(CardId),
    Edited(CardId),
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Move(CardId, String),
    Create {
        template: String,
        title: String,
        column: String,
    },
    Edit(CardId),
    LoadBoard(String),
    Quit,
}

impl App {
    pub fn new(board: Board, keys: KeyMap, now: Stamp, home: PathBuf) -> App {
        App {
            mode: Mode::Board,
            sel: initial_selection(&board),
            board,
            keys,
            status: None,
            now,
            warnings: Vec::new(),
            home,
        }
    }

    pub fn config_error(text: String, now: Stamp) -> App {
        let defaults = crate::core::board_file::parse("", None);
        let board = Board {
            name: String::new(),
            path: Default::default(),
            meta: BoardMeta {
                letter: defaults.letter,
                columns: defaults.columns,
                next: 1,
            },
            cards: Vec::new(),
            warnings: Vec::new(),
        };
        App {
            mode: Mode::ConfigError(text),
            ..App::new(board, KeyMap::defaults(), now, PathBuf::new())
        }
    }

    pub fn selected_card(&self) -> Option<&Card> {
        self.board
            .cards
            .iter()
            .find(|c| Some(c.id) == self.sel.card)
    }
}

/// The open card of newest activity and its column, else column 0 (FR-19, A-27).
pub fn initial_selection(board: &Board) -> Selection {
    let newest = board
        .open_cards()
        .into_iter()
        .max_by_key(|c| (c.activity().map(|s| s.timestamp()), c.id.number));
    let Some(card) = newest else {
        return Selection {
            column: 0,
            card: board.cards_in(0).first().map(|c| c.id),
        };
    };
    let column = (0..board.column_list().len())
        .find(|&i| board.cards_in(i).iter().any(|c| c.id == card.id))
        .unwrap_or(0);
    Selection {
        column,
        card: Some(card.id),
    }
}

/// Replaces the board, keeping the selected card wherever it now is; a vanished card gives
/// the card at its old row in the same column, clamped (7.4, A-E4-07).
fn reload(app: &mut App, board: Board) {
    let sel = &app.sel;
    let row = app
        .board
        .cards_in(sel.column)
        .iter()
        .position(|c| Some(c.id) == sel.card)
        .unwrap_or(0);
    let columns = board.column_list().len();
    let kept = sel
        .card
        .and_then(|id| (0..columns).find(|&i| board.cards_in(i).iter().any(|c| c.id == id)));
    app.sel = match kept {
        Some(column) => Selection {
            column,
            card: sel.card,
        },
        None => {
            let column = sel.column.min(columns.saturating_sub(1));
            let cards = board.cards_in(column);
            Selection {
                column,
                card: cards
                    .get(row.min(cards.len().saturating_sub(1)))
                    .map(|c| c.id),
            }
        }
    };
    app.board = board;
}

pub fn update(app: &mut App, input: Input) -> Vec<Effect> {
    // Nothing but Quit leaves ConfigError, so no effect can name a folder (FR-45).
    if let Mode::ConfigError(_) = app.mode {
        return match input {
            Input::Key(e) if e.kind != KeyEventKind::Release => vec![Effect::Quit],
            _ => Vec::new(),
        };
    }
    let event = match input {
        Input::Key(event) => event,
        Input::Done(Outcome::Failed(text)) => {
            app.status = Some(text);
            return Vec::new();
        }
        Input::Tick(now) => {
            app.now = now;
            return Vec::new();
        }
        Input::Reloaded(board) if board.name == app.board.name => {
            reload(app, board);
            return Vec::new();
        }
        _ => return Vec::new(),
    };
    app.status = None;
    if let Mode::Help { back } | Mode::Message { back, .. } = &mut app.mode {
        if event.kind != KeyEventKind::Release {
            app.mode = std::mem::replace(&mut **back, Mode::Board);
        }
        return Vec::new();
    }
    let action = key_press(&event)
        .zip(context(&app.mode))
        .and_then(|(p, ctx)| app.keys.action(ctx, &p));
    match (&app.mode, action) {
        (Mode::Board, Some(Action::Quit)) => vec![Effect::Quit],
        (Mode::Board, Some(Action::Open)) => {
            if let Some(card) = app.selected_card() {
                let note = card.notes.len().saturating_sub(1);
                app.mode = Mode::Detail {
                    card: card.id,
                    note,
                    scroll: 0,
                };
            }
            Vec::new()
        }
        // Same row in the target column, clamped to its last card; nothing wraps (A-E4-06).
        (Mode::Board, Some(a @ (Action::Up | Action::Down | Action::Left | Action::Right))) => {
            let (b, col) = (&app.board, app.sel.column);
            let row = b
                .cards_in(col)
                .iter()
                .position(|c| Some(c.id) == app.sel.card)
                .unwrap_or(0);
            let (column, row) = match a {
                Action::Up => (col, row.saturating_sub(1)),
                Action::Down => (col, row + 1),
                Action::Left => (col.saturating_sub(1), row),
                _ => ((col + 1).min(b.column_list().len() - 1), row),
            };
            let cards = b.cards_in(column);
            let card = cards.get(row.min(cards.len().saturating_sub(1)));
            app.sel = Selection {
                column,
                card: card.map(|c| c.id),
            };
            Vec::new()
        }
        (Mode::Detail { .. }, Some(a @ (Action::Up | Action::Down))) => {
            if let Mode::Detail { card, note, .. } = &mut app.mode {
                let last = app
                    .board
                    .card(*card)
                    .map_or(0, |c| c.notes.len().saturating_sub(1));
                *note = match a {
                    Action::Up => (*note + 1).min(last),
                    _ => note.saturating_sub(1),
                };
            }
            Vec::new()
        }
        (Mode::Detail { .. }, Some(Action::Quit)) => {
            app.mode = Mode::Board;
            Vec::new()
        }
        (Mode::Board, Some(Action::Help)) => {
            app.mode = Mode::Help {
                back: Box::new(Mode::Board),
            };
            Vec::new()
        }
        _ => Vec::new(),
    }
}

/// The one crossterm conversion point; release events give `None` (A-E4-26).
pub fn key_press(event: &KeyEvent) -> Option<KeyPress> {
    if event.kind == KeyEventKind::Release {
        return None;
    }
    let key = match event.code {
        KeyCode::Char(' ') => Key::Space,
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Esc,
        KeyCode::Tab => Key::Tab,
        KeyCode::BackTab => Key::BackTab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::F(n @ 1..=12) => Key::F(n),
        _ => return None,
    };
    let m = event.modifiers;
    Some(KeyPress {
        key,
        ctrl: m.contains(KeyModifiers::CONTROL),
        alt: m.contains(KeyModifiers::ALT),
        // crossterm reports Shift+Tab as BackTab with SHIFT set; the SHIFT is part of the key.
        shift: m.contains(KeyModifiers::SHIFT) && key != Key::BackTab,
    })
}

/// Help, Message and ConfigError take any key, so they have no context.
pub fn context(mode: &Mode) -> Option<KeyContext> {
    match mode {
        Mode::Board => Some(KeyContext::Board),
        Mode::Detail { .. } => Some(KeyContext::Detail),
        Mode::TemplatePick { .. } | Mode::BoardPick { .. } => Some(KeyContext::Picker),
        Mode::TitleInput { .. } => Some(KeyContext::TitleInput),
        Mode::Message { .. } | Mode::Help { .. } | Mode::ConfigError(_) => None,
    }
}
