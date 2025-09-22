use crate::{U32Set, empty_roaring};
use intern::IU32HashSet;
use nohash::{IntMap, IntSet};
use std::{
    collections::{hash_map::Entry, hash_set},
    mem::replace,
    sync::OnceLock,
};

type Set = IntSet<u32>;

#[derive(Clone, Default)]
pub struct Tree {
    children: IntMap<u32, IU32HashSet>,
    cycles: Set,
    descendants: IntMap<u32, IU32HashSet>,
    parents: IntMap<u32, u32>,
}

impl Tree {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ancestors(&self, node: u32) -> TreeAncestorIter<'_> {
        let mut it = self.ancestors_with_self(node);
        it.next();
        it
    }

    pub fn ancestors_with_self(&self, node: u32) -> TreeAncestorIter<'_> {
        TreeAncestorIter {
            child: Some(node),
            cycles: &self.cycles,
            parents: &self.parents,
        }
    }

    /// Applies an entire `TreeLog` snapshot to this tree.
    /// Returns `true` if anything changed.
    pub fn apply(&mut self, log: TreeLog) -> bool {
        fn apply_bitmap(
            target: &mut IntMap<u32, IU32HashSet>,
            source: IntMap<u32, U32Set>,
        ) -> bool {
            let mut changed = false;

            for (k, b) in source {
                match target.entry(k) {
                    Entry::Occupied(o) if b.is_empty() => {
                        o.remove();
                        changed = true;
                    }
                    Entry::Occupied(mut o) if b != *o.get().as_set() => {
                        o.insert(b.into());
                        changed = true;
                    }
                    Entry::Vacant(v) if !b.is_empty() => {
                        v.insert(b.into());
                        changed = true;
                    }
                    _ => {}
                }
            }

            if changed {
                target.shrink_to_fit();
            }

            changed
        }

        let mut changed = false;

        // ---------- cycles ----------
        if let Some(c) = log.cycles {
            if self.cycles != c {
                self.cycles = c;
                changed = true;
            }
        }

        // ---------- parents ----------
        for (child, new_parent) in log.parents {
            changed |= match new_parent {
                Some(p) => self.parents.insert(child, p).is_none() || self.parents[&child] != p,
                None => self.parents.remove(&child).is_some(),
            };
        }

        if changed {
            self.parents.shrink_to_fit();
        }

        // ---------- children & descendants ----------
        changed |= apply_bitmap(&mut self.children, log.children);
        changed |= apply_bitmap(&mut self.descendants, log.descendants);

        changed
    }

    pub fn all_nodes(&self) -> U32Set {
        let mut b = U32Set::default();

        for (&c, set) in &self.children {
            b.insert(c);
            b.extend(set.as_set().iter().copied());
        }

        for (&p, &n) in &self.parents {
            b.insert(p);
            b.insert(n);
        }

        b
    }

    pub fn children(&self, node: u32) -> &U32Set {
        self.children
            .get(&node)
            .map_or_else(|| empty_roaring(), IU32HashSet::as_set)
    }

    #[inline]
    pub fn children_with_self(&self, node: u32) -> ItemsView<'_> {
        ItemsView {
            node,
            inner: self.children(node),
        }
    }

    #[inline]
    pub fn cycles(&self) -> hash_set::Iter<'_, u32> {
        self.cycles.iter()
    }

    pub fn depth(&self, node: u32) -> Result<usize, CycleError> {
        let mut cur = Some(node);
        let mut d = 0;
        while let Some(n) = cur {
            if self.has_cycle(n) {
                return Err(CycleError(n));
            }
            d += 1;
            cur = self.parent(n);
        }
        Ok(d)
    }

    pub fn descendants(&self, node: u32) -> &U32Set {
        self.descendants
            .get(&node)
            .map_or_else(|| empty_roaring(), IU32HashSet::as_set)
    }

    #[inline]
    pub fn descendants_with_self(&self, node: u32) -> ItemsView<'_> {
        ItemsView {
            node,
            inner: self.descendants(node),
        }
    }

    #[inline]
    pub fn has_cycle(&self, node: u32) -> bool {
        self.cycles.contains(&node)
    }

    #[inline]
    pub fn is_descendant_of(&self, child: u32, parent: u32) -> bool {
        self.descendants(parent).contains(&child)
    }

    #[inline]
    pub fn parent(&self, child: u32) -> Option<u32> {
        self.parents.get(&child).copied()
    }
}

impl FromIterator<(u32, Option<u32>)> for Tree {
    fn from_iter<I: IntoIterator<Item = (u32, Option<u32>)>>(iter: I) -> Self {
        let mut log = TreeLog::new();

        for (child, parent) in iter {
            log.parents.insert(child, parent);
        }

        let mut tree = Tree::new();
        tree.apply(log);
        tree
    }
}

