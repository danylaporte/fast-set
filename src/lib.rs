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
use intern::U32HashSet;
pub use tree::{Tree, TreeIndexLog};

pub type U32Set = nohash::IntSet<u32>;

#[doc(hidden)]
pub use intern::IU32HashSet;

fn empty_roaring() -> &'static U32HashSet {
    static B: std::sync::OnceLock<U32HashSet> = std::sync::OnceLock::new();
    B.get_or_init(U32HashSet::default)
}

fn default_iu32_hashset() -> &'static IU32HashSet {
    static B: std::sync::OnceLock<IU32HashSet> = std::sync::OnceLock::new();
    B.get_or_init(|| U32HashSet::default().into())
}
