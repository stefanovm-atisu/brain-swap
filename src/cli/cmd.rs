//! One function per subcommand (TECHSPEC 2.1).

use crate::adapters::herdr;
use crate::adapters::runner::{ProcessRunner, Runner};
use crate::cli::args::{Cli, Command};
use crate::cli::out::{self, Out};
use crate::core::config::{self, Config};
use crate::core::env::Env;
use crate::core::error::Error;
use crate::core::guess;
use crate::core::model::{Board, Card, CardId, NewNote, SessionRef};
use crate::core::session::{read_reference, write_reference};
use crate::core::store::{self, CreateRequest};
use crate::core::template::{self, Template};
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

/// A clap failure: help and version print and exit 0, anything else is
/// `Error::Usage` (exit 2), followed in text mode by clap's usage lines.
pub fn parse_failure(err: clap::Error, argv: &[OsString]) -> i32 {
    use clap::error::ErrorKind;
    if matches!(
        err.kind(),
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
    ) {
        let _ = err.print();
        return 0;
    }
    let json = argv
        .iter()
        .skip(1)
        .take_while(|a| *a != "--")
        .any(|a| a == "--json");
    let text = err.render().to_string();
    let mut lines = text.lines();
    let msg = lines.next().unwrap_or_default();
    let mut out = Out::new(json);
    let code = out.error(&Error::Usage(msg.trim_start_matches("error: ").into()));
    if !json {
        for l in lines {
            let _ = writeln!(out.stderr, "{l}");
        }
    }
    code
}

/// What every board subcommand loads first: the config, the path written on first run
/// and the warnings to print.
pub struct Ctx {
    pub config: Config,
    pub created: Option<PathBuf>,
    pub warnings: Vec<String>,
    pub env: Env,
}

impl Ctx {
    /// Loads `config.toml` (a broken one is `Error::Config`, never defaults, section 10)
    /// and drops a `BRAIN_SWAP_LOG` inside a board folder (section 11).
    pub fn load(env: &mut Env) -> Result<Ctx, Error> {
        let loaded = config::load(env)?;
        let mut warnings = loaded.warnings;
        warnings.extend(crate::core::log::guard_board_folders(
            env,
            &loaded.config.boards,
        ));
        Ok(Ctx {
            config: loaded.config,
            created: loaded.created,
            warnings,
            env: env.clone(),
        })
    }

    /// Board `name`, else the default board; an unknown name is `NotFound` (6.2).
    pub fn board(&self, name: Option<&str>) -> Result<Board, Error> {
        let (name, path) = self
            .config
            .board(name.unwrap_or(&self.config.default_board))?;
        crate::core::board::load(name, path, &self.env)
    }
}

/// Template `name` among the built-ins and the user's templates (6.2); prints the
/// template load warnings (5.4).
pub fn find_template(ctx: &Ctx, name: &str, out: &mut Out) -> Result<Template, Error> {
    let (ts, warnings) = template::load(&ctx.env.templates_dir());
    warnings.iter().for_each(|w| out.warn(w));
    template::find(&ts, name).cloned()
}

/// Card `id` and its board, found by letter over the configured boards (6.1);
/// an unknown card is `NotFound("<ID> not found")` (6.2).
pub fn find_card(ctx: &Ctx, id: CardId) -> Result<(Board, Card), Error> {
    let board = crate::core::board::find_card(&ctx.config.boards, id, &ctx.env)?;
    let card = board.card(id).cloned().expect("find_card checked the card");
    Ok((board, card))
}

/// TECHSPEC 6.4: steps 1 and 2 with their warnings, then herdr's pane session (step 3) when
/// it passes I8, else none; never fails.
pub fn resolve_session(
    env: &Env,
    flag: Option<&str>,
    runner: &dyn Runner,
) -> (Option<String>, Vec<String>) {
    use crate::core::session::{IdCheck, check_id, resolve};
    let r = resolve(flag, env);
    let session = r
        .session
        .or_else(|| herdr::pane_session(env, runner).filter(|s| check_id(s) == IdCheck::Valid));
    (session, r.warnings)
}

