//! Card file grammar (TECHSPEC 4.4).

use std::ops::Range;
use std::path::{Path, PathBuf};

use jiff::tz::TimeZone;

use crate::core::error::Error;
use crate::core::frontmatter::{FmRead, read};
use crate::core::model::{Card, CardId, HerdrPlace, NewCard, Note, Place};
use crate::core::time::{format_stamp, parse_stamp};

#[derive(Debug)]
pub struct Parsed {
    pub card: Card,
    pub warnings: Vec<String>,
}

/// Reads a card; the ID always comes from `id` (FR-01, I2), `mtime` is left `None`.
pub fn parse(id: CardId, path: &Path, bytes: &[u8], default_column: &str, tz: &TimeZone) -> Parsed {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let problem = read_only(&name, bytes);
    let text = String::from_utf8_lossy(bytes).replace("\r\n", "\n");
    let fm = match read(&text) {
        FmRead::Valid(fm) => Some(fm),
        _ => None,
    };
    let key = |k| {
        fm.as_ref()
            .and_then(|f| f.get(k))
            .map(str::trim)
            .filter(|v| !v.is_empty())
    };
    let (title, body, _) = layout(&text);
    let card = Card {
        id,
        path: path.to_path_buf(),
        title: title.unwrap_or("(untitled)").to_string(),
        column: key("column").unwrap_or(default_column).to_string(),
        template: key("template").map(String::from),
        created: key("created").and_then(|s| parse_stamp(s, tz)),
        body: text[body].to_string(),
        notes: note_blocks(&text)
            .into_iter()
            .map(|r| note(&text[r], tz))
            .collect(),
        writable: problem.is_none(),
        mtime: None,
    };
    Parsed {
        card,
        warnings: problem.into_iter().collect(),
    }
}

/// A new card's canonical bytes (R4, A-E1-19) and its warnings; the ID is never written (I2).
pub fn render_new(card: &NewCard) -> (Vec<u8>, Vec<String>) {
    let mut warnings = Vec::new();
    let (mut body, mut fence) = (String::new(), None);
    for line in card.body.lines() {
        if fence.is_some_and(|f| line.starts_with(f)) {
            fence = None;
        } else if fence.is_none() {
            fence = ["```", "~~~"].into_iter().find(|f| line.starts_with(f));
            if line.trim_end() == "## Timeline" {
                warnings = vec!["## Timeline in body dropped".to_string()];
                continue;
            }
        }
        body += line;
        body += "\n";
    }
    // An unclosed fence would swallow the timeline (layout), so close it.
    if let Some(f) = fence {
        body += f;
    }
    let body = body.trim_end();
    let body = if body.is_empty() {
        String::new()
    } else {
        format!("{body}\n\n")
    };
    let text = format!(
        "---\ncolumn: {}\ntemplate: {}\ncreated: {}\n---\n# {}\n\n{body}## Timeline\n",
        card.column,
        card.template,
        format_stamp(&card.created),
        card.title
    );
    (text.into_bytes(), warnings)
}

/// Refuses a read-only card (4.7): invalid frontmatter, non-UTF-8 bytes or a conflict marker.
pub fn check_writable(name: &str, bytes: &[u8]) -> crate::core::error::Result<()> {
    read_only(name, bytes).map_or(Ok(()), |w| Err(Error::Unreadable(w)))
}

/// The warning for the first read-only problem of a card, if any.
fn read_only(name: &str, bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(bytes);
    if let Some(n) = text.lines().position(|l| l.starts_with("<<<<<<<")) {
        return Some(format!("{name}:{}: conflict marker, read only", n + 1));
    }
    if let FmRead::Invalid { line, problem } = read(&text) {
        return Some(format!("{name}:{line}: invalid frontmatter: {problem}"));
    }
    std::str::from_utf8(bytes)
        .is_err()
        .then(|| format!("{name}: not UTF-8, read only"))
}

/// Each note's bytes, from its `### ` heading to its last non-blank line (7.6, A-E1-20).
pub fn note_blocks(text: &str) -> Vec<Range<usize>> {
    let mut blocks: Vec<Range<usize>> = Vec::new();
    let Some(tl) = layout(text).2 else {
        return blocks;
    };
    let mut at = tl.start;
    for raw in text[tl].split_inclusive('\n') {
        let (from, line) = (at, raw.trim_end());
        at += raw.len();
        if line.starts_with("### ") {
            blocks.push(from..from + line.len());
        } else if let Some(b) = blocks.last_mut()
            && !line.is_empty()
        {
            b.end = from + line.len();
        }
    }
    blocks
}

