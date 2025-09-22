use crate::{IntSet, u32based};
use std::marker::PhantomData;

#[repr(transparent)]
pub struct Tree<K> {
    pub(crate) erased: u32based::Tree,
    _k: PhantomData<K>,
}

impl<K> Tree<K> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn all_nodes(&self) -> IntSet<K>
    where
        K: Into<u32>,
    {
        unsafe { IntSet::from_set(self.erased.all_nodes()) }
    }

    #[inline]
    pub fn apply(&mut self, log: TreeIndexLog<K>) -> bool {
        self.erased.apply(log.erased)
    }

    #[inline]
    pub fn children(&self, parent: K) -> &IntSet<K>
    where
        K: Into<u32>,
    {
        unsafe { IntSet::from_u32set_ref(self.erased.children(parent.into())) }
    }

    #[inline]
    pub fn children_with_self(&self, node: K) -> impl Iterator<Item = K> + '_
    where
        K: From<u32> + Into<u32>,
    {
        self.erased
            .children_with_self(node.into())
            .into_iter()
            .map(K::from)
    }

    #[inline]
    pub fn descendants(&self, parent: K) -> &IntSet<K>
    where
        K: Into<u32>,
    {
        unsafe { IntSet::from_u32set_ref(self.erased.descendants(parent.into())) }
    }

    #[inline]
    pub fn descendants_with_self(&self, node: K) -> impl Iterator<Item = K> + '_
    where
        K: From<u32> + Into<u32>,
    {
        self.erased
            .descendants_with_self(node.into())
            .into_iter()
            .map(K::from)
    }

    #[inline]
    pub fn cycles(&self) -> impl Iterator<Item = K> + '_
    where
        K: From<u32>,
    {
        self.erased.cycles().copied().map(K::from)
    }

    #[inline]
    pub fn parent(&self, child: K) -> Option<K>
    where
        K: From<u32> + Into<u32>,
    {
        self.erased.parent(child.into()).map(Into::into)
    }

    #[inline]
    pub fn depth(&self, node: K) -> Result<usize, CycleError<K>>
    where
        K: From<u32> + Into<u32>,
    {
        self.erased
            .depth(node.into())
            .map_err(|e| CycleError(K::from(e.0)))
    }

    #[inline]
    pub fn is_descendant_of(&self, child: K, parent: K) -> bool
    where
        K: Into<u32>,
    {
        self.erased.is_descendant_of(child.into(), parent.into())
    }

    #[inline]
    pub fn has_cycle(&self, node: K) -> bool
    where
        K: Into<u32>,
    {
        self.erased.has_cycle(node.into())
    }

    #[inline]
    pub fn ancestors(&self, child: K) -> impl Iterator<Item = K> + Clone + '_
    where
        K: From<u32> + Into<u32>,
    {
        self.erased.ancestors(child.into()).map(Into::into)
    }

    #[inline]
    pub fn ancestors_with_self(&self, child: K) -> impl Iterator<Item = K> + Clone + '_
    where
        K: From<u32> + Into<u32>,
    {
        self.erased
            .ancestors_with_self(child.into())
            .map(Into::into)
    }
}

impl<K> Clone for Tree<K> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            erased: self.erased.clone(),
            _k: PhantomData,
        }
    }
}

impl<K> Default for Tree<K> {
    #[inline]
    fn default() -> Self {
        Self {
            erased: Default::default(),
            _k: PhantomData,
        }
    }
}

impl<K> FromIterator<(K, Option<K>)> for Tree<K>
where
    K: Into<u32>,
{
    fn from_iter<I: IntoIterator<Item = (K, Option<K>)>>(iter: I) -> Self {
        Self {
            erased: iter
                .into_iter()
                .map(|(n, p)| (n.into(), p.map(Into::into)))
                .collect(),
            _k: PhantomData,
        }
    }
}

#[repr(transparent)]
pub struct TreeIndexLog<K> {
    pub(crate) erased: u32based::TreeLog,
    _k: PhantomData<K>,
}