pub struct ItemsView<'a> {
    node: u32,
    inner: &'a U32Set,
}

impl<'a> ItemsView<'a> {
    #[inline]
    pub fn contains(&self, value: u32) -> bool {
        value == self.node || self.inner.contains(&value)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        false
    }

    #[inline]
    pub fn iter(
        &self,
    ) -> std::iter::Chain<std::iter::Once<u32>, std::iter::Copied<hash_set::Iter<'a, u32>>> {
        std::iter::once(self.node).chain(self.inner.iter().copied())
    }

    #[inline]
    pub fn len(&self) -> u64 {
        1 + self.inner.len() as u64
    }

    #[inline]
    pub fn to_bitmap(&self) -> U32Set {
        let mut b = self.inner.clone();
        b.insert(self.node);
        b
    }
}

impl From<ItemsView<'_>> for U32Set {
    #[inline]
    fn from(value: ItemsView<'_>) -> Self {
        value.to_bitmap()
    }
}

impl<'a> IntoIterator for ItemsView<'a> {
    type Item = u32;
    type IntoIter =
        std::iter::Chain<std::iter::Once<u32>, std::iter::Copied<hash_set::Iter<'a, u32>>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a ItemsView<'a> {
    type Item = u32;
    type IntoIter =
        std::iter::Chain<std::iter::Once<u32>, std::iter::Copied<hash_set::Iter<'a, u32>>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone, Default)]
pub struct TreeLog {
    children: IntMap<u32, U32Set>,
    cycles: Option<Set>,
    descendants: IntMap<u32, U32Set>,
    parents: IntMap<u32, Option<u32>>,
}

impl TreeLog {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ancestors<'a>(&'a self, base: &'a Tree, node: u32) -> TreeLogAncestorIter<'a> {
        let mut it = self.ancestors_with_self(base, node);
        it.next();
        it
    }

    pub fn ancestors_with_self<'a>(&'a self, base: &'a Tree, node: u32) -> TreeLogAncestorIter<'a> {
        TreeLogAncestorIter {
            child: Some(node),
            cycles: self.cycles(base),
            log: self,
            base,
        }
    }

    pub fn children<'a>(&'a self, base: &'a Tree, node: u32) -> &'a U32Set {
        self.children
            .get(&node)
            .unwrap_or_else(|| base.children(node))
    }

    fn children_mut(&mut self, base: &Tree, node: u32) -> &mut U32Set {
        self.children
            .entry(node)
            .or_insert_with(|| base.children(node).clone())
    }

    #[inline]
    pub fn children_with_self<'a>(&'a self, base: &'a Tree, node: u32) -> ItemsView<'a> {
        ItemsView {
            node,
            inner: self.children(base, node),
        }
    }

    #[inline]
    pub fn cycles<'a>(&'a self, base: &'a Tree) -> &'a Set {
        self.cycles.as_ref().unwrap_or(&base.cycles)
    }

    fn cycles_mut(&mut self, base: &Tree) -> &mut Set {
        self.cycles.get_or_insert_with(|| base.cycles.clone())
    }

    pub fn depth(&self, base: &Tree, node: u32) -> Result<usize, CycleError> {
        let mut cur = Some(node);
        let mut depth = 0;
        let cycles = self.cycles(base);

        while let Some(current) = cur {
            if cycles.contains(&current) {
                return Err(CycleError(current));
            }
            depth += 1;
            cur = self.parent(base, current);
        }
        Ok(depth)
    }

    pub fn descendants<'a>(&'a self, base: &'a Tree, node: u32) -> &'a U32Set {
        self.descendants
            .get(&node)
            .unwrap_or_else(|| base.descendants(node))
    }

    fn descendants_mut(&mut self, base: &Tree, node: u32) -> &mut U32Set {
        self.descendants
            .entry(node)
            .or_insert_with(|| base.descendants(node).clone())
    }

    #[inline]
    pub fn descendants_with_self<'a>(&'a self, base: &'a Tree, node: u32) -> ItemsView<'a> {
        ItemsView {
            node,
            inner: self.descendants(base, node),
        }
    }

    /// Marks every node that belongs to a cycle **reachable from `start`**
    /// by walking the current (log + base) parent chain.
    fn detect_and_mark_cycles(&mut self, base: &Tree, start: u32) {
        let mut seen = IntSet::default();
        let mut path = Vec::new();
        let mut cur = Some(start);

        while let Some(node) = cur {
            if seen.contains(&node) {
                // found a cycle; mark every node in the loop
                let idx = path.iter().position(|&x| x == node).unwrap();
                for &n in &path[idx..] {
                    self.cycles_mut(base).insert(n);
                }
                self.cycles_mut(base).insert(node);
                break;
            }
            seen.insert(node);
            path.push(node);
            cur = self.parent(base, node);
        }
    }

    #[inline]
    pub fn has_cycle(&self, base: &Tree, node: u32) -> bool {
        self.cycles.as_ref().unwrap_or(&base.cycles).contains(&node)
    }

    pub fn insert(&mut self, base: &Tree, parent: Option<u32>, child: u32) {
        if self.parent(base, child) == parent {
            return;
        }

        let mut visited = IntSet::default();
        let removed_items = self.remove_impl(base, child, &mut visited);
        self.reparent_subtree(base, parent, child, removed_items, &mut visited);
        self.detect_and_mark_cycles(base, child);
    }

    #[inline]
    pub fn is_descendant_of(&self, base: &Tree, child: u32, parent: u32) -> bool {
        self.descendants(base, parent).contains(&child)
    }

    pub fn parent(&self, base: &Tree, child: u32) -> Option<u32> {
        match self.parents.get(&child) {
            Some(&opt) => opt,
            None => base.parent(child),
        }
    }

    fn parent_mut(&mut self, base: &Tree, child: u32) -> &mut Option<u32> {
        self.parents
            .entry(child)
            .or_insert_with(|| base.parent(child))
    }

    pub fn remove(&mut self, base: &Tree, node: u32) {
        let mut visited = IntSet::default();
        self.remove_impl(base, node, &mut visited);

        self.cycles_mut(base).clear();

        let parents = self.parents.keys().copied().collect::<Vec<_>>();

        for node in parents {
            self.detect_and_mark_cycles(base, node);
        }
    }

    fn remove_impl(
        &mut self,
        base: &Tree,
        node: u32,
        visited: &mut IntSet<u32>,
    ) -> IntMap<u32, RemoveItem> {
        // ----------------------------------------------------------
        // 1.  Gather the full subtree (node + descendants)
        // ----------------------------------------------------------
        let desc = replace(self.descendants_mut(base, node), U32Set::default());
        let chil = replace(self.children_mut(base, node), U32Set::default());

        // ----------------------------------------------------------
        // 2.  Record state for every node in the subtree
        // ----------------------------------------------------------
        let mut removed = IntMap::default();

        for &id in desc.iter() {
            removed.insert(
                id,
                RemoveItem {
                    children: replace(self.children_mut(base, id), U32Set::default()),
                    descendants: replace(self.descendants_mut(base, id), U32Set::default()),
                    parent: self.parent_mut(base, id).take(),
                },
            );
        }

        // ----------------------------------------------------------
        // 3.  Detach from former parent
        // ----------------------------------------------------------
        if let Some(p) = self.parent(base, node) {
            self.children_mut(base, p).remove(&node);
        }

        // ----------------------------------------------------------
        // 4.  Shrink ancestors' descendants
        // ----------------------------------------------------------
        let mut cur = self.parent(base, node);

        while let Some(p) = cur {
            if !visited.insert(p) {
                break;
            }

            let d = self.descendants_mut(base, p);

            d.remove(&node);
            d.retain(|k| !desc.contains(k));

            cur = self.parent(base, p);
        }

        removed.insert(
            node,
            RemoveItem {
                children: chil,
                descendants: desc,
                parent: self.parent_mut(base, node).take(),
            },
        );

        removed
    }

    /* ---- reparenting ---- */
    fn reparent_subtree(
        &mut self,
        base: &Tree,
        new_parent: Option<u32>,
        root: u32,
        mut removed: IntMap<u32, RemoveItem>, // <- now mut
        visited: &mut IntSet<u32>,
    ) {
        // 1. Re-attach root
        self.parents.insert(root, new_parent);

        if let Some(p) = new_parent {
            self.children_mut(base, p).insert(root);
        }

        let item = removed.remove(&root).unwrap_or_default();

        // 3–4. ancestor rebuild & cycle check stay the same
        let mut cur = new_parent;

        visited.clear();

        while let Some(p) = cur {
            if !visited.insert(p) {
                break;
            }

            let d = self.descendants_mut(base, p);
            d.extend(item.descendants.iter().copied());
            d.insert(root);

            cur = self.parent(base, p);
        }

        self.children.insert(root, item.children);
        self.descendants.insert(root, item.descendants);

        for (node, item) in removed {
            self.parents.insert(node, item.parent);
            self.children.insert(node, item.children);
            self.descendants.insert(node, item.descendants);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CycleError(pub u32);

#[derive(Clone, Default)]
struct RemoveItem {
    children: U32Set,
    descendants: U32Set,
    parent: Option<u32>,
}

#[derive(Clone)]
pub struct TreeAncestorIter<'a> {
    child: Option<u32>,
    cycles: &'a Set,
    parents: &'a IntMap<u32, u32>,
}

impl Iterator for TreeAncestorIter<'_> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        let n = self.child.take()?;

        self.child = if self.cycles.contains(&n) {
            None
        } else {
            self.parents.get(&n).copied()
        };

        Some(n)
    }
}

#[derive(Clone)]
pub struct TreeLogAncestorIter<'a> {
    child: Option<u32>,
    cycles: &'a Set,
    log: &'a TreeLog,
    base: &'a Tree,
}

impl Iterator for TreeLogAncestorIter<'_> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        let n = self.child.take()?;

