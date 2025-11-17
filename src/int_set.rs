use crate::U32Set;
use std::{
    collections::hash_set,
    marker::PhantomData,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign},
};

#[repr(transparent)]
pub struct IntSet<K>(U32Set, PhantomData<K>);

impl<K> IntSet<K> {
    #[inline]
    pub fn new() -> Self {
        Self(U32Set::default(), PhantomData)
    }

    /// # Safety
    /// The caller of the method must ensure that the generic parameter K
    /// correctly transpose to the bit representation.
    #[inline]
    pub const unsafe fn from_set(bitmap: U32Set) -> Self {
        Self(bitmap, PhantomData)
    }

    /// # Safety
    /// The caller of the method must ensure that the generic parameter K
    /// correctly transpose to the bit representation.
    ///
    /// The caller must also ensure that the raw bytes of `bitmap`
    /// are valid for `IntSet<K>` (they are, because `IntSet<K>` is
    /// `#[repr(transparent)]` over `AdaptiveBitmap`).
    #[inline]
    pub unsafe fn from_u32set_ref(bitmap: &U32Set) -> &IntSet<K> {
        // SAFETY: `IntSet<K>` is `#[repr(transparent)]` and its only
        // non-zero-sized field is the `AdaptiveBitmap`.  Therefore
        // `&AdaptiveBitmap` and `&IntSet<K>` have identical layout.
        unsafe { &*(bitmap as *const U32Set as *const IntSet<K>) }
    }

    #[inline]
    pub fn as_set(&self) -> &U32Set {
        &self.0
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub fn contains(&self, key: K) -> bool
    where
        K: Into<u32>,
    {
        self.0.contains(&key.into())
    }

    #[inline]
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = K>,
        K: Into<u32>,
    {
        self.0.extend(iter.into_iter().map(Into::into))
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn insert(&mut self, key: K) -> bool
    where
        K: Into<u32>,
    {
        self.0.insert(key.into())
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K>
    where
        K: TryFrom<u32>,
    {
        Iter(self.0.iter(), PhantomData)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn remove(&mut self, key: K) -> bool
    where
        K: Into<u32>,
    {
        self.0.remove(&key.into())
    }
}

impl<K> Clone for IntSet<K> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<K> Default for IntSet<K> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K> FromIterator<K> for IntSet<K>
where
    K: Into<u32>,
{
    fn from_iter<I: IntoIterator<Item = K>>(iter: I) -> Self {
        IntSet(
            U32Set::from_iter(iter.into_iter().map(Into::into)),
            PhantomData,
        )
    }
}

impl<K> IntoIterator for IntSet<K>
where
    K: TryFrom<u32>,
{
    type Item = K;
    type IntoIter = IntoIter<K>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter(), PhantomData)
    }
}

impl<'a, K> IntoIterator for &'a IntSet<K>
where
    K: TryFrom<u32>,
{
    type Item = K;
    type IntoIter = Iter<'a, K>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K> PartialEq for IntSet<K> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

pub struct IntoIter<K>(hash_set::IntoIter<u32>, PhantomData<K>);

impl<K> Iterator for IntoIter<K>
where
    K: TryFrom<u32>,
{
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().and_then(|k| K::try_from(k).ok())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct Iter<'a, K>(hash_set::Iter<'a, u32>, PhantomData<K>);

impl<K> Iterator for Iter<'_, K>
where
    K: TryFrom<u32>,
{
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().and_then(|v| K::try_from(*v).ok())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K> ExactSizeIterator for Iter<'_, K>
where
    K: TryFrom<u32>,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

// 2. IntSet <op>= &IntSet
impl<K> BitAndAssign<&IntSet<K>> for IntSet<K> {
    #[inline]
    fn bitand_assign(&mut self, rhs: &IntSet<K>) {
        self.0.retain(|k| rhs.0.contains(k));
    }
}

impl<K> BitOrAssign<&IntSet<K>> for IntSet<K> {
    #[inline]
    fn bitor_assign(&mut self, rhs: &IntSet<K>) {
        self.0.extend(rhs.0.iter().copied());
    }
}

impl<K> SubAssign<&IntSet<K>> for IntSet<K> {
    #[inline]
    fn sub_assign(&mut self, rhs: &IntSet<K>) {
        self.0.retain(|k| !rhs.0.contains(k));
    }
}

macro_rules! op {
    ($trait:ident, $method:ident, $trait_assign:ident, $method_assign:ident) => {
        // 1. &IntSet & &IntSet  -> IntSet
        impl<K> $trait<&IntSet<K>> for &IntSet<K> {
            type Output = IntSet<K>;

            #[inline]
            fn $method(self, rhs: &IntSet<K>) -> IntSet<K> {
                IntSet((&self.0).$method(&rhs.0), PhantomData)
            }
        }

        // 3. IntSet <op>= IntSet   (delegates to &)
        impl<K> $trait_assign<IntSet<K>> for IntSet<K> {
            #[inline]
            fn $method_assign(&mut self, rhs: IntSet<K>) {
                self.$method_assign(&rhs);
            }
        }

        // 4. IntSet <op> IntSet   (delegates to &)
        impl<K> $trait<IntSet<K>> for IntSet<K> {
            type Output = IntSet<K>;

            #[inline]
            fn $method(self, rhs: IntSet<K>) -> IntSet<K> {
                (&self).$method(&rhs)
            }
        }

        // 5. &IntSet <op> IntSet   (delegates to &)
        impl<K> $trait<IntSet<K>> for &IntSet<K> {
            type Output = IntSet<K>;

            #[inline]
            fn $method(self, rhs: IntSet<K>) -> IntSet<K> {
                self.$method(&rhs)
            }
        }

        // 6. IntSet <op> &IntSet   (delegates to &)
        impl<K> $trait<&IntSet<K>> for IntSet<K> {
            type Output = IntSet<K>;

            #[inline]
            fn $method(self, rhs: &IntSet<K>) -> IntSet<K> {
                (&self).$method(rhs)
            }
        }
    };
}

op!(BitAnd, bitand, BitAndAssign, bitand_assign);
op!(BitOr, bitor, BitOrAssign, bitor_assign);
op!(Sub, sub, SubAssign, sub_assign);
