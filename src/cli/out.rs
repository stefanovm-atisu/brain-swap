//! Result, error and warning output (TECHSPEC 6.2, 6.7).

use crate::core::error::Error;
use crate::core::model::{Board, Card};
use serde::Serialize;
use std::io::{Stderr, Stdout, Write};
use std::path::Path;

/// The 6.7 Card.
#[derive(Serialize)]
pub struct CardJson<'a> {
    pub id: String,
    pub board: &'a str,
    pub title: &'a str,
    pub column: &'a str,
    pub open: bool,
    pub path: &'a Path,
    // ponytail: always null until `note_json` exists (E2-F2-T5).
    pub latest_note: Option<()>,
}

pub fn card_json<'a>(card: &'a Card, board: &'a Board) -> CardJson<'a> {
    CardJson {
        id: card.id.to_string(),
        board: &board.name,
        title: &card.title,
        column: &card.column,
        open: board.is_open(card),
        path: &card.path,
        latest_note: None,
    }
}

/// One `--json` object: `v` first, then the flattened body.
#[derive(Serialize)]
pub struct Envelope<T> {
    pub v: u8,
    #[serde(flatten)]
    pub body: T,
}

/// Where a command's result, error and warnings go.
pub struct Out<O: Write = Stdout, E: Write = Stderr> {
    pub json: bool,
    pub stdout: O,
    pub stderr: E,
}

impl Out {
    pub fn new(json: bool) -> Out {
        Out {
            json,
            stdout: std::io::stdout(),
            stderr: std::io::stderr(),
        }
    }
}

impl<O: Write, E: Write> Out<O, E> {
    /// `text` as is, or with `--json` the envelope around `body` on one line.
    pub fn result<T: Serialize>(&mut self, text: &str, body: T) {
        if self.json {
            self.json_line(body);
        } else {
            let _ = self.stdout.write_all(text.as_bytes());
        }
    }

    fn json_line<T: Serialize>(&mut self, body: T) {
        let _ = serde_json::to_writer(&mut self.stdout, &Envelope { v: 1, body });
        let _ = writeln!(self.stdout);
    }

    /// `warning: <text>` on stderr in both modes.
    pub fn warn(&mut self, text: &str) {
        let _ = writeln!(self.stderr, "warning: {text}");
    }

    /// `brain-swap: <message>` on stderr, or the error envelope on stdout; returns the exit code.
    pub fn error(&mut self, e: &Error) -> i32 {
        #[derive(Serialize)]
        struct Body<'a> {
            error: Obj<'a>,
        }
        #[derive(Serialize)]
        struct Obj<'a> {
            code: &'a str,
            message: String,
        }
        if self.json {
            self.json_line(Body {
                error: Obj {
                    code: e.code(),
                    message: e.to_string(),
                },
            });
        } else {
            let _ = writeln!(self.stderr, "brain-swap: {e}");
        }
        e.exit_code()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> Vec<Error> {
        let s = |m: &str| m.to_string();
        vec![
            Error::Io(s("disk gone")),
            Error::Unreadable(s("conflict markers")),
            Error::InvalidInput(s("empty note")),
            Error::Config {
                path: "/c/config.toml".into(),
                line: 2,
                col: 3,
                msg: s("expected '='"),
            },
            Error::Verify(s("verify failed")),
            Error::Usage(s("unknown flag")),
            Error::NotFound(s("W-9 not found")),
            Error::NoReference(s("no session")),
            Error::Busy(s("board is busy")),
        ]
    }

    fn out(json: bool) -> Out<Vec<u8>, Vec<u8>> {
        Out {
            json,
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    fn text(b: &[u8]) -> &str {
        std::str::from_utf8(b).unwrap()
    }

    #[test]
    fn ts6_2_error_text_on_stderr() {
        let codes: Vec<i32> = all()
            .iter()
            .map(|e| {
                let mut o = out(false);
                let code = o.error(e);
                assert_eq!(text(&o.stderr), format!("brain-swap: {e}\n"));
                assert_eq!(text(&o.stdout), "");
                code
            })
            .collect();
        assert_eq!(codes, [1, 1, 1, 1, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn ts6_2_error_json_envelope() {
        let codes = [
            "io",
            "unreadable",
            "invalid_input",
            "config",
            "verify_failed",
            "usage",
            "not_found",
            "no_reference",
            "busy",
        ];
        for (e, code) in all().iter().zip(codes) {
            let mut o = out(true);
            o.error(e);
            assert_eq!(
                text(&o.stdout),
                format!("{{\"v\":1,\"error\":{{\"code\":\"{code}\",\"message\":\"{e}\"}}}}\n")
            );
            assert_eq!(text(&o.stderr), "");
        }
    }

    #[test]
    fn ts6_2_warning_prefixed_on_stderr() {
        for json in [false, true] {
            let mut o = out(json);
            o.warn("x");
            assert_eq!(text(&o.stderr), "warning: x\n");
            assert_eq!(text(&o.stdout), "");
        }
    }
}
