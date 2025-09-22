use crate::{U32Set, default_iu32_hashset};
use intern::IU32HashSet;
use std::{
    borrow::Borrow,
    collections::hash_map::{self, Entry, HashMap, Keys},
    hash::{BuildHasher, Hash, RandomState},
};

pub type U32FlatSetIndex = FlatSetIndex<u32, nohash::BuildNoHashHasher<u32>>;
pub type U32FlatSetIndexBuilder = FlatSetIndexBuilder<u32, nohash::BuildNoHashHasher<u32>>;
pub type U32FlatSetIndexLog = FlatSetIndexLog<u32, nohash::BuildNoHashHasher<u32>>;

pub struct FlatSetIndex<K, S = RandomState> {
    map: HashMap<K, IU32HashSet, S>,
    none: IU32HashSet,
}

impl<K> FlatSetIndex<K, RandomState> {
    #[inline]
    pub fn new() -> Self {
        Self::with_hasher(Default::default())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, Default::default())
    }
}

impl<K, S> FlatSetIndex<K, S> {
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self {
            map: HashMap::with_capacity_and_hasher(capacity, hasher),
            none: Default::default(),
        }
    }

    #[inline]
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            map: HashMap::with_hasher(hasher),
            none: IU32HashSet::default(),
        }
    }

    pub fn apply(&mut self, log: FlatSetIndexLog<K, S>) -> bool
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        let mut changed = false;

        for (key, val) in log.map {
            match self.map.entry(key) {
                Entry::Occupied(mut o) => {
                    if val.is_empty() {
                        o.remove();
                        changed = true;
                    } else if *o.get() != val {
                        o.insert(val.into());
                        changed = true;
                    }
                }
                Entry::Vacant(v) => {
                    if !val.is_empty() {
                        changed = true;
                        v.insert(val.into());
                    }
                }
            }
        }

        if let Some(log) = log.none {
            if self.none != log {
                self.none = log.into();
                changed = true;
            }
        }

        changed
    }

    #[inline]
    pub fn contains<Q>(&self, k: &Q, val: u32) -> bool
    where
        K: Borrow<Q> + Eq + Hash,
        Q: ?Sized + Eq + Hash,
        S: BuildHasher,
    {
        self.map.get(k).is_some_and(|b| b.as_set().contains(&val))
    }

    #[inline]
    pub fn contains_none(&self, val: u32) -> bool {
        self.none.as_set().contains(&val)
    }

    #[inline]
    pub fn get<Q>(&self, k: &Q) -> &IU32HashSet
    where
        K: Borrow<Q> + Eq + Hash,
        Q: ?Sized + Eq + Hash,
        S: BuildHasher,
    {
        self.map.get(k).unwrap_or_else(|| default_iu32_hashset())
    }

    #[inline]
    pub fn iter(&self) -> hash_map::Iter<'_, K, IU32HashSet> {
        self.map.iter()
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, K, IU32HashSet> {
        self.map.keys()
    }

    #[inline]
    pub fn none(&self) -> &IU32HashSet {
        &self.none
    }

    pub fn values(&self) -> U32Set {
        let mut b = self.none.as_set().clone();

        for item in self.map.values() {
            b.extend(item.as_set());
        }

        b
    }
}

impl<K: Clone, S: Clone> Clone for FlatSetIndex<K, S> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
            none: self.none.clone(),
        }
    }
}

impl<K, S: Default> Default for FlatSetIndex<K, S> {
    #[inline]
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

pub struct FlatSetIndexBuilder<K, S = RandomState> {
    base: FlatSetIndex<K, S>,
    log: FlatSetIndexLog<K, S>,
}

impl<K> FlatSetIndexBuilder<K, RandomState> {
    #[inline]
    pub fn new() -> Self {
        Self::with_hasher(Default::default())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, Default::default())
    }
}

impl<K, S> FlatSetIndexBuilder<K, S> {
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self
    where
        S: Clone,
    {
        Self {
            base: FlatSetIndex::with_capacity_and_hasher(capacity, hasher.clone()),
            log: FlatSetIndexLog::with_capacity_and_hasher(capacity, hasher),
        }
    }

    #[inline]
    pub fn with_hasher(hasher: S) -> Self
    where
        S: Clone,
    {
        Self {
            base: FlatSetIndex::with_hasher(hasher.clone()),
            log: FlatSetIndexLog::with_hasher(hasher),
        }
    }

