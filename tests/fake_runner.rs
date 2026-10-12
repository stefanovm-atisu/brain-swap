//! TECHSPEC 12.1: `FakeRunner` scripts herdr answers for in-process tests without sleeping.

use brain_swap::adapters::runner::{Answer, FakeRunner, RunOutcome, RunResult, Runner};
use std::time::{Duration, Instant};

const TWO_S: Duration = Duration::from_secs(2);

fn exited(code: i32, stdout: &str, stderr: &str) -> RunOutcome {
    RunOutcome::Exited {
        code: Some(code),
        stdout: stdout.into(),
        stderr: stderr.into(),
    }
}

#[test]
fn ts12_1_fake_runner_answers_and_records_argv() {
    let fake = FakeRunner::new().on("pane get w4V:p9", Answer::ok("{}"));
    for _ in 0..2 {
        let r = fake.run(&["pane", "get", "w4V:p9"], TWO_S);
        assert_eq!(r.outcome, exited(0, "{}", ""));
    }
    assert_eq!(fake.calls(), ["pane get w4V:p9", "pane get w4V:p9"]);
}

#[test]
fn ts12_1_fake_runner_records_timeouts() {
    let fake = FakeRunner::new().on("pane get w4V:p9", Answer::ok("{}"));
    fake.run(&["pane", "get", "w4V:p9"], TWO_S);
    fake.run(&["pane", "get", "w4V:p9"], Duration::from_millis(500));
    assert_eq!(fake.timeouts(), [TWO_S, Duration::from_millis(500)]);
}

#[test]
fn ts12_1_fake_runner_hang_times_out_without_waiting() {
    let fake = FakeRunner::new().on("pane get x", Answer::Hang);
    let start = Instant::now();
    let r = fake.run(&["pane", "get", "x"], TWO_S);
    assert_eq!(
        r,
        RunResult {
            outcome: RunOutcome::TimedOut,
            took: TWO_S
        }
    );
    assert!(start.elapsed() < Duration::from_millis(100));
}

#[test]
fn ts12_1_fake_runner_slow_answer_beyond_timeout_times_out() {
    let slow = FakeRunner::new().on("a", Answer::ok("{}").taking(Duration::from_millis(2500)));
    let r = slow.run(&["a"], TWO_S);
    assert_eq!(
        r,
        RunResult {
            outcome: RunOutcome::TimedOut,
            took: TWO_S
        }
    );
    let quick = FakeRunner::new().on("a", Answer::ok("{}").taking(Duration::from_millis(1500)));
    let r = quick.run(&["a"], TWO_S);
    assert_eq!(
        r,
        RunResult {
            outcome: exited(0, "{}", ""),
            took: Duration::from_millis(1500)
        }
    );
}

#[test]
fn ts12_1_fake_runner_fail_and_spawn_failed() {
    let fake = FakeRunner::new()
        .on("a", Answer::fail(1, "boom"))
        .on("b", Answer::SpawnFailed("no such file".into()));
    assert_eq!(fake.run(&["a"], TWO_S).outcome, exited(1, "", "boom"));
    assert_eq!(
        fake.run(&["b"], TWO_S),
        RunResult {
            outcome: RunOutcome::SpawnFailed("no such file".into()),
            took: Duration::ZERO
        }
    );
}

#[test]
#[should_panic(expected = "no answer for 'tab focus x'")]
fn ts12_1_fake_runner_unplanned_call_panics() {
    FakeRunner::new().run(&["tab", "focus", "x"], TWO_S);
}
