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

        changes
    }

    #[inline]
    pub fn get(&self, index: u32) -> Option<&V> {
        self.data.get(index as usize).and_then(|v| v.as_ref())
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
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (u32, V)>,
    {
        let mut base = OneIndex::new();
        let mut log = OneIndexLog::new();

        for (k, v) in iter {
            log.insert(&base, k, v);
        }

        base.apply(log);
        base
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
                if base.data.get(index as usize).is_some() {
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
