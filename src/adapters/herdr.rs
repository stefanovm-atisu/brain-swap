//! The herdr adapter (TECHSPEC 9): herdr runs through its public CLI only.

use crate::adapters::runner::Runner;
use crate::core::env::Env;
use crate::core::model::HerdrPlace;
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
