//! The `BRAIN_SWAP_LOG` sink (TECHSPEC 11, FR-05).

use crate::core::env::Env;
use crate::core::time::format_stamp;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

/// Appends `<now> <kind> <text>` to `env.log` in one write, so concurrent writers do not splice
/// lines; errors are ignored so logging never fails a command.
pub fn event(env: &Env, kind: &str, text: &str) {
    let Some(path) = &env.log else { return };
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
    {
        let line = format!("{} {kind} {text}\n", format_stamp(&env.now));
        let _ = f.write_all(line.as_bytes());
    }
}

/// Drops a log path that lies inside a board folder and returns the warning (FR-05).
pub fn guard_board_folders(env: &mut Env, boards: &[(String, PathBuf)]) -> Option<String> {
    let log = lexical(&env.cwd.join(env.log.as_ref()?));
    let (name, _) = boards
        .iter()
        .find(|(_, dir)| log.starts_with(lexical(dir)))?;
    env.log = None;
    Some(format!("BRAIN_SWAP_LOG inside board {name}, ignored"))
}

/// Drops `.` and resolves `..` against the preceding component, without touching the file system.
fn lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            c => out.push(c),
        }
    }
    out
}
