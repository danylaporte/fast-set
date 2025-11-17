use crate::u32based::one_index;
use std::marker::PhantomData;

pub struct OneIndex<K, V> {
    index: one_index::OneIndex<V>,
    _k: PhantomData<K>,
}

impl<K, V> OneIndex<K, V> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            index: one_index::OneIndex::new(),
            _k: PhantomData,
        }
    }

    pub fn apply(&mut self, log: OneIndexLog<K, V>) -> bool
    where
        V: PartialEq,
    {
        self.index.apply(log.log)
    }

    #[inline]
    pub fn get(&self, key: K) -> Option<&V>
    where
        K: Into<u32>,
    {
        self.index.get(key.into())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> + '_
    where
        K: TryFrom<u32>,
    {
        self.index
            .iter()
            .filter_map(|(k, v)| Some((K::try_from(k).ok()?, v)))
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.index.len()
    }

    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = K> + '_
    where
        K: TryFrom<u32>,
    {
        self.index.keys().filter_map(|k| K::try_from(k).ok())
    }
}

impl<K, V> Default for OneIndex<K, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> FromIterator<(K, V)> for OneIndex<K, V>
where
    K: Into<u32>,
    V: PartialEq,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut builder = OneIndexBuilder::new();

        for (k, v) in iter {
            builder.insert(k, v);
        }

        builder.build()
    }
}

pub struct OneIndexLog<K, V> {
    log: one_index::OneIndexLog<V>,
    _k: PhantomData<K>,
}

impl<K, V> OneIndexLog<K, V> {
    #[inline]
    pub fn new() -> Self {
        Self {
            log: one_index::OneIndexLog::new(),
            _k: PhantomData,
        }
    }

    #[inline]
    pub fn get<'a>(&'a self, base: &'a OneIndex<K, V>, key: K) -> Option<&'a V>
    where
        K: Into<u32>,
    {
        self.log.get(&base.index, key.into())
    }

    #[inline]
    pub fn insert(&mut self, base: &OneIndex<K, V>, key: K, value: V)
    where
        K: Into<u32>,
        V: PartialEq,
    {
        self.log.insert(&base.index, key.into(), value)
    }

    #[inline]
    pub fn remove(&mut self, base: &OneIndex<K, V>, key: K)
    where
        K: Into<u32>,
        V: PartialEq,
    {
        self.log.remove(&base.index, key.into())
    }
}

impl<K, V> Default for OneIndexLog<K, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

pub struct OneIndexBuilder<K, V> {
    base: OneIndex<K, V>,
    log: OneIndexLog<K, V>,
}

impl<K, V> OneIndexBuilder<K, V> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn build(mut self) -> OneIndex<K, V>
    where
        V: PartialEq,
    {
        self.base.index.apply(self.log.log);
        self.base
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V)
    where
        K: Into<u32>,
        V: PartialEq,
    {
        self.log.insert(&self.base, key, value)
    }
}

impl<K, V> Default for OneIndexBuilder<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            base: OneIndex::new(),
            log: OneIndexLog::new(),
        }
    }
}

pub struct OneIndexTrx<'a, K, V> {
    base: &'a OneIndex<K, V>,
    log: &'a OneIndexLog<K, V>,
}

impl<'a, K, V> OneIndexTrx<'a, K, V> {
    #[inline]
    pub fn new(base: &'a OneIndex<K, V>, log: &'a OneIndexLog<K, V>) -> Self {
        Self { base, log }
    }

    #[inline]
    pub fn get(&self, key: K) -> Option<&V>
    where
        K: Into<u32>,
    {
        self.log.get(self.base, key)
    }
}
