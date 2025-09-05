use super::{Tree, TreeLog};
use crate::interner::{IRoaringBitmap, default_i_roaring_bitmap};
use nohash::IntMap;
use roaring::RoaringBitmap;
use std::collections::hash_map::Entry;

#[derive(Clone, Default)]
pub struct NodeSetIndex {
    direct_items: IntMap<u32, IRoaringBitmap>,
    subtree_items: IntMap<u32, IRoaringBitmap>,
}

impl NodeSetIndex {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&mut self, log: NodeSetIndexLog) -> bool {
        fn apply_map(
            target: &mut IntMap<u32, IRoaringBitmap>,
            source: IntMap<u32, RoaringBitmap>,
        ) -> bool {
            let mut changed = false;

            for (k, b) in source {
                match target.entry(k) {
                    Entry::Occupied(o) if b.is_empty() => {
                        o.remove();
                        changed = true;
                    }
                    Entry::Occupied(mut o) if b != *o.get() => {
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
        changed |= apply_map(&mut self.direct_items, log.direct_items);
        changed |= apply_map(&mut self.subtree_items, log.subtree_items);
        changed
    }

    #[inline]
    pub fn direct_items(&self, node: u32) -> &IRoaringBitmap {
        self.direct_items
            .get(&node)
            .unwrap_or_else(|| default_i_roaring_bitmap())
    }

    #[inline]
    pub fn subtree_items(&self, node: u32) -> &IRoaringBitmap {
        self.subtree_items
            .get(&node)
            .unwrap_or_else(|| default_i_roaring_bitmap())
    }

    pub fn values(&self) -> RoaringBitmap {
        let mut b = RoaringBitmap::new();

        for bm in self.direct_items.values() {
            b |= &**bm;
        }

        b
    }
}

#[derive(Clone, Default)]
pub struct NodeSetIndexLog {
    direct_items: IntMap<u32, RoaringBitmap>,
    subtree_items: IntMap<u32, RoaringBitmap>,
}

impl NodeSetIndexLog {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn direct_items<'a>(&'a self, base: &'a NodeSetIndex, node: u32) -> &'a RoaringBitmap {
        self.direct_items
            .get(&node)
            .unwrap_or_else(|| base.direct_items(node))
    }

    fn direct_items_mut(&mut self, base: &NodeSetIndex, node: u32) -> &mut RoaringBitmap {
        self.direct_items
            .entry(node)
            .or_insert_with(|| base.direct_items(node).into())
    }

    pub fn insert(
        &mut self,
        base: &NodeSetIndex,
        base_tree: &Tree,
        log_tree: &TreeLog,
        node: u32,
        item: u32,
    ) {
        if self.direct_items_mut(base, node).insert(item) {
            // Get all ancestors including those in cycles
            let mut visited = std::collections::HashSet::new();
            let mut stack = vec![node];

            while let Some(current) = stack.pop() {
                if visited.insert(current) {
                    // Insert item for this ancestor
                    self.subtree_items_mut(base, current).insert(item);

                    // Add parents to stack
                    if let Some(parent) = log_tree.parent(base_tree, current) {
                        stack.push(parent);
                    }
                }
            }
        }
    }

    /// Re-insert a previously detached subtree.
    /// The parent of `data.root` is taken from the current tree state;
    /// no tree mutation occurs.
    pub fn insert_subtree(
        &mut self,
        base: &NodeSetIndex,
        base_tree: &Tree,
        log_tree: &TreeLog,
        data: DetachedSubtree,
    ) {
        // 1. Restore the bitmaps that belong strictly to the detached nodes.
        for (id, b) in data.direct {
            self.direct_items.insert(id, b);
        }

        for (id, b) in data.subtree {
            self.subtree_items.insert(id, b);
        }

        // 2. All items now in the subtree.
        let subtree_items: RoaringBitmap = self
            .subtree_items
            .get(&data.root)
            .into_iter()
            .flat_map(|b| b.iter())
            .collect();

        // 3. Push those items up every ancestor **above** the root’s parent.
        let mut cur = log_tree.parent(base_tree, data.root);
        let mut seen = std::collections::HashSet::new();

        while let Some(ancestor) = cur {
            if !seen.insert(ancestor) {
                break; // cycle
            }

            let dst = self.subtree_items_mut(base, ancestor);
            *dst |= &subtree_items;
            cur = log_tree.parent(base_tree, ancestor);
        }
    }

    pub fn remove(
        &mut self,
        base: &NodeSetIndex,
        base_tree: &Tree,
        log_tree: &TreeLog,
        node: u32,
        item: u32,
    ) {
        if !self.direct_items_mut(base, node).remove(item) {
            return;
        }

        // Use the same approach as insert to handle cycles
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![node];

        while let Some(current) = stack.pop() {
            if visited.insert(current) {
                // Remove item from this ancestor
                self.subtree_items_mut(base, current).remove(item);

                // Add parents to stack
                if let Some(parent) = log_tree.parent(base_tree, current) {
                    // Only push if we haven't visited it yet
                    if !visited.contains(&parent) {
                        stack.push(parent);
                    }
                }
            }
        }
    }

    /// Detach the entire subtree rooted at `root`, returning its index data.
    pub fn remove_subtree(
        &mut self,
        base: &NodeSetIndex,
        base_tree: &Tree,
        log_tree: &TreeLog,
        root: u32,
    ) -> DetachedSubtree {
        let nodes: RoaringBitmap = log_tree
            .descendants_with_self(base_tree, root)
            .iter()
            .collect();

        let mut direct = IntMap::default();
        let mut subtree = IntMap::default();

        for id in nodes.iter() {
            if let Some(b) = self.direct_items.remove(&id) {
                direct.insert(id, b);
            } else if let Some(b) = base.direct_items.get(&id) {
                direct.insert(id, b.into());
            }

            if let Some(b) = self.subtree_items.remove(&id) {
                subtree.insert(id, b);
            } else if let Some(b) = base.subtree_items.get(&id) {
                subtree.insert(id, b.into());
            }
        }

        DetachedSubtree {
            root,
            direct,
            subtree,
        }
    }

    #[inline]
    pub fn subtree_items<'a>(&'a self, base: &'a NodeSetIndex, node: u32) -> &'a RoaringBitmap {
        self.subtree_items
            .get(&node)
            .unwrap_or_else(|| base.subtree_items(node))
    }

    fn subtree_items_mut(&mut self, base: &NodeSetIndex, node: u32) -> &mut RoaringBitmap {
        self.subtree_items
            .entry(node)
            .or_insert_with(|| base.subtree_items(node).into())
    }
}

#[derive(Clone, Default)]
pub struct DetachedSubtree {
    root: u32,
    direct: IntMap<u32, RoaringBitmap>,
    subtree: IntMap<u32, RoaringBitmap>,
}

pub struct ItemsView<'a> {
    node: u32,
    inner: &'a RoaringBitmap,
}

impl<'a> ItemsView<'a> {
    #[inline]
    pub fn contains(&self, value: u32) -> bool {
        value == self.node || self.inner.contains(value)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        false
    }

    #[inline]
    pub fn iter(&self) -> std::iter::Chain<std::iter::Once<u32>, roaring::bitmap::Iter<'a>> {
        std::iter::once(self.node).chain(self.inner.iter())
    }

    #[inline]
    pub fn len(&self) -> u64 {
        1 + self.inner.len()
    }

    #[inline]
    pub fn to_bitmap(&self) -> RoaringBitmap {
        let mut b = self.inner.clone();
        b.insert(self.node);
        b
    }
}

impl From<ItemsView<'_>> for RoaringBitmap {
    #[inline]
    fn from(v: ItemsView<'_>) -> Self {
        v.to_bitmap()
    }
}