    pub fn build(mut self) -> FlatSetIndex<K, S>
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        self.base.apply(self.log);
        self.base
    }

    #[inline]
    pub fn difference(&mut self, key: K, rhs: &U32Set)
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        self.log.difference(&self.base, key, rhs);
    }

    #[inline]
    pub fn difference_none(&mut self, rhs: &U32Set) {
        self.log.difference_none(&self.base, rhs);
    }

    #[inline]
    pub fn insert(&mut self, key: K, val: u32) -> bool
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        self.log.insert(&self.base, key, val)
    }

    #[inline]
    pub fn insert_none(&mut self, val: u32) -> bool {
        self.log.insert_none(&self.base, val)
    }

    #[inline]
    pub fn intersection(&mut self, key: K, rhs: &U32Set)
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        self.log.intersection(&self.base, key, rhs);
    }

    #[inline]
    pub fn intersection_none(&mut self, rhs: &U32Set) {
        self.log.intersection_none(&self.base, rhs);
    }

    #[inline]
    pub fn remove(&mut self, key: K, val: u32) -> bool
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        self.log.remove(&self.base, key, val)
    }

    #[inline]
    pub fn remove_none(&mut self, val: u32) -> bool {
        self.log.remove_none(&self.base, val)
    }

    #[inline]
    pub fn union(&mut self, key: K, rhs: &U32Set)
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        self.log.union(&self.base, key, rhs);
    }

    #[inline]
    pub fn union_none(&mut self, rhs: &U32Set) {
        self.log.union_none(&self.base, rhs);
    }
}

impl<K, S: Default> Default for FlatSetIndexBuilder<K, S> {
    fn default() -> Self {
        Self {
            base: Default::default(),
            log: Default::default(),
        }
    }
}

pub struct FlatSetIndexLog<K, S> {
    map: HashMap<K, U32Set, S>,
    none: Option<U32Set>,
}

impl<K> FlatSetIndexLog<K, RandomState> {
    #[inline]
    pub fn new() -> Self {
        Self::with_hasher(Default::default())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, Default::default())
    }
}

impl<K, S> FlatSetIndexLog<K, S> {
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self {
            map: HashMap::with_capacity_and_hasher(capacity, hasher),
            none: None,
        }
    }

    #[inline]
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            map: HashMap::with_hasher(hasher),
            none: None,
        }
    }

    #[inline]
    pub fn contains<Q>(&self, base: &FlatSetIndex<K, S>, k: &Q, val: u32) -> bool
    where
        K: Borrow<Q> + Eq + Hash,
        Q: ?Sized + Eq + Hash,
        S: BuildHasher,
    {
        match self.map.get(k) {
            Some(log) => log.contains(&val),
            None => base.contains(k, val),
        }
    }

    #[inline]
    pub fn contains_none(&self, base: &FlatSetIndex<K, S>, val: u32) -> bool {
        match &self.none {
            Some(log) => log.contains(&val),
            None => base.contains_none(val),
        }
    }

    pub fn difference(&mut self, base: &FlatSetIndex<K, S>, key: K, rhs: &U32Set)
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        let v = self.get_mut(base, key);
        *v = v.difference(rhs).copied().collect();
    }

    pub fn difference_none(&mut self, base: &FlatSetIndex<K, S>, rhs: &U32Set) {
        let v = self.none_mut(base);
        *v = v.difference(rhs).copied().collect();
    }

    #[inline]
    pub fn get<'a, Q>(&'a self, base: &'a FlatSetIndex<K, S>, k: &Q) -> &'a U32Set
    where
        K: Borrow<Q> + Eq + Hash,
        Q: ?Sized + Eq + Hash,
        S: BuildHasher,
    {
        match self.map.get(k) {
            Some(log) => log,
            None => base.get(k).as_set(),
        }
    }

    fn get_mut(&mut self, base: &FlatSetIndex<K, S>, key: K) -> &mut U32Set
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        match self.map.entry(key) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let b = base.get(v.key()).as_set().clone();
                v.insert(b)
            }
        }
    }

    #[inline]
    pub fn insert(&mut self, base: &FlatSetIndex<K, S>, key: K, val: u32) -> bool
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        self.get_mut(base, key).insert(val)
    }

    #[inline]
    pub fn insert_none(&mut self, base: &FlatSetIndex<K, S>, val: u32) -> bool {
        self.none_mut(base).insert(val)
    }

    pub fn intersection(&mut self, base: &FlatSetIndex<K, S>, key: K, rhs: &U32Set)
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        let v = self.get_mut(base, key);
        *v = v.intersection(rhs).copied().collect();
    }

    pub fn intersection_none(&mut self, base: &FlatSetIndex<K, S>, rhs: &U32Set) {
        let v = self.none_mut(base);
        *v = v.intersection(rhs).copied().collect();
    }

    #[inline]
    pub fn none<'a>(&'a self, base: &'a FlatSetIndex<K, S>) -> &'a U32Set {
        match &self.none {
            Some(log) => log,
            None => base.none().as_set(),
        }
    }

    fn none_mut(&mut self, base: &FlatSetIndex<K, S>) -> &mut U32Set {
        self.none.get_or_insert_with(|| base.none.as_set().clone())
    }

    #[inline]
    pub fn remove(&mut self, base: &FlatSetIndex<K, S>, key: K, val: u32) -> bool
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        self.get_mut(base, key).remove(&val)
    }

    #[inline]
    pub fn remove_none(&mut self, base: &FlatSetIndex<K, S>, val: u32) -> bool {
        self.none_mut(base).remove(&val)
    }

    pub fn union(&mut self, base: &FlatSetIndex<K, S>, key: K, rhs: &U32Set)
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        self.get_mut(base, key).extend(rhs.iter().copied());
    }

    pub fn union_none(&mut self, base: &FlatSetIndex<K, S>, rhs: &U32Set) {
        self.none_mut(base).extend(rhs.iter().copied());
    }
}