/// The title, the body's range and the timeline's range (after `## Timeline` up to the post).
fn layout(text: &str) -> (Option<&str>, Range<usize>, Option<Range<usize>>) {
    let start = match read(text) {
        FmRead::Valid(fm) => fm.end,
        _ => 0,
    };
    let (mut title, mut body, mut timeline) = (None, start..text.len(), None::<Range<usize>>);
    let mut fence = None;
    let mut at = start;
    for raw in text[start..].split_inclusive('\n') {
        let (from, line) = (at, raw.trim_end());
        at += raw.len();
        if let Some(f) = fence {
            if line.starts_with(f) {
                fence = None;
            }
        } else if let Some(f) = ["```", "~~~"].into_iter().find(|f| line.starts_with(f)) {
            fence = Some(f);
        } else if let Some(t) = &mut timeline {
            if line.starts_with("# ") || line.starts_with("## ") {
                t.end = from;
                break;
            }
        } else if line == "## Timeline" {
            body.end = from;
            timeline = Some(at..text.len());
        } else if title.is_none()
            && let Some(t) = line.strip_prefix("# ")
        {
            title = Some(t.trim());
            body.start = at;
        }
    }
    (title, body, timeline)
}

fn note(block: &str, tz: &TimeZone) -> Note {
    let mut lines = block.lines();
    let heading = lines.next().unwrap_or_default().trim_end();
    let stamp = heading.get(4..).unwrap_or_default().trim();
    let (stamp, auto) = match stamp.strip_suffix(" auto") {
        Some(s) => (s.trim_end(), true),
        None => (stamp, false),
    };
    let mut parts: [String; 3] = Default::default();
    let (mut place, mut extra, mut cur) = (Place::default(), Vec::new(), None);
    for line in lines {
        if let (Some(i), Some(c)) = (cur, line.strip_prefix("  ")) {
            parts[i] = format!("{}\n{c}", parts[i]);
            continue;
        }
        cur = None;
        let l = line.strip_prefix("- ").unwrap_or(line);
        let (l, b) = l.strip_prefix("**").map_or((l, ""), |l| (l, "**"));
        if let Some((label, t)) = l.split_once(':')
            && let Some(i) = ["doing", "next", "watch out"].iter().position(|x| {
                label
                    .strip_suffix(b)
                    .unwrap_or(label)
                    .eq_ignore_ascii_case(x)
            })
        {
            let t = t.strip_prefix(b).unwrap_or(t);
            parts[i] = t.strip_prefix(' ').unwrap_or(t).to_string();
            cur = Some(i);
        } else if let Some(pairs) = line
            .strip_prefix("<!-- where: ")
            .and_then(|r| r.trim_end().strip_suffix(" -->"))
        {
            place = where_line(pairs);
        } else if !line.trim().is_empty() {
            extra.push(line.to_string());
        }
    }
    let [doing, next, watch_out] = parts;
    let words: Vec<&str> = stamp.split_whitespace().collect();
    Note {
        heading: heading.to_string(),
        at: [2, 1]
            .into_iter()
            .find_map(|n| parse_stamp(&words.get(..n)?.join(" "), tz)),
        auto,
        doing,
        next,
        watch_out,
        place,
        extra,
    }
}

/// `k=v` pairs of a place line, values bare or quoted with `\"` and `\\`; unknown keys are ignored here.
fn where_line(pairs: &str) -> Place {
    let mut p = Place::default();
    let (mut pane, mut tab, mut workspace) = (None, None, None);
    let mut toks = vec![String::new()];
    let (mut quoted, mut escaped) = (false, false);
    for c in pairs.chars() {
        match c {
            _ if escaped => escaped = false,
            '\\' if quoted => {
                escaped = true;
                continue;
            }
            '"' => {
                quoted = !quoted;
                continue;
            }
            ' ' if !quoted => {
                toks.push(String::new());
                continue;
            }
            _ => {}
        }
        toks.last_mut().unwrap().push(c);
    }
    for (k, v) in toks.iter().filter_map(|p| p.split_once('=')) {
        let v = Some(v.to_string());
        match k {
            "cwd" => p.cwd = v.map(PathBuf::from),
            "pane" => pane = v,
            "tab" => tab = v,
            "workspace" => workspace = v,
            "session" => p.session = v,
            _ => {}
        }
    }
    p.herdr = pane.map(|pane| HerdrPlace {
        pane,
        tab,
        workspace,
    });
    p
}
