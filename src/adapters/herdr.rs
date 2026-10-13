//! The herdr adapter (TECHSPEC 9): herdr runs through its public CLI only.

use crate::adapters::runner::Runner;
use crate::core::env::Env;
use crate::core::model::HerdrPlace;
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
}
