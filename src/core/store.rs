//! Every write to a board (TECHSPEC 4.3, 4.6, 10, 12.1).

use crate::core::env::{Env, encode_file_name};
use crate::core::error::{Error, Result};
use crate::core::{failpoint, log};
use std::fs::{self, File, TryLockError};
use std::io::Write;
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
