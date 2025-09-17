use crate::{IRoaringBitmap, IntSet, u32based};
use fxhash::FxBuildHasher;
use roaring::RoaringBitmap;
use std::{borrow::Borrow, collections::hash_map, hash::Hash, marker::PhantomData};

#[repr(transparent)]
pub struct HashFlatSetIndex<K, V> {
    inner: u32based::FlatSetIndex<K, FxBuildHasher>,
    _kv: PhantomData<(K, V)>,
}

impl<K, V> HashFlatSetIndex<K, V> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: u32based::FlatSetIndex::with_capacity_and_hasher(capacity, Default::default()),
            _kv: PhantomData,
        }
    }

    #[inline]
    pub fn apply(&mut self, log: HashFlatSetIndexLog<K, V>) -> bool
    where
        K: Eq + Hash,
    {
        self.inner.apply(log.inner)
    }

    #[inline]
    pub fn contains<Q>(&self, k: &Q, value: V) -> bool
    where
        K: Borrow<Q> + Eq + Hash,
        Q: ?Sized + Eq + Hash,
        V: Into<u32>,
    {
        self.inner.contains(k, value.into())
    }

    #[inline]
    pub fn contains_none(&self, value: V) -> bool
    where
        V: Into<u32>,
    {
        self.inner.contains_none(value.into())
    }

    #[inline]
    pub fn get<Q>(&self, k: &Q) -> &IntSet<V>
    where
        K: Borrow<Q> + Eq + Hash,
        Q: ?Sized + Eq + Hash,
    {
        unsafe { IntSet::from_bitmap_ref(self.inner.get(k).as_bitmap()) }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &IntSet<V>)>
    where
        V: Into<u32>,
    {
        self.inner
            .iter()
            .map(|(k, v)| (k, unsafe { IntSet::from_bitmap_ref(v.as_bitmap()) }))
    }

    #[inline]
    pub fn keys(&self) -> hash_map::Keys<'_, K, IRoaringBitmap> {
        self.inner.keys()
    }

    #[inline]
    pub fn none(&self) -> &IntSet<V> {
        unsafe { IntSet::from_bitmap_ref(self.inner.none().as_bitmap()) }
    }

    #[inline]
    pub fn values(&self) -> IntSet<V> {
        unsafe { IntSet::from_bitmap(self.inner.values()) }
    }
}

impl<K: Clone, V> Clone for HashFlatSetIndex<K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _kv: PhantomData,
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> Default for HashFlatSetIndex<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _kv: PhantomData,
        }
    }
}

pub struct HashFlatSetIndexBuilder<K, V> {
    base: HashFlatSetIndex<K, V>,
    log: HashFlatSetIndexLog<K, V>,
}

impl<K, V> HashFlatSetIndexBuilder<K, V> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            base: HashFlatSetIndex::new(),
            log: HashFlatSetIndexLog::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn build(mut self) -> HashFlatSetIndex<K, V>
    where
        K: Eq + Hash,
    {
        self.base.apply(self.log);
        self.base
    }

    #[inline]
    pub fn difference(&mut self, key: K, rhs: &IntSet<V>)
    where
        K: Eq + Hash,
    {
        self.log.difference(&self.base, key, rhs.as_bitmap());
    }

    #[inline]
    pub fn difference_none(&mut self, rhs: &IntSet<V>) {
        self.log.difference_none(&self.base, rhs.as_bitmap());
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> bool
    where
        K: Eq + Hash,
        V: Into<u32>,
    {
        self.log.insert(&self.base, key, value)
    }

    #[inline]
    pub fn insert_none(&mut self, value: V) -> bool
    where
        V: Into<u32>,
    {
        self.log.insert_none(&self.base, value)
    }

    #[inline]
    pub fn intersection(&mut self, key: K, rhs: &IntSet<V>)
    where
        K: Eq + Hash,
    {
        self.log.intersection(&self.base, key, rhs.as_bitmap());
    }

    #[inline]
    pub fn intersection_none(&mut self, rhs: &IntSet<V>) {
        self.log.intersection_none(&self.base, rhs.as_bitmap());
    }

    #[inline]
    pub fn remove(&mut self, key: K, value: V) -> bool
    where
        K: Eq + Hash,
        V: Into<u32>,
    {
        self.log.remove(&self.base, key, value)
    }

    #[inline]
    pub fn remove_none(&mut self, value: V) -> bool
    where
        V: Into<u32>,
    {
        self.log.remove_none(&self.base, value)
    }

    #[inline]
    pub fn union(&mut self, key: K, rhs: &IntSet<V>)
    where
        K: Eq + Hash,
    {
        self.log.union(&self.base, key, rhs.as_bitmap());
    }

    #[inline]
    pub fn union_none(&mut self, rhs: &IntSet<V>) {
        self.log.union_none(&self.base, rhs.as_bitmap());
    }
}

impl<K, V> Default for HashFlatSetIndexBuilder<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            base: Default::default(),
            log: Default::default(),
        }
    }
}

pub struct HashFlatSetIndexLog<K, V> {
    inner: u32based::FlatSetIndexLog<K, FxBuildHasher>,
    _v: PhantomData<V>,
}

