//! The external editor on a copy of the card, merged back through `core::store` (TECHSPEC 7.6).

use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::process::ExitStatus;

use super::app::Outcome;
use super::runtime::TerminalGuard;
use crate::core::env::Env;
use crate::core::model::{Board, CardId};
use crate::core::store::merge_edit;

pub trait Launcher {
    fn launch(&mut self, argv: &[String]) -> io::Result<ExitStatus>;
}

/// `<edit_dir>/<board>-<ID>.md`.
pub fn copy_path(env: &Env, board: &str, id: CardId) -> PathBuf {
    env.edit_dir().join(format!("{board}-{id}.md"))
}

/// Copies `C0`, runs the editor on the copy between `leave` and `enter`, merges a zero-exit save.
/// Holds no lock: only `merge_edit` takes one (TECHSPEC 10).
pub fn edit_card(
    env: &Env,
    board: &Board,
    id: CardId,
    config_editor: &str,
    launcher: &mut dyn Launcher,
    guard: &mut dyn TerminalGuard,
) -> Outcome {
    edit(env, board, id, config_editor, launcher, guard)
        .unwrap_or_else(|e| Outcome::Failed(e.to_string()))
}

fn edit(
    env: &Env,
    board: &Board,
    id: CardId,
    config_editor: &str,
    launcher: &mut dyn Launcher,
    guard: &mut dyn TerminalGuard,
) -> io::Result<Outcome> {
    let c0 = match fs::read(board.path.join(id.file_name())) {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Ok(Outcome::Failed(format!("{id} not found")));
        }
        r => r?,
    };
    let copy = copy_path(env, &board.name, id);
    fs::create_dir_all(env.edit_dir())?;
    fs::write(&copy, &c0)?;
    // ponytail: E4-F5-T2 replaces this with `editor_argv` ($VISUAL, $EDITOR).
    let mut argv: Vec<String> = config_editor
        .split_ascii_whitespace()
        .map(String::from)
        .collect();
    if argv.is_empty() {
        argv.push("vi".into());
    }
    argv.push(copy.to_string_lossy().into_owned());
    guard.leave()?;
    let status = launcher.launch(&argv);
    guard.enter()?;
    let status = status?;
    if !status.success() {
        return Ok(Outcome::Failed(format!("editor: {status}")));
    }
    let edited = fs::read(&copy)?;
    Ok(match merge_edit(env, board, id, &c0, &edited) {
        Ok(_) => {
            fs::remove_file(&copy)?;
            Outcome::Edited(id)
        }
        Err(e) => Outcome::Failed(e.to_string()),
    })
}
