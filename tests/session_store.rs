//! TECHSPEC 3 (I8) and 6.4: session ID check and resolution, the reference store.

mod common;

use brain_swap::core::env::Env;
use brain_swap::core::error::Error;
use brain_swap::core::model::{CardId, SessionRef};
use brain_swap::core::session::{
    IdCheck, PaneHint, Resolution, check_id, read_pane_hint, read_reference, resolve,
    valid_pane_id, write_reference,
};
use brain_swap::core::time::{format_stamp, parse_stamp};
use common::env::core_env;
use jiff::tz::TimeZone;
use std::fs;
use std::path::{Path, PathBuf};

const UUID: &str = "55583127-a22a-49bc-a803-e777c377a595";

fn run(flag: Option<&str>, session: Option<&str>) -> Resolution {
    let mut env = core_env(Path::new("/h"), "2026-10-08T11:52:00+03:00");
    env.session = session.map(str::to_string);
    resolve(flag, &env)
}

fn res(session: Option<&str>, warnings: &[&str]) -> Resolution {
    Resolution {
        session: session.map(str::to_string),
        warnings: warnings.iter().map(|w| w.to_string()).collect(),
    }
}

#[test]
fn fr41_valid_flag_resolves_without_warning() {
    assert_eq!(run(Some(UUID), None), res(Some(UUID), &[]));
}

#[test]
fn i8_valid_ids_match_pattern() {
    for s in ["a", "A.b_c-1", &"x".repeat(128)] {
        assert_eq!(check_id(s), IdCheck::Valid, "{s}");
    }
}

#[test]
fn i8_empty_dollar_or_brace_is_placeholder() {
    for s in ["", "${CLAUDE_SESSION_ID}", "$X", "{x", "x}"] {
        assert_eq!(check_id(s), IdCheck::Placeholder, "{s}");
    }
}

#[test]
fn i8_other_bad_ids_are_invalid() {
    for s in ["a b", "a/b", "é", &"x".repeat(129)] {
        assert_eq!(check_id(s), IdCheck::Invalid, "{s}");
    }
}

#[test]
fn fr41_empty_flag_warns_and_falls_back_to_env() {
    assert_eq!(
        run(Some(""), Some("b")),
        res(Some("b"), &["session id not substituted"])
    );
}

#[test]
fn fr41_placeholder_flag_without_env_resolves_none() {
    assert_eq!(
        run(Some("${CLAUDE_SESSION_ID}"), None),
        res(None, &["session id not substituted"])
    );
}

#[test]
fn fr41_invalid_flag_resolves_none_with_warning() {
    assert_eq!(run(Some("a b"), None), res(None, &["invalid session id"]));
}

#[test]
fn ts6_4_missing_flag_uses_env_without_warning() {
    assert_eq!(run(None, Some("b")), res(Some("b"), &[]));
}

#[test]
fn ts6_4_valid_flag_wins_over_env() {
    assert_eq!(run(Some("a"), Some("b")).session.as_deref(), Some("a"));
}

#[test]
fn ts6_4_nothing_resolves_silently() {
    assert_eq!(run(None, None), res(None, &[]));
}

#[test]
fn ts6_4_invalid_env_session_warns_and_falls_through() {
    assert_eq!(
        run(None, Some("${X}")),
        res(None, &["session id not substituted"])
    );
    assert_eq!(run(None, Some("a b")), res(None, &["invalid session id"]));
    assert_eq!(run(None, Some("")), res(None, &[]));
}

#[test]
fn ts6_4_same_warning_given_once() {
    assert_eq!(
        run(Some(""), Some("${X}")),
        res(None, &["session id not substituted"])
    );
}

const NOW: &str = "2026-10-08T11:52:00+03:00";

fn sref(card: &str, pane: Option<&str>, cwd: Option<&str>) -> SessionRef {
    SessionRef {
        card: CardId::parse(card).unwrap(),
        board: "work".to_string(),
        pane: pane.map(str::to_string),
        cwd: cwd.map(PathBuf::from),
        set_at: parse_stamp(NOW, &TimeZone::UTC).unwrap(),
    }
}

fn full() -> SessionRef {
    sref(
        "W-12",
        Some("w4V:p9"),
        Some("/home/smilen/Work/ati.billing"),
    )
}

/// Every regular file under `dir`, recursively, sorted.
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(files(&p));
        } else {
            out.push(p);
        }
    }
    out.sort();
    out
}