        self.child = if self.cycles.contains(&n) {
            None
        } else {
            self.log.parent(self.base, n)
        };

        Some(n)
    }
}

pub fn empty_tree() -> &'static Tree {
    static EMPTY: OnceLock<Tree> = OnceLock::new();
    EMPTY.get_or_init(Tree::default)
}

pub fn empty_tree_log() -> &'static TreeLog {
    static EMPTY: OnceLock<TreeLog> = OnceLock::new();
    EMPTY.get_or_init(TreeLog::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    /* ---------- helpers ---------- */
    fn collect_children(log: &TreeLog, base: &Tree, node: u32) -> Vec<u32> {
        log.children_with_self(base, node)
            .iter()
            .collect::<Vec<_>>()
    }

    fn collect_descendants(log: &TreeLog, base: &Tree, node: u32) -> Vec<u32> {
        log.descendants_with_self(base, node)
            .iter()
            .collect::<Vec<_>>()
    }

    /* ---------- basic insert & remove ---------- */
    #[test]
    fn simple_insert_remove() {
        let mut log = TreeLog::new();
        let base = Tree::new();

        log.insert(&base, None, 1);
        log.insert(&base, Some(1), 2);

        assert_eq!(collect_children(&log, &base, 1), vec![1, 2]);
        assert_eq!(collect_descendants(&log, &base, 1), vec![1, 2]);
        assert_eq!(log.parent(&base, 2), Some(1));

        log.remove(&base, 2);
        assert_eq!(collect_children(&log, &base, 1), vec![1]);
        assert_eq!(collect_descendants(&log, &base, 1), vec![1]);
        assert_eq!(log.parent(&base, 2), None);
    }

    /* ---------- deep subtree ---------- */
    #[test]
    fn deep_tree() {
        let mut log = TreeLog::new();
        let base = Tree::new();

        // 0 → 1 → 2 → 3 → 4
        log.insert(&base, None, 0);
        log.insert(&base, Some(0), 1);
        log.insert(&base, Some(1), 2);
        log.insert(&base, Some(2), 3);
        log.insert(&base, Some(3), 4);

        assert_eq!(log.depth(&base, 4).unwrap(), 5);
        assert_eq!(collect_descendants(&log, &base, 0), vec![0, 1, 2, 3, 4]);

        log.remove(&base, 2); // removes 2,3,4
        assert_eq!(collect_descendants(&log, &base, 0), vec![0, 1]);
        assert_eq!(log.depth(&base, 1).unwrap(), 2);
    }

    /* ---------- cycle detection ---------- */
    #[test]
    fn detect_cycle() {
        let mut log = TreeLog::new();
        let base = Tree::new();

        log.insert(&base, None, 1);
        log.insert(&base, Some(1), 2);
        log.insert(&base, Some(2), 3);

        // create 3 → 1 cycle
        log.insert(&base, Some(3), 1);

        assert!(log.has_cycle(&base, 1));
        assert!(log.has_cycle(&base, 2));
        assert!(log.has_cycle(&base, 3));

        // depth must error on any node in the cycle
        assert!(log.depth(&base, 1).is_err());
        assert!(log.depth(&base, 2).is_err());
        assert!(log.depth(&base, 3).is_err());

        // break the cycle
        log.remove(&base, 3);
        assert!(!log.has_cycle(&base, 1));
        assert!(log.depth(&base, 3).is_ok());
    }

    /* ---------- apply round-trip ---------- */
    #[test]
    fn apply_round_trip() {
        let base = Tree::new();
        let mut log = TreeLog::new();

        log.insert(&base, None, 5);
        log.insert(&base, Some(5), 6);
        log.insert(&base, Some(5), 7);

        let mut other = Tree::new();
        assert!(other.apply(log.clone()));

        assert_eq!(collect_children(&log, &other, 5), vec![5, 6, 7]);
        assert_eq!(collect_descendants(&log, &other, 5), vec![5, 6, 7]);
    }
}
