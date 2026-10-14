//! One function per subcommand (TECHSPEC 2.1).

use crate::adapters::claude::{self, InstallOpts};
use crate::adapters::herdr;
use crate::adapters::runner::{ProcessRunner, Runner};
use crate::cli::args::{Cli, Command, InstallTarget};
use crate::cli::out::{self, Out};
use crate::core::config::{self, Config};
use crate::core::env::Env;
use crate::core::error::{EXIT_JUMP_NOT_PERFORMED, Error};
use crate::core::guess;
use crate::core::model::{Board, Card, CardId, NewNote, SessionRef};
use crate::core::session::{read_reference, write_reference};
use crate::core::store::{self, CreateRequest};
use crate::core::template::{self, Template};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

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
                first_run(&c, &mut out);
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
                    let (text, cards) = picker(env, &ctx, &board);
                    #[derive(serde::Serialize)]
                    struct Body<'a> {
                        board: &'a str,
                        cards: Vec<out::CandidateJson<'a>>,
                        other_boards: Vec<&'a str>,
                    }
                    out.result(
                        &text,
                        Body {
                            board: &board.name,
                            cards,
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
        Some(Command::Context { session }) => {
            context(env, (cli.board.as_deref(), session.as_deref()), &mut out)
        }
        Some(Command::Link { id, session }) => link(
            env,
            &ctx.expect("loaded above"),
            id,
            session.as_deref(),
            &mut out,
        ),
        Some(Command::Locate { id, note }) => {
            locate(env, &ctx.expect("loaded above"), id, note, &mut out)
        }
        Some(Command::Jump { id, note }) => {
            jump(env, &ctx.expect("loaded above"), id, note, &mut out)
        }
        Some(Command::Install(InstallTarget::Herdr)) => install_herdr(env, &mut out),
        Some(Command::Install(InstallTarget::Claude {
            link,
            force,
            remove: false,
        })) => {
            let exe = env.exe.clone();
            install_claude(env, &InstallOpts { link, force, exe }, &mut out)
        }
        _ => todo!(),
    }
}

/// `install claude` (8.8): one `installed <folder>` line per skill, then the notices; with
/// `--json` the 6.7 object on stdout and the notices as warnings on stderr.
pub fn install_claude(env: &Env, opts: &InstallOpts, out: &mut Out) -> i32 {
    let report = match claude::install(env, opts) {
        Ok(r) => r,
        Err(e) => return out.error(&e),
    };
    let mut text = String::new();
    for p in &report.linked {
        text += &format!("installed {}\n", p.display());
    }
    for n in &report.notices {
        match out.json {
            true => out.warn(n),
            false => text += &format!("{n}\n"),
        }
    }
    #[derive(serde::Serialize)]
    struct Body<'a> {
        linked: &'a [PathBuf],
        removed: &'a [PathBuf],
    }
    let (linked, removed) = (&report.linked, &report.removed);
    out.result(&text, Body { linked, removed });
    0
}

/// `install herdr` (9.4): prints the key binding for the canonical binary path; reads no
/// config, writes no file and runs no herdr command (FR-64).
pub fn install_herdr(env: &Env, out: &mut Out) -> i32 {
    let Some(exe) = &env.exe else {
        return out.error(&Error::Io(
            "cannot determine the brain-swap binary path".into(),
        ));
    };
    let snippet = herdr::key_snippet(&herdr_command(env, exe));
    out.result(
        &format!("{snippet}\nthen run: herdr server reload-config\n"),
        out::SnippetJson { snippet: &snippet },
    );
    0
}

/// The popup command of 9.4: the SP-2 fallback `env` prefix before the binary path.
/// A path outside a plain character set is single-quoted, since herdr may run the command
/// through a shell (SP-2 not run).
pub fn herdr_command(env: &Env, exe: &Path) -> String {
    let word = |p: &Path| {
        let s = p.display().to_string();
        if s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-+,:@%".contains(&b))
        {
            s
        } else {
            format!("'{}'", s.replace('\'', r"'\''"))
        }
    };
    let herdr = match &env.herdr_bin_path {
        Some(p) if !p.as_os_str().is_empty() => word(p),
        _ => "herdr".to_string(),
    };
    format!("env HERDR_ENV=1 HERDR_BIN_PATH={herdr} {}", word(exe))
}

