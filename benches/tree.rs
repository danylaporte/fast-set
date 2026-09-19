use fast_set::u32based::{Tree, TreeLog};
use std::{hint::black_box, time::Instant};

const NODES: u32 = 100_000;
const FANOUT: u32 = 4;
const ITERS: usize = 20;

/// Balanced tree: node `n` (n > 0) has parent `(n - 1) / FANOUT`.
fn parent_of(n: u32) -> Option<u32> {
    (n > 0).then(|| (n - 1) / FANOUT)
}

fn build_base() -> Tree {
    (0..NODES).map(|n| (n, parent_of(n))).collect()
}

fn bench(name: &str, mut f: impl FnMut()) {
    let filters: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .collect();
    if !filters.is_empty() && !filters.iter().any(|f| name.contains(f.as_str())) {
        return;
    }
    for _ in 0..2 {
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
        "{name:<32} median {:>10.2} us   min {:>10.2} us",
        median.as_secs_f64() * 1e6,
        min.as_secs_f64() * 1e6
    );
}

fn main() {
    let base = build_base();
    println!("base: {NODES} nodes, fanout {FANOUT}, {ITERS} iterations\n");

    bench("build_from_iter", || {
        black_box(build_base());
    });

    bench("build_chain_1000", || {
        let t: Tree = (0..1000u32).map(|n| (n, (n > 0).then(|| n - 1))).collect();
        black_box(t);
    });

    bench("log_insert_leaves_1000", || {
        let mut log = TreeLog::new();
        for i in 0..1000u32 {
            log.insert(&base, Some(NODES / 2 + i), NODES + i);
        }
        black_box(log);
    });

    bench("log_insert_noop_1000", || {
        let mut log = TreeLog::new();
        for n in NODES - 1000..NODES {
            log.insert(&base, parent_of(n), n);
        }
        black_box(log);
    });

    bench("log_remove_leaves_1000", || {
        let mut log = TreeLog::new();
        for n in NODES - 1000..NODES {
            log.remove(&base, n);
        }
        black_box(log);
    });

    bench("log_move_leaves_1000", || {
        let mut log = TreeLog::new();
        for n in NODES - 1000..NODES {
            log.insert(&base, Some(1), n);
        }
        black_box(log);
    });

    bench("log_move_subtree_x10", || {
        let mut log = TreeLog::new();
        for i in 0..10u32 {
            // node 5 has ~NODES/16 descendants; alternate between two parents
            log.insert(&base, Some(2 + (i % 2)), 5);
        }
        black_box(log);
    });

    bench("log_remove_subtree", || {
        let mut log = TreeLog::new();
        log.remove(&base, 5);
        black_box(log);
    });

    bench("apply_insert_leaves_1000", || {
        let mut log = TreeLog::new();
        for i in 0..1000u32 {
            log.insert(&base, Some(NODES / 2 + i), NODES + i);
        }
        let mut t = base.clone();
        black_box(t.apply(log));
        black_box(t);
    });

    bench("apply_noop", || {
        let mut log = TreeLog::new();
        for n in NODES - 1000..NODES {
            log.insert(&base, parent_of(n), n);
        }
        let mut t = base.clone();
        black_box(t.apply(log));
        black_box(t);
    });

    bench("query_depth_all", || {
        let mut s = 0usize;
        for n in 0..NODES {
            s += base.depth(n).unwrap();
        }
        black_box(s);
    });

    bench("query_ancestors_all", || {
        let mut s = 0usize;
        for n in 0..NODES {
            s += base.ancestors(n).count();
        }
        black_box(s);
    });

    bench("query_is_descendant_all", || {
        let mut s = 0usize;
        for n in 0..NODES {
            s += base.is_descendant_of(n, 0) as usize;
        }
        black_box(s);
    });
}
