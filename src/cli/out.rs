//! Result, error and warning output (TECHSPEC 6.2, 6.7).

use crate::adapters::herdr::{Focused, PaneStatus};
use crate::cli::cmd::Ctx;
use crate::core::error::Error;
use crate::core::model::{Board, Card, Note, Place};
use crate::core::time::{Age, Stamp, age, age_minutes, format_stamp};
use serde::Serialize;
use serde::ser::{SerializeStruct, Serializer};
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

/// `<n>. <ID> <title> [<column>] <age>[, <label>]: <preview>` (6.3), without a newline.
pub fn picker_line(n: usize, card: &Card, label: Option<&str>, now: &Stamp) -> String {
    let (age, preview) = match card.latest_note() {
        Some(note) => (age(note.at.as_ref(), now).to_string(), preview(note)),
        None => ("no note".to_string(), String::new()),
    };
    let label = label.map(|l| format!(", {l}")).unwrap_or_default();
    let preview = match preview.is_empty() {
        true => preview,
        false => format!(": {preview}"),
    };
    let (id, title, column) = (card.id, &card.title, &card.column);
    format!("{n}. {id} {title} [{column}] {age}{label}{preview}")
}

/// The 6.7 Candidate: a Card plus its 1-based printed position and 6.6 reason (A-E2-20).
#[derive(Serialize)]
pub struct CandidateJson<'a> {
    #[serde(flatten)]
    pub card: CardJson<'a>,
    pub rank: usize,
    pub reason: &'static str,
}

/// The configured boards other than `shown`, in config order (A-E2-19).
pub fn other_boards<'a>(ctx: &'a Ctx, shown: &str) -> Vec<&'a str> {
    let names = ctx.config.boards.iter().map(|(n, _)| n.as_str());
    names.filter(|n| *n != shown).collect()
}

/// `o. other board: <names>`, or `None` when no other board is configured (A-E2-19).
pub fn other_boards_line(ctx: &Ctx, shown: &str) -> Option<String> {
    let names = other_boards(ctx, shown);
    (!names.is_empty()).then(|| format!("o. other board: {}", names.join(", ")))
}

/// The first non-blank line of Next, else Doing, cut at the last word boundary within 40 characters.
pub fn preview(note: &Note) -> String {
    let mut lines = note.next.lines().chain(note.doing.lines()).map(str::trim);
    let line = lines
        .find(|l| !l.is_empty())
        .unwrap_or_default()
        .to_string();
    let chars: Vec<char> = line.chars().collect();
    if chars.len() <= 40 {
        return line;
    }
    // A space at position 40 still keeps the 40 characters before it whole.
    let cut = chars[..41]
        .iter()
        .rposition(|c| c.is_whitespace())
        .unwrap_or(40);
    chars[..cut]
        .iter()
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// The `locate` lines (6.3): the status, then `Where: <cwd>`.
pub fn locate_lines(status: &PaneStatus, place: &Place) -> String {
    let head = match status {
        PaneStatus::Live(p) => format!("live {}", p.pane),
        PaneStatus::Moved(p) => format!("moved {}", p.pane),
        PaneStatus::Closed => format!("closed {}", place.herdr.as_ref().map_or("", |h| &h.pane)),
        PaneStatus::OutsideHerdr => "not in herdr".into(),
        PaneStatus::NoPane => "no pane recorded".into(),
        PaneStatus::NotResponding => "herdr not responding".into(),
        PaneStatus::HerdrError(m) => format!("herdr: {m}"),
    };
    let cwd = place
        .cwd
        .as_ref()
        .map_or("none".into(), |c| c.display().to_string());
    format!("{head}\nWhere: {cwd}\n")
}

/// The 6.7 `locate` object: `pane` is the live pane for `live` and `moved`, else the recorded one.
#[derive(Serialize)]
pub struct LocateJson<'a> {
    pub status: &'static str,
    pub pane: Option<&'a str>,
    pub cwd: Option<&'a Path>,
}

impl<'a> LocateJson<'a> {
    pub fn new(status: &'a PaneStatus, place: &'a Place) -> LocateJson<'a> {
        let pane = match status {
            PaneStatus::Live(p) | PaneStatus::Moved(p) => Some(p.pane.as_str()),
            _ => place.herdr.as_ref().map(|h| h.pane.as_str()),
        };
        LocateJson {
            status: status.json_status(),
            pane,
            cwd: place.cwd.as_deref(),
        }
    }
}

/// The 6.7 `jump` object: the `locate` fields plus `focused` (`"pane"`, `"tab"` or `false`).
pub struct JumpJson<'a> {
    pub locate: LocateJson<'a>,
    pub focused: Option<Focused>,
}

impl Serialize for JumpJson<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_struct("JumpJson", 4)?;
        m.serialize_field("status", self.locate.status)?;
        m.serialize_field("pane", &self.locate.pane)?;
        m.serialize_field("cwd", &self.locate.cwd)?;
        match self.focused {
            Some(Focused::Agent) => m.serialize_field("focused", "pane")?,
            Some(Focused::Tab) => m.serialize_field("focused", "tab")?,
            None => m.serialize_field("focused", &false)?,
        }
        m.end()
    }
}

/// The `install herdr` body (6.7).
#[derive(Serialize)]
pub struct SnippetJson<'a> {
    pub snippet: &'a str,
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

    fn card(notes: Vec<crate::core::model::Note>) -> Card {
        Card {
            id: crate::core::model::CardId::parse("W-1").unwrap(),
            path: "W-1.md".into(),
            title: "t".into(),
            column: "Todo".into(),
            template: None,
            created: None,
            body: String::new(),
            notes,
            writable: true,
            mtime: None,
        }
    }

    #[test]
    fn ts6_3_picker_preview_cut_at_word_boundary() {
        let mut n = note("### 2026-10-08T11:00:00+03:00");
        n.next = "open the MR, ask the DBA for the v13 deploy list".into();
        assert_eq!(preview(&n), "open the MR, ask the DBA for the v13");
        let forty = "abcd efghi ".repeat(4)[..40].to_string();
        n.next = forty.clone();
        assert_eq!(preview(&n), forty);
        n.next = "x".repeat(50);
        assert_eq!(preview(&n), "x".repeat(40));
        n.next = "\u{fc}".repeat(45);
        assert_eq!(preview(&n), "\u{fc}".repeat(40));
    }

    #[test]
    fn ts6_3_picker_preview_skips_blank_leading_lines() {
        let mut n = note("### 2026-10-08T11:00:00+03:00");
        n.next = "\nopen the MR".into();
        assert_eq!(preview(&n), "open the MR");
        n.next = String::new();
        n.doing = "\nmigration test passes".into();
        assert_eq!(preview(&n), "migration test passes");
    }

    #[test]
    fn ts6_3_picker_preview_falls_back_to_doing() {
        let now =
            crate::core::time::parse_stamp("2026-10-08T11:52:00+03:00", &jiff::tz::TimeZone::UTC)
                .unwrap();
        let mut n = note("### 2026-10-08T11:00:00+03:00");
        n.next = String::new();
        n.doing = "a\nb".into();
        assert_eq!(preview(&n), "a");
        n.doing = String::new();
        let line = picker_line(1, &card(vec![n]), None, &now);
        assert!(line.ends_with("52 min"), "{line}");
        assert!(!line.contains(": "), "{line}");
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
