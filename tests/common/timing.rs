//! Timing helpers of the `#[ignore]` performance tests (A-E6-04).

/// The sample at rank `ceil(0.95 * n)` of the sorted samples.
pub fn p95_ms(samples: &mut [f64]) -> f64 {
    let _ = samples;
    todo!()
}

/// Calls `f` `warmup` times untimed, then `runs` times, returning wall milliseconds.
pub fn time_runs(warmup: usize, runs: usize, f: impl FnMut()) -> Vec<f64> {
    let _ = (warmup, runs, f);
    todo!()
}
