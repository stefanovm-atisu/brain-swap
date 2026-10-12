//! Board loading (TECHSPEC 3, 4.1).

use std::cmp::Reverse;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::core::board_file;
use crate::core::card_file;
use crate::core::env::Env;
use crate::core::error::{Error, Result};
use crate::core::model::{Board, BoardMeta, Card, CardId};

/// Reads the board folder `path`; creates and locks nothing (FR-04).
pub fn load(name: &str, path: &Path, env: &Env) -> Result<Board> {
    let mut warnings = Vec::new();
    let md = match fs::read(path.join("board.md")) {
        Err(e) if e.kind() != ErrorKind::NotFound => {
            warnings.push(format!("board.md: {e}"));
            None
        }
        r => r.ok(),
    };
    let bf = board_file::parse(name, md.as_deref());
    warnings.extend(bf.warnings);
    let mut names = match fs::read_dir(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(Error::Io(format!("board {name}: {e}"))),
        Ok(rd) => rd
            .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
            .collect::<std::io::Result<Vec<_>>>()
            .map_err(|e| Error::Io(format!("board {name}: {e}")))?,
    };
    names.sort();
    let mut cards = Vec::new();
    for file in names {
        let p = path.join(&file);
        let Some(id) = CardId::from_file_name(&file).filter(|_| p.is_file()) else {
            continue;
        };
        if id.letter != bf.letter {
            warnings.push(format!(
                "{file}: letter {} is not this board's {}, ignored",
                id.letter, bf.letter
            ));
            continue;
        }
        let bytes = match fs::read(&p) {
            Ok(b) => b,
            Err(e) => {
                warnings.push(format!("{file}: {e}"));
                continue;
            }
        };
        let mut parsed = card_file::parse(id, &p, &bytes, &bf.columns[0], &env.tz);
        parsed.card.mtime = fs::metadata(&p)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| jiff::Timestamp::try_from(t).ok())
            .map(|t| t.to_zoned(env.tz.clone()));
        warnings.extend(parsed.warnings);
        cards.push(parsed.card);
    }
    // Sorted by number so extra columns follow first appearance by card number (A-E1-30).
    cards.sort_by_key(|c| c.id);
    for c in &mut cards {
        let lower = c.column.to_lowercase();
        match bf.columns.iter().find(|k| k.to_lowercase() == lower) {
            Some(k) => c.column = k.clone(),
            None => warnings.push(format!("{}: unknown column '{}'", c.id, c.column)),
        }
    }
    let highest = cards.iter().map(|c| c.id.number).max().unwrap_or(0);
    Ok(Board {
        name: name.to_string(),
        path: path.to_path_buf(),
        meta: BoardMeta {
            letter: bf.letter,
            columns: bf.columns,
            next: bf.next.unwrap_or(1).max(highest.saturating_add(1)),
        },
        cards,
        warnings,
    })
}

/// The letter of board `name` at `path` and the warnings, reading only `board.md`.
pub fn board_letter(name: &str, path: &Path) -> (char, Vec<String>) {
    // An unreadable board.md reads as absent here; `load` warns about it if the board is chosen.
    let bf = board_file::parse(name, fs::read(path.join("board.md")).ok().as_deref());
    (bf.letter, bf.warnings)
}

/// Loads the first board whose letter is `id.letter` and checks it holds `id` (FR-10, A-E1-31).
pub fn find_card(boards: &[(String, PathBuf)], id: CardId, env: &Env) -> Result<Board> {
    let not_found = || Error::NotFound(format!("{id} not found"));
    let mut hits = boards
        .iter()
        .filter(|(n, p)| board_letter(n, p).0 == id.letter);
    let (name, path) = hits.next().ok_or_else(not_found)?;
    let mut b = load(name, path, env)?;
    for (other, _) in hits {
        b.warnings.push(format!(
            "boards {name} and {other} share letter {}; using {name}",
            id.letter
        ));
    }
    if b.card(id).is_none() {
        return Err(not_found());
    }
    Ok(b)
}

/// A board column; `unknown` marks an extra column holding unmatched cards (I5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub unknown: bool,
}

impl Board {
    /// Configured column names, then the unmatched ones in card number order (I5).
    fn names(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.meta.columns.iter().map(String::as_str).collect();
        for c in &self.cards {
            if !v.contains(&c.column.as_str()) {
                v.push(&c.column);
            }
        }
        v
    }

    pub fn column_list(&self) -> Vec<Column> {
        let n = self.meta.columns.len();
        self.names()
            .into_iter()
            .enumerate()
            .map(|(i, name)| Column {
                name: if i < n {
                    name.to_string()
                } else {
                    format!("{name} (unknown)")
                },
                unknown: i >= n,
            })
            .collect()
    }

    /// The cards of column `column_index`, newest activity first, ties to the higher number (FR-09).
    pub fn cards_in(&self, column_index: usize) -> Vec<&Card> {
        let name = self.names()[column_index];
        let mut v: Vec<&Card> = self.cards.iter().filter(|c| c.column == name).collect();
        v.sort_by_key(|c| Reverse((c.activity().map(|s| s.timestamp()), c.id.number)));
        v
    }

    /// Not in the last configured column; extra columns are open (FR-08, I5).
    pub fn is_open(&self, card: &Card) -> bool {
        self.meta.columns.last() != Some(&card.column)
    }

    pub fn open_cards(&self) -> Vec<&Card> {
        self.cards.iter().filter(|c| self.is_open(c)).collect()
    }

    pub fn card(&self, id: CardId) -> Option<&Card> {
        self.cards.iter().find(|c| c.id == id)
    }

    /// The configured column matching `name` ignoring case; extra columns never match (6.2, 4.7).
    pub fn column(&self, name: &str) -> Result<&str> {
        let lower = name.to_lowercase();
        self.meta
            .columns
            .iter()
            .find(|c| c.to_lowercase() == lower)
            .map(String::as_str)
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "unknown column '{name}' on {} ({})",
                    self.name,
                    self.meta.columns.join(", ")
                ))
            })
    }
}