/// Dispatches one subcommand and returns its 6.2 exit code.
pub fn run(env: &mut Env, cli: Cli) -> i32 {
    let mut out = Out::new(cli.json);
    let mut ctx = None;
    if !matches!(
        cli.command,
        Some(Command::Templates | Command::Install(_) | Command::Context { .. })
    ) {
        match Ctx::load(env) {
            Ok(c) => {
                if let Some(p) = &c.created {
                    let notice = format!("created config {}", p.display());
                    if out.json {
                        out.warn(&notice);
                    } else {
                        let _ = writeln!(out.stderr, "{notice}");
                    }
                }
                c.warnings.iter().for_each(|w| out.warn(w));
                ctx = Some(c);
            }
            Err(e) => return out.error(&e),
        }
    }
    match cli.command {
        Some(Command::New {
            title,
            template,
            column,
            body,
            session,
        }) => {
            let ctx = ctx.expect("loaded above");
            let runner = ProcessRunner::new(env);
            let (session, warnings) = resolve_session(env, session.as_deref(), &runner);
            warnings.iter().for_each(|w| out.warn(w));
            let names = (cli.board.as_deref(), template.as_deref(), column.as_deref());
            match new(env, &ctx, &title.join(" "), names, body.is_some(), &mut out) {
                Ok((board, created)) => {
                    let card = &created.card;
                    for w in board.warnings.iter().chain(&created.warnings) {
                        out.warn(w);
                    }
                    set_reference(
                        env,
                        session.as_deref(),
                        card.id,
                        &board.name,
                        &runner,
                        &mut out,
                    );
                    #[derive(serde::Serialize)]
                    struct Body<'a> {
                        card: out::CardJson<'a>,
                    }
                    let text = format!(
                        "created {}: {} ({}, {})\n",
                        card.id, card.title, card.column, board.name
                    );
                    out.result(
                        &text,
                        Body {
                            card: out::card_json(card, &board, &env.now),
                        },
                    );
                    0
                }
                Err(e) => out.error(&e),
            }
        }
        Some(Command::Show { id, all }) => {
            let ctx = ctx.expect("loaded above");
            match find_card(&ctx, id) {
                Ok((board, card)) => {
                    board.warnings.iter().for_each(|w| out.warn(w));
                    #[derive(serde::Serialize)]
                    struct Body<'a> {
                        card: out::CardJson<'a>,
                        notes: Vec<out::NoteJson<'a>>,
                    }
                    let view = out::SessionView::Unresolved;
                    let mut text = out::card_block(&card, &board, &env.now, &view);
                    // Notes before the latest, numbered from the oldest as 1 (A-E2-13).
                    let older = if all {
                        card.notes.len().saturating_sub(1)
                    } else {
                        0
                    };
                    for (i, n) in card.notes[..older].iter().enumerate().rev() {
                        text += &format!(
                            "\n{}",
                            out::note_block(&format!("note {}", i + 1), n, &env.now, &view)
                        );
                    }
                    let notes = card.notes.iter().rev().take(older + 1);
                    out.result(
                        &text,
                        Body {
                            card: out::card_json(&card, &board, &env.now),
                            notes: notes.map(|n| out::note_json(n, &env.now)).collect(),
                        },
                    );
                    0
                }
                Err(e) => out.error(&e),
            }
        }
        Some(Command::Move { id, column }) => {
            let ctx = ctx.expect("loaded above");
            let moved = find_card(&ctx, id).and_then(|(board, _)| {
                let column = board.column(&column)?.to_string();
                Ok((store::move_card(env, &board, id, &column)?, board, column))
            });
            match moved {
                Ok((moved, board, column)) => {
                    board.warnings.iter().for_each(|w| out.warn(w));
                    #[derive(serde::Serialize)]
                    struct Body<'a> {
                        card: out::CardJson<'a>,
                    }
                    out.result(
                        &format!("moved {id} to {column}\n"),
                        Body {
                            card: out::card_json(&moved.card, &board, &env.now),
                        },
                    );
                    0
                }
                Err(e) => out.error(&e),
            }
        }
        Some(Command::Ls { open, guess }) => {
            let ctx = ctx.expect("loaded above");
            match ctx.board(cli.board.as_deref()) {
                Ok(board) if guess => {
                    board.warnings.iter().for_each(|w| out.warn(w));
                    let cands = guess::best_guess(&board, &guess::GuessInput::from_env(env));
                    let mut text = format!("open cards on {}, best guess first:\n", board.name);
                    for (i, c) in cands.iter().enumerate() {
                        text += &out::picker_line(i + 1, c.card, c.tier.label(), &env.now);
                        text.push('\n');
                    }
                    if let Some(l) = out::other_boards_line(&ctx, &board.name) {
                        text += &l;
                        text.push('\n');
                    }
                    #[derive(serde::Serialize)]
                    struct Body<'a> {
                        board: &'a str,
                        cards: Vec<out::CandidateJson<'a>>,
                        other_boards: Vec<&'a str>,
                    }
                    let cards = cands.iter().enumerate().map(|(i, c)| out::CandidateJson {
                        card: out::card_json(c.card, &board, &env.now),
                        rank: i + 1,
                        reason: c.tier.reason(),
                    });
                    out.result(
                        &text,
                        Body {
                            board: &board.name,
                            cards: cards.collect(),
                            other_boards: out::other_boards(&ctx, &board.name),
                        },
                    );
                    0
                }
                Ok(board) => {
                    board.warnings.iter().for_each(|w| out.warn(w));
                    let cards: Vec<&Card> = (0..board.column_list().len())
                        .flat_map(|i| board.cards_in(i))
                        .filter(|c| !open || board.is_open(c))
                        .collect();
                    let mut text = format!("cards on {}:\n", board.name);
                    for (i, c) in cards.iter().enumerate() {
                        text += &out::picker_line(i + 1, c, None, &env.now);
                        text.push('\n');
                    }
                    #[derive(serde::Serialize)]
                    struct Body<'a> {
                        board: &'a str,
                        cards: Vec<out::CardJson<'a>>,
                    }
                    let cards = cards.iter().map(|c| out::card_json(c, &board, &env.now));
                    out.result(
                        &text,
                        Body {
                            board: &board.name,
                            cards: cards.collect(),
                        },
                    );
                    0
                }
                Err(e) => out.error(&e),
            }
        }
        Some(Command::Templates) => {
            let (ts, warnings) = template::load(&env.templates_dir());
            for w in &warnings {
                out.warn(w);
            }
            #[derive(serde::Serialize)]
            struct Body<'a> {
                templates: &'a [Template],
            }
            out.result(&templates(&ts), Body { templates: &ts });
            0
        }
        Some(Command::Park {
            card: flag,
            session,
            auto,
        }) => {
            let ctx = ctx.expect("loaded above");
            let runner = ProcessRunner::new(env);
            let (session, warnings) = resolve_session(env, session.as_deref(), &runner);
            warnings.iter().for_each(|w| out.warn(w));
            // I9: the reference counts while its board is configured and holds its card.
            let target = match flag {
                Some(id) => find_card(&ctx, id).map(|(board, _)| (board, id)),
                None => session
                    .as_deref()
                    .and_then(|s| read_reference(env, s))
                    .and_then(|r| {
                        let board = ctx.board(Some(&r.board)).ok()?;
                        board.card(r.card).is_some().then_some((board, r.card))
                    })
                    .ok_or_else(|| Error::NoReference("no reference: pass --card <ID>".into())),
            };
            let parked = target.and_then(|(board, id)| {
                let stdin = std::io::stdin();
                let terminal = std::io::IsTerminal::is_terminal(&stdin);
                let n = read_note(&mut stdin.lock(), &mut std::io::stderr(), terminal)?;
                let note = NewNote {
                    at: env.now.clone(),
                    auto,
                    doing: n.doing,
                    next: n.next,
                    watch_out: n.watch_out,
                    cwd: env.cwd.clone(),
                    herdr: herdr::capture(env, &runner),
                    session: session.clone(),
                };
                Ok((store::park(env, &board, id, &note)?, board))
            });
            match parked {
                Ok((parked, board)) => {
                    let card = &parked.card;
                    for w in board.warnings.iter().chain(&parked.warnings) {
                        out.warn(w);
                    }
                    if flag.is_some() {
                        let s = session.as_deref();
                        set_reference(env, s, card.id, &board.name, &runner, &mut out);
                    }
                    #[derive(serde::Serialize)]
                    struct Body<'a> {
                        card: out::CardJson<'a>,
                    }
                    out.result(
                        &format!("parked to {}: {}\n", card.id, card.title),
                        Body {
                            card: out::card_json(card, &board, &env.now),
                        },
                    );
                    0
                }
                Err(e) => out.error(&e),
            }
        }
        _ => todo!(),
    }
}

