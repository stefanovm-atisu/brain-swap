//! The herdr adapter (TECHSPEC 9): herdr runs through its public CLI only.

use crate::adapters::runner::{RunOutcome, Runner};
use crate::core::env::Env;
use crate::core::model::{HerdrPlace, Place};
use serde_json::Value;
use std::time::Duration;

/// The timeout every herdr call asks for (T-15).
pub const CALL_TIMEOUT: Duration = Duration::from_secs(2);

/// The herdr part of a note's place at park time (9.1, FR-15, FR-58); runs no herdr command.
pub fn capture(env: &Env, _runner: &dyn Runner) -> Option<HerdrPlace> {
    let set = |v: &Option<String>| v.clone().filter(|v| !v.is_empty());
    if !env.herdr_active() {
        return None;
    }
    Some(HerdrPlace {
        pane: set(&env.herdr_pane_id)?,
        tab: set(&env.herdr_tab_id),
        workspace: set(&env.herdr_workspace_id),
    })
}

/// The current pane's `agent_session.value` (9.2, 6.4 step 3); `None` outside herdr or on any failure.
pub fn pane_session(env: &Env, runner: &dyn Runner) -> Option<String> {
    let pane = env.herdr_pane_id.as_deref().filter(|p| !p.is_empty());
    let pane = pane.filter(|_| env.herdr_active())?;
    let RunOutcome::Exited {
        code: Some(0),
        stdout,
        ..
    } = runner.run(&["pane", "get", pane], CALL_TIMEOUT).outcome
    else {
        return None;
    };
    let v: Value = serde_json::from_str(&stdout).ok()?;
    Some(PaneJson::from_response(&v)?.session()?.to_string())
}

/// A tolerant reader over one herdr pane object (9.3); no serde-derived herdr types.
pub struct PaneJson<'a>(&'a Value);

impl<'a> PaneJson<'a> {
    /// The pane at `result.pane` of a `pane get` answer, else `None`.
    pub fn from_response(v: &'a Value) -> Option<PaneJson<'a>> {
        v.pointer("/result/pane")
            .filter(|p| p.is_object())
            .map(PaneJson)
    }

    /// A pane object taken from a list.
    pub fn new(v: &'a Value) -> PaneJson<'a> {
        PaneJson(v)
    }

    fn text(&self, path: &str) -> Option<&'a str> {
        self.0.pointer(path)?.as_str().filter(|s| !s.is_empty())
    }

    pub fn id(&self) -> Option<&str> {
        self.text("/pane_id")
    }

    pub fn tab(&self) -> Option<&str> {
        self.text("/tab_id")
    }

    pub fn workspace(&self) -> Option<&str> {
        self.text("/workspace_id")
    }

    /// `agent_session.value`, non-empty (9.2).
    pub fn session(&self) -> Option<&str> {
        self.text("/agent_session/value")
    }

    /// The `LivePane.agent` rule of 9.3.
    pub fn has_agent(&self) -> bool {
        let status = self.0.get("agent_status").and_then(Value::as_str);
        self.0.get("agent").is_some()
            || self.0.get("agent_session").is_some()
            || status.is_some_and(|s| s != "unknown")
    }
}

/// `error.code` of herdr's JSON stderr, else `None` (9.3).
pub fn error_code(stderr: &str) -> Option<String> {
    let v: Value = serde_json::from_str(stderr).ok()?;
    Some(v.pointer("/error/code")?.as_str()?.to_string())
}

/// A pane herdr answered for, with live IDs (TECHSPEC 3, 9.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivePane {
    pub pane: String,
    pub tab: String,
    pub workspace: String,
    pub agent: bool,
}

/// What `locate` found for a note's place (TECHSPEC 3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaneStatus {
    Live(LivePane),
    Moved(LivePane),
    Closed,
    OutsideHerdr,
    NoPane,
    NotResponding,
    HerdrError(String),
}

impl PaneStatus {
    /// The `status` name of the `locate` and `jump` objects (6.7).
    pub fn json_status(&self) -> &'static str {
        match self {
            PaneStatus::Live(_) => "live",
            PaneStatus::Moved(_) => "moved",
            PaneStatus::Closed => "closed",
            PaneStatus::OutsideHerdr => "outside",
            PaneStatus::NoPane => "none",
            PaneStatus::NotResponding => "not_responding",
            PaneStatus::HerdrError(_) => "herdr_error",
        }
    }
}

