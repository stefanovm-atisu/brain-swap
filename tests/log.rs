//! TECHSPEC 11 and FR-05: the `BRAIN_SWAP_LOG` sink.

mod common;

use brain_swap::core::log::{event, guard_board_folders};
use common::env::core_env;
use std::fs;
use std::path::PathBuf;

const NOW: &str = "2026-10-08T11:52:00+03:00";

fn files_under(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(files_under(&p));
        } else {
            out.push(p);
        }
    }
    out
}

#[test]
fn ts11_event_appends_one_line_per_call() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), NOW);
    let log = tmp.path().join("bs.log");
    env.log = Some(log.clone());
    event(&env, "lock_wait", "/b 150");
    event(&env, "lock_wait", "/b 150");
    let line = "2026-10-08T11:52:00+03:00 lock_wait /b 150\n";
    assert_eq!(fs::read_to_string(log).unwrap(), format!("{line}{line}"));
}

#[test]
fn ts11_no_log_when_unset() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), NOW);
    env.log = None;
    event(&env, "lock_wait", "/b 150");
    assert!(files_under(tmp.path()).is_empty());
}

#[test]
fn ts11_event_keeps_existing_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), NOW);
    let log = tmp.path().join("bs.log");
    fs::write(&log, "old\n").unwrap();
    env.log = Some(log.clone());
    event(&env, "merge", "A-1");
    assert_eq!(
        fs::read_to_string(log).unwrap(),
        "old\n2026-10-08T11:52:00+03:00 merge A-1\n"
    );
}

#[test]
fn fr05_log_inside_board_folder_is_ignored_with_warning() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("work");
    fs::create_dir(&work).unwrap();
    let mut env = core_env(tmp.path(), NOW);
    env.log = Some(work.join("bs.log"));
    let warning = guard_board_folders(&mut env, &[("work".to_string(), work.clone())]);
    assert_eq!(
        warning.as_deref(),
        Some("BRAIN_SWAP_LOG inside board work, ignored")
    );
    assert_eq!(env.log, None);
    event(&env, "reload", "work");
    assert!(files_under(&work).is_empty());
}

#[test]
fn fr05_log_outside_board_folders_is_kept() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("work");
    let mut env = core_env(tmp.path(), NOW);
    let log = tmp.path().join("state").join("bs.log");
    env.log = Some(log.clone());
    let warning = guard_board_folders(&mut env, &[("work".to_string(), work)]);
    assert_eq!(warning, None);
    assert_eq!(env.log, Some(log));
}

#[test]
fn ts11_unwritable_log_does_not_fail() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), NOW);
    env.log = Some(tmp.path().to_path_buf());
    event(&env, "verify_failed", "A-1");
}

#[test]
fn ts11_concurrent_events_do_not_splice_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), NOW);
    let log = tmp.path().join("bs.log");
    env.log = Some(log.clone());
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let env = env.clone();
            std::thread::spawn(move || {
                for _ in 0..500 {
                    event(&env, "lock_wait", "/b 150");
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    let text = fs::read_to_string(log).unwrap();
    assert_eq!(text.lines().count(), 4000);
    assert!(
        text.lines()
            .all(|l| l == "2026-10-08T11:52:00+03:00 lock_wait /b 150")
    );
}

#[test]
fn fr05_log_with_dotdot_inside_board_folder_is_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("work");
    let mut env = core_env(tmp.path(), NOW);
    env.cwd = tmp.path().join("proj");
    env.log = Some(PathBuf::from("../work/./bs.log"));
    let warning = guard_board_folders(&mut env, &[("work".to_string(), work)]);
    assert_eq!(
        warning.as_deref(),
        Some("BRAIN_SWAP_LOG inside board work, ignored")
    );
    assert_eq!(env.log, None);
}
