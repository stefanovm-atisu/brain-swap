//! The 6.1 and 6.3 command line surface.

use crate::core::model::CardId;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

fn card_id(s: &str) -> Result<CardId, String> {
    CardId::parse(s).ok_or_else(|| format!("invalid card ID '{s}'"))
}

/// The TECHSPEC 6.2 exit code table, also in README.
const EXIT_CODES: &str = "Exit codes:
  0 success
  1 runtime error
  2 usage
  3 not found
  4 no reference
  5 busy
  6 jump not performed";

#[derive(Parser, Debug)]
#[command(
    name = "brain-swap",
    version,
    about = "Keyboard-first kanban with switch notes",
    after_help = EXIT_CODES
)]
pub struct Cli {
    /// Board for commands without a card ID (default: default_board)
    #[arg(long, global = true)]
    pub board: Option<String>,
    /// Print one JSON object
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Command {
    /// Create a card
    New {
        #[arg(long)]
        template: Option<String>,
        #[arg(long)]
        column: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long, value_parser = ["-"])]
        body: Option<String>,
        title: Vec<String>,
    },
    /// Append a switch note to a card
    Park {
        #[arg(long, value_parser = card_id)]
        card: Option<CardId>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        auto: bool,
    },
    /// Print the re-entry context of a session
    Context {
        #[arg(long)]
        session: Option<String>,
    },
    /// Print a card
    Show {
        #[arg(value_parser = card_id)]
        id: CardId,
        #[arg(long)]
        all: bool,
    },
    /// List cards
    Ls {
        #[arg(long)]
        open: bool,
        #[arg(long)]
        guess: bool,
    },
    /// Link a session to a card
    Link {
        #[arg(value_parser = card_id)]
        id: CardId,
        #[arg(long)]
        session: Option<String>,
    },
    /// Move a card to a column
    Move {
        #[arg(value_parser = card_id)]
        id: CardId,
        column: String,
    },
    /// Report where a card's pane is
    Locate {
        #[arg(value_parser = card_id)]
        id: CardId,
        /// Locate note <n> (1 is the oldest) instead of the latest note with a place
        #[arg(long)]
        note: Option<u32>,
    },
    /// Focus a card's pane
    Jump {
        #[arg(value_parser = card_id)]
        id: CardId,
        /// Jump to note <n> (1 is the oldest) instead of the latest note with a place
        #[arg(long)]
        note: Option<u32>,
    },
    /// List card templates
    Templates,
    /// Install an adapter
    #[command(subcommand)]
    Install(InstallTarget),
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum InstallTarget {
    /// Install the Claude Code pack
    Claude {
        #[arg(long)]
        link: Option<PathBuf>,
        /// Replace entries at the skill paths that brain-swap did not create
        #[arg(long)]
        force: bool,
        #[arg(long)]
        remove: bool,
    },
    /// Print the herdr key binding to paste; edits nothing
    Herdr,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(form: &str) -> Cli {
        let argv = std::iter::once("brain-swap").chain(form.split(' '));
        Cli::try_parse_from(argv).unwrap()
    }

    fn id(s: &str) -> CardId {
        CardId::parse(s).unwrap()
    }

    fn s(v: &str) -> Option<String> {
        Some(v.to_string())
    }

    #[test]
    fn ts6_1_every_subcommand_parses() {
        let cases = [
            (
                "new --template bug --column Doing --session s --body - fix it",
                Command::New {
                    template: s("bug"),
                    column: s("Doing"),
                    session: s("s"),
                    body: s("-"),
                    title: vec!["fix".into(), "it".into()],
                },
            ),
            (
                "park --card W-12 --session s --auto",
                Command::Park {
                    card: Some(id("W-12")),
                    session: s("s"),
                    auto: true,
                },
            ),
            ("context --session s", Command::Context { session: s("s") }),
            (
                "show w-12 --all",
                Command::Show {
                    id: CardId {
                        letter: 'W',
                        number: 12,
                    },
                    all: true,
                },
            ),
            (
                "ls --open --guess",
                Command::Ls {
                    open: true,
                    guess: true,
                },
            ),
            (
                "link W-12 --session s",
                Command::Link {
                    id: id("W-12"),
                    session: s("s"),
                },
            ),
            (
                "move W-12 Doing",
                Command::Move {
                    id: id("W-12"),
                    column: "Doing".into(),
                },
            ),
            (
                "locate W-12 --note 2",
                Command::Locate {
                    id: id("W-12"),
                    note: Some(2),
                },
            ),
            (
                "jump W-12 --note 1",
                Command::Jump {
                    id: id("W-12"),
                    note: Some(1),
                },
            ),
            ("templates", Command::Templates),
            (
                "install claude --link /r --force",
                Command::Install(InstallTarget::Claude {
                    link: Some("/r".into()),
                    force: true,
                    remove: false,
                }),
            ),
            (
                "install claude --remove",
                Command::Install(InstallTarget::Claude {
                    link: None,
                    force: false,
                    remove: true,
                }),
            ),
            ("install herdr", Command::Install(InstallTarget::Herdr)),
        ];
        for (form, want) in cases {
            assert_eq!(parse(form).command, Some(want), "{form}");
        }
    }

    #[test]
    fn ts6_1_board_and_json_are_global() {
        for form in ["ls --board home --json", "--board home --json ls"] {
            let cli = parse(form);
            assert_eq!(cli.board.as_deref(), Some("home"), "{form}");
            assert!(cli.json, "{form}");
        }
    }
}
