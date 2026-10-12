//! `config.toml` (TECHSPEC 5.2) parsed into a `Config`.

use crate::core::env::Env;
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

/// An `Error::Config` at the end of `before`, the text preceding the offending byte.
fn config_error(before: &str, path: &Path, msg: String) -> Error {
    Error::Config {
        path: path.to_path_buf(),
        line: before.matches('\n').count() + 1,
        col: before.rsplit('\n').next().unwrap_or("").chars().count() + 1,
        msg,
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
    let fail = |start: usize, msg: String| config_error(&text[..start], path, msg);
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

/// A loaded config; `created` is the path this process just wrote on first run (FR-66).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    pub config: Config,
    pub warnings: Vec<String>,
    pub created: Option<PathBuf>,
}

/// The 5.2 file with each board at `<data_dir>/<name>`, as `~/...` under `HOME`.
pub fn default_text(env: &Env) -> String {
    let dir = |name: &str| {
        let p = env.default_board_dir(name);
        match p.strip_prefix(&env.home) {
            Ok(rel) => format!("~/{}", rel.display()),
            Err(_) => p.display().to_string(),
        }
    };
    format!(
        r#"default_board = "work"   # must be in [boards]
editor = ""              # empty: $VISUAL, $EDITOR, vi; arguments allowed ("code --wait")

[boards]                 # name = folder; names match [a-z][a-z0-9_-]*
work = "{}"
home = "{}"
personal = "{}"

[keys]                   # action = key or list of keys
up = ["k", "up"]
down = ["j", "down"]
left = ["h", "left"]
right = ["l", "right"]
move_left = "H"
move_right = "L"
open = "o"
jump = "enter"
new = "n"
edit = "e"
switch_board = "b"
scroll_down = "ctrl+d"
scroll_up = "ctrl+u"
help = "?"
quit = ["q", "esc"]
confirm = "enter"
cancel = "esc"
"#,
        dir("work"),
        dir("home"),
        dir("personal")
    )
}

/// Reads `env.config_file()`, writing `default_text` first when it is missing.
pub fn load(env: &Env) -> Result<Loaded> {
    if env.config_file().exists() {
        read(env, None)
    } else {
        write_default(env)
    }
}

/// Writes `default_text` to a synced temp file and hard-links it into place, so the
/// config is never seen half-written and an existing one is never replaced (FR-66).
#[doc(hidden)]
pub fn write_default(env: &Env) -> Result<Loaded> {
    let path = env.config_file();
    let dir = path.parent().unwrap_or(Path::new("/"));
    let tmp = dir.join(format!(".config.toml.bs-tmp-{}", env.pid));
    let io = |e: std::io::Error| Error::Io(format!("{}: {e}", path.display()));
    std::fs::create_dir_all(dir).map_err(io)?;
    let mut f = std::fs::File::create(&tmp).map_err(io)?;
    std::io::Write::write_all(&mut f, default_text(env).as_bytes()).map_err(io)?;
    f.sync_all().map_err(io)?;
    let linked = std::fs::hard_link(&tmp, &path);
    let _ = std::fs::remove_file(&tmp);
    match linked {
        Ok(()) => read(env, Some(path.clone())),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => read(env, None),
        Err(e) => Err(io(e)),
    }
}

fn read(env: &Env, created: Option<PathBuf>) -> Result<Loaded> {
    let path = env.config_file();
    let bytes = std::fs::read(&path).map_err(|e| Error::Io(format!("{}: {e}", path.display())))?;
    let text = String::from_utf8(bytes).map_err(|e| {
        let valid = &e.as_bytes()[..e.utf8_error().valid_up_to()];
        config_error(
            std::str::from_utf8(valid).unwrap_or(""),
            &path,
            "invalid UTF-8".to_string(),
        )
    })?;
    let (config, warnings) = parse(&text, &path, &env.home)?;
    Ok(Loaded {
        config,
        warnings,
        created,
    })
}
