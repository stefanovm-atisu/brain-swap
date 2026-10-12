//! `Sandbox`: a temp root with `home/`, `config/`, `data/`, `state/` for CLI tests.

use super::spawn::{Scenario, brain_swap};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Sandbox {
    tmp: tempfile::TempDir,
    scenario: Scenario,
}

impl Sandbox {
    pub fn new() -> Sandbox {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for d in ["home", "config", "data", "state", "elsewhere"] {
            fs::create_dir_all(root.join(d)).unwrap();
        }
        let scenario = Scenario::new(root).xdg(
            &root.join("config"),
            &root.join("data"),
            &root.join("state"),
        );
        Sandbox { tmp, scenario }
    }

    pub fn root(&self) -> &Path {
        self.tmp.path()
    }

    pub fn config_file(&self) -> PathBuf {
        self.root().join("config/brain-swap/config.toml")
    }

    /// Boards `work`, `home`, `personal` at `<root>/boards/<name>`, default `work`.
    pub fn write_config(&self) {
        let b = |n: &str| self.root().join("boards").join(n).display().to_string();
        let text = format!(
            "default_board = \"work\"\n\n[boards]\nwork = \"{}\"\nhome = \"{}\"\npersonal = \"{}\"\n",
            b("work"),
            b("home"),
            b("personal")
        );
        let path = self.config_file();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    /// Copies `tests/fixtures/boards/<fixture>/` to `<root>/boards/<board>`, `@SANDBOX@` as the root.
    pub fn copy_board(&self, fixture: &str, board: &str) {
        let from = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/boards")
            .join(fixture);
        let to = self.root().join("boards").join(board);
        fs::create_dir_all(&to).unwrap();
        let root = self.root().display().to_string();
        for e in fs::read_dir(from).unwrap() {
            let p = e.unwrap().path();
            let text = fs::read_to_string(&p).unwrap().replace("@SANDBOX@", &root);
            fs::write(to.join(p.file_name().unwrap()), text).unwrap();
        }
    }

    pub fn cmd(&self, args: &[&str]) -> assert_cmd::Command {
        let mut c = brain_swap(&self.scenario);
        c.args(args).current_dir(self.root().join("elsewhere"));
        c
    }

    pub fn redact(&self, text: &str) -> String {
        text.replace(&self.root().display().to_string(), "<sandbox>")
    }

    pub fn scenario_mut(&mut self) -> &mut Scenario {
        &mut self.scenario
    }
}
