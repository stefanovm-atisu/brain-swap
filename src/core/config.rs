//! `config.toml` (TECHSPEC 5.2) parsed into a `Config`.

use crate::core::error::{Error, Result};
use std::path::{Path, PathBuf};
use toml::{Table, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub path: PathBuf,
    pub default_board: String,
    pub editor: String,
    pub boards: Vec<(String, PathBuf)>,
    pub keys: Vec<(String, Vec<String>)>,
}

impl Config {
    /// The board named `name`, ignoring case (6.2).
    pub fn board(&self, name: &str) -> Result<(&str, &Path)> {
        self.boards
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(n, p)| (n.as_str(), p.as_path()))
            .ok_or_else(|| {
                let names: Vec<&str> = self.boards.iter().map(|(n, _)| n.as_str()).collect();
                Error::NotFound(format!("unknown board '{name}' ({})", names.join(", ")))
            })
    }
}

/// Parses `text` read from `path`; a leading `~/` in a board path expands to `home`.
// ponytail: values of the wrong type are skipped here; E1-F3-T2 turns them into config errors.
pub fn parse(text: &str, path: &Path, home: &Path) -> Result<(Config, Vec<String>)> {
    let t: Table = text.parse().map_err(|e: toml::de::Error| {
        let before = &text[..e.span().map_or(0, |s| s.start)];
        Error::Config {
            path: path.to_path_buf(),
            line: before.matches('\n').count() + 1,
            col: before.rsplit('\n').next().unwrap_or("").chars().count() + 1,
            msg: e.message().to_string(),
        }
    })?;
    let s = |k: &str, d: &str| t.get(k).and_then(Value::as_str).unwrap_or(d).to_string();
    let table = |k: &str| {
        t.get(k)
            .and_then(Value::as_table)
            .cloned()
            .unwrap_or_default()
    };
    let boards = table("boards")
        .into_iter()
        .filter_map(|(n, v)| {
            let p = v.as_str()?;
            Some((n, p.strip_prefix("~/").map_or(p.into(), |r| home.join(r))))
        })
        .collect();
    let keys = table("keys")
        .into_iter()
        .map(|(a, v)| {
            let ks = match v {
                Value::Array(a) => a
                    .iter()
                    .filter_map(|k| k.as_str().map(String::from))
                    .collect(),
                v => v.as_str().map(String::from).into_iter().collect(),
            };
            (a, ks)
        })
        .collect();
    let config = Config {
        path: path.to_path_buf(),
        default_board: s("default_board", "work"),
        editor: s("editor", ""),
        boards,
        keys,
    };
    Ok((config, Vec::new()))
}