/// The note `locate` and `jump` target (6.3): the jump note, or note `n` counted from the
/// oldest; `n` outside 1 to the note count is `NotFound`.
pub fn target_note(card: &Card, note: Option<u32>) -> Result<Option<usize>, Error> {
    match note {
        None => Ok(card.jump_note()),
        Some(n) if (1..=card.notes.len()).contains(&(n as usize)) => Ok(Some(n as usize - 1)),
        Some(_) => Err(Error::NotFound(format!(
            "{} has {} notes",
            card.id,
            card.notes.len()
        ))),
    }
}

/// `locate` (6.3): the target note's pane status and its `Where:` line, or `no note yet`;
/// exit 0 for every status.
pub fn locate(env: &Env, ctx: &Ctx, id: CardId, note: Option<u32>, out: &mut Out) -> i32 {
    let (board, card) = match find_card(ctx, id) {
        Ok(found) => found,
        Err(e) => return out.error(&e),
    };
    board.warnings.iter().for_each(|w| out.warn(w));
    let i = match target_note(&card, note) {
        Ok(Some(i)) => i,
        Ok(None) => {
            let json = out::LocateJson {
                status: "no_note",
                pane: None,
                cwd: None,
            };
            out.result("no note yet\n", json);
            return 0;
        }
        Err(e) => return out.error(&e),
    };
    let place = &card.notes[i].place;
    let status = herdr::locate(place, env, &ProcessRunner::new(env));
    let text = out::locate_lines(&status, place);
    out.result(&text, out::LocateJson::new(&status, place));
    0
}

/// `jump` (6.3): `locate`, then `focus` a live or moved pane within one shared 3 second
/// budget; exit 0 when focused, else the `locate` lines and exit 6.
pub fn jump(env: &Env, ctx: &Ctx, id: CardId, note: Option<u32>, out: &mut Out) -> i32 {
    let (board, card) = match find_card(ctx, id) {
        Ok(found) => found,
        Err(e) => return out.error(&e),
    };
    board.warnings.iter().for_each(|w| out.warn(w));
    let i = match target_note(&card, note) {
        Ok(Some(i)) => i,
        Ok(None) => {
            let locate = out::LocateJson {
                status: "no_note",
                pane: None,
                cwd: None,
            };
            let json = out::JumpJson {
                locate,
                focused: None,
            };
            out.result("no note yet\n", json);
            return EXIT_JUMP_NOT_PERFORMED;
        }
        Err(e) => return out.error(&e),
    };
    let place = &card.notes[i].place;
    let process = ProcessRunner::new(env);
    let budget = herdr::Budget::new(&process, herdr::BUDGET);
    let mut status = herdr::locate(place, env, &budget);
    let focus = match &status {
        herdr::PaneStatus::Live(p) | herdr::PaneStatus::Moved(p) => Some(herdr::focus(p, &budget)),
        _ => None,
    };
    if let Some(Err(herdr::FocusError::NotResponding)) = focus {
        status = herdr::PaneStatus::NotResponding;
    }
    let lines = out::locate_lines(&status, place);
    let text = match (&status, focus) {
        (herdr::PaneStatus::Moved(p), Some(Ok(_))) => format!("pane moved: now {}\n", p.pane),
        (herdr::PaneStatus::Live(p), Some(Ok(herdr::Focused::Agent))) => {
            format!("jumped to {} (pane {})\n", card.id, p.pane)
        }
        (herdr::PaneStatus::Live(p), Some(Ok(herdr::Focused::Tab))) => {
            format!(
                "jumped to {} (tab {}; pane not focusable)\n",
                card.id, p.tab
            )
        }
        // Keep the `Where:` line of the locate lines.
        (herdr::PaneStatus::Live(p) | herdr::PaneStatus::Moved(p), Some(Err(_))) => {
            format!(
                "focus failed: {}\n{}",
                p.pane,
                lines.split_once('\n').unwrap().1
            )
        }
        _ => lines,
    };
    let focused = focus.and_then(Result::ok);
    let locate = out::LocateJson::new(&status, place);
    out.result(&text, out::JumpJson { locate, focused });
    match focused {
        Some(_) => 0,
        None => EXIT_JUMP_NOT_PERFORMED,
    }
}

/// `created config <path>` on stderr after the first run, a warning under `--json` (10).
fn first_run(ctx: &Ctx, out: &mut Out) {
    if let Some(p) = &ctx.created {
        let notice = format!("created config {}", p.display());
        if out.json {
            out.warn(&notice);
        } else {
            let _ = writeln!(out.stderr, "{notice}");
        }
    }
}

