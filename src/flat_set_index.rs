use crate::{IntSet, U32Set, u32based};
use std::{hash::Hash, marker::PhantomData};

#[repr(transparent)]
pub struct FlatSetIndex<K, V> {
    inner: u32based::U32FlatSetIndex,
    _kv: PhantomData<(K, V)>,
}

impl<K, V> FlatSetIndex<K, V> {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
            _kv: PhantomData,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: u32based::U32FlatSetIndex::with_capacity_and_hasher(
                capacity,
                Default::default(),
            ),
            _kv: PhantomData,
        }
    }

    #[inline]
    pub fn apply(&mut self, log: FlatSetIndexLog<K, V>) -> bool {
        self.inner.apply(log.inner)
    }

    #[inline]
    pub fn contains(&self, key: K, value: V) -> bool
    where
        K: Into<u32>,
        V: Into<u32>,
    {
        self.inner.contains(&key.into(), value.into())
    }

    #[inline]
    pub fn contains_none(&self, value: V) -> bool
    where
        V: Into<u32>,
    {
        self.inner.contains_none(value.into())
    }

    #[inline]
    pub fn get(&self, key: K) -> &IntSet<V>
    where
        K: Into<u32>,
    {
        unsafe { IntSet::from_u32set_ref(self.inner.get(&key.into()).as_set()) }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (K, &IntSet<V>)>
    where
        K: From<u32>,
        V: Into<u32>,
    {
        self.inner
            .iter()
            .map(|(k, v)| (K::from(*k), unsafe { IntSet::from_u32set_ref(v.as_set()) }))
    }

    #[inline]
    pub fn keys(&self) -> impl ExactSizeIterator<Item = K>
    where
        K: From<u32>,
    {
        self.inner.keys().copied().map(K::from)
    }

    #[inline]
    pub fn none(&self) -> &IntSet<V> {
        unsafe { IntSet::from_u32set_ref(self.inner.none().as_set()) }
    }

    #[inline]
    pub fn values(&self) -> IntSet<V> {
        unsafe { IntSet::from_set(self.inner.values()) }
    }
}

impl<K, V> Clone for FlatSetIndex<K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _kv: PhantomData,
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> Default for FlatSetIndex<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _kv: PhantomData,
        }
    }
}

pub struct FlatSetIndexBuilder<K, V> {
    base: FlatSetIndex<K, V>,
    log: FlatSetIndexLog<K, V>,
}

impl<K, V> FlatSetIndexBuilder<K, V> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            base: FlatSetIndex::with_capacity(capacity),
            log: FlatSetIndexLog::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn build(mut self) -> FlatSetIndex<K, V>
    where
        K: Eq + Hash,
    {
        self.base.apply(self.log);
        self.base
    }

    #[inline]
    pub fn difference(&mut self, key: K, rhs: &IntSet<V>)
    where
        K: From<u32> + Into<u32>,
    {
        self.log.difference(&self.base, key, rhs.as_set());
    }

    #[inline]
    pub fn difference_none(&mut self, rhs: &IntSet<V>) {
        self.log.difference_none(&self.base, rhs.as_set());
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> bool
    where
        K: From<u32> + Into<u32>,
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
        K: From<u32> + Into<u32>,
    {
        self.log.intersection(&self.base, key, rhs.as_set());
    }

    #[inline]
    pub fn intersection_none(&mut self, rhs: &IntSet<V>) {
        self.log.intersection_none(&self.base, rhs.as_set());
    }

    #[inline]
    pub fn remove(&mut self, key: K, value: V) -> bool
    where
        K: From<u32> + Into<u32>,
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
        K: From<u32> + Into<u32>,
    {
        self.log.union(&self.base, key, rhs.as_set());
    }

    #[inline]
    pub fn union_none(&mut self, rhs: &IntSet<V>) {
        self.log.union_none(&self.base, rhs.as_set());
    }
}

impl<K, V> Default for FlatSetIndexBuilder<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            base: Default::default(),
            log: Default::default(),
        }
    }
}

#[repr(transparent)]
pub struct FlatSetIndexLog<K, V> {
    inner: u32based::U32FlatSetIndexLog,
    _kv: PhantomData<(K, V)>,
}