impl<K, V> HashFlatSetIndexLog<K, V> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: u32based::FlatSetIndexLog::with_capacity_and_hasher(
                capacity,
                Default::default(),
            ),
            _v: PhantomData,
        }
    }

    #[inline]
    pub fn contains<Q>(&self, base: &HashFlatSetIndex<K, V>, k: &Q, value: V) -> bool
    where
        Q: ?Sized + Eq + Hash,
        K: Borrow<Q> + Eq + Hash,
        V: Into<u32>,
    {
        self.inner.contains(&base.inner, k, value.into())
    }

    #[inline]
    pub fn contains_none(&self, base: &HashFlatSetIndex<K, V>, value: V) -> bool
    where
        u32: From<V>,
    {
        self.inner.contains_none(&base.inner, value.into())
    }

    #[inline]
    pub fn get<'a, Q>(&'a self, base: &'a HashFlatSetIndex<K, V>, k: &Q) -> &'a IntSet<V>
    where
        Q: ?Sized + Eq + Hash,
        K: Borrow<Q> + Eq + Hash,
    {
        unsafe { IntSet::from_bitmap_ref(self.inner.get(&base.inner, k)) }
    }

    #[inline]
    pub fn none<'a>(&'a self, base: &'a HashFlatSetIndex<K, V>) -> &'a IntSet<V> {
        unsafe { IntSet::from_bitmap_ref(self.inner.none(&base.inner)) }
    }

    #[inline]
    pub fn insert(&mut self, base: &HashFlatSetIndex<K, V>, key: K, value: V) -> bool
    where
        K: Eq + Hash,
        V: Into<u32>,
    {
        self.inner.insert(&base.inner, key, value.into())
    }

    #[inline]
    pub fn insert_none(&mut self, base: &HashFlatSetIndex<K, V>, value: V) -> bool
    where
        V: Into<u32>,
    {
        self.inner.insert_none(&base.inner, value.into())
    }

    #[inline]
    pub fn remove(&mut self, base: &HashFlatSetIndex<K, V>, key: K, value: V) -> bool
    where
        K: Eq + Hash,
        V: Into<u32>,
    {
        self.inner.remove(&base.inner, key, value.into())
    }

    #[inline]
    pub fn remove_none(&mut self, base: &HashFlatSetIndex<K, V>, value: V) -> bool
    where
        V: Into<u32>,
    {
        self.inner.remove_none(&base.inner, value.into())
    }

    /* ---- bulk operations --------------------------------------------- */

    #[inline]
    pub fn union(&mut self, base: &HashFlatSetIndex<K, V>, key: K, rhs: &RoaringBitmap)
    where
        K: Eq + Hash,
    {
        self.inner.union(&base.inner, key, rhs)
    }

    #[inline]
    pub fn union_none(&mut self, base: &HashFlatSetIndex<K, V>, rhs: &RoaringBitmap) {
        self.inner.union_none(&base.inner, rhs)
    }

    #[inline]
    pub fn difference(&mut self, base: &HashFlatSetIndex<K, V>, key: K, rhs: &RoaringBitmap)
    where
        K: Eq + Hash,
    {
        self.inner.difference(&base.inner, key, rhs)
    }

    #[inline]
    pub fn difference_none(&mut self, base: &HashFlatSetIndex<K, V>, rhs: &RoaringBitmap) {
        self.inner.difference_none(&base.inner, rhs)
    }

    #[inline]
    pub fn intersection(&mut self, base: &HashFlatSetIndex<K, V>, key: K, rhs: &RoaringBitmap)
    where
        K: Eq + Hash,
    {
        self.inner.intersection(&base.inner, key, rhs)
    }

    #[inline]
    pub fn intersection_none(&mut self, base: &HashFlatSetIndex<K, V>, rhs: &RoaringBitmap) {
        self.inner.intersection_none(&base.inner, rhs)
    }
}

impl<K, V> Default for HashFlatSetIndexLog<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _v: PhantomData,
        }
    }
}

pub struct HashFlatSetIndexTrx<'a, K, V> {
    base: &'a HashFlatSetIndex<K, V>,
    log: &'a HashFlatSetIndexLog<K, V>,
}

impl<'a, K, V> HashFlatSetIndexTrx<'a, K, V> {
    #[inline]
    pub fn new(base: &'a HashFlatSetIndex<K, V>, log: &'a HashFlatSetIndexLog<K, V>) -> Self {
        Self { base, log }
    }

    #[inline]
    pub fn contains<Q>(&self, k: &Q, value: V) -> bool
    where
        K: Borrow<Q> + Eq + Hash,
        Q: ?Sized + Eq + Hash,
        V: Into<u32>,
    {
        self.log.contains(self.base, k, value)
    }

    #[inline]
    pub fn contains_none(&self, value: V) -> bool
    where
        u32: From<V>,
    {
        self.log.contains_none(self.base, value)
    }

    #[inline]
    pub fn get<Q>(&self, k: &Q) -> &IntSet<V>
    where
        K: Borrow<Q> + Eq + Hash,
        Q: ?Sized + Eq + Hash,
    {
        self.log.get(self.base, k)
    }

    #[inline]
    pub fn none(&self) -> &IntSet<V> {
        self.log.none(self.base)
    }
}