/// The 6.7 `context` body.
#[derive(serde::Serialize)]
struct ContextJson<'a> {
    session: Option<&'a str>,
    reference: Option<String>,
    card: Option<out::CardJson<'a>>,
    candidates: Vec<out::CandidateJson<'a>>,
    other_boards: Vec<&'a str>,
    problem: Option<Problem<'a>>,
    warnings: Vec<String>,
}

#[derive(serde::Serialize)]
struct Problem<'a> {
    code: &'a str,
    message: String,
    hint: String,
}

/// The `context` header lines: version, session and one `warning:` line per warning.
fn context_head(session: Option<&str>, warnings: &[String]) -> String {
    let mut text = format!(
        "brain-swap context v1\nsession: {}\n",
        session.unwrap_or("none")
    );
    for w in warnings {
        text += &format!("warning: {w}\n");
    }
    text
}

/// `context` ending in `error:` and `hint:` lines (6.3), JSON `problem`; exit 0 (6.2).
fn context_problem(
    out: &mut Out,
    session: Option<&str>,
    warnings: Vec<String>,
    e: &Error,
    hint: String,
) -> i32 {
    let text = format!(
        "{}error: {e}\nhint: {hint}\n",
        context_head(session, &warnings)
    );
    let body = ContextJson {
        session,
        reference: None,
        card: None,
        candidates: Vec::new(),
        other_boards: Vec::new(),
        problem: Some(Problem {
            code: e.code(),
            message: e.to_string(),
            hint,
        }),
        warnings,
    };
    out.result(&text, body);
    0
}

/// The `hint:` of a board that failed to load: an unknown name, else its folder (6.3).
fn board_hint(ctx: &Ctx, name: &str, e: &Error) -> String {
    match e {
        Error::NotFound(_) => "use one of the listed boards".to_string(),
        _ => {
            let n = ctx.config.board(name).map_or(name, |(n, _)| n);
            let path = ctx.config.path.display();
            format!("check the folder of board {n} in {path}")
        }
    }
}

/// `context` (6.3, 6.7): the reference's card block, else the picker; config, session and
/// board warnings become `warning:` lines and expected problems `error:` lines; exit 0.
fn context(env: &mut Env, (name, flag): (Option<&str>, Option<&str>), out: &mut Out) -> i32 {
    let loaded = Ctx::load(env);
    let runner = ProcessRunner::new(env);
    let (session, mut warnings) = resolve_session(env, flag, &runner);
    let ctx = match loaded {
        Ok(ctx) => ctx,
        Err(e) => {
            let hint = format!(
                "fix {}, then run the command again",
                env.config_file().display()
            );
            return context_problem(out, session.as_deref(), warnings, &e, hint);
        }
    };
    first_run(&ctx, out);
    warnings.splice(0..0, ctx.warnings.iter().cloned());
    let reference = session.as_deref().and_then(|s| read_reference(env, s));
    // An unconfigured reference board reads as a missing card (I9), an unreadable one is
    // a problem.
    let mut found = None;
    if let Some(r) = &reference {
        match ctx.board(Some(&r.board)) {
            Ok(board) => found = board.card(r.card).cloned().map(|c| (board, c)),
            Err(Error::NotFound(_)) => {}
            Err(e) => {
                let hint = board_hint(&ctx, &r.board, &e);
                return context_problem(out, session.as_deref(), warnings, &e, hint);
            }
        }
    }
    let Some((board, card)) = found else {
        let board = match ctx.board(name) {
            Ok(board) => board,
            Err(e) => {
                let hint = board_hint(&ctx, name.unwrap_or(&ctx.config.default_board), &e);
                return context_problem(out, session.as_deref(), warnings, &e, hint);
            }
        };
        warnings.extend(board.warnings.iter().cloned());
        let head = context_head(session.as_deref(), &warnings);
        let reference = match reference {
            Some(r) => {
                warnings.push(format!("{} not found", r.card));
                format!("none ({} not found)", r.card)
            }
            None => "none".to_string(),
        };
        let (lines, mut candidates) = picker(env, &ctx, &board);
        for n in candidates
            .iter_mut()
            .filter_map(|c| c.card.latest_note.as_mut())
        {
            n.by_this_session = n.place.session.map(|s| Some(s) == session.as_deref());
        }
        let body = ContextJson {
            session: session.as_deref(),
            reference: None,
            card: None,
            candidates,
            other_boards: out::other_boards(&ctx, &board.name),
            problem: None,
            warnings,
        };
        out.result(&format!("{head}reference: {reference}\n{lines}"), body);
        return 0;
    };
    warnings.extend(board.warnings.iter().cloned());
    let view = out::SessionView::Resolved(session.clone());
    let text = format!(
        "{}reference: {}\n{}",
        context_head(session.as_deref(), &warnings),
        card.id,
        out::card_block(&card, &board, &env.now, &view)
    );
    let mut card_json = out::card_json(&card, &board, &env.now);
    if let Some(n) = &mut card_json.latest_note {
        n.by_this_session = n.place.session.map(|s| Some(s) == session.as_deref());
    }
    let body = ContextJson {
        session: session.as_deref(),
        reference: Some(card.id.to_string()),
        card: Some(card_json),
        candidates: Vec::new(),
        other_boards: Vec::new(),
        problem: None,
        warnings,
    };
    out.result(&text, body);
    0
}

