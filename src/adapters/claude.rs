//! The Claude Code pack embedded in the binary and its install (TECHSPEC 8.8).

use crate::core::env::Env;
use crate::core::error::Error;
use crate::core::store;
use std::fs;
use std::path::PathBuf;

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

/// Writes each embedded skill as a plain folder `<skills>/<name>/SKILL.md` through a temp
/// file renamed into place (SP-6 fallback: copies, not symlinks); edits no settings.
pub fn install(env: &Env, _opts: &InstallOpts) -> Result<InstallReport, Error> {
    let dir = skills_dir(env);
    let io = |e: std::io::Error| Error::Io(format!("{}: {e}", dir.display()));
    let created = !dir.is_dir();
    let mut report = InstallReport::default();
    for skill in &SKILLS {
        let folder = dir.join(skill.name);
        fs::create_dir_all(&folder).map_err(io)?;
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
