//! TECHSPEC 4.8 R7 and 10: the verify-then-rename atomic write.

mod common;

use brain_swap::core::error::Error;
use brain_swap::core::store::{temp_path, write_atomic};
use common::env::core_env;
use std::fs;
use std::path::{Path, PathBuf};

const NOW: &str = "2026-10-08T11:52:00+03:00";

fn names(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect()
}

fn setup(tmp: &Path) -> PathBuf {
    let target = tmp.join("b/W-12.md");
    fs::create_dir(tmp.join("b")).unwrap();
    fs::write(&target, "a").unwrap();
    target
}

#[test]
fn ts10_write_atomic_replaces_content() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let target = setup(tmp.path());
    write_atomic(&env, &target, b"b", &|_| true, None).unwrap();
    assert_eq!(fs::read_to_string(&target).unwrap(), "b");
}

#[test]
fn ts10_temp_file_is_dot_name_bs_tmp_pid() {
    assert_eq!(
        temp_path(Path::new("/b/W-12.md"), 4711),
        PathBuf::from("/b/.W-12.md.bs-tmp-4711")
    );
}

#[test]
fn r7_verify_failure_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let target = setup(tmp.path());
    let err = write_atomic(&env, &target, b"b", &|_| false, None).unwrap_err();
    assert!(matches!(err, Error::Verify(_)), "{err:?}");
    assert_eq!(err.code(), "verify_failed");
    assert_eq!(err.exit_code(), 1);
    assert_eq!(fs::read_to_string(&target).unwrap(), "a");
    assert_eq!(names(&tmp.path().join("b")), vec![target]);
}

#[test]
fn r7_verify_failure_is_logged() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), NOW);
    let log = tmp.path().join("bs.log");
    env.log = Some(log.clone());
    let target = setup(tmp.path());
    write_atomic(&env, &target, b"b", &|_| false, None).unwrap_err();
    let text = fs::read_to_string(&log).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1, "{text}");
    let fields: Vec<&str> = lines[0].split(' ').collect();
    assert_eq!(fields[1], "verify_failed");
    assert_eq!(fields[2], target.to_string_lossy());
}

#[test]
fn ts10_no_temp_left_after_success() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let target = setup(tmp.path());
    write_atomic(&env, &target, b"b", &|_| true, None).unwrap();
    assert_eq!(names(&tmp.path().join("b")), vec![target]);
}

#[cfg(feature = "failpoints")]
#[test]
fn ts12_1_failpoint_after_tmp_leaves_temp_and_target() {
    use brain_swap::core::failpoint::Failpoint;
    let tmp = tempfile::tempdir().unwrap();
    let mut env = core_env(tmp.path(), NOW);
    env.failpoint = Some(Failpoint {
        name: "park:after_tmp".to_string(),
        abort: false,
    });
    let target = setup(tmp.path());
    let err = write_atomic(&env, &target, b"b", &|_| true, Some("park:after_tmp")).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "{err:?}");
    assert_eq!(fs::read_to_string(&target).unwrap(), "a");
    let tmp_file = temp_path(&target, env.pid);
    assert_eq!(fs::read_to_string(tmp_file).unwrap(), "b");
}

#[test]
fn ts11_write_error_names_target() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let target = tmp.path().join("W-12.md");
    fs::create_dir(&target).unwrap();
    let err = write_atomic(&env, &target, b"b", &|_| true, None).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "{err:?}");
    assert!(err.to_string().contains("W-12.md"), "{err}");
}
