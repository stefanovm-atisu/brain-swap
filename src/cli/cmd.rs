//! One function per subcommand (TECHSPEC 2.1).

use crate::cli::args::{Cli, Command};
use crate::core::env::Env;
use crate::core::template;

/// Dispatches one subcommand and returns its 6.2 exit code.
pub fn run(env: &mut Env, cli: Cli) -> i32 {
    match cli.command {
        Some(Command::Templates) => {
            print!("{}", templates(env));
            0
        }
        _ => todo!(),
    }
}

/// `templates` (6.3): header line and skeleton per template, one blank line between;
/// never reads `config.toml`.
pub fn templates(env: &Env) -> String {
    let (ts, _warnings) = template::load(&env.templates_dir());
    ts.iter()
        .map(|t| {
            let key = t.key.map(|k| format!(" (key {k})")).unwrap_or_default();
            let skeleton = t.skeleton.trim_end_matches('\n');
            format!("template: {}{key}\n{skeleton}\n", t.name)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