fn write_raw(env: &Env, name: &str, text: &[u8]) {
    fs::create_dir_all(env.sessions_dir()).unwrap();
    fs::write(env.sessions_dir().join(name), text).unwrap();
}

const VALID: &str = "card = \"W-12\"\nboard = \"work\"\nset_at = \"2026-10-08T11:52:00+03:00\"\n";

#[test]
fn fr57_reference_round_trips() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    write_reference(&env, "s1", &full()).unwrap();
    assert_eq!(read_reference(&env, "s1"), Some(full()));
}

#[test]
fn ts6_4_reference_file_format() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    write_reference(&env, "s1", &full()).unwrap();
    let text =
        fs::read_to_string(home.path().join(".local/state/brain-swap/sessions/s1.toml")).unwrap();
    insta::assert_snapshot!(text);
}

#[test]
fn ts6_4_absent_pane_and_cwd_are_left_out() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    let r = sref("W-12", None, None);
    write_reference(&env, "s1", &r).unwrap();
    let text = fs::read_to_string(env.sessions_dir().join("s1.toml")).unwrap();
    assert_eq!(text, VALID);
    let back = read_reference(&env, "s1").unwrap();
    assert_eq!((back.pane, back.cwd), (None, None));
}

#[test]
fn fr57_new_session_id_starts_unlinked() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    write_reference(&env, "s1", &full()).unwrap();
    assert_eq!(read_reference(&env, "s2"), None);
}

#[test]
fn fr57_setting_again_replaces_reference() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    write_reference(&env, "s1", &sref("W-12", None, None)).unwrap();
    write_reference(&env, "s1", &sref("W-7", None, None)).unwrap();
    assert_eq!(
        read_reference(&env, "s1").unwrap().card,
        CardId::parse("W-7").unwrap()
    );
}

#[test]
fn ts6_4_unparsable_file_reads_as_none() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    let board = "board = \"work\"\n";
    let set_at = "set_at = \"2026-10-08T11:52:00+03:00\"\n";
    let card = "card = \"W-12\"\n";
    let cases: [(&str, Vec<u8>); 5] = [
        ("a", format!("card = \n{board}{set_at}").into_bytes()),
        ("b", format!("card = \"W12\"\n{board}{set_at}").into_bytes()),
        (
            "c",
            format!("{card}{board}set_at = \"yesterday\"\n").into_bytes(),
        ),
        ("d", format!("{card}{set_at}").into_bytes()),
        (
            "e",
            [
                format!("{card}{board}{set_at}# ").as_bytes(),
                &[0xFF, b'\n'],
            ]
            .concat(),
        ),
    ];
    for (id, bytes) in &cases {
        write_raw(&env, &format!("{id}.toml"), bytes);
    }
    for (id, _) in &cases {
        assert_eq!(read_reference(&env, id), None, "{id}");
    }
}

#[test]
fn ts6_4_set_at_without_offset_reads_as_utc() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    write_raw(
        &env,
        "s1.toml",
        b"card = \"W-12\"\nboard = \"work\"\nset_at = \"2026-10-08 08:52:00\"\n",
    );
    let r = read_reference(&env, "s1").unwrap();
    assert_eq!(format_stamp(&r.set_at), "2026-10-08T08:52:00+00:00");
}

#[test]
fn i8_invalid_session_id_is_neither_written_nor_read() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    assert_eq!(
        write_reference(&env, "../x", &full()),
        Err(Error::InvalidInput("invalid session id".to_string()))
    );
    assert_eq!(files(&env.state_dir()), Vec::<PathBuf>::new());
    write_raw(&env, "a b.toml", VALID.as_bytes());
    assert_eq!(read_reference(&env, "a b"), None);
}

#[test]
fn fr05_reference_lives_in_state_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let env = core_env(tmp.path(), NOW);
    let board = tmp.path().join("boards/work");
    fs::create_dir_all(&board).unwrap();
    fs::write(board.join("W-12.md"), "# W-12\n").unwrap();
    let before = files(tmp.path());
    write_reference(&env, "s1", &sref("W-12", None, Some("/x"))).unwrap();
    let new: Vec<PathBuf> = files(tmp.path())
        .into_iter()
        .filter(|p| !before.contains(p))
        .collect();
    assert_eq!(
        new,
        [tmp.path().join(".local/state/brain-swap/sessions/s1.toml")]
    );
}

