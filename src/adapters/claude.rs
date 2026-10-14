//! The Claude Code pack embedded in the binary and its install (TECHSPEC 8.8).

use crate::core::env::Env;
use crate::core::error::Error;
use crate::core::store;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Skill {
    pub name: &'static str,
    pub body: &'static str,
}

macro_rules! skill {
    ($name:literal) => {
        Skill {
            name: $name,
            body: include_str!(concat!("../../pack/claude/skills/", $name, "/SKILL.md")),
        }
    };
}

pub const SKILLS: [Skill; 4] = [
    skill!("bs-card"),
    skill!("bs-park"),
    skill!("bs-back"),
    skill!("bs-link"),
];

/// `brain-swap install claude` options (8.8).
#[derive(Debug, Default)]
pub struct InstallOpts {
    pub link: Option<PathBuf>,
    pub force: bool,
    pub exe: Option<PathBuf>,
}

/// What `install` or `remove` did: `notices` are the 8.8 hints, printed after the paths.
#[derive(Debug, Default)]
pub struct InstallReport {
    pub linked: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
    pub notices: Vec<String>,
    pub warnings: Vec<String>,
}

/// Claude Code's skills folder: `$HOME/.claude/skills`.
pub fn skills_dir(env: &Env) -> PathBuf {
    env.home.join(".claude/skills")
}

/// True when the frontmatter of `skill_md` holds `metadata:` with an indented
/// `pack: brain-swap` (8.8 ours rule).
pub fn is_ours(skill_md: &str) -> bool {
    let mut lines = skill_md.lines();
    if lines.next() != Some("---") {
        return false;
    }
    let mut in_metadata = false;
    for line in lines.take_while(|l| *l != "---") {
        if !line.starts_with([' ', '\t']) {
            in_metadata = line.trim_end() == "metadata:";
        } else if in_metadata && line.trim() == "pack: brain-swap" {
            return true;
        }
    }
    false
}

/// Writes each embedded skill as a plain folder `<skills>/<name>/SKILL.md` through a temp
/// file renamed into place (SP-6 fallback: copies, not symlinks); edits no settings.
pub fn install(env: &Env, opts: &InstallOpts) -> Result<InstallReport, Error> {
    let dir = skills_dir(env);
    let created = !dir.is_dir();
    let folders: Vec<PathBuf> = SKILLS.iter().map(|s| dir.join(s.name)).collect();
    let mut report = InstallReport {
        removed: clear_foreign(&folders, opts.force)?,
        ..Default::default()
    };
    for skill in &SKILLS {
        let folder = dir.join(skill.name);
        // No symlink is left at `folder` here, so this cannot write through one.
        fs::create_dir_all(&folder).map_err(|e| io(&folder, e))?;
        let body = skill.body.as_bytes();
        store::write_atomic(env, &folder.join("SKILL.md"), body, &|_| true, None)?;
        report.linked.push(folder);
    }
    if created {
        report
            .notices
            .push("run /reload-skills in sessions already open".into());
    }
    report.notices.push(
        "tip: allow Bash(brain-swap:*) in your Claude Code settings so moves need no prompt".into(),
    );
    Ok(report)
}

fn io(path: &Path, e: std::io::Error) -> Error {
    Error::Io(format!("{}: {e}", path.display()))
}

/// The 8.8 foreign check: refuses the first of `folders` that exists and is not ours (a
/// folder without our `SKILL.md`, a file, any symlink) unless `force`, which deletes them
/// all and returns them; on refusal nothing is touched.
fn clear_foreign(folders: &[PathBuf], force: bool) -> Result<Vec<PathBuf>, Error> {
    let ours = |f: &Path| match fs::symlink_metadata(f) {
        Err(_) => true,
        Ok(m) => m.is_dir() && fs::read_to_string(f.join("SKILL.md")).is_ok_and(|t| is_ours(&t)),
    };
    let foreign: Vec<PathBuf> = folders.iter().filter(|f| !ours(f)).cloned().collect();
    match foreign.first() {
        Some(f) if !force => Err(Error::InvalidInput(format!(
            "{} exists and was not created by brain-swap (use --force)",
            f.display()
        ))),
        _ => {
            for f in &foreign {
                let is_dir = fs::symlink_metadata(f).is_ok_and(|m| m.is_dir());
                match is_dir {
                    true => fs::remove_dir_all(f),
                    false => fs::remove_file(f),
                }
                .map_err(|e| io(f, e))?;
            }
            Ok(foreign)
        }
    }
}
