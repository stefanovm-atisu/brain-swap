//! One function per subcommand (TECHSPEC 2.1).

use crate::cli::args::{Cli, Command};
use crate::cli::out::{self, Out};
use crate::core::config::{self, Config};
use crate::core::env::Env;
use crate::core::error::Error;
use crate::core::model::Board;
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
        Some(Command::New { title, .. }) => {
            let ctx = ctx.expect("loaded above");
            match new(env, &ctx, &title.join(" "), &mut out) {
                Ok((board, created)) => {
                    let card = &created.card;
                    for w in board.warnings.iter().chain(&created.warnings) {
                        out.warn(w);
                    }
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
                            card: out::card_json(card, &board),
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
        _ => todo!(),
    }
}

/// `new` (6.3): a `feature` card in the first column of the default board.
fn new(env: &Env, ctx: &Ctx, title: &str, out: &mut Out) -> Result<(Board, store::Created), Error> {
    let board = ctx.board(None)?;
    let template = find_template(ctx, "feature", out)?;
    let req = CreateRequest {
        title,
        template: &template,
        column: None,
        body: None,
    };
    let created = store::create(env, &board, &req)?;
    Ok((board, created))
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