/// `open cards on <board>, best guess first:`, the 6.6 picker lines and the other boards
/// line (6.3) with their 6.7 Candidates; shared by `ls --guess` and `context`.
fn picker<'a>(env: &Env, ctx: &'a Ctx, board: &'a Board) -> (String, Vec<out::CandidateJson<'a>>) {
    let cands = guess::best_guess(board, &guess::GuessInput::from_env(env));
    let mut text = format!("open cards on {}, best guess first:\n", board.name);
    for (i, c) in cands.iter().enumerate() {
        text += &out::picker_line(i + 1, c.card, c.tier.label(), &env.now);
        text.push('\n');
    }
    if let Some(l) = out::other_boards_line(ctx, &board.name) {
        text += &l;
        text.push('\n');
    }
    let json = cands.iter().enumerate().map(|(i, c)| out::CandidateJson {
        card: out::card_json(c.card, board, &env.now),
        rank: i + 1,
        reason: c.tier.reason(),
    });
    (text, json.collect())
}

/// `link` (6.3): an unknown card or no session changes nothing; otherwise the reference
/// is set and the block printed against the resolved session.
fn link(env: &Env, ctx: &Ctx, id: CardId, flag: Option<&str>, out: &mut Out) -> i32 {
    let (board, card) = match find_card(ctx, id) {
        Ok(found) => found,
        Err(e) => return out.error(&e),
    };
    board.warnings.iter().for_each(|w| out.warn(w));
    let runner = ProcessRunner::new(env);
    let (session, warnings) = resolve_session(env, flag, &runner);
    warnings.iter().for_each(|w| out.warn(w));
    let Some(session) = session else {
        return out.error(&Error::InvalidInput("no session".into()));
    };
    set_reference(env, Some(&session), id, &board.name, &runner, out);
    #[derive(serde::Serialize)]
    struct Body<'a> {
        card: out::CardJson<'a>,
    }
    let mut json = out::card_json(&card, &board, &env.now);
    if let Some(n) = &mut json.latest_note {
        n.by_this_session = n.place.session.map(|s| s == session);
    }
    let view = out::SessionView::Resolved(Some(session.clone()));
    let text = format!("linked to {id}: {}\n", card.title)
        + &out::card_block(&card, &board, &env.now, &view);
    out.result(&text, Body { card: json });
    0
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
    use super::{herdr_command, parse_note, read_note};
    use crate::core::env::Env;
    use crate::core::error::Error;
    use jiff::tz::TimeZone;
    use std::path::Path;

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

    #[test]
    fn ts9_4_env_prefix_without_herdr_bin_path_uses_herdr() {
        let vars = [("HOME".to_string(), "/h".to_string())];
        let now = jiff::Timestamp::UNIX_EPOCH.to_zoned(TimeZone::UTC);
        let env = Env::from_vars(vars, "/cwd".into(), 1, TimeZone::UTC, now).unwrap();
        assert_eq!(env.herdr_bin_path, None);
        assert_eq!(
            herdr_command(&env, Path::new("/x/brain-swap")),
            "env HERDR_ENV=1 HERDR_BIN_PATH=herdr /x/brain-swap"
        );
    }

    #[test]
    fn ts9_4_env_prefix_quotes_a_path_with_spaces() {
        let vars = [
            ("HOME".to_string(), "/h".to_string()),
            ("HERDR_BIN_PATH".to_string(), "/a b/herdr".to_string()),
        ];
        let now = jiff::Timestamp::UNIX_EPOCH.to_zoned(TimeZone::UTC);
        let env = Env::from_vars(vars, "/cwd".into(), 1, TimeZone::UTC, now).unwrap();
        assert_eq!(
            herdr_command(&env, Path::new("/x/it's/brain-swap")),
            r"env HERDR_ENV=1 HERDR_BIN_PATH='/a b/herdr' '/x/it'\''s/brain-swap'"
        );
    }
}
