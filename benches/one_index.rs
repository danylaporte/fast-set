use fast_set::u32based::{OneIndex, OneIndexLog};
use std::{hint::black_box, time::Instant};

const N: u32 = 200_000;
const ITERS: usize = 30;

fn bench(name: &str, mut f: impl FnMut()) {
    let filters: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .collect();
    if !filters.is_empty() && !filters.iter().any(|f| name.contains(f.as_str())) {
        return;
    }
    for _ in 0..3 {
        f();
    }
    let mut samples = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let t = Instant::now();
        f();
        samples.push(t.elapsed());
    }
    samples.sort();
    println!(
        "{name:<28} median {:>10.2} us   min {:>10.2} us",
        samples[ITERS / 2].as_secs_f64() * 1e6,
        samples[0].as_secs_f64() * 1e6
    );
}

fn main() {
    let base: OneIndex<u64> = (0..N).map(|k| (k, k as u64 * 3)).collect();
    println!("base: {N} entries, {ITERS} iterations\n");

    bench("build_from_iter", || {
        let idx: OneIndex<u64> = (0..N).map(|k| (k, k as u64 * 3)).collect();
        black_box(idx);
    });

    bench("log_insert_noop_10k", || {
        let mut log = OneIndexLog::new();
        for k in 0..10_000 {
            log.insert(&base, k, k as u64 * 3);
        }
        black_box(log);
    });

    bench("log_insert_change_10k", || {
        let mut log = OneIndexLog::new();
        for k in 0..10_000 {
            log.insert(&base, k, k as u64);
        }
        black_box(log);
    });

    bench("apply_change_10k", || {
        let mut log = OneIndexLog::new();
        for k in 0..10_000 {
            log.insert(&base, k, k as u64);
        }
        let mut idx: OneIndex<u64> = (0..N).map(|k| (k, k as u64 * 3)).collect();
        black_box(idx.apply(log));
        black_box(idx);
    });

    bench("apply_remove_tail_10k", || {
        let mut log = OneIndexLog::new();
        for k in N - 10_000..N {
            log.remove(&base, k);
        }
        let mut idx: OneIndex<u64> = (0..N).map(|k| (k, k as u64 * 3)).collect();
        black_box(idx.apply(log));
        black_box(idx);
    });

    bench("get_all", || {
        let mut s = 0u64;
        for k in 0..N {
            s = s.wrapping_add(*base.get(k).unwrap());
        }
        black_box(s);
    });

    bench("iter_all", || {
        black_box(base.iter().map(|(_, v)| *v).sum::<u64>());
    });
}
