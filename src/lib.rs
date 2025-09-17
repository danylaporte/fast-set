pub mod flat_set_index;
pub mod hash_flat_set_index;
pub mod int_set;
pub mod one_index;
pub mod tree;
pub mod u32based;

pub use flat_set_index::{FlatSetIndex, FlatSetIndexBuilder, FlatSetIndexLog};
pub use hash_flat_set_index::{
    HashFlatSetIndex, HashFlatSetIndexBuilder, HashFlatSetIndexLog, HashFlatSetIndexTrx,
};
pub use int_set::IntSet;
use roaring::RoaringBitmap;
pub use tree::{Tree, TreeIndexLog};

#[doc(hidden)]
pub use intern::IRoaringBitmap;

fn empty_roaring() -> &'static RoaringBitmap {
    static B: std::sync::OnceLock<RoaringBitmap> = std::sync::OnceLock::new();
    B.get_or_init(RoaringBitmap::new)
}

fn default_i_roaring_bitmap() -> &'static IRoaringBitmap {
    static B: std::sync::OnceLock<IRoaringBitmap> = std::sync::OnceLock::new();
    B.get_or_init(|| RoaringBitmap::new().into())
}
