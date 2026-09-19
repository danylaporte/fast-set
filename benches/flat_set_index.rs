use fast_set::{
    U32Set,
    u32based::{U32FlatSetIndex, U32FlatSetIndexBuilder, U32FlatSetIndexLog},
};
use std::{hint::black_box, time::Instant};

const KEYS: u32 = 32;
const PER_KEY: u32 = 4096;
const ITERS: usize = 200;

fn key_base(key: u32) -> u32 {
    key * 1_000_000
}

/// Values `[start, start+n)` for `key`; present in base only when `start + n <= PER_KEY`.
fn range(key: u32, start: u32, n: u32) -> U32Set {
    (key_base(key) + start..key_base(key) + start + n).collect()
}

fn builder() -> U32FlatSetIndexBuilder {
    U32FlatSetIndexBuilder::with_capacity_and_hasher(KEYS as usize, Default::default())
}

fn log() -> U32FlatSetIndexLog {
    U32FlatSetIndexLog::with_hasher(Default::default())
}

fn base() -> U32FlatSetIndex {
    let mut b = builder();
    for k in 0..KEYS {
        b.union(k, &range(k, 0, PER_KEY));
    }
    b.build()
}

fn bench(name: &str, mut f: impl FnMut()) {
    let filters: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .collect();
    if !filters.is_empty() && !filters.iter().any(|f| name.contains(f.as_str())) {
        return;
    }
    for _ in 0..5 {
        f();
    }
    let mut samples = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let t = Instant::now();
        f();
        samples.push(t.elapsed());
    }
    samples.sort();
    let median = samples[ITERS / 2];
    let min = samples[0];
    println!(
        "{name:<28} median {:>10.2} us   min {:>10.2} us",
        median.as_secs_f64() * 1e6,
        min.as_secs_f64() * 1e6
    );
}

fn main() {
    let base = base();
    let idx_len = base.iter().map(|(_, s)| s.as_set().len()).sum::<usize>();
    println!("base: {KEYS} keys x {PER_KEY} values = {idx_len} values, {ITERS} iterations\n");

    let existing = |k| range(k, 0, 256);
    let fresh = |k| range(k, PER_KEY, 256);
    let small_existing = |k| range(k, 0, 64);
    let small_fresh = |k| range(k, PER_KEY, 64);
    let superset = |k| range(k, 0, PER_KEY + 64);
    let half = |k| range(k, 0, PER_KEY / 2);

    let existing_sets: Vec<_> = (0..KEYS).map(existing).collect();
    let fresh_sets: Vec<_> = (0..KEYS).map(fresh).collect();
    let small_existing_sets: Vec<_> = (0..KEYS).map(small_existing).collect();
    let small_fresh_sets: Vec<_> = (0..KEYS).map(small_fresh).collect();
    let superset_sets: Vec<_> = (0..KEYS).map(superset).collect();
    let half_sets: Vec<_> = (0..KEYS).map(half).collect();

    bench("build_fresh", || {
        let mut b = builder();
        for k in 0..KEYS {
            for v in 0..PER_KEY {
                b.insert(k, key_base(k) + v);
            }
        }
        black_box(b.build());
    });

    bench("log_insert_existing (noop)", || {
        let mut log = log();
        for k in 0..KEYS {
            for v in 0..256 {
                log.insert(&base, k, key_base(k) + v);
            }
        }
        black_box(log);
    });

    bench("log_insert_new", || {
        let mut log = log();
        for k in 0..KEYS {
            for v in 0..256 {
                log.insert(&base, k, key_base(k) + PER_KEY + v);
            }
        }
        black_box(log);
    });

    bench("log_remove_missing (noop)", || {
        let mut log = log();
        for k in 0..KEYS {
            for v in 0..256 {
                log.remove(&base, k, key_base(k) + PER_KEY + v);
            }
        }
        black_box(log);
    });

    bench("log_remove_existing", || {
        let mut log = log();
        for k in 0..KEYS {
            for v in 0..256 {
                log.remove(&base, k, key_base(k) + v);
            }
        }
        black_box(log);
    });

    bench("log_union_subset (noop)", || {
        let mut log = log();
        for k in 0..KEYS {
            log.union(&base, k, &existing_sets[k as usize]);
        }
        black_box(log);
    });

    bench("log_union_new", || {
        let mut log = log();
        for k in 0..KEYS {
            log.union(&base, k, &fresh_sets[k as usize]);
        }
        black_box(log);
    });

    bench("log_difference_disjoint (noop)", || {
        let mut log = log();
        for k in 0..KEYS {
            log.difference(&base, k, &small_fresh_sets[k as usize]);
        }
        black_box(log);
    });

    bench("log_difference_small", || {
        let mut log = log();
        for k in 0..KEYS {
            log.difference(&base, k, &small_existing_sets[k as usize]);
        }
        black_box(log);
    });

    bench("log_difference_half", || {
        let mut log = log();
        for k in 0..KEYS {
            log.difference(&base, k, &half_sets[k as usize]);
        }
        black_box(log);
    });

    bench("log_intersection_superset (noop)", || {
        let mut log = log();
        for k in 0..KEYS {
            log.intersection(&base, k, &superset_sets[k as usize]);
        }
        black_box(log);
    });

    bench("log_intersection_half", || {
        let mut log = log();
        for k in 0..KEYS {
            log.intersection(&base, k, &half_sets[k as usize]);
        }
        black_box(log);
    });

    bench("apply_noop_log", || {
        let mut idx = base.clone();
        let mut log = log();
        for k in 0..KEYS {
            log.insert(&base, k, key_base(k));
        }
        black_box(idx.apply(log));
        black_box(idx);
    });

    bench("apply_changing_log", || {
        let mut idx = base.clone();
        let mut log = log();
        for k in 0..KEYS {
            log.insert(&base, k, key_base(k) + PER_KEY);
        }
        black_box(idx.apply(log));
        black_box(idx);
    });

    bench("values", || {
        black_box(base.values());
    });
}
