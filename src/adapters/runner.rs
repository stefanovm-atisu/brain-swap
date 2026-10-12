//! The one process boundary of the herdr adapter (TECHSPEC 2.2, 9.1): herdr runs through its CLI only.

use crate::core::env::Env;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Runs one herdr command; object safe, so callers take `&dyn Runner`.
pub trait Runner {
    fn run(&self, args: &[&str], timeout: Duration) -> RunResult;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunResult {
    pub outcome: RunOutcome,
    pub took: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunOutcome {
    Exited {
        code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    TimedOut,
    SpawnFailed(String),
}

impl RunResult {
    pub fn succeeded(&self) -> bool {
        matches!(self.outcome, RunOutcome::Exited { code: Some(0), .. })
    }
}

/// Starts the real herdr binary, without a shell.
#[derive(Clone, Debug)]
pub struct ProcessRunner {
    env: Env,
    program: PathBuf,
}

impl ProcessRunner {
    pub fn new(env: &Env) -> ProcessRunner {
        let program = match &env.herdr_bin_path {
            Some(p) if !p.as_os_str().is_empty() => p.clone(),
            _ => PathBuf::from("herdr"),
        };
        ProcessRunner {
            env: env.clone(),
            program,
        }
    }

    pub fn program(&self) -> &Path {
        &self.program
    }
}

impl Runner for ProcessRunner {
    fn run(&self, args: &[&str], timeout: Duration) -> RunResult {
        let start = Instant::now();
        let spawned = Command::new(&self.program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let outcome = match spawned {
            Err(e) => RunOutcome::SpawnFailed(e.to_string()),
            Ok(mut child) => {
                let out = drain(child.stdout.take());
                let err = drain(child.stderr.take());
                let rest = || timeout.saturating_sub(start.elapsed());
                loop {
                    match child.try_wait() {
                        Ok(None) if start.elapsed() < timeout => {
                            thread::sleep(Duration::from_millis(10))
                        }
                        Ok(Some(status)) => {
                            break RunOutcome::Exited {
                                code: status.code(),
                                // A stream still held open past the deadline reads as empty.
                                stdout: out.recv_timeout(rest()).unwrap_or_default(),
                                stderr: err.recv_timeout(rest()).unwrap_or_default(),
                            };
                        }
                        // Readers are left detached: a grandchild may still hold the pipes.
                        _ => {
                            let _ = child.kill();
                            let _ = child.wait();
                            break RunOutcome::TimedOut;
                        }
                    }
                }
            }
        };
        let took = start.elapsed();
        let code = match &outcome {
            RunOutcome::Exited { code: Some(c), .. } => c.to_string(),
            RunOutcome::Exited { .. } => "signal".into(),
            RunOutcome::TimedOut => "timeout".into(),
            RunOutcome::SpawnFailed(_) => "spawn_failed".into(),
        };
        let text = format!("{} exit={code} ms={}", args.join(" "), took.as_millis());
        crate::core::log::event(&self.env, "herdr", &text);
        RunResult { outcome, took }
    }
}

/// Reads a child's stream to its end on its own thread, so a full pipe cannot stall the child.
fn drain(stream: Option<impl Read + Send + 'static>) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut s) = stream {
            let _ = s.read_to_end(&mut buf);
        }
        let _ = tx.send(String::from_utf8_lossy(&buf).into_owned());
    });
    rx
}

/// A scripted `Runner` for in-process tests (TECHSPEC 12.1); it never sleeps and never starts a process.
#[derive(Debug, Default)]
pub struct FakeRunner {
    answers: std::collections::HashMap<String, Answer>,
    calls: std::cell::RefCell<Vec<(String, Duration)>>,
}

/// What a `FakeRunner` answers for one argv.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    Exit {
        code: i32,
        stdout: String,
        stderr: String,
        took: Duration,
    },
    Hang,
    SpawnFailed(String),
}

impl Answer {
    pub fn ok(stdout: &str) -> Answer {
        Answer::Exit {
            code: 0,
            stdout: stdout.into(),
            stderr: String::new(),
            took: Duration::ZERO,
        }
    }

    pub fn fail(code: i32, stderr: &str) -> Answer {
        Answer::Exit {
            code,
            stdout: String::new(),
            stderr: stderr.into(),
            took: Duration::ZERO,
        }
    }

    /// Sets how long an `Exit` answer pretends to take; other answers are unchanged.
    pub fn taking(self, d: Duration) -> Answer {
        match self {
            Answer::Exit {
                code,
                stdout,
                stderr,
                ..
            } => Answer::Exit {
                code,
                stdout,
                stderr,
                took: d,
            },
            other => other,
        }
    }
}

impl FakeRunner {
    pub fn new() -> FakeRunner {
        FakeRunner::default()
    }

    pub fn on(mut self, args: &str, answer: Answer) -> Self {
        self.answers.insert(args.to_string(), answer);
        self
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.borrow().iter().map(|c| c.0.clone()).collect()
    }

    pub fn timeouts(&self) -> Vec<Duration> {
        self.calls.borrow().iter().map(|c| c.1).collect()
    }
}

impl Runner for FakeRunner {
    fn run(&self, args: &[&str], timeout: Duration) -> RunResult {
        let argv = args.join(" ");
        self.calls.borrow_mut().push((argv.clone(), timeout));
        let answer = self.answers.get(&argv);
        let answer = answer.unwrap_or_else(|| panic!("FakeRunner: no answer for '{argv}'"));
        let timed_out = RunResult {
            outcome: RunOutcome::TimedOut,
            took: timeout,
        };
        match answer.clone() {
            Answer::Exit { took, .. } if took > timeout => timed_out,
            Answer::Exit {
                code,
                stdout,
                stderr,
                took,
            } => RunResult {
                outcome: RunOutcome::Exited {
                    code: Some(code),
                    stdout,
                    stderr,
                },
                took,
            },
            Answer::Hang => timed_out,
            Answer::SpawnFailed(msg) => RunResult {
                outcome: RunOutcome::SpawnFailed(msg),
                took: Duration::ZERO,
            },
        }
    }
}
