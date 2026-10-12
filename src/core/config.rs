//! `config.toml` (TECHSPEC 5.2) parsed into a `Config`.

use crate::core::error::{Error, Result};
use std::path::{Path, PathBuf};
use toml::Spanned;
use toml::de::{DeString, DeTable, DeValue};

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
///
/// Fails `Error::Config` at the offending key (or the parser's span) with one of:
/// - the TOML parser's own message for a syntax error;
/// - `default_board '<x>' is not in [boards]`;
/// - `[boards] is empty` (at the `[boards]` header, or 1:1 when it is missing);
/// - `board name '<n>' must match [a-z][a-z0-9_-]*`;
/// - `<key>: expected <type>` for a known key of the wrong type;
/// - `board <name>: path must be absolute or start with ~/`.
///
/// Warns `config: unknown key '<key>'` for an unknown top-level key.
pub fn parse(text: &str, path: &Path, home: &Path) -> Result<(Config, Vec<String>)> {
    let fail = |start: usize, msg: String| {
        let before = &text[..start];
        Error::Config {
            path: path.to_path_buf(),
            line: before.matches('\n').count() + 1,
            col: before.rsplit('\n').next().unwrap_or("").chars().count() + 1,
            msg,
        }
    };
    let doc = DeTable::parse(text)
        .map_err(|e| fail(e.span().map_or(0, |s| s.start), e.message().to_string()))?
        .into_inner();
    let expected = |k: &Spanned<DeString>, ty: &str| {
        fail(k.span().start, format!("{}: expected {ty}", k.get_ref()))
    };
    let str_of = |k: &Spanned<DeString>, v: &Spanned<DeValue>| {
        v.get_ref()
            .as_str()
            .map(String::from)
            .ok_or_else(|| expected(k, "string"))
    };
    let mut warnings = Vec::new();
    let (mut default_board, mut editor) = (None, String::new());
    let (mut boards, mut keys, mut boards_at) = (Vec::new(), Vec::new(), 0);
    for (k, v) in &doc {
        match k.get_ref().as_ref() {
            "default_board" => default_board = Some((k.span().start, str_of(k, v)?)),
            "editor" => editor = str_of(k, v)?,
            "boards" => {
                boards_at = k.span().start;
                for (n, p) in v.get_ref().as_table().ok_or_else(|| expected(k, "table"))? {
                    let name = n.get_ref();
                    let mut cs = name.chars();
                    if !(cs.next().is_some_and(|c| c.is_ascii_lowercase())
                        && cs.all(|c| {
                            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-'
                        }))
                    {
                        return Err(fail(
                            n.span().start,
                            format!("board name '{name}' must match [a-z][a-z0-9_-]*"),
                        ));
                    }
                    let p = str_of(n, p)?;
                    let p = match p.strip_prefix("~/") {
                        Some(r) => home.join(r),
                        None if Path::new(&p).is_absolute() => p.into(),
                        None => {
                            return Err(fail(
                                n.span().start,
                                format!("board {name}: path must be absolute or start with ~/"),
                            ));
                        }
                    };
                    boards.push((name.to_string(), p));
                }
            }
            "keys" => {
                for (a, v) in v.get_ref().as_table().ok_or_else(|| expected(k, "table"))? {
                    let ks = match v.get_ref() {
                        DeValue::String(s) => Some(vec![s.to_string()]),
                        DeValue::Array(xs) => xs
                            .iter()
                            .map(|x| x.get_ref().as_str().map(String::from))
                            .collect(),
                        _ => None,
                    };
                    keys.push((
                        a.get_ref().to_string(),
                        ks.ok_or_else(|| expected(a, "string or list of strings"))?,
                    ));
                }
            }
            other => warnings.push(format!("config: unknown key '{other}'")),
        }
    }
    if boards.is_empty() {
        return Err(fail(boards_at, "[boards] is empty".to_string()));
    }
    let (at, default_board) = default_board.unwrap_or((0, "work".to_string()));
    if !boards.iter().any(|(n, _)| *n == default_board) {
        return Err(fail(
            at,
            format!("default_board '{default_board}' is not in [boards]"),
        ));
    }
    let config = Config {
        path: path.to_path_buf(),
        default_board,
        editor,
        boards,
        keys,
    };
    Ok((config, warnings))
}