/// 6.3 and 6.4: with a session, writes its reference (and inside herdr the pane hint)
/// through `core::session`; a failed write is a warning, the command already succeeded.
fn set_reference(
    env: &Env,
    session: Option<&str>,
    card: CardId,
    board: &str,
    runner: &dyn Runner,
    out: &mut Out,
) {
    let Some(session) = session else { return };
    let r = SessionRef {
        card,
        board: board.to_string(),
        pane: herdr::capture(env, runner).map(|h| h.pane),
        cwd: Some(env.cwd.clone()),
        set_at: env.now.clone(),
    };
    if let Err(e) = write_reference(env, session, &r) {
        out.warn(&e.to_string());
    }
}

/// `new` (6.3): the board, template and column names default to the default board, `feature`
/// and the first column; `store::create` resolves the column (`Board::column`).
fn new(
    env: &Env,
    ctx: &Ctx,
    title: &str,
    (board, template, column): (Option<&str>, Option<&str>, Option<&str>),
    stdin_body: bool,
    out: &mut Out,
) -> Result<(Board, store::Created), Error> {
    let board = ctx.board(board)?;
    let template = find_template(ctx, template.unwrap_or("feature"), out)?;
    let stdin = match stdin_body {
        true => Some(split_body(
            &std::io::read_to_string(std::io::stdin()).map_err(|e| Error::Io(e.to_string()))?,
        )),
        false => None,
    };
    let title = match &stdin {
        Some((Some(t), _)) if title.trim().is_empty() => t,
        _ => title,
    };
    let req = CreateRequest {
        title,
        template: &template,
        column,
        body: stdin.as_ref().map(|(_, b)| b.as_str()),
    };
    let created = store::create(env, &board, &req)?;
    Ok((board, created))
}

