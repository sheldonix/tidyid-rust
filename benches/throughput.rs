#![forbid(unsafe_code)]

use std::hint::black_box;
use std::time::{Duration, Instant};

use tidyid::tidyid;

const SAMPLES: usize = 20;

fn measure(
    name: &str,
    calls_per_sample: usize,
    operations_per_call: usize,
    mut operation: impl FnMut(),
) {
    let warmup_until = Instant::now() + Duration::from_millis(200);
    while Instant::now() < warmup_until {
        operation();
    }

    let mut samples = [0.0; SAMPLES];
    for sample in &mut samples {
        let started = Instant::now();
        for _ in 0..calls_per_sample {
            operation();
        }
        *sample =
            started.elapsed().as_nanos() as f64 / (calls_per_sample * operations_per_call) as f64;
    }
    samples.sort_unstable_by(f64::total_cmp);
    let p50 = samples[SAMPLES / 2];
    let p95 = samples[SAMPLES * 95 / 100];
    let mean = samples.iter().sum::<f64>() / SAMPLES as f64;
    println!(
        "{name:<24} {:>10.0} ids/s  mean={mean:>7.1}ns p50={p50:>7.1}ns p95={p95:>7.1}ns",
        1_000_000_000.0 / mean,
    );
}

fn main() {
    for length in [8, 10, 16, 32, 64, 256] {
        measure(&format!("tidyid/{length}"), 50_000, 1, || {
            black_box(tidyid(length, false).unwrap());
        });
    }

    measure("tidyid/32/uppercase", 50_000, 1, || {
        black_box(tidyid(32, true).unwrap());
    });
}
