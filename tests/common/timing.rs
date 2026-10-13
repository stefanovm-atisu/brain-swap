//! Timing helpers of the `#[ignore]` performance tests (A-E6-04).

use std::time::Instant;

/// The sample at rank `ceil(0.95 * n)` of the sorted samples.
pub fn p95_ms(samples: &mut [f64]) -> f64 {
    samples.sort_by(f64::total_cmp);
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

/// Calls `f` `warmup` times untimed, then `runs` times, returning wall milliseconds.
pub fn time_runs(warmup: usize, runs: usize, mut f: impl FnMut()) -> Vec<f64> {
    (0..warmup).for_each(|_| f());
    (0..runs)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_secs_f64() * 1000.0
        })
        .collect()
}

/// The sum of the `ms=` durations of every `herdr` line of a `BRAIN_SWAP_LOG` text.
pub fn herdr_ms(log: &str) -> f64 {
    log.lines()
        .filter(|l| l.split(' ').nth(1) == Some("herdr"))
        .filter_map(|l| l.rsplit_once(" ms=")?.1.parse::<f64>().ok())
        .sum()
}
