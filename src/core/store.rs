//! Every write to a board (TECHSPEC 4.3, 4.6, 10, 12.1).

use crate::core::env::{Env, encode_file_name};
use crate::core::error::{Error, Result};
use crate::core::model::{Board, Card, CardId, NewCard, NewNote};
use crate::core::template::Template;
use crate::core::{board_file, card_file, failpoint, log};
use jiff::tz::TimeZone;
use std::fs::{self, File, TryLockError};
use std::io::{ErrorKind, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// The exclusive per-board lock; dropping it closes the file and releases the lock.
pub struct BoardLock {
    _file: File,
}

fn io(e: std::io::Error) -> Error {
    Error::Io(e.to_string())
}

/// `<locks>/<encoded canonical board_dir>.lock`; creates nothing.
pub fn lock_path(env: &Env, board_dir: &Path) -> Result<PathBuf> {
    let canonical = board_dir.canonicalize().map_err(io)?;
    let name = encode_file_name(&canonical.to_string_lossy());
    Ok(env.locks_dir().join(format!("{name}.lock")))
}

/// Creates `board_dir`, then takes its lock within 2 seconds or fails `Busy` (10).
pub fn lock_board(env: &Env, board_dir: &Path) -> Result<BoardLock> {
    fs::create_dir_all(board_dir).map_err(io)?;
    let path = lock_path(env, board_dir)?;
    fs::create_dir_all(env.locks_dir()).map_err(io)?;
    let file = File::options()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .map_err(io)?;
    let start = Instant::now();
    loop {
        match file.try_lock() {
            Ok(()) => break,
            Err(TryLockError::Error(e)) => return Err(io(e)),
            Err(TryLockError::WouldBlock) if start.elapsed() >= Duration::from_secs(2) => {
                return Err(Error::Busy("board busy, try again".to_string()));
            }
            Err(TryLockError::WouldBlock) => sleep(Duration::from_millis(10)),
        }
    }
    let ms = start.elapsed().as_millis();
    if ms > 100 {
        let canonical = board_dir.canonicalize().unwrap_or_default();
        log::event(env, "lock_wait", &format!("{} {ms}", canonical.display()));
    }
    Ok(BoardLock { _file: file })
}

/// `.<file name>.bs-tmp-<pid>` beside `target`.
pub fn temp_path(target: &Path, pid: u32) -> PathBuf {
    let name = target.file_name().unwrap_or_default().to_string_lossy();
    target.with_file_name(format!(".{name}.bs-tmp-{pid}"))
}

/// Verifies `bytes`, then writes them to a temp file and renames it over `target` (R7, TECHSPEC 10).
/// Errors remove the temp file, except a failpoint, which leaves it as a crash would.
pub fn write_atomic(
    env: &Env,
    target: &Path,
    bytes: &[u8],
    verify: &dyn Fn(&[u8]) -> bool,
    failpoint: Option<&str>,
) -> Result<()> {
    if !verify(bytes) {
        log::event(env, "verify_failed", &target.display().to_string());
        let name = target.file_name().unwrap_or_default().to_string_lossy();
        return Err(Error::Verify(format!("{name}: verify failed")));
    }
    let tmp = temp_path(target, env.pid);
    let io = |e: std::io::Error| Error::Io(format!("{}: {e}", target.display()));
    let written = File::create(&tmp).and_then(|mut f| {
        f.write_all(bytes)?;
        f.sync_all()
    });
    if let Err(e) = written {
        let _ = fs::remove_file(&tmp);
        return Err(io(e));
    }
    if let Some(name) = failpoint {
        failpoint::check(env, name)?;
    }
    if let Err(e) = fs::rename(&tmp, target) {
        let _ = fs::remove_file(&tmp);
        return Err(io(e));
    }
    let dir = target.parent().unwrap_or(Path::new("/"));
    File::open(dir).and_then(|d| d.sync_all()).map_err(io)
}

// Create (TECHSPEC 4.6 steps 1 to 3, 4.3, R4, R7).

/// Maps a `hard_link` error other than `AlreadyExists` to `Error::Io` for board `board` (4.6, T-13).
pub fn link_error(board: &str, e: &std::io::Error) -> Error {
    match e.kind() {
        ErrorKind::Unsupported | ErrorKind::PermissionDenied => {
            Error::Io(format!("board {board}: hard links not supported"))
        }
        _ => Error::Io(format!("board {board}: {e}")),
    }
}

/// Link attempts before `create` gives up on finding a free number (4.6, A-E1-33).
pub const MAX_LINK_ATTEMPTS: u32 = 100;

/// What `create` writes: the column defaults to the board's first (I3), the body to the skeleton.
pub struct CreateRequest<'a> {
    pub title: &'a str,
    pub template: &'a Template,
    pub column: Option<&'a str>,
    pub body: Option<&'a str>,
}

