//! TECHSPEC 10 and 11: the per-board lock in the state folder.

mod common;

use brain_swap::core::env::encode_file_name;
use brain_swap::core::error::Error;
use brain_swap::core::store::{lock_board, lock_path};
use common::env::core_env;
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const NOW: &str = "2026-10-08T11:52:00+03:00";

fn expected_lock(state_home: &Path, board: &Path) -> std::path::PathBuf {
    let canonical = board.canonicalize().unwrap();
    state_home.join("brain-swap/locks").join(format!(
        "{}.lock",
        encode_file_name(&canonical.to_string_lossy())
    ))
}

/// Thread A holds the lock for 150 ms while the caller locks; returns the caller's wait.
fn contended_wait(env: &brain_swap::core::env::Env, board: &Path) -> Duration {
    let (tx, rx) = mpsc::channel();
    let (a_env, a_board) = (env.clone(), board.to_path_buf());
    let a = thread::spawn(move || {
        let lock = lock_board(&a_env, &a_board).unwrap();
        tx.send(()).unwrap();
        thread::sleep(Duration::from_millis(150));
        drop(lock);
    });
    rx.recv().unwrap();
    let start = Instant::now();
    let lock = lock_board(env, board);
    let waited = start.elapsed();
    assert!(lock.is_ok());
    a.join().unwrap();
    waited
}

#[test]
fn fr05_lock_file_lives_in_state_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let board = tmp.path().join("b");
    fs::create_dir(&board).unwrap();
    let _lock = lock_board(&env, &board).unwrap();
    assert!(expected_lock(&env.state_home, &board).is_file());
    assert_eq!(fs::read_dir(&board).unwrap().count(), 0);
}

#[test]
fn fr04_lock_creates_missing_board_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let board = tmp.path().join("new/b");
    let _lock = lock_board(&env, &board).unwrap();
    assert!(board.is_dir());
}

#[test]
fn ts10_second_locker_waits_until_release() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let board = tmp.path().join("b");
    let waited = contended_wait(&env, &board);
    assert!(waited >= Duration::from_millis(100), "{waited:?}");
    assert!(waited < Duration::from_secs(2), "{waited:?}");
}

#[test]
fn ts10_lock_wait_over_100_ms_is_logged() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), NOW);
    let board = tmp.path().join("b");
    let log = tmp.path().join("bs.log");
    env.log = Some(log.clone());
    fs::create_dir(&board).unwrap();
    // Only B logs: A's uncontended lock is never a wait over 100 ms.
    contended_wait(&env, &board);
    let text = fs::read_to_string(&log).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1, "{text}");
    let canonical = board.canonicalize().unwrap();
    let fields: Vec<&str> = lines[0].split(' ').collect();
    assert_eq!(fields[1], "lock_wait");
    assert_eq!(fields[2], canonical.to_string_lossy());
}

#[test]
fn ts10_short_lock_wait_is_not_logged() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), NOW);
    let log = tmp.path().join("bs.log");
    env.log = Some(log.clone());
    let _lock = lock_board(&env, &tmp.path().join("b")).unwrap();
    assert!(fs::read_to_string(&log).unwrap_or_default().is_empty());
}

#[test]
fn ts10_lock_times_out_busy_after_2_seconds() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let board = tmp.path().join("b");
    fs::create_dir(&board).unwrap();
    let path = expected_lock(&env.state_home, &board);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let holder = fs::File::create(&path).unwrap();
    holder.try_lock().unwrap();
    let start = Instant::now();
    let err = lock_board(&env, &board).err().unwrap();
    let waited = start.elapsed();
    assert_eq!(err, Error::Busy("board busy, try again".to_string()));
    assert_eq!(err.exit_code(), 5);
    assert!(waited >= Duration::from_secs(2), "{waited:?}");
    assert!(waited <= Duration::from_millis(2500), "{waited:?}");
}

#[test]
fn ts10_lock_is_released_on_drop() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let board = tmp.path().join("b");
    drop(lock_board(&env, &board).unwrap());
    let start = Instant::now();
    let _lock = lock_board(&env, &board).unwrap();
    assert!(start.elapsed() < Duration::from_millis(100));
}

#[test]
fn ts10_lock_path_is_encoded_canonical_board_path() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let board = tmp.path().join("b");
    fs::create_dir(&board).unwrap();
    let link = tmp.path().join("link");
    std::os::unix::fs::symlink(&board, &link).unwrap();
    let expected = expected_lock(&env.state_home, &board);
    assert_eq!(lock_path(&env, &link).unwrap(), expected);
    assert!(!env.locks_dir().exists());
    let _lock = lock_board(&env, &board).unwrap();
    let files: Vec<_> = fs::read_dir(env.locks_dir())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(files, vec![expected]);
}
