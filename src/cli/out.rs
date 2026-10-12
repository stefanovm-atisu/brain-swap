//! Result, error and warning output (TECHSPEC 6.2, 6.7).

use crate::core::error::Error;
use crate::core::model::{Board, Card, Note};
use crate::core::time::{Age, Stamp, age, age_minutes, format_stamp};
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
    pub latest_note: Option<NoteJson<'a>>,
}

pub fn card_json<'a>(card: &'a Card, board: &'a Board, now: &Stamp) -> CardJson<'a> {
    CardJson {
        id: card.id.to_string(),
        board: &board.name,
        title: &card.title,
        column: &card.column,
        open: board.is_open(card),
        path: &card.path,
        latest_note: card.latest_note().map(|n| note_json(n, now)),
    }
}

/// The 6.7 note.
#[derive(Serialize)]
pub struct NoteJson<'a> {
    pub at: Option<String>,
    pub age_min: Option<i64>,
    pub auto: bool,
    pub doing: &'a str,
    pub next: &'a str,
    pub watch_out: &'a str,
    pub by_this_session: Option<bool>,
    pub place: PlaceJson<'a>,
}

#[derive(Serialize)]
pub struct PlaceJson<'a> {
    pub cwd: Option<&'a Path>,
    pub session: Option<&'a str>,
    pub herdr: Option<HerdrJson<'a>>,
}

#[derive(Serialize)]
pub struct HerdrJson<'a> {
    pub pane: &'a str,
    pub tab: Option<&'a str>,
    pub workspace: Option<&'a str>,
}

/// `by_this_session` stays null: `show` resolves no session (A-E2-21).
pub fn note_json<'a>(n: &'a Note, now: &Stamp) -> NoteJson<'a> {
    NoteJson {
        at: n.at.as_ref().map(format_stamp),
        age_min: age_minutes(n.at.as_ref(), now),
        auto: n.auto,
        doing: &n.doing,
        next: &n.next,
        watch_out: &n.watch_out,
        by_this_session: None,
        place: PlaceJson {
            cwd: n.place.cwd.as_deref(),
            session: n.place.session.as_deref(),
            herdr: n.place.herdr.as_ref().map(|h| HerdrJson {
                pane: &h.pane,
                tab: h.tab.as_deref(),
                workspace: h.workspace.as_deref(),
            }),
        },
    }
}

/// The session a block's `latest note:` ending is judged against.
pub enum SessionView {
    /// `show`: no session resolved, the ending names the note's session.
    Unresolved,
    /// `link` and `context`: the resolved session, if any.
    Resolved(Option<String>),
}

/// The block from `card:` on (6.3).
pub fn card_block(card: &Card, board: &Board, now: &Stamp, session: &SessionView) -> String {
    let s = format!(
        "card: {} {}\nboard: {}, column: {}\n",
        card.id, card.title, board.name, card.column
    );
    match card.latest_note() {
        Some(n) => s + &note_block("latest note", n, now, session),
        None => s + "latest note: none\n",
    }
}

/// `<label>: <stamp>, <age>[, auto], <ending>`, the three parts and the `Where:` line.
pub fn note_block(label: &str, n: &Note, now: &Stamp, session: &SessionView) -> String {
    let stamp = n.at.as_ref().map(format_stamp).unwrap_or_else(|| {
        let h = n.heading.get(4..).unwrap_or_default().trim();
        h.strip_suffix(" auto").unwrap_or(h).trim_end().to_string()
    });
    let age = match age(n.at.as_ref(), now) {
        Age::Now => "now".to_string(),
        a => format!("{a} ago"),
    };
    let auto = if n.auto { ", auto" } else { "" };
    let ending = match (&n.place.session, session) {
        (None, _) => "no session".to_string(),
        (Some(id), SessionView::Unresolved) => format!("by session {id}"),
        (Some(id), SessionView::Resolved(Some(me))) if id == me => "by this session".to_string(),
        (Some(_), SessionView::Resolved(_)) => "by another session".to_string(),
    };
    let mut s = format!("{label}: {stamp}, {age}{auto}, {ending}\n");
    for (label, part) in [
        ("Doing", &n.doing),
        ("Next", &n.next),
        ("Watch out", &n.watch_out),
    ] {
        let part = part.replace('\n', "\n  ");
        s += &format!("{label}:{}{part}\n", if part.is_empty() { "" } else { " " });
    }
    let cwd = n
        .place
        .cwd
        .as_ref()
        .map_or("none".into(), |c| c.display().to_string());
    let pane = n
        .place
        .herdr
        .as_ref()
        .map(|h| format!(" (pane {})", h.pane))
        .unwrap_or_default();
    s + &format!("Where: {cwd}{pane}\n")
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

    fn block(note: crate::core::model::Note) -> String {
        use crate::core::model::{BoardMeta, CardId};
        use crate::core::time::parse_stamp;
        let card = Card {
            id: CardId::parse("W-1").unwrap(),
            path: "W-1.md".into(),
            title: "t".into(),
            column: "Todo".into(),
            template: None,
            created: None,
            body: String::new(),
            notes: vec![note],
            writable: true,
            mtime: None,
        };
        let board = Board {
            name: "work".into(),
            path: "work".into(),
            meta: BoardMeta {
                letter: 'W',
                columns: vec!["Todo".into()],
                next: 2,
            },
            cards: Vec::new(),
            warnings: Vec::new(),
        };
        let now = parse_stamp("2026-10-08T11:52:00+03:00", &jiff::tz::TimeZone::UTC).unwrap();
        card_block(&card, &board, &now, &SessionView::Unresolved)
    }

    fn note(heading: &str) -> crate::core::model::Note {
        let at = crate::core::time::parse_stamp(&heading[4..], &jiff::tz::TimeZone::UTC);
        crate::core::model::Note {
            heading: heading.into(),
            at,
            auto: false,
            doing: "d".into(),
            next: "n".into(),
            watch_out: "w".into(),
            place: Default::default(),
            extra: Vec::new(),
        }
    }

    #[test]
    fn ts6_3_block_age_now_and_bad_stamp() {
        let b = block(note("### 2026-10-08T11:51:40+03:00"));
        assert!(
            b.contains("\nlatest note: 2026-10-08T11:51:40+03:00, now, "),
            "{b}"
        );
        let b = block(note("### yesterday"));
        assert!(b.contains("\nlatest note: yesterday, ? ago, "), "{b}");
    }

    #[test]
    fn ts6_3_block_continuation_and_missing_place() {
        let mut n = note("### 2026-10-08T11:00:00+03:00");
        n.next = "a\nb".into();
        n.watch_out = String::new();
        let b = block(n);
        assert!(
            b.ends_with("\nNext: a\n  b\nWatch out:\nWhere: none\n"),
            "{b}"
        );
    }

    #[test]
    fn ts6_7_note_json_absent_values_null() {
        let now =
            crate::core::time::parse_stamp("2026-10-08T11:52:00+03:00", &jiff::tz::TimeZone::UTC)
                .unwrap();
        let mut w9 = note("### 2026-10-06T17:05:12+03:00");
        w9.place.cwd = Some("/s/brain-swap".into());
        let v = serde_json::to_value(note_json(&w9, &now)).unwrap();
        assert!(v["place"]["herdr"].is_null(), "{v}");
        assert!(v["place"]["session"].is_null(), "{v}");
        assert!(v["by_this_session"].is_null(), "{v}");
        let v = serde_json::to_value(note_json(&note("### yesterday"), &now)).unwrap();
        assert!(v["at"].is_null(), "{v}");
        assert!(v["age_min"].is_null(), "{v}");
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
