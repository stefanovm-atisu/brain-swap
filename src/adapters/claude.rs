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

/// Claude Code's skills folder: `$CLAUDE_CONFIG_DIR/skills` when set and non-empty
/// (unverified assumption, SP-6), else `$HOME/.claude/skills` (8.8).
pub fn skills_dir(env: &Env) -> PathBuf {
    match &env.claude_config_dir {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join("skills"),
        _ => env.home.join(".claude/skills"),
    }
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

/// Writes each embedded skill (or, with `--link`, the checkout's copy) as a plain folder
/// `<skills>/<name>/SKILL.md` through a temp file renamed into place (SP-6 fallback:
/// copies, not symlinks); edits no settings.
pub fn install(env: &Env, opts: &InstallOpts) -> Result<InstallReport, Error> {
    let dir = skills_dir(env);
    let mut bodies: Vec<Vec<u8>> = SKILLS.iter().map(|s| s.body.into()).collect();
    if let Some(repo) = &opts.link {
        let repo = fs::canonicalize(repo).unwrap_or_else(|_| repo.clone());
        for (skill, body) in SKILLS.iter().zip(&mut bodies) {
            let path = repo
                .join("pack/claude/skills")
                .join(skill.name)
                .join("SKILL.md");
            if !path.is_file() {
                let msg = format!("not a brain-swap checkout: {} missing", path.display());
                return Err(Error::InvalidInput(msg));
            }
            *body = fs::read(&path).map_err(|e| io(&path, e))?;
        }
    }
    if let Some(exe) = &opts.exe {
        let exe = exe.display().to_string();
        for body in &mut bodies {
            *body = with_exe(&String::from_utf8_lossy(body), &exe).into_bytes();
        }
    }
    let created = !dir.is_dir();
    let folders: Vec<PathBuf> = SKILLS.iter().map(|s| dir.join(s.name)).collect();
    let mut report = InstallReport {
        removed: clear_foreign(&folders, opts.force)?,
        ..Default::default()
    };
    for (folder, body) in folders.into_iter().zip(&bodies) {
        // No symlink is left at `folder` here, so this cannot write through one.
        fs::create_dir_all(&folder).map_err(|e| io(&folder, e))?;
        store::write_atomic(env, &folder.join("SKILL.md"), body, &|_| true, None)?;
        report.linked.push(folder);
    }
    if created {
        report
            .notices
            .push("run /reload-skills in sessions already open".into());
    }
    let program = opts
        .exe
        .as_ref()
        .map_or("brain-swap".into(), |e| e.display().to_string());
    report.notices.push(format!(
        "tip: allow Bash({program}:*) in your Claude Code settings so moves need no prompt"
    ));
    Ok(report)
}

/// Deletes each `<skills>/<name>` that is ours (8.8), never following a symlink; anything
/// else there is left alone.
pub fn remove(env: &Env) -> Result<InstallReport, Error> {
    let mut report = InstallReport::default();
    for skill in &SKILLS {
        let folder = skills_dir(env).join(skill.name);
        if is_our_folder(&folder) {
            fs::remove_dir_all(&folder).map_err(|e| io(&folder, e))?;
            report.removed.push(folder);
        }
    }
    Ok(report)
}

/// A real folder (not a symlink) whose `SKILL.md` is ours.
fn is_our_folder(f: &Path) -> bool {
    fs::symlink_metadata(f).is_ok_and(|m| m.is_dir())
        && fs::read_to_string(f.join("SKILL.md")).is_ok_and(|t| is_ours(&t))
}

/// `body` with `brain-swap ` at each command position (start of a code span, inline
/// command included, or of a fenced line) and the `allowed-tools` value naming `exe`
/// (SP-3 fallback, 8.8).
fn with_exe(body: &str, exe: &str) -> String {
    let mut fenced = false;
    let mut out = String::new();
    for line in body.split_inclusive('\n') {
        let mut line = line.replace("`brain-swap ", &format!("`{exe} "));
        if line.starts_with("allowed-tools:") {
            line = format!("allowed-tools: Bash({exe}:*)\n");
        } else if line.trim_start().starts_with("```") {
            fenced = !fenced;
        } else if fenced && line.trim_start().starts_with("brain-swap ") {
            line = line.replacen("brain-swap ", &format!("{exe} "), 1);
        }
        out += &line;
    }
    out
}

fn io(path: &Path, e: std::io::Error) -> Error {
    Error::Io(format!("{}: {e}", path.display()))
}

/// The 8.8 foreign check: refuses the first of `folders` that exists and is not ours (a
/// folder without our `SKILL.md`, a file, any symlink) unless `force`, which deletes them
/// all and returns them; on refusal nothing is touched.
fn clear_foreign(folders: &[PathBuf], force: bool) -> Result<Vec<PathBuf>, Error> {
    let ours = |f: &Path| fs::symlink_metadata(f).is_err() || is_our_folder(f);
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
