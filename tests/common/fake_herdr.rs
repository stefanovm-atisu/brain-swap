//! The fake herdr script's argv log and scenario names (TECHSPEC 12.1, A-E5-03, A-E5-04).

use std::fs;
use std::path::Path;

/// The lines of `$HOME/fake-herdr.log`, one per call; empty when there is no log.
pub fn calls(home: &Path) -> Vec<String> {
    fs::read_to_string(home.join("fake-herdr.log"))
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect()
}

/// The scenario names under `tests/fake_herdr/scenarios/`, sorted.
pub fn scenarios() -> Vec<String> {
    let dir = Path::new(super::spawn::FAKE_HERDR).with_file_name("scenarios");
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| {
            let name = e.unwrap().file_name().into_string().unwrap();
            name.strip_suffix(".sh").map(String::from)
        })
        .collect();
    names.sort();
    names
}