impl<K> TreeIndexLog<K> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn children<'a>(&'a self, base: &'a Tree<K>, parent: K) -> &'a IntSet<K>
    where
        K: Into<u32>,
    {
        unsafe { IntSet::from_u32set_ref(self.erased.children(&base.erased, parent.into())) }
    }

    #[inline]
    pub fn children_with_self<'a>(
        &'a self,
        base: &'a Tree<K>,
        node: K,
    ) -> impl Iterator<Item = K> + 'a
    where
        K: From<u32> + Into<u32>,
    {
        self.erased
            .children_with_self(&base.erased, node.into())
            .into_iter()
            .map(K::from)
    }

    #[inline]
    pub fn descendants<'a>(&'a self, base: &'a Tree<K>, parent: K) -> &'a IntSet<K>
    where
        K: Into<u32>,
    {
        unsafe { IntSet::from_u32set_ref(self.erased.descendants(&base.erased, parent.into())) }
    }

    #[inline]
    pub fn descendants_with_self<'a>(
        &'a self,
        base: &'a Tree<K>,
        node: K,
    ) -> impl Iterator<Item = K> + 'a
    where
        K: From<u32> + Into<u32>,
    {
        self.erased
            .descendants_with_self(&base.erased, node.into())
            .into_iter()
            .map(K::from)
    }

    #[inline]
    pub fn cycles<'a>(&'a self, base: &'a Tree<K>) -> impl Iterator<Item = K> + 'a
    where
        K: From<u32>,
    {
        self.erased
            .cycles(&base.erased)
            .iter()
            .copied()
            .map(K::from)
    }

    pub fn depth(&self, base: &Tree<K>, node: K) -> Result<usize, CycleError<K>>
    where
        K: From<u32> + Into<u32>,
    {
        self.erased
            .depth(&base.erased, node.into())
            .map_err(|e| CycleError(K::from(e.0)))
    }

    #[inline]
    pub fn has_cycle(&self, base: &Tree<K>, node: K) -> bool
    where
        K: Into<u32>,
    {
        self.erased.cycles(&base.erased).contains(&node.into())
    }

    #[inline]
    pub fn parent(&self, base: &Tree<K>, child: K) -> Option<K>
    where
        K: From<u32> + Into<u32>,
    {
        self.erased.parent(&base.erased, child.into()).map(K::from)
    }

    #[inline]
    pub fn insert(&mut self, base: &Tree<K>, parent: Option<K>, child: K)
    where
        K: Into<u32>,
    {
        self.erased
            .insert(&base.erased, parent.map(Into::into), child.into());
    }

    #[inline]
    pub fn is_descendant_of(&self, base: &Tree<K>, child: K, parent: K) -> bool
    where
        K: Into<u32>,
    {
        self.erased
            .is_descendant_of(&base.erased, child.into(), parent.into())
    }

    #[inline]
    pub fn remove(&mut self, base: &Tree<K>, node: K)
    where
        K: Into<u32>,
    {
        self.erased.remove(&base.erased, node.into());
    }

    #[inline]
    pub fn ancestors<'a>(
        &'a self,
        base: &'a Tree<K>,
        child: K,
    ) -> impl Iterator<Item = K> + Clone + 'a
    where
        K: From<u32> + Into<u32>,
    {
        self.erased
            .ancestors(&base.erased, child.into())
            .map(K::from)
    }

    #[inline]
    pub fn ancestors_with_self<'a>(
        &'a self,
        base: &'a Tree<K>,
        child: K,
    ) -> impl Iterator<Item = K> + Clone + 'a
    where
        K: From<u32> + Into<u32>,
    {
        self.erased
            .ancestors_with_self(&base.erased, child.into())
            .map(K::from)
    }
}

impl<K> Clone for TreeIndexLog<K> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            erased: self.erased.clone(),
            _k: PhantomData,
        }
    }
}

