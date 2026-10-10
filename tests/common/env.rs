//! `core_env`: an `Env` for core tests that never reads the process environment.

use brain_swap::core::env::Env;
use brain_swap::core::time::parse_stamp;
use jiff::tz::TimeZone;
use std::path::Path;

/// `HOME=home`, XDG defaults under it, `now` as the clock and its fixed offset as the zone.
pub fn core_env(home: &Path, now: &str) -> Env {
    let now = parse_stamp(now, &TimeZone::UTC).unwrap();
    let home = home.to_string_lossy().into_owned();
    Env::from_vars(
        [("HOME".to_string(), home.clone())],
        home.into(),
        std::process::id(),
        now.time_zone().clone(),
        now,
    )
    .unwrap()
}
