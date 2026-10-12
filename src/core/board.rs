//! Board loading (TECHSPEC 3, 4.1).

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::core::board_file;
use crate::core::card_file;
use crate::core::env::Env;
use crate::core::error::{Error, Result};
use crate::core::model::{Board, BoardMeta, CardId};

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
