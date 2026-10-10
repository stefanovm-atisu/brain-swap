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
}

pub mod tui {
    pub mod app;
    pub mod editor;
    pub mod runtime;
}

pub mod adapters {
    pub mod claude;
    pub mod herdr;
    pub mod runner;
}
