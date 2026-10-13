//! TECHSPEC 12.1 (crash, process level), 4.6, I1 to I4: the TUI editor merge aborts at
//! `edit:after_tmp` in a re-run of this test binary (A-E4-20).
#![cfg(feature = "failpoints")]

mod common;

use brain_swap::core::env::Env;
use brain_swap::tui::app::Input;
use brain_swap::tui::editor::{Launcher, ProcessLauncher};
use brain_swap::tui::runtime::Runtime;
use brain_swap::tui::{gate, prepare};
use common::sound::assert_board_sound;
use common::spawn::{Scenario, command};
use common::tui::{
    Ev, FakeEvents, FakeGuard, FakeLauncher, TuiFixture, fake_runtime_with, key, no_runner,
    scripted_editor,
};
use ratatui::backend::TestBackend;
use std::fs;
use std::process::Stdio;

/// Parks on W-12 through the real CLI, the note on a quoted heredoc.
const PARK: &str = "@BS@ park --card W-12 --session test <<'EOF'
Doing: parked mid-edit
Next: check the merge
Watch out: nothing
EOF";

const V13: &str = "# migrate invoices to v13";
const V14: &str = "# migrate invoices to v14";

/// `tui::gate` and `tui::prepare` over `env`, then a runtime with `launcher` and a `FakeGuard`.
fn runtime(env: &Env, launcher: Box<dyn Launcher>) -> (Runtime<TestBackend>, FakeEvents) {
    let mut env = env.clone();
    let start = gate(&env, None, true).unwrap();
    let prepared = prepare(&mut env, start).unwrap();
    let guard = FakeGuard::default();
    let (rt, _, events) = fake_runtime_with(&env, prepared, 80, 24, launcher, guard, no_runner());
    (rt, events)
}

/// The re-run child: `Env` from the process environment as `main` builds it, then `l e q`
/// (W-12, edit, quit) under `ProcessLauncher`; a no-op without `BRAIN_SWAP_FAILPOINT`.
#[test]
#[ignore]
fn tui_crash_child() {
    if std::env::var_os("BRAIN_SWAP_FAILPOINT").is_none() {
        return;
    }
    let tz = jiff::tz::TimeZone::system();
    let clock = jiff::Timestamp::now().to_zoned(tz.clone());
    let cwd = std::env::current_dir().unwrap();
    let env = Env::from_vars(std::env::vars(), cwd, std::process::id(), tz, clock).unwrap();
    let (mut rt, events) = runtime(&env, Box::new(ProcessLauncher));
    events.push([Ev::Key("l"), Ev::Key("e"), Ev::Key("q")]);
    rt.run();
}

/// `work_4_9` with a scripted editor that parks on W-12 and saves the v14 title, then the child
/// spawned with `BRAIN_SWAP_FAILPOINT=edit:after_tmp`; returns the fixture, W-12 before, the
/// saved bytes and the child's pid.
fn crash() -> (TuiFixture, String, String, u32) {
    let fx = TuiFixture::new(Some("work_4_9"));
    let tmp = fx.home.parent().unwrap().to_path_buf();
    let before = fs::read_to_string(fx.board_dir.join("W-12.md")).unwrap();
    let edited = before.replace(V13, V14);
    fs::write(tmp.join("edited.md"), &edited).unwrap();
    let scenario = Scenario::new(&tmp);
    fx.set_editor(&scripted_editor(
        &scenario,
        &tmp,
        &[PARK],
        &tmp.join("edited.md"),
        0,
    ));
    let exe = std::env::current_exe().unwrap();
    let child = command(&scenario.failpoint("edit:after_tmp"), &exe)
        .args([
            "--ignored",
            "--exact",
            "tui_crash_child",
            "--test-threads=1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = child.id();
    let out = child.wait_with_output().unwrap();
    assert!(
        !out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    (fx, before, edited, pid)
}

/// The card IDs `fx`'s board loads, by number.
fn ids(fx: &TuiFixture) -> Vec<String> {
    let mut ids: Vec<_> = fx.board().cards.iter().map(|c| c.id).collect();
    ids.sort_by_key(|id| id.number);
    ids.iter().map(|id| id.to_string()).collect()
}

#[test]
fn ts12_1_edit_after_tmp_crash_leaves_card_sound() {
    let (fx, before, edited, pid) = crash();
    let after = fs::read_to_string(fx.board_dir.join("W-12.md")).unwrap();
    let block = after
        .strip_prefix(before.as_str())
        .expect("bytes outside the note");
    assert!(block.starts_with("\n### "), "{block}");
    assert!(block.contains("- Doing: parked mid-edit\n"), "{block}");
    assert_eq!(ids(&fx), ["W-9", "W-11", "W-12"]);
    assert_board_sound(&fx.env, &fx.board(), &[12]);
    let temp = fx.board_dir.join(format!(".W-12.md.bs-tmp-{pid}"));
    assert_eq!(
        fs::read_to_string(temp).unwrap(),
        format!("{edited}{block}")
    );
    assert!(
        fx.home
            .join(".local/state/brain-swap/edit/work-W-12.md")
            .exists()
    );
}

#[test]
fn ts12_1_edit_after_crash_succeeds_without_failpoint() {
    let (fx, _, _, pid) = crash();
    let card = fx.board_dir.join("W-12.md");
    let parked = fs::read_to_string(&card).unwrap();
    let launcher = FakeLauncher::new(Some(0), |copy| {
        let text = fs::read_to_string(copy).unwrap();
        fs::write(copy, text.replace(V13, V14)).unwrap();
    });
    let (mut rt, _) = runtime(&fx.env, Box::new(launcher));
    // The parked note is the board's newest, so start-up selects W-12.
    assert_eq!(rt.app().sel.card.unwrap().to_string(), "W-12");
    rt.dispatch(Input::Key(key("e")));
    assert_eq!(fs::read_to_string(&card).unwrap(), parked.replace(V13, V14));
    assert!(parked.contains("- Doing: parked mid-edit\n"), "{parked}");
    assert!(fx.board_dir.join(format!(".W-12.md.bs-tmp-{pid}")).exists());
    assert!(fx.board().warnings.is_empty());
    assert_eq!(ids(&fx), ["W-9", "W-11", "W-12"]);
}