#[test]
fn ts6_4_reference_write_is_atomic() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    let (a, b) = (sref("W-12", None, None), sref("W-7", None, None));
    write_reference(&env, "s1", &a).unwrap();
    let cards = std::thread::scope(|s| {
        s.spawn(|| {
            for i in 0..200 {
                write_reference(&env, "s1", if i % 2 == 0 { &b } else { &a }).unwrap();
            }
        });
        let reader = s.spawn(|| {
            (0..200)
                .map(|_| read_reference(&env, "s1").map(|r| r.card))
                .collect::<Vec<_>>()
        });
        reader.join().unwrap()
    });
    for c in cards {
        assert!(c == Some(a.card) || c == Some(b.card), "{c:?}");
    }
}

#[test]
fn ts6_4_read_creates_nothing() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    assert_eq!(read_reference(&env, "s1"), None);
    assert!(!env.state_dir().exists());
}

fn hint(card: &str, session: &str) -> Option<PaneHint> {
    Some(PaneHint {
        card: CardId::parse(card).unwrap(),
        session: session.to_string(),
    })
}

#[test]
fn ts6_4_reference_with_pane_writes_hint() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    write_reference(&env, "s1", &full()).unwrap();
    assert_eq!(read_pane_hint(&env, "w4V:p9"), hint("W-12", "s1"));
    assert_eq!(
        files(&env.panes_dir()),
        [home.path().join(".local/state/brain-swap/panes/w4V:p9")]
    );
}

#[test]
fn ts6_4_pane_hint_file_format() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    write_reference(&env, "s1", &full()).unwrap();
    let text = fs::read_to_string(env.panes_dir().join("w4V:p9")).unwrap();
    assert_eq!(files(&env.panes_dir()).len(), 1);
    insta::assert_snapshot!(text);
}

#[test]
fn ts6_4_reference_without_pane_writes_no_hint() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    write_reference(&env, "s1", &sref("W-12", None, None)).unwrap();
    assert_eq!(files(&env.panes_dir()), Vec::<PathBuf>::new());
}

#[test]
fn ts6_4_last_reference_in_pane_wins() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    write_reference(&env, "s1", &sref("W-12", Some("w4V:p9"), None)).unwrap();
    write_reference(&env, "s2", &sref("W-7", Some("w4V:p9"), None)).unwrap();
    assert_eq!(read_pane_hint(&env, "w4V:p9"), hint("W-7", "s2"));
    assert_eq!(
        read_reference(&env, "s1").unwrap().card,
        CardId::parse("W-12").unwrap()
    );
    assert_eq!(files(&env.panes_dir()).len(), 1);
}

#[test]
fn ts6_4_pane_id_outside_pattern_writes_no_hint() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    let long = "x".repeat(129);
    for (i, pane) in ["w4V p9", "a/b", "%", &long].into_iter().enumerate() {
        let session = format!("s{i}");
        write_reference(&env, &session, &sref("W-12", Some(pane), None)).unwrap();
        assert!(env.sessions_dir().join(format!("{session}.toml")).is_file());
        assert_eq!(read_pane_hint(&env, pane), None, "{pane}");
    }
    assert_eq!(files(&env.panes_dir()), Vec::<PathBuf>::new());
}

#[test]
fn ts6_4_dot_pane_ids_write_no_hint() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    for pane in [".", ".."] {
        assert!(!valid_pane_id(pane), "{pane}");
        assert_eq!(
            write_reference(&env, "s1", &sref("W-12", Some(pane), None)),
            Ok(())
        );
    }
    assert_eq!(files(&env.panes_dir()), Vec::<PathBuf>::new());
}

#[test]
fn ts6_4_unparsable_hint_reads_as_none() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    fs::create_dir_all(env.panes_dir()).unwrap();
    for text in ["card = 3\n", "card = \"W12\"\nsession = \"s1\"\n"] {
        fs::write(env.panes_dir().join("w4V:p9"), text).unwrap();
        assert_eq!(read_pane_hint(&env, "w4V:p9"), None, "{text}");
    }
}

#[test]
fn fr57_hint_outlives_session_change_without_park() {
    let home = tempfile::tempdir().unwrap();
    let env = core_env(home.path(), NOW);
    write_reference(&env, "old", &sref("W-11", Some("w4V:p11"), None)).unwrap();
    assert_eq!(read_reference(&env, "new"), None);
    assert_eq!(read_pane_hint(&env, "w4V:p11"), hint("W-11", "old"));
}