/// `new --body -` (6.3): a first non-blank line starting with `# ` is the title and is
/// removed; the rest, from its first non-blank line, is the body.
pub fn split_body(text: &str) -> (Option<String>, String) {
    let skip_blank = |t: &str| {
        let end = t.len() - t.trim_start().len();
        t[t[..end].rfind('\n').map_or(0, |i| i + 1)..].to_string()
    };
    let text = skip_blank(text);
    let (first, rest) = text.split_once('\n').unwrap_or((&text, ""));
    match first.strip_prefix("# ") {
        Some(t) => (Some(t.trim().to_string()), skip_blank(rest)),
        None => (None, text.clone()),
    }
}

/// The three parts of a note read from stdin (6.5).
#[derive(Debug, Default)]
pub struct NoteText {
    pub doing: String,
    pub next: String,
    pub watch_out: String,
}

/// The 6.5 grammar: a label line starts its part, other non-blank lines continue the
/// current one (Doing before any label), a repeated label continues its part (A-E2-24).
pub fn parse_note(text: &str) -> NoteText {
    let mut parts = [String::new(), String::new(), String::new()];
    let mut at = 0;
    for line in text.lines() {
        let rest = match label(line) {
            Some((i, rest)) => {
                at = i;
                rest
            }
            None => line,
        }
        .trim();
        if rest.is_empty() {
            continue;
        }
        if !parts[at].is_empty() {
            parts[at].push('\n');
        }
        parts[at].push_str(rest);
    }
    let [doing, next, watch_out] = parts;
    NoteText {
        doing,
        next,
        watch_out,
    }
}

/// A note from `input`: the 6.5 grammar, or on a terminal one verbatim line per prompt
/// written to `prompts`, end of input leaving the rest empty (A-E2-23).
pub fn read_note(
    input: &mut dyn std::io::BufRead,
    prompts: &mut dyn Write,
    terminal: bool,
) -> Result<NoteText, Error> {
    let io = |e: std::io::Error| Error::Io(format!("stdin: {e}"));
    let n = if terminal {
        let mut parts = [String::new(), String::new(), String::new()];
        for (part, prompt) in parts.iter_mut().zip(["Doing: ", "Next: ", "Watch out: "]) {
            let _ = prompts
                .write_all(prompt.as_bytes())
                .and_then(|_| prompts.flush());
            if input.read_line(part).map_err(io)? == 0 {
                break;
            }
            part.truncate(part.trim_end_matches(['\r', '\n']).len());
        }
        let [doing, next, watch_out] = parts;
        NoteText {
            doing,
            next,
            watch_out,
        }
    } else {
        parse_note(&std::io::read_to_string(input).map_err(io)?)
    };
    if [&n.doing, &n.next, &n.watch_out]
        .iter()
        .all(|p| p.trim().is_empty())
    {
        return Err(Error::InvalidInput("empty note".to_string()));
    }
    Ok(n)
}

