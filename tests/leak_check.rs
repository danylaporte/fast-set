use dhat::{Alloc, Profiler};
use fast_set::IRoaringBitmap;
use roaring::RoaringBitmap;

#[global_allocator]
static ALLOC: Alloc = Alloc;

#[test]
fn no_leak_on_intern() {
    let _profiler = Profiler::builder().testing().build();

    let rb = RoaringBitmap::from_iter(0..1_000);
    let ib = IRoaringBitmap::from(&rb);
    let _clone = ib.clone();
    // Everything dropped here → allocations should be zero at exit
}