impl<K> Default for TreeIndexLog<K> {
    #[inline]
    fn default() -> Self {
        Self {
            erased: Default::default(),
            _k: PhantomData,
        }
    }
}

pub struct TreeTrx<'a, K> {
    base: &'a Tree<K>,
    log: &'a TreeIndexLog<K>,
}

impl<'a, K> TreeTrx<'a, K> {
    pub fn new(base: &'a Tree<K>, log: &'a TreeIndexLog<K>) -> Self {
        Self { base, log }
    }
    /// Returns an iterator over ancestors, stops at cycle nodes
    #[inline]
    pub fn ancestors(&self, child: K) -> impl Iterator<Item = K> + '_
    where
        K: From<u32> + Into<u32>,
    {
        let mut iter = self.ancestors_with_self(child);
        iter.next();
        iter
    }

    /// Returns an iterator over ancestors **including** the start node
    #[inline]
    pub fn ancestors_with_self(&self, child: K) -> impl Iterator<Item = K> + '_
    where
        K: From<u32> + Into<u32>,
    {
        self.log.ancestors_with_self(self.base, child)
    }

    #[inline]
    pub fn children(&self, node: K) -> &IntSet<K>
    where
        K: Into<u32>,
    {
        self.log.children(self.base, node)
    }

    #[inline]
    pub fn children_with_self(&self, node: K) -> impl Iterator<Item = K> + '_
    where
        K: From<u32> + Into<u32>,
    {
        self.log.children_with_self(self.base, node)
    }

    /// Iterator over cycle nodes
    #[inline]
    pub fn cycles(&self) -> impl Iterator<Item = K> + '_
    where
        K: From<u32>,
    {
        self.log.cycles(self.base)
    }

    #[inline]
    pub fn depth(&self, node: K) -> Result<usize, CycleError<K>>
    where
        K: From<u32> + Into<u32>,
    {
        self.log.depth(self.base, node)
    }

    #[inline]
    pub fn descendants(&self, parent: K) -> &IntSet<K>
    where
        K: Into<u32>,
    {
        self.log.descendants(self.base, parent)
    }

    #[inline]
    pub fn descendants_with_self(&self, parent: K) -> impl Iterator<Item = K> + '_
    where
        K: From<u32> + Into<u32>,
    {
        self.log.descendants_with_self(self.base, parent)
    }

    #[inline]
    pub fn has_cycle(&self, id: K) -> bool
    where
        K: Into<u32>,
    {
        self.log.has_cycle(self.base, id)
    }

    #[inline]
    pub fn is_descendant_of(&self, child: K, parent: K) -> bool
    where
        K: Into<u32>,
    {
        self.log.is_descendant_of(self.base, child, parent)
    }

    #[inline]
    pub fn parent(&self, child: K) -> Option<K>
    where
        K: From<u32> + Into<u32>,
    {
        self.log.parent(self.base, child)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CycleError<K>(pub K);

pub fn empty_tree<K>() -> &'static Tree<K> {
    let empty = u32based::tree::empty_tree();
    // SAFETY:
    // - `EMPTY_LOG` has static lifetime, hence the address is valid forever.
    // - `Tree<K>` is `#[repr(transparent)]` and zero-sized, so the
    //   reference to the inner value can be transmuted to a reference to the
    //   wrapper without changing the address or violating any aliasing rules.
    unsafe { core::mem::transmute(&empty) }
}

pub fn empty_tree_log<K>() -> &'static TreeIndexLog<K> {
    let empty = u32based::tree::empty_tree_log();
    // SAFETY:
    // - `EMPTY_LOG` has static lifetime, hence the address is valid forever.
    // - `TreeLog<K>` is `#[repr(transparent)]` and zero-sized, so the
    //   reference to the inner value can be transmuted to a reference to the
    //   wrapper without changing the address or violating any aliasing rules.
    unsafe { core::mem::transmute(&empty) }
}
