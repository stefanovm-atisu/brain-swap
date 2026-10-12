//! One function per subcommand (TECHSPEC 2.1).

use crate::cli::args::{Cli, Command};
use crate::cli::out::Out;
use crate::core::env::Env;
use crate::core::template::{self, Template};

/// Dispatches one subcommand and returns its 6.2 exit code.
pub fn run(env: &mut Env, cli: Cli) -> i32 {
    let mut out = Out::new(cli.json);
    match cli.command {
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
