//! TECHSPEC 10: pruning `sessions/` and `panes/` after 30 days (FR-05, T-17).

mod common;

use brain_swap::core::env::Env;
use brain_swap::core::session::prune;
use common::env::core_env;
use jiff::SignedDuration;
use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const NOW: &str = "2026-10-08T12:00:00+00:00";

fn days(n: i64) -> SignedDuration {
    SignedDuration::from_hours(24 * n)
}

/// Creates `path` (and its folder) with its mtime `age` before `env.now`.
fn file_aged(env: &Env, path: &Path, age: SignedDuration) -> PathBuf {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mtime = SystemTime::from(env.now.timestamp() - age);
    File::create(path).unwrap().set_modified(mtime).unwrap();
    path.to_path_buf()
}

#[test]
fn ts10_prune_removes_session_files_older_than_30_days() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    let a = file_aged(&env, &env.sessions_dir().join("a.toml"), days(31));
    let b = file_aged(&env, &env.sessions_dir().join("b.toml"), days(29));
    prune(&env);
    assert!(!a.exists());
    assert!(b.exists());
}

#[test]
fn ts10_prune_removes_pane_hints_older_than_30_days() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    let old = file_aged(&env, &env.panes_dir().join("w4V:p9"), days(31));
    let new = file_aged(&env, &env.panes_dir().join("w4V:p11"), days(29));
    prune(&env);
    assert!(!old.exists());
    assert!(new.exists());
}

#[test]
fn ts10_prune_boundary_is_strictly_over_30_days() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    let hours = SignedDuration::from_hours(720);
    let x = file_aged(&env, &env.sessions_dir().join("x.toml"), hours);
    let y = file_aged(
        &env,
        &env.sessions_dir().join("y.toml"),
        hours + SignedDuration::from_secs(1),
    );
    prune(&env);
    assert!(x.exists());
    assert!(!y.exists());
}

#[test]
fn ts10_prune_measures_age_from_env_now() {
    let home = tempfile::tempdir().unwrap();
    let early = core_env(home.path(), "2026-09-20T12:00:00+00:00");
    let late = core_env(home.path(), "2026-10-08T12:00:00+00:00");
    let a = early.sessions_dir().join("a.toml");
    fs::create_dir_all(early.sessions_dir()).unwrap();
    let mtime = core_env(home.path(), "2026-09-01T12:00:00+00:00").now;
    File::create(&a)
        .unwrap()
        .set_modified(SystemTime::from(mtime.timestamp()))
        .unwrap();
    prune(&early);
    assert!(a.exists());
    prune(&late);
    assert!(!a.exists());
}

#[test]
fn ts10_prune_removes_old_crash_leftovers() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    let tmp = file_aged(
        &env,
        &env.sessions_dir().join(".s1.toml.bs-tmp-4711"),
        days(31),
    );
    prune(&env);
    assert!(!tmp.exists());
}

#[test]
fn ts10_prune_touches_nothing_else() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    let lock = file_aged(&env, &env.locks_dir().join("x.lock"), days(365));
    let edit = file_aged(&env, &env.edit_dir().join("work-W-12.md"), days(365));
    prune(&env);
    assert!(lock.exists());
    assert!(edit.exists());
}

#[test]
fn ts10_prune_without_folders_creates_nothing() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    prune(&env);
    assert!(!env.state_dir().exists());
}

#[test]
fn ts10_prune_ignores_errors() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    let a = file_aged(&env, &env.sessions_dir().join("a.toml"), days(31));
    let pane = file_aged(&env, &env.panes_dir().join("w4V:p9"), days(31));
    let sessions = env.sessions_dir();
    fs::set_permissions(&sessions, fs::Permissions::from_mode(0o555)).unwrap();
    // Root ignores the mode: if a write into the folder succeeds, the test cannot run.
    if File::create(sessions.join("probe")).is_ok() {
        return;
    }
    prune(&env);
    fs::set_permissions(&sessions, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(a.exists());
    assert!(!pane.exists());
}
