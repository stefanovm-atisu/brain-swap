use brain_swap::cli::args::Cli;
use brain_swap::core::env::Env;
use brain_swap::core::error::{Error, Result};
use clap::Parser;
use jiff::Timestamp;
use jiff::tz::TimeZone;
use std::io::IsTerminal;

fn main() {
    let cli = Cli::try_parse_from(std::env::args_os()).unwrap_or_else(|e| e.exit());
    let code = match env() {
        Err(e) => brain_swap::cli::out::Out::new(cli.json).error(&e),
        Ok(env) if cli.command.is_none() => brain_swap::tui::run(
            env,
            cli.board,
            std::io::stdin().is_terminal() && std::io::stdout().is_terminal(),
        ),
        Ok(mut env) => brain_swap::cli::run(&mut env, cli),
    };
    std::process::exit(code);
}

/// The only place that reads the process environment (TECHSPEC 6.1).
fn env() -> Result<Env> {
    let cwd = std::env::current_dir().map_err(|e| Error::Io(format!("current dir: {e}")))?;
    let tz = TimeZone::system();
    let clock = Timestamp::now().to_zoned(tz.clone());
    // A name or value that is not UTF-8 is dropped instead of panicking (a Latin-1 OLDPWD).
    let vars = std::env::vars_os()
        .filter_map(|(k, v)| Some((k.into_string().ok()?, v.into_string().ok()?)));
    let mut env = Env::from_vars(vars, cwd, std::process::id(), tz, clock)?;
    env.exe = std::env::current_exe().and_then(std::fs::canonicalize).ok();
    Ok(env)
}
