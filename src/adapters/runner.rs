//! The one process boundary of the herdr adapter (TECHSPEC 2.2, 9.1): herdr runs through its CLI only.

use crate::core::env::Env;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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
    #[allow(dead_code)] // read by the log line (E5-F1-T3)
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
    fn run(&self, args: &[&str], _timeout: Duration) -> RunResult {
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
                let code = child.wait().ok().and_then(|s| s.code());
                RunOutcome::Exited {
                    code,
                    stdout: out.join().unwrap_or_default(),
                    stderr: err.join().unwrap_or_default(),
                }
            }
        };
        RunResult {
            outcome,
            took: start.elapsed(),
        }
    }
}

/// Reads a child's stream to its end on its own thread, so a full pipe cannot stall the child.
fn drain(stream: Option<impl Read + Send + 'static>) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut s) = stream {
            let _ = s.read_to_end(&mut buf);
        }
        String::from_utf8_lossy(&buf).into_owned()
    })
}
