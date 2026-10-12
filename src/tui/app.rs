//! TUI state machine (TECHSPEC 7.1): pure, no I/O.

use std::path::PathBuf;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::core::keys::{Action, Key, KeyContext, KeyMap, KeyPress};
use crate::core::model::{Board, BoardMeta, Card, CardId};
use crate::core::time::Stamp;

/// Lines `scroll_down` and `scroll_up` move the Detail body (A-E4-12).
pub const SCROLL_STEP: u16 = 10;

/// The card body without its leading and trailing blank lines.
pub fn body_lines(body: &str) -> Vec<&str> {
    body.trim_start_matches(['\r', '\n'])
        .trim_end()
        .lines()
        .collect()
}

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
    /// Name and final hotkey, in load order; `App::new` starts with the built-ins.
    pub templates: Vec<(String, Option<char>)>,
    /// Name and folder of each configured board, in config order.
    pub boards: Vec<(String, PathBuf)>,
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
            templates: crate::core::template::builtins()
                .into_iter()
                .map(|t| (t.name, t.key))
                .collect(),
            boards: Vec::new(),
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

/// The neighbouring configured column; a card in an extra column goes to the first (7.3, 4.7).
pub fn move_target(board: &Board, card: &Card, right: bool) -> Option<String> {
    let cols = &board.meta.columns;
    let Some(i) = cols.iter().position(|c| *c == card.column) else {
        return cols.first().cloned();
    };
    let j = if right { i + 1 } else { i.checked_sub(1)? };
    cols.get(j).cloned()
}

