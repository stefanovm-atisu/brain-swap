//! `PackEnv`: temp boards `work` and `home` from `tests/fixtures/pack/` for running skill commands.

use super::spawn::{Scenario, command};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/pack");

/// The 12.1 Pack context states an inline command must survive.
#[derive(Clone, Copy, Debug)]
pub enum ContextState {
    NoConfig,
    BrokenConfig,
    MissingFolder,
    NoReference,
    DeletedCard,
    EmptySession,
    Unsubstituted,
    Placeholder,
    WithNote,
    WithoutNote,
}

pub struct PackEnv {
    tmp: tempfile::TempDir,
    scenario: Scenario,
}

impl PackEnv {
    /// Boards `work`, `home` (fixture copies) and `personal` (missing), default `work`.
    pub fn new() -> PackEnv {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for b in ["work", "home"] {
            let to = root.join("boards").join(b);
            fs::create_dir_all(&to).unwrap();
            for e in fs::read_dir(Path::new(FIXTURES).join(b)).unwrap() {
                let p = e.unwrap().path();
                fs::copy(&p, to.join(p.file_name().unwrap())).unwrap();
            }
        }
        let b = |n: &str| root.join("boards").join(n).display().to_string();
        let config = root.join("config/brain-swap");
        fs::create_dir_all(&config).unwrap();
        fs::write(
            config.join("config.toml"),
            format!(
                "default_board = \"work\"\n\n[boards]\nwork = \"{}\"\nhome = \"{}\"\npersonal = \"{}\"\n",
                b("work"),
                b("home"),
                b("personal")
            ),
        )
        .unwrap();
        let bin = Path::new(env!("CARGO_BIN_EXE_brain-swap"))
            .parent()
            .unwrap();
        let scenario = Scenario::new(root)
            .xdg(
                &root.join("config"),
                &root.join("data"),
                &root.join("state"),
            )
            .now("2026-10-08T11:12:00+03:00")
            .path(&format!("{}:/usr/bin:/bin", bin.display()));
        PackEnv { tmp, scenario }
    }

    /// The env of `state` (from `new().with_w12()`) and the text that replaces
    /// `${CLAUDE_SESSION_ID}` in the inline command (`substitute`).
    pub fn for_state(state: ContextState) -> (PackEnv, &'static str) {
        use super::skill_md::SESSION_A;
        let env = PackEnv::new().with_w12();
        let root = env.tmp.path();
        let config = root.join("config/brain-swap/config.toml");
        let edit = |from: &str, to: &str| {
            let text = fs::read_to_string(&config).unwrap();
            fs::write(&config, text.replacen(from, to, 1)).unwrap();
        };
        let session = match state {
            ContextState::NoConfig => {
                fs::remove_file(&config).unwrap();
                SESSION_A
            }
            ContextState::BrokenConfig => {
                edit("\n\n[boards]", "\n[boards");
                SESSION_A
            }
            ContextState::MissingFolder => {
                let work = root.join("boards/work").display().to_string();
                edit(&work, &root.join("absent").display().to_string());
                SESSION_A
            }
            ContextState::NoReference => SESSION_A,
            ContextState::DeletedCard => {
                env.link("W-12", SESSION_A);
                fs::remove_file(root.join("boards/work/W-12.md")).unwrap();
                SESSION_A
            }
            ContextState::EmptySession => "",
            ContextState::Unsubstituted => "${CLAUDE_SESSION_ID}",
            ContextState::Placeholder => "\\${CLAUDE_SESSION_ID}",
            ContextState::WithNote => {
                env.link("W-12", SESSION_A);
                SESSION_A
            }
            ContextState::WithoutNote => {
                env.link("W-11", SESSION_A);
                SESSION_A
            }
        };
        (env, session)
    }

    pub fn with_w12(self) -> PackEnv {
        let to = self.tmp.path().join("boards/work/W-12.md");
        fs::copy(Path::new(FIXTURES).join("W-12.md"), to).unwrap();
        self
    }

    /// `sh -c cmd` with `stdin`, from the temp root: exit code, stdout, stderr.
    pub fn run_sh(&self, cmd: &str, stdin: &str) -> (i32, String, String) {
        let mut child = command(&self.scenario, Path::new("/bin/sh"))
            .args(["-c", cmd])
            .current_dir(self.tmp.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        let text = |b: Vec<u8>| String::from_utf8(b).unwrap();
        (
            out.status.code().unwrap(),
            text(out.stdout),
            text(out.stderr),
        )
    }

    /// Seeds the reference of `session` to `id`.
    pub fn link(&self, id: &str, session: &str) {
        let (code, _, err) = self.run_sh(&format!("brain-swap link {id} --session {session}"), "");
        assert_eq!(code, 0, "{err}");
    }
}
