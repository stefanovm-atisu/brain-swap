//! Best guess: the order of a board's open cards (TECHSPEC 6.6, FR-38).

use std::cmp::Reverse;
use std::path::{Path, PathBuf};

use crate::core::model::{Board, Card, CardId};

/// A candidate's tier, best first (6.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    LinkedHere,
    SamePane,
    SameFolder,
    MiddleColumn,
    Other,
}

impl Tier {
    /// The JSON `reason` (6.6, 6.7).
    pub fn reason(&self) -> &'static str {
        match self {
            Tier::LinkedHere => "linked_here",
            Tier::SamePane => "same_pane",
            Tier::SameFolder => "same_folder",
            Tier::MiddleColumn => "middle_column",
            Tier::Other => "other",
        }
    }

    /// The text label; tiers 3 and 4 have none (6.6).
    pub fn label(&self) -> Option<&'static str> {
        match self {
            Tier::LinkedHere => Some("linked here"),
            Tier::SamePane => Some("same pane"),
            Tier::SameFolder => Some("same folder"),
            Tier::MiddleColumn | Tier::Other => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate<'a> {
    pub card: &'a Card,
    pub tier: Tier,
}

/// What the guess knows about where it runs: pane only inside herdr, cwd, `$HOME`, pane hint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuessInput {
    pub pane: Option<String>,
    pub cwd: PathBuf,
    pub home: PathBuf,
    pub hint: Option<CardId>,
}

/// The open cards of `board` in 6.6 order; pure: no `Env`, no I/O, no clock.
pub fn best_guess<'a>(board: &'a Board, input: &GuessInput) -> Vec<Candidate<'a>> {
    let cols = &board.meta.columns;
    let middle = cols
        .get(1..cols.len().saturating_sub(1))
        .unwrap_or_default();
    let mut v: Vec<Candidate> = board
        .open_cards()
        .into_iter()
        .map(|card| {
            let pane = card.notes.iter().rev().find_map(|n| n.place.herdr.as_ref());
            let cwd = card.notes.iter().rev().find_map(|n| n.place.cwd.as_deref());
            let tier = if pane.is_some_and(|h| input.pane.as_deref() == Some(h.pane.as_str())) {
                Tier::SamePane
            } else if cwd.is_some_and(|c| same_folder(c, input)) {
                Tier::SameFolder
            } else if middle.contains(&card.column) {
                Tier::MiddleColumn
            } else {
                Tier::Other
            };
            Candidate { card, tier }
        })
        .collect();
    // No activity is `None`, which `Reverse` sorts after every instant (A-E1-43).
    v.sort_by_key(|c| {
        let k = (c.card.activity().map(|s| s.timestamp()), c.card.id.number);
        (c.tier, Reverse(k))
    });
    v
}

/// 6.6 tier 2, lexical and component-wise (`Path::starts_with`); a `/` or `$HOME` note cwd never contains the current one.
fn same_folder(c: &Path, input: &GuessInput) -> bool {
    c.starts_with(&input.cwd)
        || (input.cwd.starts_with(c) && c != Path::new("/") && c != input.home)
}