impl<K, V> FlatSetIndexLog<K, V> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: u32based::U32FlatSetIndexLog::with_capacity_and_hasher(
                capacity,
                Default::default(),
            ),
            _kv: PhantomData,
        }
    }

    #[inline]
    pub fn contains(&self, base: &FlatSetIndex<K, V>, key: K, value: V) -> bool
    where
        K: Into<u32>,
        V: Into<u32>,
    {
        self.inner.contains(&base.inner, &key.into(), value.into())
    }

    #[inline]
    pub fn contains_none(&self, base: &FlatSetIndex<K, V>, value: V) -> bool
    where
        u32: From<V>,
    {
        self.inner.contains_none(&base.inner, value.into())
    }

    #[inline]
    pub fn get<'a>(&'a self, base: &'a FlatSetIndex<K, V>, key: K) -> &'a IntSet<V>
    where
        K: Into<u32>,
    {
        unsafe { IntSet::from_u32set_ref(self.inner.get(&base.inner, &key.into())) }
    }

    #[inline]
    pub fn none<'a>(&'a self, base: &'a FlatSetIndex<K, V>) -> &'a IntSet<V> {
        unsafe { IntSet::from_u32set_ref(self.inner.none(&base.inner)) }
    }

    #[inline]
    pub fn insert(&mut self, base: &FlatSetIndex<K, V>, key: K, value: V) -> bool
    where
        K: Into<u32>,
        V: Into<u32>,
    {
        self.inner.insert(&base.inner, key.into(), value.into())
    }

    #[inline]
    pub fn insert_none(&mut self, base: &FlatSetIndex<K, V>, value: V) -> bool
    where
        V: Into<u32>,
    {
        self.inner.insert_none(&base.inner, value.into())
    }

    #[inline]
    pub fn remove(&mut self, base: &FlatSetIndex<K, V>, key: K, value: V) -> bool
    where
        K: Into<u32>,
        V: Into<u32>,
    {
        self.inner.remove(&base.inner, key.into(), value.into())
    }

    #[inline]
    pub fn remove_none(&mut self, base: &FlatSetIndex<K, V>, value: V) -> bool
    where
        V: Into<u32>,
    {
        self.inner.remove_none(&base.inner, value.into())
    }

    /* ---- bulk operations --------------------------------------------- */

    #[inline]
    pub fn union(&mut self, base: &FlatSetIndex<K, V>, key: K, rhs: &U32Set)
    where
        K: Into<u32>,
    {
        self.inner.union(&base.inner, key.into(), rhs)
    }

    #[inline]
    pub fn union_none(&mut self, base: &FlatSetIndex<K, V>, rhs: &U32Set) {
        self.inner.union_none(&base.inner, rhs)
    }

    #[inline]
    pub fn difference(&mut self, base: &FlatSetIndex<K, V>, key: K, rhs: &U32Set)
    where
        K: Into<u32>,
    {
        self.inner.difference(&base.inner, key.into(), rhs)
    }

    #[inline]
    pub fn difference_none(&mut self, base: &FlatSetIndex<K, V>, rhs: &U32Set) {
        self.inner.difference_none(&base.inner, rhs)
    }

    #[inline]
    pub fn intersection(&mut self, base: &FlatSetIndex<K, V>, key: K, rhs: &U32Set)
    where
        K: Into<u32>,
    {
        self.inner.intersection(&base.inner, key.into(), rhs)
    }

    #[inline]
    pub fn intersection_none(&mut self, base: &FlatSetIndex<K, V>, rhs: &U32Set) {
        self.inner.intersection_none(&base.inner, rhs)
    }
}

impl<K, V> Default for FlatSetIndexLog<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _kv: PhantomData,
        }
    }
}

pub struct FlatSetIndexTrx<'a, K, V> {
    base: &'a FlatSetIndex<K, V>,
    log: &'a FlatSetIndexLog<K, V>,
}

impl<'a, K, V> FlatSetIndexTrx<'a, K, V> {
    #[inline]
    pub fn new(base: &'a FlatSetIndex<K, V>, log: &'a FlatSetIndexLog<K, V>) -> Self {
        Self { base, log }
    }

    #[inline]
    pub fn contains(&self, key: K, value: V) -> bool
    where
        K: Into<u32>,
        V: Into<u32>,
    {
        self.log.contains(self.base, key, value)
    }

    #[inline]
    pub fn contains_none(&self, value: V) -> bool
    where
        u32: From<V>,
    {
        self.log.contains_none(self.base, value)
    }

    #[inline]
    pub fn get(&self, key: K) -> &IntSet<V>
    where
        K: Into<u32>,
    {
        self.log.get(self.base, key)
    }

    #[inline]
    pub fn none(&self) -> &IntSet<V> {
        self.log.none(self.base)
    }
}
