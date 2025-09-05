use crate::{
    IntSet, Tree, TreeIndexLog,
    u32based::{self},
};
use std::marker::PhantomData;

pub struct NodeSetIndex<K, V> {
    erased: u32based::NodeSetIndex,
    _kv: PhantomData<(K, V)>,
}

impl<K, V> NodeSetIndex<K, V> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn apply(&mut self, log: NodeSetIndexLog<K, V>) -> bool {
        self.erased.apply(log.erased)
    }

    #[inline]
    pub fn direct_items(&self, node: K) -> &IntSet<V>
    where
        K: Into<u32>,
    {
        unsafe { IntSet::from_bitmap_ref(self.erased.direct_items(node.into())) }
    }

    #[inline]
    pub fn subtree_items(&self, node: K) -> &IntSet<V>
    where
        K: Into<u32>,
    {
        unsafe { IntSet::from_bitmap_ref(self.erased.subtree_items(node.into())) }
    }

    #[inline]
    pub fn values(&self) -> IntSet<V> {
        unsafe { IntSet::from_bitmap(self.erased.values()) }
    }
}

impl<K, V> Clone for NodeSetIndex<K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            erased: self.erased.clone(),
            _kv: PhantomData,
        }
    }
}

impl<K, V> Default for NodeSetIndex<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            erased: Default::default(),
            _kv: PhantomData,
        }
    }
}

pub struct NodeSetIndexLog<K, V> {
    erased: u32based::NodeSetIndexLog,
    _kv: PhantomData<(K, V)>,
}

impl<K, V> NodeSetIndexLog<K, V> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn direct_items<'a>(&'a self, base: &'a NodeSetIndex<K, V>, node: K) -> &'a IntSet<V>
    where
        K: Into<u32>,
    {
        unsafe { IntSet::from_bitmap_ref(self.erased.direct_items(&base.erased, node.into())) }
    }

    #[inline]
    pub fn insert(
        &mut self,
        base: &NodeSetIndex<K, V>,
        base_h: &Tree<K>,
        log_h: &TreeIndexLog<K>,
        node: K,
        item: V,
    ) where
        K: Into<u32>,
        V: Into<u32>,
    {
        self.erased.insert(
            &base.erased,
            &base_h.erased,
            &log_h.erased,
            node.into(),
            item.into(),
        );
    }

    #[inline]
    pub fn remove(
        &mut self,
        base: &NodeSetIndex<K, V>,
        base_h: &Tree<K>,
        log_h: &TreeIndexLog<K>,
        node: K,
        item: V,
    ) where
        K: Into<u32>,
        V: Into<u32>,
    {
        self.erased.remove(
            &base.erased,
            &base_h.erased,
            &log_h.erased,
            node.into(),
            item.into(),
        );
    }

    #[inline]
    pub fn subtree_items<'a>(&'a self, base: &'a NodeSetIndex<K, V>, node: K) -> &'a IntSet<V>
    where
        K: Into<u32>,
    {
        unsafe { IntSet::from_bitmap_ref(self.erased.subtree_items(&base.erased, node.into())) }
    }
}

impl<K, V> Clone for NodeSetIndexLog<K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            erased: self.erased.clone(),
            _kv: PhantomData,
        }
    }
}

impl<K, V> Default for NodeSetIndexLog<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            erased: Default::default(),
            _kv: PhantomData,
        }
    }
}

pub struct NodeSetIndexTrx<'a, K, V> {
    base: &'a NodeSetIndex<K, V>,
    log: &'a NodeSetIndexLog<K, V>,
}

impl<'a, K, V> NodeSetIndexTrx<'a, K, V> {
    pub fn new(base: &'a NodeSetIndex<K, V>, log: &'a NodeSetIndexLog<K, V>) -> Self {
        Self { base, log }
    }

    #[inline]
    pub fn direct_items(&self, node: K) -> &IntSet<V>
    where
        K: Into<u32>,
    {
        self.log.direct_items(self.base, node)
    }

    #[inline]
    pub fn subtree_items(&self, node: K) -> &IntSet<V>
    where
        K: Into<u32>,
    {
        self.log.subtree_items(self.base, node)
    }
}

pub struct NodeSetIndexBuilder<K, V> {
    base: NodeSetIndex<K, V>,
    log: NodeSetIndexLog<K, V>,
}

impl<K, V> NodeSetIndexBuilder<K, V> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn build(mut self) -> NodeSetIndex<K, V> {
        self.base.apply(self.log);
        self.base
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V, tree: &Tree<K>)
    where
        K: Into<u32>,
        V: Into<u32>,
    {
        self.log
            .insert(&self.base, tree, &TreeIndexLog::default(), key, value)
    }

    #[inline]
    pub fn remove(&mut self, key: K, value: V, tree: &Tree<K>)
    where
        K: Into<u32>,
        V: Into<u32>,
    {
        self.log
            .remove(&self.base, tree, &TreeIndexLog::default(), key, value)
    }
}

impl<K, V> Default for NodeSetIndexBuilder<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            base: Default::default(),
            log: Default::default(),
        }
    }
}
