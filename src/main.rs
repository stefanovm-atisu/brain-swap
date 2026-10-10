use brain_swap::core::env::Env;
use brain_swap::core::error::{Error, Result};
use jiff::Timestamp;
use jiff::tz::TimeZone;

fn main() {
    if let Err(e) = run() {
        eprintln!("brain-swap: {e}");
        std::process::exit(e.exit_code());
    }
}

/// The only place that reads the process environment (TECHSPEC 6.1).
fn run() -> Result<()> {
    let cwd = std::env::current_dir().map_err(|e| Error::Io(format!("current dir: {e}")))?;
    let tz = TimeZone::system();
    let clock = Timestamp::now().to_zoned(tz.clone());
    // A name or value that is not UTF-8 is dropped instead of panicking (a Latin-1 OLDPWD).
    let vars = std::env::vars_os()
        .filter_map(|(k, v)| Some((k.into_string().ok()?, v.into_string().ok()?)));
    let mut env = Env::from_vars(vars, cwd, std::process::id(), tz, clock)?;
    env.exe = std::env::current_exe().and_then(std::fs::canonicalize).ok();
    let _ = env;
    Ok(())
}