/// `^\s*-?\s*(\*\*)?(doing|next|watch out|watch)\s*:\s*(\*\*)?\s*(.*)$` in any case, matched
/// on characters: the part index and the captured rest.
fn label(line: &str) -> Option<(usize, &str)> {
    let l = line.trim_start();
    let l = l.strip_prefix('-').unwrap_or(l).trim_start();
    let l = l.strip_prefix("**").unwrap_or(l);
    [(0, "doing"), (1, "next"), (2, "watch out"), (2, "watch")]
        .into_iter()
        .find_map(|(i, word)| {
            let mut cs = l.chars();
            word.chars()
                .all(|w| cs.next().is_some_and(|c| c.to_ascii_lowercase() == w))
                .then_some(())?;
            let rest = cs.as_str().trim_start().strip_prefix(':')?.trim_start();
            Some((i, rest.strip_prefix("**").unwrap_or(rest)))
        })
}

/// `templates` (6.3): header line and skeleton per template, one blank line between;
/// never reads `config.toml`.
pub fn templates(ts: &[Template]) -> String {
    ts.iter()
        .map(|t| {
            let key = t.key.map(|k| format!(" (key {k})")).unwrap_or_default();
            let skeleton = t.skeleton.trim_end_matches('\n');
            format!("template: {}{key}\n{skeleton}\n", t.name)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{parse_note, read_note};
    use crate::core::error::Error;

    fn parts(text: &str) -> [String; 3] {
        let n = parse_note(text);
        [n.doing, n.next, n.watch_out]
    }

    #[test]
    fn ts6_5_labels_start_parts() {
        assert_eq!(parts("Doing: a\nNext: b\nWatch out: c"), ["a", "b", "c"]);
    }

    #[test]
    fn ts6_5_dash_bold_and_spacing_variants() {
        let text = "- Doing: a\n**Next:** b\n  watch out :  c";
        assert_eq!(parts(text), ["a", "b", "c"]);
    }

    #[test]
    fn ts6_5_labels_match_any_case() {
        assert_eq!(parts("DOING: a\nnExT: b\nWatch Out: c"), ["a", "b", "c"]);
    }

    #[test]
    fn ts6_5_watch_alone_is_watch_out() {
        assert_eq!(parts("watch: timeout"), ["", "", "timeout"]);
    }

    #[test]
    fn ts6_5_unlabelled_leading_lines_are_doing() {
        let text = "rerun the next test\nNext: x";
        assert_eq!(parts(text), ["rerun the next test", "x", ""]);
    }

    #[test]
    fn ts6_5_label_word_without_colon_is_text() {
        assert_eq!(parts("next step is x"), ["next step is x", "", ""]);
    }

    #[test]
    fn ts6_5_other_lines_continue_part() {
        assert_eq!(parts("Next: a\nb\n\nc"), ["", "a\nb\nc", ""]);
    }

    #[test]
    fn ts6_5_colon_after_closing_bold_is_text() {
        assert_eq!(parts("**Next**: x"), ["**Next**: x", "", ""]);
    }

    #[test]
    fn ts6_5_repeated_label_continues_part() {
        assert_eq!(parts("Doing: a\nDoing: b"), ["a\nb", "", ""]);
    }

    #[test]
    fn ts6_5_non_ascii_text_kept() {
        assert_eq!(parts("Doing: über\nwätch: x"), ["über\nwätch: x", "", ""]);
    }

    fn read(input: &str, terminal: bool) -> (Result<[String; 3], Error>, String) {
        let mut prompts = Vec::new();
        let n = read_note(&mut input.as_bytes(), &mut prompts, terminal);
        let n = n.map(|n| [n.doing, n.next, n.watch_out]);
        (n, String::from_utf8(prompts).unwrap())
    }

    #[test]
    fn fr34_terminal_prompts_for_each_part() {
        let (n, prompts) = read("a\nb\nc\n", true);
        assert_eq!(prompts, "Doing: Next: Watch out: ");
        assert_eq!(n.unwrap(), ["a", "b", "c"]);
    }

    #[test]
    fn fr34_terminal_answer_taken_verbatim() {
        let (n, _) = read("next: x\n\n\n", true);
        assert_eq!(n.unwrap(), ["next: x", "", ""]);
    }

    #[test]
    fn fr34_terminal_end_of_input_stops_prompting() {
        let (n, prompts) = read("a\n", true);
        assert_eq!(prompts, "Doing: Next: ");
        assert_eq!(n.unwrap(), ["a", "", ""]);
    }

    #[test]
    fn fr18_terminal_three_empty_answers_rejected() {
        let (n, _) = read("\n\n\n", true);
        assert_eq!(n, Err(Error::InvalidInput("empty note".to_string())));
    }

    #[test]
    fn fr34_non_terminal_parses_without_prompts() {
        let (n, prompts) = read("Next: b\n", false);
        assert_eq!(prompts, "");
        assert_eq!(n.unwrap(), ["", "b", ""]);
    }
}
