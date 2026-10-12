//! TECHSPEC 4.3, 4.6 and R4: creating a card inside the board lock.

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use brain_swap::core::board::load;
use brain_swap::core::env::Env;
use brain_swap::core::error::{Error, Result};
use brain_swap::core::store::{CreateRequest, Created, create, link_error};
use brain_swap::core::template::{Template, builtins, find};
use brain_swap::core::time::parse_stamp;
use common::bytes::assert_only_lines_changed;
use common::env::core_env;
use jiff::tz::TimeZone;

const NOW: &str = "2026-10-08T11:52:00+03:00";
const WORK_4_9: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/boards/work_4_9"
);

/// A temp home holding a copy of `work_4_9` at `<home>/work`.
fn copy() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("work");
    fs::create_dir(&dir).unwrap();
    for e in fs::read_dir(WORK_4_9).unwrap() {
        let e = e.unwrap();
        fs::copy(e.path(), dir.join(e.file_name())).unwrap();
    }
    (tmp, dir)
}

/// A temp home with an empty folder `<home>/work`.
fn empty() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("work");
    fs::create_dir(&dir).unwrap();
    (tmp, dir)
}

fn feature() -> Template {
    find(&builtins(), "Feature").unwrap().clone()
}

fn files(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

fn new_in(
    env: &Env,
    dir: &Path,
    title: &str,
    column: Option<&str>,
    body: Option<&str>,
) -> Result<Created> {
    let board = load("work", dir, env).unwrap();
    let template = feature();
    let req = CreateRequest {
        title,
        template: &template,
        column,
        body,
    };
    create(env, &board, &req)
}

fn new(home: &Path, dir: &Path) -> Result<Created> {
    new_in(&core_env(home, NOW), dir, "new task", None, None)
}

fn board_md(dir: &Path) -> String {
    fs::read_to_string(dir.join("board.md")).unwrap()
}

#[test]
fn fr06_create_takes_next_number() {
    let (tmp, dir) = copy();
    let c = new(tmp.path(), &dir).unwrap();
    assert_eq!(c.card.id.to_string(), "W-13");
    assert!(dir.join("W-13.md").is_file());
    assert!(board_md(&dir).contains("next: 14\n"));
}

#[test]
fn fr13_create_uses_template_skeleton() {
    let (tmp, dir) = copy();
    let c = new(tmp.path(), &dir).unwrap();
    for h in ["## Goal", "## Acceptance", "## Context"] {
        assert!(c.card.body.contains(h), "{:?}", c.card.body);
    }
    assert_eq!(c.card.template.as_deref(), Some("Feature"));
    let text = fs::read_to_string(dir.join("W-13.md")).unwrap();
    assert!(text.contains("## Goal") && text.contains("template: Feature\n"));
}

#[test]
fn fr13_create_uses_supplied_body() {
    let (tmp, dir) = copy();
    let env = core_env(tmp.path(), NOW);
    let c = new_in(&env, &dir, "new task", None, Some("## Notes\nx\n")).unwrap();
    assert_eq!(c.card.body.trim(), "## Notes\nx");
    assert!(!c.card.body.contains("## Goal"));
}

#[test]
fn i3_create_enters_first_column_unless_given() {
    let (tmp, dir) = copy();
    let env = core_env(tmp.path(), NOW);
    let c = new_in(&env, &dir, "a", None, None).unwrap();
    assert_eq!(c.card.column, "Todo");
    let c = new_in(&env, &dir, "b", Some("doing"), None).unwrap();
    assert_eq!(c.card.column, "Doing");
    let before = files(&dir);
    let err = new_in(&env, &dir, "c", Some("nope"), None).unwrap_err();
    assert!(matches!(err, Error::NotFound(_)), "{err:?}");
    assert_eq!(files(&dir), before);
}

#[test]
fn fr04_create_makes_missing_board_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("work");
    new(tmp.path(), &dir).unwrap();
    assert_eq!(files(&dir), ["W-1.md", "board.md"]);
    assert_eq!(board_md(&dir), "---\nnext: 2\n---\n");
}

#[test]
fn nfr06_create_changes_only_next_line_of_board_md() {
    let (tmp, dir) = copy();
    let before = fs::read(dir.join("board.md")).unwrap();
    let files_before = files(&dir);
    new(tmp.path(), &dir).unwrap();
    let after = fs::read(dir.join("board.md")).unwrap();
    assert_only_lines_changed(&before, &after, &["next: 14\n"]);
    let added: Vec<_> = files(&dir)
        .into_iter()
        .filter(|f| !files_before.contains(f))
        .collect();
    assert_eq!(added, ["W-13.md"]);
}

#[test]
fn ts4_3_create_prepends_frontmatter_to_plain_board_md() {
    let (tmp, dir) = empty();
    fs::write(dir.join("board.md"), "# work\n").unwrap();
    new(tmp.path(), &dir).unwrap();
    assert_eq!(board_md(&dir), "---\nnext: 2\n---\n# work\n");
}

#[test]
fn ts4_3_create_with_invalid_board_md_is_unreadable() {
    let (tmp, dir) = empty();
    fs::write(dir.join("board.md"), "---\nletter W\n---\n").unwrap();
    let err = new(tmp.path(), &dir).unwrap_err();
    assert!(matches!(err, Error::Unreadable(_)), "{err:?}");
    assert!(err.to_string().starts_with("board.md:2: "), "{err}");
    assert_eq!(err.exit_code(), 1);
    assert_eq!(files(&dir), ["board.md"]);
    assert_eq!(board_md(&dir), "---\nletter W\n---\n");
}