/// Turns a note's place into a `PaneStatus` (9.3).
pub fn locate(place: &Place, env: &Env, runner: &dyn Runner) -> PaneStatus {
    if !env.herdr_active() {
        return PaneStatus::OutsideHerdr;
    }
    let Some(h) = &place.herdr else {
        return PaneStatus::NoPane;
    };
    let stdout = match runner.run(&["pane", "get", &h.pane], CALL_TIMEOUT).outcome {
        RunOutcome::Exited {
            code: Some(0),
            stdout,
            ..
        } => stdout,
        RunOutcome::TimedOut => return PaneStatus::NotResponding,
        RunOutcome::SpawnFailed(msg) => return PaneStatus::HerdrError(msg),
        RunOutcome::Exited { code, stderr, .. } => {
            if error_code(&stderr).as_deref() == Some("pane_not_found") {
                // ponytail: the moved search by place.session is E5-F2-T3's.
                return PaneStatus::Closed;
            }
            let code = code.map_or("signal".into(), |c| c.to_string());
            let first = stderr.lines().find(|l| !l.trim().is_empty());
            return PaneStatus::HerdrError(first.map_or(format!("exit {code}"), String::from));
        }
    };
    let v: Value = serde_json::from_str(&stdout).unwrap_or_default();
    let Some(p) = PaneJson::from_response(&v) else {
        return PaneStatus::HerdrError("pane get: unreadable answer".into());
    };
    let or = |live: Option<&str>, stored: &Option<String>| {
        live.or(stored.as_deref()).unwrap_or_default().to_string()
    };
    let live = LivePane {
        pane: or(p.id(), &Some(h.pane.clone())),
        tab: or(p.tab(), &h.tab),
        workspace: or(p.workspace(), &h.workspace),
        agent: p.has_agent(),
    };
    match live.pane == h.pane {
        true => PaneStatus::Live(live),
        false => PaneStatus::Moved(live),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AGENT: &str = include_str!("../../tests/fixtures/herdr/pane_agent.json");
    const SHELL: &str = include_str!("../../tests/fixtures/herdr/pane_shell.json");
    const NOT_FOUND: &str = include_str!("../../tests/fixtures/herdr/pane_not_found.stderr");

    fn json(s: &str) -> Value {
        serde_json::from_str(s).unwrap()
    }

    fn with(s: &str, edit: impl FnOnce(&mut serde_json::Map<String, Value>)) -> Value {
        let mut v = json(s);
        edit(v["result"]["pane"].as_object_mut().unwrap());
        v
    }

    #[test]
    fn fr41_pane_json_reads_agent_session_value() {
        let v = json(AGENT);
        let p = PaneJson::from_response(&v).unwrap();
        assert_eq!(p.session(), Some("55583127-a22a-49bc-a803-e777c377a595"));
    }

    #[test]
    fn ts9_2_shell_pane_has_no_session() {
        let v = json(SHELL);
        assert_eq!(PaneJson::from_response(&v).unwrap().session(), None);
    }

    #[test]
    fn ts9_2_empty_session_value_is_none() {
        let v = with(AGENT, |p| p["agent_session"]["value"] = "".into());
        assert_eq!(PaneJson::from_response(&v).unwrap().session(), None);
    }

    #[test]
    fn ts9_3_pane_json_reads_ids() {
        let v = json(AGENT);
        let p = PaneJson::from_response(&v).unwrap();
        assert_eq!(p.id(), Some("w4V:p9"));
        assert_eq!(p.tab(), Some("w4V:t1"));
        assert_eq!(p.workspace(), Some("w4V"));
    }

    #[test]
    fn ts9_3_from_response_needs_result_pane() {
        assert!(PaneJson::from_response(&json(r#"{"result":{}}"#)).is_none());
        assert!(PaneJson::from_response(&json("[]")).is_none());
    }

    #[test]
    fn ts9_3_agent_session_key_means_agent() {
        let v = with(AGENT, |p| {
            p.insert("agent_status".into(), "unknown".into());
        });
        assert!(PaneJson::from_response(&v).unwrap().has_agent());
    }

    #[test]
    fn ts9_3_agent_key_means_agent() {
        let v = with(SHELL, |p| {
            p.insert("agent".into(), "claude".into());
        });
        assert!(PaneJson::from_response(&v).unwrap().has_agent());
    }

    #[test]
    fn ts9_3_known_agent_status_means_agent() {
        let v = with(SHELL, |p| {
            p.insert("agent_status".into(), "working".into());
        });
        assert!(PaneJson::from_response(&v).unwrap().has_agent());
    }

    #[test]
    fn ts9_3_unknown_or_missing_status_means_no_agent() {
        let v = json(SHELL);
        assert!(!PaneJson::from_response(&v).unwrap().has_agent());
        let v = with(SHELL, |p| {
            p.remove("agent_status");
        });
        assert!(!PaneJson::from_response(&v).unwrap().has_agent());
    }

    #[test]
    fn ts9_3_error_code_reads_pane_not_found() {
        assert_eq!(error_code(NOT_FOUND), Some("pane_not_found".into()));
    }

    #[test]
    fn ts9_3_error_code_none_for_plain_text() {
        assert_eq!(error_code("herdr: cannot connect"), None);
        assert_eq!(error_code(""), None);
    }

    #[test]
    fn ts6_7_status_json_names() {
        let p = LivePane {
            pane: "w4V:p9".into(),
            tab: "w4V:t1".into(),
            workspace: "w4V".into(),
            agent: true,
        };
        let all = [
            PaneStatus::Live(p.clone()),
            PaneStatus::Moved(p),
            PaneStatus::Closed,
            PaneStatus::OutsideHerdr,
            PaneStatus::NoPane,
            PaneStatus::NotResponding,
            PaneStatus::HerdrError("x".into()),
        ];
        let names: Vec<_> = all.iter().map(PaneStatus::json_status).collect();
        assert_eq!(
            names,
            [
                "live",
                "moved",
                "closed",
                "outside",
                "none",
                "not_responding",
                "herdr_error"
            ]
        );
    }
}
