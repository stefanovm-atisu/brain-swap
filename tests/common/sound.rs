//! `assert_board_sound`: the TECHSPEC 12.1 crash check, shared by the in-process and process layers.

use brain_swap::core::board::load;
use brain_swap::core::board_file;
use brain_swap::core::env::Env;
use brain_swap::core::model::{Board, CardId};
use brain_swap::core::store::{CreateRequest, create};
use brain_swap::core::template::{builtins, find};
use std::collections::HashSet;
use std::fs;

/// Reloads `board` with `env` (no failpoint) and asserts that every card file parses without
/// warnings, I1 to I4 hold on disk, and a following create gets a number above every number in
/// `used` and on disk; returns the created ID.
pub fn assert_board_sound(env: &Env, board: &Board, used: &[u32]) -> CardId {
    let b = load(&board.name, &board.path, env).unwrap();
    assert!(b.warnings.is_empty(), "warnings: {:?}", b.warnings);
    let bf = board_file::parse(&b.name, fs::read(b.path.join("board.md")).ok().as_deref());
    assert!(bf.letter.is_ascii_uppercase(), "I1: letter {}", bf.letter);
    assert!(!bf.columns.is_empty(), "I3: no columns");
    let lower: HashSet<String> = bf.columns.iter().map(|c| c.to_lowercase()).collect();
    assert_eq!(lower.len(), bf.columns.len(), "I3: {:?}", bf.columns);
    for c in &b.cards {
        assert!(c.writable, "{} does not parse", c.id);
        assert_eq!(c.id.letter, bf.letter, "I1: {}", c.id);
        assert!(c.id.number >= 1, "I1: {}", c.id);
        assert_eq!(c.path, b.path.join(format!("{}.md", c.id)), "I2");
        assert!(
            bf.next.unwrap_or(1) > c.id.number,
            "I4: next {:?} {}",
            bf.next,
            c.id
        );
    }
    let seen = used
        .iter()
        .chain(b.cards.iter().map(|c| &c.id.number))
        .max()
        .copied()
        .unwrap_or(0);
    let template = find(&builtins(), "Feature").unwrap().clone();
    let req = CreateRequest {
        title: "after the crash",
        template: &template,
        column: None,
        body: None,
    };
    let id = create(env, &b, &req).unwrap().card.id;
    assert!(id.number > seen, "{id} reuses a number up to {seen}");
    id
}