impl<K, S: Default> Default for FlatSetIndexLog<K, S> {
    #[inline]
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /* ---------- helpers ---------- */

    fn bitmap(vals: &[u32]) -> U32Set {
        U32Set::from_iter(vals.iter().copied())
    }

    /* ---------- basic consistency ---------- */

    #[test]
    fn empty_index_is_consistent() {
        let idx = FlatSetIndex::<u32, _>::new();
        assert!(idx.none().as_set().is_empty());
        assert!(idx.map.is_empty());
    }

    #[test]
    fn insert_and_contains() {
        let mut builder = FlatSetIndexBuilder::new();
        assert!(builder.insert(1, 10));
        assert!(builder.insert(1, 20));
        assert!(!builder.insert(1, 10)); // duplicate
        assert!(builder.insert_none(30));

        let idx = builder.build();
        assert!(idx.contains(&1, 10));
        assert!(idx.contains(&1, 20));
        assert!(!idx.contains(&1, 30));
        assert!(idx.contains_none(30));
    }

    #[test]
    fn union_difference_sequence() {
        let mut builder = FlatSetIndexBuilder::new();
        builder.union(1, &bitmap(&[1, 2, 3]));
        builder.difference(1, &bitmap(&[2]));
        builder.union_none(&bitmap(&[4, 5]));

        let idx = builder.build();
        assert!(idx.contains(&1, 1));
        assert!(idx.contains(&1, 3));
        assert!(!idx.contains(&1, 2));
        assert!(idx.contains_none(4));
        assert!(idx.contains_none(5));
    }

    #[test]
    fn intersection_makes_empty() {
        let mut builder = FlatSetIndexBuilder::new();
        builder.union(1, &bitmap(&[1, 2, 3]));
        builder.intersection(1, &bitmap(&[2, 3, 4]));
        let idx = builder.build();
        assert!(!idx.contains(&1, 1));
        assert!(idx.contains(&1, 2));
        assert!(idx.contains(&1, 3));
    }

    #[test]
    fn remove_and_reapply() {
        let mut builder = FlatSetIndexBuilder::new();
        builder.union(1, &bitmap(&[1, 2, 3]));
        builder.remove(1, 2);
        builder.remove(1, 99); // non-existent
        let idx = builder.build();
        assert!(idx.contains(&1, 1));
        assert!(!idx.contains(&1, 2));
        assert!(idx.contains(&1, 3));
        assert_eq!(idx.get(&1).as_set().len(), 2);
    }

    #[test]
    fn large_random_sequence() {
        use rand::prelude::*;

        let mut rng = StdRng::seed_from_u64(0xC0FFEE);
        let mut builder = FlatSetIndexBuilder::with_capacity(100);

        // 100 random mutations
        for _ in 0..100 {
            let key = rng.random_range(0..50);
            let val = rng.random_range(0..1_000);
            match rng.random_range(0..4) {
                0 => {
                    builder.insert(key, val);
                }
                1 => {
                    builder.remove(key, val);
                }
                2 => {
                    builder.union(key, &bitmap(&[val, val + 1, val + 2]));
                }
                3 => {
                    builder.difference(key, &bitmap(&[val]));
                }
                _ => unreachable!(),
            };
        }

        // ensure the log applies cleanly
        let idx = builder.build();
        for k in 0..50 {
            let rb: U32Set = idx.get(&k).inner().clone();
            for v in rb.iter() {
                assert!(idx.contains(&k, *v));
            }
        }
    }

    /* ---------- log-only consistency ---------- */

    #[test]
    fn log_operations_are_consistent() {
        let base = FlatSetIndex::new();
        let mut log = FlatSetIndexLog::new();

        assert!(log.insert(&base, 1, 10));
        assert!(!log.insert(&base, 1, 10)); // duplicate
        assert!(log.insert_none(&base, 20));

        // log queries mirror the final index
        assert!(log.contains(&base, &1, 10));
        assert!(!log.contains(&base, &1, 15));
        assert!(log.contains_none(&base, 20));
    }

    /* ---------- miri-friendly threaded stress ---------- */

    #[test]
    fn concurrent_build_is_safe() {
        use std::thread;

        let mut handles = vec![];
        for i in 0..8 {
            handles.push(thread::spawn(move || {
                let mut b = FlatSetIndexBuilder::with_capacity(100);
                for j in 0..1_000 {
                    let k = i * 1_000 + j;
                    b.insert(k % 50, k as u32);
                }
                b.build()
            }));
        }

        for h in handles {
            let idx = h.join().unwrap();
            assert!(idx.get(&0).as_set().len() > 0);
        }
    }
}
