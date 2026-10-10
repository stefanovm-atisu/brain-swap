//! The process environment read once by `main` (TECHSPEC 5.1, 6.1).

use crate::core::error::{Error, Result};
use crate::core::failpoint::Failpoint;
use crate::core::time::{Stamp, parse_stamp};
use jiff::tz::TimeZone;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Env {
    pub cwd: PathBuf,
    pub home: PathBuf,
    pub config_home: PathBuf,
    pub data_home: PathBuf,
    pub state_home: PathBuf,
    pub visual: Option<String>,
    pub editor: Option<String>,
    pub herdr_env: Option<String>,
    pub herdr_bin_path: Option<PathBuf>,
    pub herdr_pane_id: Option<String>,
    pub herdr_tab_id: Option<String>,
    pub herdr_workspace_id: Option<String>,
    pub session: Option<String>,
    pub log: Option<PathBuf>,
    pub claude_config_dir: Option<PathBuf>,
    pub exe: Option<PathBuf>,
    pub failpoint: Option<Failpoint>,
    pub pid: u32,
    pub tz: TimeZone,
    pub now: Stamp,
}

impl Env {
    /// Reads only the 6.1 names from `vars`; `BRAIN_SWAP_NOW` replaces `clock`, both land in `tz`.
    pub fn from_vars(
        vars: impl IntoIterator<Item = (String, String)>,
        cwd: PathBuf,
        pid: u32,
        tz: TimeZone,
        clock: Stamp,
    ) -> Result<Env> {
        let vars: HashMap<String, String> = vars.into_iter().collect();
        let get = |k: &str| vars.get(k).cloned();
        let home = PathBuf::from(
            get("HOME")
                .filter(|h| !h.is_empty())
                .ok_or_else(|| Error::Io("HOME not set".to_string()))?,
        );
        let xdg = |k: &str, default: &str| {
            get(k)
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .unwrap_or_else(|| home.join(default))
        };
        let now = match get("BRAIN_SWAP_NOW") {
            Some(v) => parse_stamp(&v, &tz).ok_or_else(|| {
                Error::InvalidInput(format!("BRAIN_SWAP_NOW: invalid stamp '{v}'"))
            })?,
            None => clock,
        };
        Ok(Env {
            cwd,
            config_home: xdg("XDG_CONFIG_HOME", ".config"),
            data_home: xdg("XDG_DATA_HOME", ".local/share"),
            state_home: xdg("XDG_STATE_HOME", ".local/state"),
            visual: get("VISUAL"),
            editor: get("EDITOR"),
            herdr_env: get("HERDR_ENV"),
            herdr_bin_path: get("HERDR_BIN_PATH").map(PathBuf::from),
            herdr_pane_id: get("HERDR_PANE_ID"),
            herdr_tab_id: get("HERDR_TAB_ID"),
            herdr_workspace_id: get("HERDR_WORKSPACE_ID"),
            session: get("BRAIN_SWAP_SESSION"),
            log: get("BRAIN_SWAP_LOG").map(PathBuf::from),
            claude_config_dir: get("CLAUDE_CONFIG_DIR").map(PathBuf::from),
            exe: None,
            failpoint: get("BRAIN_SWAP_FAILPOINT")
                .filter(|_| cfg!(feature = "failpoints"))
                .map(|name| Failpoint { name, abort: true }),
            pid,
            now: now.timestamp().to_zoned(tz.clone()),
            tz,
            home,
        })
    }

    /// herdr integration is active only for `HERDR_ENV=1` (9.1, FR-58).
    pub fn herdr_active(&self) -> bool {
        self.herdr_env.as_deref() == Some("1")
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_home.join("brain-swap/config.toml")
    }

    pub fn templates_dir(&self) -> PathBuf {
        self.config_home.join("brain-swap/templates")
    }

    pub fn data_dir(&self) -> PathBuf {
        self.data_home.join("brain-swap")
    }

    pub fn default_board_dir(&self, name: &str) -> PathBuf {
        self.data_dir().join(name)
    }

    pub fn state_dir(&self) -> PathBuf {
        self.state_home.join("brain-swap")
    }

    pub fn sessions_dir(&self) -> PathBuf {
        self.state_dir().join("sessions")
    }

    pub fn panes_dir(&self) -> PathBuf {
        self.state_dir().join("panes")
    }

    pub fn locks_dir(&self) -> PathBuf {
        self.state_dir().join("locks")
    }

    pub fn edit_dir(&self) -> PathBuf {
        self.state_dir().join("edit")
    }
}

/// A path as one file name (10, 6.4): `%` becomes `%25`, then `/` becomes `%2F`.
pub fn encode_file_name(name: &str) -> String {
    name.replace('%', "%25").replace('/', "%2F")
}