#[derive(Debug)]
pub struct Created {
    pub card: Card,
    pub warnings: Vec<String>,
}

/// Allocates `<L>-<n>.md` inside the board lock and links the rendered card there (4.6).
pub fn create(env: &Env, board: &Board, req: &CreateRequest) -> Result<Created> {
    let title = req.title.trim();
    if title.is_empty() {
        return Err(Error::InvalidInput("no title".to_string()));
    }
    let column = match req.column {
        Some(c) => board.column(c)?,
        None => &board.meta.columns[0],
    };
    let _lock = lock_board(env, &board.path)?;
    let md_path = board.path.join("board.md");
    let md = match fs::read(&md_path) {
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        r => Some(r.map_err(io)?),
    };
    let bf = board_file::parse(&board.name, md.as_deref());
    if !bf.writable {
        return Err(Error::Unreadable(bf.warnings.concat()));
    }
    let mut highest = 0;
    for e in fs::read_dir(&board.path).map_err(io)? {
        let e = e.map_err(io)?;
        if let Some(id) = CardId::from_file_name(&e.file_name().to_string_lossy())
            && id.letter == bf.letter
            && e.path().is_file()
        {
            highest = highest.max(id.number);
        }
    }
    let mut n = bf.next.unwrap_or(1).max(highest + 1);
    for _ in 0..MAX_LINK_ATTEMPTS {
        let next = board_file::set_next(md.as_deref(), n + 1)?;
        let verify = |b: &[u8]| board_file::parse(&board.name, Some(b)).next == Some(n + 1);
        write_atomic(env, &md_path, &next, &verify, None)?;
        failpoint::check(env, "create:after_next")?;

        let id = CardId {
            letter: bf.letter,
            number: n,
        };
        let target = board.path.join(id.file_name());
        let new = NewCard {
            title: title.to_string(),
            template: req.template.name.clone(),
            column: column.to_string(),
            created: env.now.clone(),
            body: req.body.unwrap_or(&req.template.skeleton).to_string(),
        };
        let (bytes, warnings) = card_file::render_new(&new);
        let card = card_file::parse(id, &target, &bytes, column, &env.tz).card;
        if (
            &card.title,
            &card.column,
            card.template.as_ref(),
            card.created.as_ref(),
        ) != (
            &new.title,
            &new.column,
            Some(&new.template),
            Some(&new.created),
        ) {
            log::event(env, "verify_failed", &target.display().to_string());
            return Err(Error::Verify(format!("{id}.md: verify failed")));
        }
        let tmp = temp_path(&target, env.pid);
        File::create(&tmp)
            .and_then(|mut f| {
                f.write_all(&bytes)?;
                f.sync_all()
            })
            .map_err(io)?;
        failpoint::check(env, "create:after_tmp")?;
        // Linking never overwrites: a name taken meanwhile fails with AlreadyExists (4.6).
        if let Err(e) = fs::hard_link(&tmp, &target) {
            let _ = fs::remove_file(&tmp);
            if e.kind() == ErrorKind::AlreadyExists {
                n += 1;
                continue;
            }
            return Err(link_error(&board.name, &e));
        }
        failpoint::check(env, "create:after_link")?;
        fs::remove_file(&tmp).map_err(io)?;
        return Ok(Created { card, warnings });
    }
    Err(Error::Io(format!(
        "board {}: no free card number after {MAX_LINK_ATTEMPTS} tries",
        board.name
    )))
}

