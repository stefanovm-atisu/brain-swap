//! TUI state machine (TECHSPEC 7.1): pure, no I/O.

use ratatui::crossterm::event::KeyEvent;

use crate::core::keys::KeyMap;
use crate::core::model::{Board, Card, CardId};
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
    pub fn new(board: Board, keys: KeyMap, now: Stamp) -> App {
        App {
            mode: Mode::Board,
            sel: initial_selection(&board),
            board,
            keys,
            status: None,
            now,
            warnings: Vec::new(),
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

pub fn update(_app: &mut App, _input: Input) -> Vec<Effect> {
    Vec::new()
}
