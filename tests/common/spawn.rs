//! The one spawn helper (TECHSPEC 12.1): every process a test starts gets a cleared
//! environment plus exactly the scenario's variables.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// The fake herdr script that `HERDR_BIN_PATH` always names.
pub const FAKE_HERDR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fake_herdr/herdr");

pub struct Scenario {
    vars: BTreeMap<String, String>,
}

impl Scenario {
    /// The 12.1 defaults under `<tmp>/home`, which it creates (the fake herdr logs there).
    pub fn new(tmp: &Path) -> Scenario {
        let home = tmp.join("home");
        fs::create_dir_all(&home).unwrap();
        let h = |p: &str| home.join(p).to_string_lossy().into_owned();
        let vars = [
            ("HOME", home.to_string_lossy().into_owned()),
            ("XDG_CONFIG_HOME", h(".config")),
            ("XDG_DATA_HOME", h(".local/share")),
            ("XDG_STATE_HOME", h(".local/state")),
            ("TZ", "UTC".to_string()),
            ("BRAIN_SWAP_NOW", "2026-10-08T11:52:00+03:00".to_string()),
            ("PATH", "/usr/bin:/bin".to_string()),
            ("HERDR_BIN_PATH", FAKE_HERDR.to_string()),
        ];
        Scenario {
            vars: vars.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        }
    }

    fn set(mut self, name: &str, value: impl Into<String>) -> Scenario {
        self.vars.insert(name.to_string(), value.into());
        self
    }

    pub fn herdr(self, pane: &str, tab: &str, workspace: &str) -> Scenario {
        self.set("HERDR_ENV", "1")
            .set("HERDR_PANE_ID", pane)
            .set("HERDR_TAB_ID", tab)
            .set("HERDR_WORKSPACE_ID", workspace)
    }

    pub fn fake_scenario(self, name: &str) -> Scenario {
        self.set("FAKE_HERDR_SCENARIO", name)
    }

    pub fn failpoint(self, name: &str) -> Scenario {
        self.set("BRAIN_SWAP_FAILPOINT", name)
    }

    pub fn session(self, id: &str) -> Scenario {
        self.set("BRAIN_SWAP_SESSION", id)
    }

    pub fn log(self, path: &Path) -> Scenario {
        self.set("BRAIN_SWAP_LOG", path.to_string_lossy())
    }

    pub fn visual(self, cmd: &str) -> Scenario {
        self.set("VISUAL", cmd)
    }

    pub fn editor(self, cmd: &str) -> Scenario {
        self.set("EDITOR", cmd)
    }

    pub fn now(self, stamp: &str) -> Scenario {
        self.set("BRAIN_SWAP_NOW", stamp)
    }

    pub fn xdg(self, config: &Path, data: &Path, state: &Path) -> Scenario {
        self.set("XDG_CONFIG_HOME", config.to_string_lossy())
            .set("XDG_DATA_HOME", data.to_string_lossy())
            .set("XDG_STATE_HOME", state.to_string_lossy())
    }

    /// Drops one `XDG_*` variable; any other name panics so `HERDR_BIN_PATH` stays the fake.
    pub fn unset(mut self, name: &str) -> Scenario {
        assert!(
            ["XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_STATE_HOME"].contains(&name),
            "unset({name}): only XDG_* variables may be dropped"
        );
        self.vars.remove(name);
        self
    }

    pub fn path(self, value: &str) -> Scenario {
        self.set("PATH", value)
    }

    /// Exactly the variables `command` sets, sorted by name.
    pub fn vars(&self) -> Vec<(String, String)> {
        self.vars.clone().into_iter().collect()
    }
}

pub fn command(s: &Scenario, program: &Path) -> std::process::Command {
    let mut c = std::process::Command::new(program);
    c.env_clear().envs(&s.vars);
    c
}

pub fn brain_swap(s: &Scenario) -> assert_cmd::Command {
    assert_cmd::Command::from_std(command(s, assert_cmd::cargo::cargo_bin!("brain-swap")))
}
