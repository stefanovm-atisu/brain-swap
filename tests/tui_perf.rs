//! NFR-02 timings on a 300-card board; run with `cargo test --release -- --ignored` (A-E4-24).

mod common;

use brain_swap::core::config::load;
use brain_swap::tui::app::Input;
use brain_swap::tui::{TuiStart, prepare};
use common::big_board::write_big_board;
use common::tui::{TuiFixture, fake_runtime, key};
use std::time::{Duration, Instant};

fn big() -> TuiFixture {
    let fx = TuiFixture::new(None);
    write_big_board(&fx.board_dir, 300, 20);
    fx
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

#[test]
#[ignore]
fn nfr02_first_frame_within_150_ms() {
    let fx = big();
    let mut runs: Vec<Duration> = (0..6)
        .map(|_| {
            let mut env = fx.env.clone();
            let start = TuiStart {
                config: load(&env),
                board: None,
            };
            let t = Instant::now();
            let prepared = prepare(&mut env, start).unwrap();
            let (mut rt, _, _) = fake_runtime(&env, prepared, 120, 40);
            rt.draw().unwrap();
            t.elapsed()
        })
        .skip(1)
        .collect();
    runs.sort();
    let median = ms(runs[2]);
    println!("nfr02 first frame: median {median:.1} ms");
    assert!(median < 150.0, "median {median} ms");
}

#[test]
#[ignore]
fn nfr02_key_reaction_within_50_ms() {
    let fx = big();
    let (mut rt, _, _) = fx.runtime(120, 40);
    rt.draw().unwrap();
    let mut runs: Vec<Duration> = ["j", "l", "k", "h"]
        .iter()
        .cycle()
        .take(100)
        .map(|k| {
            let t = Instant::now();
            assert_eq!(rt.dispatch(Input::Key(key(k))), None);
            rt.draw().unwrap();
            t.elapsed()
        })
        .collect();
    runs.sort();
    let p95 = ms(runs[94]);
    println!("nfr02 key reaction: p95 {p95:.1} ms");
    assert!(p95 < 50.0, "p95 {p95} ms");
}