impl<'a> IntoIterator for ItemsView<'a> {
    type Item = u32;
    type IntoIter = std::iter::Chain<std::iter::Once<u32>, roaring::bitmap::Iter<'a>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a ItemsView<'a> {
    type Item = u32;
    type IntoIter = std::iter::Chain<std::iter::Once<u32>, roaring::bitmap::Iter<'a>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /* ---------- helper utilities ---------- */
    fn collect_log(log: &NodeSetIndexLog, base: &NodeSetIndex, n: u32) -> HashSet<u32> {
        log.subtree_items(base, n).iter().collect()
    }

    /// Panics if any invariant is broken.
    fn check_invariants(
        base: &NodeSetIndex,
        log: &NodeSetIndexLog,
        tree: &Tree,
        log_tree: &TreeLog,
    ) {
        // collect every node that exists in the tree (root = 0 is *not* guaranteed)
        let all_nodes: RoaringBitmap = {
            let mut b = RoaringBitmap::new();
            // walk the tree and collect every reachable id
            let mut stack = vec![];
            for r in tree.children(0).iter() {
                stack.push(r);
            }
            while let Some(n) = stack.pop() {
                b.insert(n);
                stack.extend(tree.children(n).iter());
            }
            b
        };

        for node in all_nodes.iter() {
            let mut expected = HashSet::new();
            // union of direct_items(d) for every descendant d of node
            for d in log_tree.descendants_with_self(tree, node) {
                expected.extend(log.direct_items(base, d).iter());
            }

            let actual = collect_log(log, base, node);
            assert_eq!(
                expected, actual,
                "subtree invariant violated for node {node}"
            );
        }
    }

    /* ---------- tests ---------- */

    #[test]
    fn simple_insert_remove() {
        let base_tree = Tree::new();
        let mut log_tree = TreeLog::new();
        let mut log = NodeSetIndexLog::new();
        let base = NodeSetIndex::new();

        log_tree.insert(&base_tree, None, 1);
        log_tree.insert(&base_tree, Some(1), 2);
        log_tree.insert(&base_tree, Some(2), 3);

        log.insert(&base, &base_tree, &log_tree, 3, 42);
        check_invariants(&base, &log, &base_tree, &log_tree);

        log.insert(&base, &base_tree, &log_tree, 2, 43);
        check_invariants(&base, &log, &base_tree, &log_tree);

        log.remove(&base, &base_tree, &log_tree, 2, 43);
        check_invariants(&base, &log, &base_tree, &log_tree);
    }

    #[test]
    fn remove_insert_subtree() {
        let base_tree = Tree::new();
        let mut log_tree = TreeLog::new();
        let mut log = NodeSetIndexLog::new();
        let base = NodeSetIndex::new();

        // 1 → 2 → 3
        log_tree.insert(&base_tree, None, 1);
        log_tree.insert(&base_tree, Some(1), 2);
        log_tree.insert(&base_tree, Some(2), 3);

        log.insert(&base, &base_tree, &log_tree, 3, 100);
        log.insert(&base, &base_tree, &log_tree, 2, 101);
        check_invariants(&base, &log, &base_tree, &log_tree);

        // detach 2..=3
        let snap = log.remove_subtree(&base, &base_tree, &log_tree, 2);
        check_invariants(&base, &log, &base_tree, &log_tree);

        // move tree: 1 → 4 → 2 → 3
        log_tree.insert(&base_tree, Some(1), 4);
        log_tree.insert(&base_tree, Some(4), 2); // re-attach 2 under 4

        log.insert_subtree(&base, &base_tree, &log_tree, snap);
        check_invariants(&base, &log, &base_tree, &log_tree);

        // 100 and 101 must now appear for 1,4,2,3
        assert!(collect_log(&log, &base, 1).contains(&100));
        assert!(collect_log(&log, &base, 1).contains(&101));
    }

    #[test]
    fn apply_preserves_invariants() {
        let base_tree = Tree::new();
        let mut log_tree = TreeLog::new();
        let mut log = NodeSetIndexLog::new();
        let mut base = NodeSetIndex::new();

        // build 1 → 2 → 3
        log_tree.insert(&base_tree, None, 1);
        log_tree.insert(&base_tree, Some(1), 2);
        log_tree.insert(&base_tree, Some(2), 3);

        log.insert(&base, &base_tree, &log_tree, 2, 7);
        log.insert(&base, &base_tree, &log_tree, 3, 8);

        check_invariants(&base, &log, &base_tree, &log_tree);

        // apply snapshot into base
        base.apply(log.clone());
        check_invariants(&base, &log, &base_tree, &log_tree);
    }

    #[test]
    fn empty_cases() {
        let base_tree = Tree::new();
        let log_tree = TreeLog::new();
        let log = NodeSetIndexLog::new();
        let base = NodeSetIndex::new();

        // no crash on empty nodes
        check_invariants(&base, &log, &base_tree, &log_tree);
    }
}
