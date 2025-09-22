use dhat::{Alloc, Profiler};
use fast_set::{IU32HashSet, U32Set};

#[global_allocator]
static ALLOC: Alloc = Alloc;

#[test]
fn no_leak_on_intern() {
    let _profiler = Profiler::builder().testing().build();

    let rb = U32Set::from_iter(0..1_000);
    let ib = IU32HashSet::from(&rb);
    let _clone = ib.clone();
    // Everything dropped here → allocations should be zero at exit
}
