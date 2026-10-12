//! In-memory boards for tests of pure functions over `Board` (board `work`, letter `W`).

use std::path::PathBuf;

use brain_swap::core::model::{Board, BoardMeta, Card, CardId, HerdrPlace, Note, Place};
use brain_swap::core::time::{Stamp, parse_stamp};
use jiff::tz::TimeZone;

/// A full stamp, or a bare `HH:MM` meaning that time on 2026-10-08 at `+03:00`.
fn stamp(at: &str) -> Stamp {
    let full = if at.len() == 5 {
        format!("2026-10-08T{at}:00+03:00")
    } else {
        at.to_string()
    };
    parse_stamp(&full, &TimeZone::UTC).unwrap()
}

pub fn mem_board(columns: &[&str], cards: Vec<Card>) -> Board {
    Board {
        name: "work".to_string(),
        path: PathBuf::from("work"),
        meta: BoardMeta {
            letter: 'W',
            columns: columns.iter().map(|c| c.to_string()).collect(),
            next: cards.iter().map(|c| c.id.number).max().unwrap_or(0) + 1,
        },
        cards,
        warnings: Vec::new(),
    }
}

pub fn mem_card(id: &str, column: &str, created: Option<&str>, notes: Vec<Note>) -> Card {
    Card {
        id: CardId::parse(id).unwrap(),
        path: PathBuf::from(format!("work/{id}.md")),
        title: id.to_string(),
        column: column.to_string(),
        template: None,
        created: created.map(stamp),
        body: String::new(),
        notes,
        writable: true,
        mtime: None,
    }
}

pub fn mem_note(at: &str, pane: Option<&str>, cwd: Option<&str>) -> Note {
    Note {
        heading: format!("### {at}"),
        at: Some(stamp(at)),
        auto: false,
        doing: "something".to_string(),
        next: String::new(),
        watch_out: String::new(),
        place: Place {
            cwd: cwd.map(PathBuf::from),
            herdr: pane.map(|p| HerdrPlace {
                pane: p.to_string(),
                tab: None,
                workspace: None,
            }),
            session: None,
        },
        extra: Vec::new(),
    }
}