/// The card after a move; `changed` is false when it already sat in the column.
#[derive(Debug)]
pub struct Moved {
    pub card: Card,
    pub changed: bool,
}

/// Moves card `id` to `column`, re-reading its file inside the lock (6.2, 10, R2, R7).
pub fn move_card(env: &Env, board: &Board, id: CardId, column: &str) -> Result<Moved> {
    let column = board.column(column)?;
    let _lock = lock_board(env, &board.path)?;
    let name = id.file_name();
    let path = board.path.join(&name);
    let before = match fs::read(&path) {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Err(Error::NotFound(format!("{id} not found")));
        }
        r => r.map_err(io)?,
    };
    card_file::check_writable(&name, &before)?;
    let parse = |b: &[u8]| card_file::parse(id, &path, b, &board.meta.columns[0], &env.tz).card;
    let card = parse(&before);
    if card.column.to_lowercase() == column.to_lowercase() {
        return Ok(Moved {
            card,
            changed: false,
        });
    }
    let after = card_file::set_column(&name, &before, column)?;
    write_atomic(
        env,
        &path,
        &after,
        &|a| verify_move(&before, a, column),
        None,
    )?;
    Ok(Moved {
        card: parse(&after),
        changed: true,
    })
}

/// R7 for a move: `after` parses to `column` and differs from `before` only as R2 allows.
pub fn verify_move(before: &[u8], after: &[u8], column: &str) -> bool {
    let id = CardId {
        letter: 'X',
        number: 1,
    };
    let parsed = card_file::parse(id, Path::new(""), after, "", &TimeZone::UTC).card;
    let b: Vec<&[u8]> = before.split_inclusive(|&c| c == b'\n').collect();
    let a: Vec<&[u8]> = after.split_inclusive(|&c| c == b'\n').collect();
    let pre = b.iter().zip(&a).take_while(|(x, y)| x == y).count();
    let suf = b[pre..]
        .iter()
        .rev()
        .zip(a[pre..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (removed, added) = (b.len() - pre - suf, a.len() - pre - suf);
    // R2: one line changed, added or already right, or a three-line frontmatter put in front.
    let r2 = (removed <= added && added <= 1) || (pre == 0 && removed == 0 && added == 3);
    r2 && parsed.column == column
}

// Park (FR-16, R3, R7).

/// A parked card re-read from the written bytes, and the splice warnings.
#[derive(Debug)]
pub struct Parked {
    pub card: Card,
    pub warnings: Vec<String>,
}

/// R7 for park: `after` holds exactly one more note and equals `before` outside `inserted`.
pub fn verify_park(before: &[u8], after: &[u8], inserted: Range<usize>) -> bool {
    let notes = |b: &[u8]| card_file::note_blocks(&String::from_utf8_lossy(b)).len();
    let (s, e) = (inserted.start, inserted.end);
    after.len() == before.len() + inserted.len()
        && after.get(..s) == before.get(..s)
        && after.get(e..) == before.get(s..)
        && notes(after) == notes(before) + 1
}

/// Appends `note` to card `id` under the board lock, re-reading the file inside it (FR-16).
pub fn park(env: &Env, board: &Board, id: CardId, note: &NewNote) -> Result<Parked> {
    if [&note.doing, &note.next, &note.watch_out]
        .iter()
        .all(|p| p.trim().is_empty())
    {
        return Err(Error::InvalidInput("empty note".to_string()));
    }
    let _lock = lock_board(env, &board.path)?;
    let path = board.path.join(id.file_name());
    let before = fs::read(&path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => Error::NotFound(format!("{id} not found")),
        _ => Error::Io(format!("{}: {e}", path.display())),
    })?;
    let a = card_file::append_note(&id.file_name(), &before, note)?;
    let verify = |after: &[u8]| verify_park(&before, after, a.inserted.clone());
    write_atomic(env, &path, &a.bytes, &verify, Some("park:after_tmp"))?;
    let card = card_file::parse(id, &path, &a.bytes, &board.meta.columns[0], &env.tz).card;
    Ok(Parked {
        card,
        warnings: a.warnings,
    })
}