/// The configured column at `column`, or the first one for an extra column (4.7, A-E4-16).
pub fn create_column(board: &Board, column: usize) -> String {
    let cols = &board.meta.columns;
    cols.get(column)
        .or(cols.first())
        .cloned()
        .unwrap_or_default()
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
            // A Detail, shown or behind Help, keeps its note clamped, or leaves when its card is gone.
            let detail = match &mut app.mode {
                Mode::Help { back } => &mut **back,
                m => m,
            };
            if let Mode::Detail { card, note, .. } = detail {
                match app.board.card(*card) {
                    Some(c) => *note = (*note).min(c.notes.len().saturating_sub(1)),
                    None => app.mode = Mode::Board,
                }
            }
            return Vec::new();
        }
        // Another board, after `LoadBoard`: the start selection (A-27, A-E4-05).
        Input::Reloaded(board) => {
            app.sel = initial_selection(&board);
            app.board = board;
            app.mode = Mode::Board;
            return Vec::new();
        }
        // Selects the card and focuses its column; the mode stays (A-E4-13).
        Input::Done(Outcome::Moved(id) | Outcome::Edited(id)) => {
            if let Some(column) = (0..app.board.column_list().len())
                .find(|&i| app.board.cards_in(i).iter().any(|c| c.id == id))
            {
                app.sel = Selection {
                    column,
                    card: Some(id),
                };
            }
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
    let press = key_press(&event);
    let action = press
        .zip(context(&app.mode))
        .and_then(|(p, ctx)| app.keys.action(ctx, &p));
    match (&app.mode, action) {
        (Mode::Board, Some(Action::Quit)) => vec![Effect::Quit],
        (Mode::Board, Some(Action::New)) => {
            app.mode = Mode::TemplatePick { sel: 0 };
            Vec::new()
        }
        // Action keys first, then a template hotkey (5.2).
        (&Mode::TemplatePick { sel }, a) => {
            let hotkey = match press {
                Some(KeyPress {
                    key: Key::Char(c),
                    ctrl: false,
                    alt: false,
                    ..
                }) if a.is_none() => app.templates.iter().position(|t| t.1 == Some(c)),
                _ => None,
            };
            let chosen = if a == Some(Action::Confirm) {
                Some(sel)
            } else {
                hotkey
            };
            let last = app.templates.len().saturating_sub(1);
            app.mode = match (a, chosen.and_then(|i| app.templates.get(i))) {
                (_, Some((name, _))) => Mode::TitleInput {
                    template: name.clone(),
                    buf: String::new(),
                },
                (Some(Action::Up), _) => Mode::TemplatePick {
                    sel: sel.saturating_sub(1),
                },
                (Some(Action::Down), _) => Mode::TemplatePick {
                    sel: (sel + 1).min(last),
                },
                (Some(Action::Cancel), _) => Mode::Board,
                _ => Mode::TemplatePick { sel },
            };
            Vec::new()
        }
        (Mode::Board, Some(Action::SwitchBoard)) => {
            let sel = app.boards.iter().position(|b| b.0 == app.board.name);
            app.mode = Mode::BoardPick {
                sel: sel.unwrap_or(0),
            };
            Vec::new()
        }
        (&Mode::BoardPick { sel }, a) => {
            let last = app.boards.len().saturating_sub(1);
            let mut effects = Vec::new();
            app.mode = match a {
                Some(Action::Up) => Mode::BoardPick {
                    sel: sel.saturating_sub(1),
                },
                Some(Action::Down) => Mode::BoardPick {
                    sel: (sel + 1).min(last),
                },
                Some(Action::Confirm) => {
                    effects.extend(app.boards.get(sel).map(|b| Effect::LoadBoard(b.0.clone())));
                    Mode::Board
                }
                Some(Action::Cancel) => Mode::Board,
                _ => Mode::BoardPick { sel },
            };
            effects
        }
        // A text field: only `confirm` and `cancel` are actions there (5.2).
        (Mode::TitleInput { template, buf }, a) => match a {
            Some(Action::Confirm) if !buf.trim().is_empty() => {
                let effect = Effect::Create {
                    template: template.clone(),
                    title: buf.trim().to_string(),
                    column: create_column(&app.board, app.sel.column),
                };
                app.mode = Mode::Board;
                vec![effect]
            }
            Some(Action::Cancel) => {
                app.mode = Mode::Board;
                Vec::new()
            }
            _ => {
                if let (None, Mode::TitleInput { buf, .. }) = (a, &mut app.mode) {
                    match press.map(|p| (p.key, p.ctrl || p.alt)) {
                        Some((Key::Backspace, _)) => {
                            buf.pop();
                        }
                        Some((Key::Char(c), false)) => buf.push(c),
                        Some((Key::Space, false)) => buf.push(' '),
                        _ => {}
                    }
                }
                Vec::new()
            }
        },
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
        (Mode::Board, Some(a @ (Action::MoveLeft | Action::MoveRight))) => app
            .selected_card()
            .and_then(|c| {
                Some(Effect::Move(
                    c.id,
                    move_target(&app.board, c, a == Action::MoveRight)?,
                ))
            })
            .into_iter()
            .collect(),
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
        (Mode::Detail { .. }, Some(a @ (Action::ScrollDown | Action::ScrollUp))) => {
            if let Mode::Detail { card, scroll, .. } = &mut app.mode {
                let lines = app
                    .board
                    .card(*card)
                    .map_or(0, |c| body_lines(&c.body).len());
                let last = u16::try_from(lines.saturating_sub(1)).unwrap_or(u16::MAX);
                *scroll = match a {
                    Action::ScrollDown => scroll.saturating_add(SCROLL_STEP).min(last),
                    _ => scroll.saturating_sub(SCROLL_STEP),
                };
            }
            Vec::new()
        }
        (Mode::Detail { .. }, Some(Action::Quit)) => {
            app.mode = Mode::Board;
            Vec::new()
        }
        (Mode::Board, Some(Action::Edit)) => app.sel.card.map(Effect::Edit).into_iter().collect(),
        (Mode::Detail { card, .. }, Some(Action::Edit)) => vec![Effect::Edit(*card)],
        (Mode::Board | Mode::Detail { .. }, Some(Action::Help)) => {
            app.mode = Mode::Help {
                back: Box::new(app.mode.clone()),
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