#[test]
fn i4_next_exceeds_every_number_after_create() {
    let (tmp, dir) = empty();
    fs::write(dir.join("board.md"), "---\nnext: 3\n---\n").unwrap();
    fs::write(dir.join("W-12.md"), "# old\n").unwrap();
    let c = new(tmp.path(), &dir).unwrap();
    assert_eq!(c.card.id.to_string(), "W-13");
    assert!(board_md(&dir).contains("next: 14\n"));
}

#[test]
fn ts4_6_deleting_cards_never_lowers_next() {
    let (tmp, dir) = copy();
    let c = new(tmp.path(), &dir).unwrap();
    assert_eq!(c.card.id.to_string(), "W-13");
    fs::remove_file(dir.join("W-13.md")).unwrap();
    let c = new(tmp.path(), &dir).unwrap();
    assert_eq!(c.card.id.to_string(), "W-14");
}

#[test]
fn ts6_3_create_with_empty_title_is_invalid_input() {
    let (tmp, dir) = copy();
    let before = files(&dir);
    let md = board_md(&dir);
    let env = core_env(tmp.path(), NOW);
    let err = new_in(&env, &dir, "  ", None, None).unwrap_err();
    assert_eq!(err, Error::InvalidInput("no title".to_string()));
    assert_eq!(files(&dir), before);
    assert_eq!(board_md(&dir), md);
    assert!(!env.locks_dir().exists());
}

#[test]
fn r4_created_card_parses_to_request_values() {
    let (tmp, dir) = copy();
    let env = core_env(tmp.path(), NOW);
    let c = new_in(&env, &dir, "migrate invoices to v13", Some("Doing"), None).unwrap();
    assert_eq!(c.card.title, "migrate invoices to v13");
    assert_eq!(c.card.column, "Doing");
    assert_eq!(c.card.template.as_deref(), Some("Feature"));
    assert_eq!(c.card.created, Some(env.now.clone()));
    assert_eq!(c.card.path, dir.join("W-13.md"));
}

#[test]
fn ts3_created_is_written_in_local_zone() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().to_string_lossy().into_owned();
    let vars = [
        ("HOME".to_string(), home.clone()),
        ("BRAIN_SWAP_NOW".to_string(), NOW.to_string()),
    ];
    let clock = parse_stamp("2000-01-01T00:00:00+00:00", &TimeZone::UTC).unwrap();
    let env = Env::from_vars(vars, home.into(), std::process::id(), TimeZone::UTC, clock).unwrap();
    let dir = tmp.path().join("work");
    new_in(&env, &dir, "new task", None, None).unwrap();
    let text = fs::read_to_string(dir.join("W-1.md")).unwrap();
    assert!(
        text.lines()
            .any(|l| l == "created: 2026-10-08T08:52:00+00:00"),
        "{text}"
    );
}

/// A copy of `work_4_9` plus directories `W-<n>.md` for each `n` in `numbers`.
fn copy_with_dirs(numbers: std::ops::RangeInclusive<u32>) -> (tempfile::TempDir, PathBuf) {
    let (tmp, dir) = copy();
    for n in numbers {
        fs::create_dir(dir.join(format!("W-{n}.md"))).unwrap();
    }
    (tmp, dir)
}

fn temp_files(dir: &Path) -> Vec<String> {
    files(dir)
        .into_iter()
        .filter(|f| f.contains(".bs-tmp-"))
        .collect()
}

#[test]
fn fr06_create_skips_name_that_appears_inside_lock() {
    let (tmp, dir) = copy_with_dirs(13..=13);
    let c = new(tmp.path(), &dir).unwrap();
    assert_eq!(c.card.id.to_string(), "W-14");
    assert_eq!(c.card.path, dir.join("W-14.md"));
    assert!(board_md(&dir).contains("next: 15\n"));
    assert!(dir.join("W-13.md").is_dir());
    assert_eq!(fs::read_dir(dir.join("W-13.md")).unwrap().count(), 0);
}

#[test]
fn fr06_create_never_overwrites_existing_file() {
    let (tmp, dir) = copy_with_dirs(13..=13);
    let snapshot = |dir: &Path| -> BTreeMap<String, Option<Vec<u8>>> {
        files(dir)
            .into_iter()
            .filter(|f| f != "board.md")
            .map(|f| {
                let p = dir.join(&f);
                (f, p.is_file().then(|| fs::read(&p).unwrap()))
            })
            .collect()
    };
    let before = snapshot(&dir);
    new(tmp.path(), &dir).unwrap();
    let after = snapshot(&dir);
    for (name, content) in &before {
        assert_eq!(after.get(name), Some(content), "{name}");
    }
    assert!(temp_files(&dir).is_empty(), "{:?}", files(&dir));
}

#[test]
fn ts4_6_create_gives_up_after_100_collisions() {
    let (tmp, dir) = copy_with_dirs(13..=112);
    let err = new(tmp.path(), &dir).unwrap_err();
    assert_eq!(
        err,
        Error::Io("board work: no free card number after 100 tries".to_string())
    );
    assert_eq!(err.exit_code(), 1);
    assert!(temp_files(&dir).is_empty(), "{:?}", files(&dir));
}

#[test]
fn ts4_6_unsupported_link_maps_to_hard_links_not_supported() {
    for kind in [ErrorKind::Unsupported, ErrorKind::PermissionDenied] {
        assert_eq!(
            link_error("work", &io::Error::from(kind)),
            Error::Io("board work: hard links not supported".to_string()),
            "{kind:?}"
        );
    }
}

#[test]
fn ts4_6_other_link_error_keeps_os_message() {
    assert_eq!(
        link_error("work", &io::Error::other("disk on fire")),
        Error::Io("board work: disk on fire".to_string())
    );
}
