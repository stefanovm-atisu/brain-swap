//! brain-swap: module layout of TECHSPEC 2.1, layering of 2.2.

pub mod core {
    pub mod board;
    pub mod board_file;
    pub mod card_file;
    pub mod config;
    pub mod env;
    pub mod error;
    pub mod failpoint;
    pub mod frontmatter;
    pub mod guess;
    pub mod keys;
    pub mod log;
    pub mod model;
    pub mod session;
    pub mod store;
    pub mod template;
    pub mod time;
}

pub mod cli {
    pub mod args;
    pub mod cmd;
    pub mod out;
    pub use cmd::{parse_failure, run};
}

pub mod tui {
    pub mod app;
    pub mod editor;
    pub mod runtime;
    pub mod view_board;
    pub mod view_detail;
    pub mod view_frame;
    pub mod view_help;
    pub mod view_message;
    pub mod view_pick;

    /// What `Runtime::new` starts from.
    pub struct Prepared {
        pub app: app::App,
        pub config: Option<crate::core::config::Config>,
        pub templates: Vec<crate::core::template::Template>,
    }

    /// The checked bare invocation; a config error is carried for ConfigError (FR-45).
    #[derive(Debug)]
    pub struct TuiStart {
        pub config: crate::core::error::Result<crate::core::config::Loaded>,
        pub board: Option<String>,
    }

    /// The bare invocation, checked in the order of A-E4-27: config, `--board`, terminal.
    pub fn gate(
        env: &crate::core::env::Env,
        board: Option<&str>,
        terminal: bool,
    ) -> crate::core::error::Result<TuiStart> {
        let config = crate::core::config::load(env);
        let board = match (&config, board) {
            (Ok(l), Some(b)) => Some(l.config.board(b)?.0.to_string()),
            (_, b) => b.map(String::from),
        };
        if !terminal {
            return Err(crate::core::error::Error::Usage("not a terminal".into()));
        }
        Ok(TuiStart { config, board })
    }

    /// The app to start: ConfigError, or the board with its warnings (7.1, 11, A-E4-08).
    pub fn prepare(
        env: &mut crate::core::env::Env,
        start: TuiStart,
    ) -> crate::core::error::Result<Prepared> {
        let loaded = match start.config {
            Ok(l) => l,
            Err(e) => {
                return Ok(Prepared {
                    app: app::App::config_error(e.to_string(), env.now.clone()),
                    config: None,
                    templates: Vec::new(),
                });
            }
        };
        let config = loaded.config;
        let guard = crate::core::log::guard_board_folders(env, &config.boards);
        let (keys, key_warnings) = crate::core::keys::KeyMap::build(&config.keys);
        let (name, dir) = config.board(start.board.as_deref().unwrap_or(&config.default_board))?;
        let board = crate::core::board::load(name, dir, env)?;
        let (mut templates, template_warnings) = crate::core::template::load(&env.templates_dir());
        let hotkeys: Vec<_> = templates
            .iter()
            .filter_map(|t| Some((t.name.clone(), t.key?)))
            .collect();
        let (kept, hotkey_warnings) = keys.check_template_hotkeys(&hotkeys);
        for t in &mut templates {
            t.key = t.key.filter(|&c| kept.contains(&(t.name.clone(), c)));
        }
        let mut app = app::App::new(board, keys, env.now.clone(), env.home.clone());
        app.templates = templates.iter().map(|t| (t.name.clone(), t.key)).collect();
        app.boards = config.boards.clone();
        app.warnings = loaded
            .warnings
            .into_iter()
            .chain(guard)
            .chain(key_warnings)
            .chain(template_warnings)
            .chain(hotkey_warnings)
            .collect();
        app.status = match loaded.created {
            Some(p) => Some(format!("created config {}", p.display())),
            None => app
                .warnings
                .iter()
                .chain(&app.board.warnings)
                .next()
                .cloned(),
        };
        Ok(Prepared {
            app,
            config: Some(config),
            templates,
        })
    }

    /// `gate` and `prepare`, then the runtime on the real terminal (6.3, 7.5); an error
    /// before it prints on stderr only, with the screen untouched.
    pub fn run(mut env: crate::core::env::Env, board: Option<String>, terminal: bool) -> i32 {
        use runtime::{
            CrosstermEvents, CrosstermGuard, Parts, Runtime, SystemClock, TerminalGuard,
        };
        let prepared =
            match gate(&env, board.as_deref(), terminal).and_then(|s| prepare(&mut env, s)) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("brain-swap: {e}");
                    return e.exit_code();
                }
            };
        // The default hook prints the panic; leave the alternate screen first so it stays visible.
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = CrosstermGuard.leave();
            previous(info);
        }));
        let terminal = match ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(
            std::io::stdout(),
        )) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("brain-swap: terminal: {e}");
                return 1;
            }
        };
        let parts = Parts {
            guard: Box::new(CrosstermGuard),
            events: Box::new(CrosstermEvents),
            clock: Box::new(SystemClock { tz: env.tz.clone() }),
            launcher: Box::new(editor::ProcessLauncher),
        };
        Runtime::new(env, prepared, terminal, parts).run()
    }
}

pub mod adapters {
    pub mod claude;
    pub mod herdr;
    pub mod runner;
}
