use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;

pub struct OneIndex<V> {
    data: Vec<Option<V>>,
    len: usize,
}

impl<V> OneIndex<V> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
            len: 0,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            len: 0,
        }
    }

    pub fn apply(&mut self, log: OneIndexLog<V>) -> bool
    where
        V: PartialEq,
    {
        let mut changes = false;

        let new_len = log
            .0
            .iter()
            .filter(|(_, v)| v.is_some())
            .map(|(k, _)| *k as usize + 1)
            .max()
            .unwrap_or_default();

        if self.data.len() < new_len {
            self.data.resize_with(new_len, || None);
        }

        for (index, value) in log.0 {
            let index = index as usize;

            match value {
                Some(v) => {
                    let new = Some(v);
                    let slot = unsafe { self.data.get_unchecked_mut(index) };

                    if *slot != new {
                        if slot.is_none() {
                            self.len += 1;
                        }

                        *slot = new;
                        changes = true;
                    }
                }
                None => {
                    if let Some(slot) = self.data.get_mut(index) {
                        let old = slot.take();

                        if old.is_some() {
                            changes = true;
                            self.len -= 1;
                        }
                    }
                }
            }
        }

        self.trim();

        changes
    }

    #[inline]
    pub fn get(&self, index: u32) -> Option<&V> {
        self.data.get(index as usize).and_then(|v| v.as_ref())
    }

    /// Direct write used while building; returns whether anything changed.
    pub(crate) fn insert(&mut self, index: u32, value: V) -> bool
    where
        V: PartialEq,
    {
        let index = index as usize;

        if self.data.len() <= index {
            self.data.resize_with(index + 1, || None);
        }

        let slot = &mut self.data[index];

        match slot {
            Some(old) if *old == value => false,
            Some(old) => {
                *old = value;
                true
            }
            None => {
                *slot = Some(value);
                self.len += 1;
                true
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (u32, &V)> + '_ {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.as_ref().map(|v| (i as u32, v)))
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn keys(&self) -> impl Iterator<Item = u32> + '_ {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.as_ref().map(|_| i as u32))
    }

    /// Drops trailing empty slots so iteration and memory track the highest live key.
    fn trim(&mut self) {
        let live = self
            .data
            .iter()
            .rposition(Option::is_some)
            .map_or(0, |i| i + 1);
        self.data.truncate(live);

        if self.data.capacity() > 2 * self.data.len() + 16 {
            self.data.shrink_to_fit();
        }
    }
}

impl<V> Default for OneIndex<V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<V> FromIterator<(u32, V)> for OneIndex<V>
where
    V: PartialEq,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (u32, V)>,
    {
        let iter = iter.into_iter();
        let mut index = OneIndex::with_capacity(iter.size_hint().0);

        for (k, v) in iter {
            index.insert(k, v);
        }

        index
    }
}

pub struct OneIndexLog<V>(
    // Some = insert / replace,
    // None = remove
    FxHashMap<u32, Option<V>>,
);

impl<V> OneIndexLog<V> {
    #[inline]
    pub fn new() -> Self {
        Self(FxHashMap::default())
    }

    #[inline]
    pub fn get<'a>(&'a self, base: &'a OneIndex<V>, index: u32) -> Option<&'a V> {
        match self.0.get(&index) {
            Some(v) => v.as_ref(),
            _ => base.get(index),
        }
    }

    pub fn insert(&mut self, base: &OneIndex<V>, index: u32, value: V)
    where
        V: PartialEq,
    {
        let new = Some(value);

        match self.0.entry(index) {
            Entry::Vacant(e) => {
                if base.data.get(index as usize).is_none_or(|v| *v != new) {
                    e.insert(new);
                }
            }
            Entry::Occupied(mut e) => {
                e.insert(new);
            }
        }
    }

    pub fn remove(&mut self, base: &OneIndex<V>, index: u32)
    where
        V: PartialEq,
    {
        match self.0.entry(index) {
            Entry::Vacant(e) => {
                if base.get(index).is_some() {
                    e.insert(None);
                }
            }
            Entry::Occupied(mut e) => {
                e.insert(None);
            }
        }
    }
}

impl<V> Default for OneIndexLog<V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter_last_write_wins_and_tracks_len() {
        let idx: OneIndex<&str> = vec![(5, "a"), (1, "b"), (5, "c")].into_iter().collect();

        assert_eq!(idx.len(), 2);
        assert_eq!(idx.get(5), Some(&"c"));
        assert_eq!(idx.get(1), Some(&"b"));
        assert_eq!(idx.get(3), None);
        assert_eq!(idx.get(99), None);
        assert_eq!(idx.keys().collect::<Vec<_>>(), vec![1, 5]);
    }

    #[test]
    fn insert_reports_changes() {
        let mut idx = OneIndex::new();

        assert!(idx.insert(2, 10));
        assert!(!idx.insert(2, 10));
        assert!(idx.insert(2, 11));
        assert_eq!(idx.len(), 1);
    }

    #[test]
    fn apply_trims_trailing_empty_slots() {
        let mut idx: OneIndex<u32> = (0..10).map(|k| (k, k)).collect();
        let mut log = OneIndexLog::new();

        for k in 5..10 {
            log.remove(&idx, k);
        }

        assert!(idx.apply(log));
        assert_eq!(idx.len(), 5);
        assert_eq!(idx.data.len(), 5);
        assert_eq!(
            idx.iter().map(|(k, _)| k).collect::<Vec<_>>(),
            (0..5).collect::<Vec<_>>()
        );
    }

    #[test]
    fn log_skips_noops() {
        let base: OneIndex<u32> = (0..4).map(|k| (k, k)).collect();
        let mut log = OneIndexLog::new();

        log.insert(&base, 1, 1);
        log.remove(&base, 7);
        assert!(log.0.is_empty());

        log.insert(&base, 1, 2);
        log.remove(&base, 3);
        assert_eq!(log.get(&base, 1), Some(&2));
        assert_eq!(log.get(&base, 3), None);
        assert_eq!(log.get(&base, 0), Some(&0));

        let mut idx: OneIndex<u32> = (0..4).map(|k| (k, k)).collect();
        assert!(idx.apply(log));
        assert_eq!(idx.get(1), Some(&2));
        assert_eq!(idx.get(3), None);
        assert_eq!(idx.len(), 3);
    }

    #[test]
    fn apply_grows_and_counts() {
        let mut idx = OneIndex::new();
        let mut log = OneIndexLog::new();

        log.insert(&idx, 100, 'x');
        log.insert(&idx, 3, 'y');

        assert!(idx.apply(log));
        assert_eq!(idx.len(), 2);
        assert_eq!(idx.get(100), Some(&'x'));
        assert!(!idx.apply(OneIndexLog::new()));
    }
}
