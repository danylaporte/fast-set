use fxhash::FxHashMap;
use roaring::RoaringBitmap;
use std::{
    borrow::{Borrow, Cow},
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    ops::Deref,
    ptr::{NonNull, addr_eq},
    sync::{Mutex, OnceLock},
};

#[static_init::dynamic]
static INTERNER: Mutex<FxHashMap<Key, u32>> = Mutex::new(FxHashMap::default());

#[static_init::dynamic]
static DEFAULT_INTERNED: IRoaringBitmap = intern(Cow::Owned(RoaringBitmap::new()));

#[repr(transparent)]
struct Bitmap(RoaringBitmap);

impl Eq for Bitmap {}

impl Hash for Bitmap {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_bitmap(&self.0).hash(state);
    }
}

impl PartialEq for Bitmap {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

fn hash_bitmap(bitmap: &RoaringBitmap) -> u64 {
    bitmap.iter().fold(0u64, |h, v| h ^ hash_single(v))
}

fn hash_single(v: u32) -> u64 {
    let mut hasher = fxhash::FxHasher::default();
    v.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Copy)]
struct Key(NonNull<Bitmap>);

impl Key {
    #[inline]
    fn as_bitmap(&self) -> &Bitmap {
        unsafe { self.0.as_ref() }
    }
}

impl Borrow<Bitmap> for Key {
    #[inline]
    fn borrow(&self) -> &Bitmap {
        self.as_bitmap()
    }
}

impl Eq for Key {}

impl Hash for Key {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bitmap().hash(state);
    }
}

impl PartialEq<Bitmap> for Key {
    #[inline]
    fn eq(&self, other: &Bitmap) -> bool {
        self.as_bitmap() == other
    }
}

impl PartialEq<RoaringBitmap> for Key {
    #[inline]
    fn eq(&self, other: &RoaringBitmap) -> bool {
        &self.as_bitmap().0 == other
    }
}

impl PartialEq for Key {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        addr_eq(self.0.as_ptr(), other.0.as_ptr())
    }
}

unsafe impl Send for Key {}
unsafe impl Sync for Key {}

/// An interned RoaringBitmap. This can only be cloned as it increments a counter.
/// On the last IAdaptiveBitmap remaining, the buffer of the bitmap is deallocated.
#[repr(transparent)]
pub struct IRoaringBitmap(Key);

impl IRoaringBitmap {
    #[inline]
    pub fn new(b: Cow<'_, RoaringBitmap>) -> Self {
        intern(b)
    }

    #[inline]
    pub fn as_bitmap(&self) -> &RoaringBitmap {
        &self.0.as_bitmap().0
    }

    #[inline]
    pub fn to_bitmap(&self) -> RoaringBitmap {
        self.as_bitmap().clone()
    }
}

impl Borrow<RoaringBitmap> for IRoaringBitmap {
    #[inline]
    fn borrow(&self) -> &RoaringBitmap {
        &self.0.as_bitmap().0
    }
}

impl Clone for IRoaringBitmap {
    fn clone(&self) -> Self {
        let mut gate = INTERNER.lock().unwrap();

        if let Some(count) = gate.get_mut(&self.0) {
            *count += 1;
        };

        Self(Key(self.0.0))
    }
}

impl Debug for IRoaringBitmap {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.as_bitmap(), f)
    }
}

impl Default for IRoaringBitmap {
    #[inline]
    fn default() -> Self {
        DEFAULT_INTERNED.clone()
    }
}

impl Deref for IRoaringBitmap {
    type Target = RoaringBitmap;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0.as_bitmap().0
    }
}

impl Drop for IRoaringBitmap {
    fn drop(&mut self) {
        let mut gate = INTERNER.lock().unwrap();

        if let Some(count) = gate.get_mut(&self.0) {
            *count -= 1;

            if *count == 0 {
                gate.remove(&self.0);

                // manually drop when count is zero. no more reference.
                drop(unsafe { Box::from_raw(self.0.0.as_ptr()) });
            }
        }
    }
}

impl Eq for IRoaringBitmap {}

impl From<&RoaringBitmap> for IRoaringBitmap {
    #[inline]
    fn from(b: &RoaringBitmap) -> Self {
        intern(Cow::Borrowed(b))
    }
}

impl From<RoaringBitmap> for IRoaringBitmap {
    #[inline]
    fn from(b: RoaringBitmap) -> Self {
        intern(Cow::Owned(b))
    }
}

impl From<&IRoaringBitmap> for RoaringBitmap {
    #[inline]
    fn from(b: &IRoaringBitmap) -> Self {
        b.to_bitmap()
    }
}

impl From<IRoaringBitmap> for RoaringBitmap {
    #[inline]
    fn from(b: IRoaringBitmap) -> Self {
        b.to_bitmap()
    }
}

impl Hash for IRoaringBitmap {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_bitmap().hash(state);
    }
}

impl PartialEq for IRoaringBitmap {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<RoaringBitmap> for IRoaringBitmap {
    #[inline]
    fn eq(&self, other: &RoaringBitmap) -> bool {
        self.as_bitmap() == other
    }
}

impl PartialEq<IRoaringBitmap> for RoaringBitmap {
    #[inline]
    fn eq(&self, other: &IRoaringBitmap) -> bool {
        self == other.as_bitmap()
    }
}

pub fn default_i_roaring_bitmap<'a>() -> &'a IRoaringBitmap {
    &DEFAULT_INTERNED
}

pub fn empty_roaring() -> &'static RoaringBitmap {
    static EMPTY: OnceLock<RoaringBitmap> = OnceLock::new();
    EMPTY.get_or_init(RoaringBitmap::new)
}

fn intern(mut b: Cow<'_, RoaringBitmap>) -> IRoaringBitmap {
    // attempt to optimize outside the lock to maximize the performance.
    if let Cow::Owned(b) = &mut b {
        b.optimize();
    }

    let mut gate = INTERNER.lock().unwrap();
    let r: &RoaringBitmap = &b;
    let q = unsafe { &*(r as *const RoaringBitmap as *const Bitmap) };

    if let Some(v) = gate.get_key_value(q) {
        let key = Key(v.0.0);

        unsafe {
            *gate.get_mut(&key).unwrap_unchecked() += 1;
        };

        return IRoaringBitmap(key);
    }

    let b = match b {
        Cow::Borrowed(b) => {
            let mut b = b.clone();
            b.optimize(); // ensure the bitmap is optimized at this point.
            b
        }
        Cow::Owned(b) => b,
    };

    let boxed = Box::new(Bitmap(b));
    let key = unsafe { Key(NonNull::new_unchecked(Box::into_raw(boxed))) };

    gate.insert(key, 1);

    IRoaringBitmap(key)
}

#[cfg(test)]
mod interner_tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::{
        hash::{Hash, Hasher},
        ptr,
    };

    /* ---------- helpers ---------- */

    fn hash_of<T: Hash>(t: &T) -> u64 {
        let mut h = DefaultHasher::new();
        t.hash(&mut h);
        h.finish()
    }

    /* ---------- basic functionality ---------- */

    #[test]
    fn intern_same_bitmap_gives_same_handle() {
        let rb = RoaringBitmap::from_iter([1, 2, 3]);
        let a = IRoaringBitmap::from(&rb);
        let b = IRoaringBitmap::from(&rb);
        assert!(ptr::eq(a.0.0.as_ptr(), b.0.0.as_ptr()));
    }

    #[test]
    fn intern_different_bitmaps_give_different_handles() {
        let a = IRoaringBitmap::from(RoaringBitmap::from_iter([1, 2, 3]));
        let b = IRoaringBitmap::from(RoaringBitmap::from_iter([4, 5]));
        assert!(!ptr::eq(a.0.0.as_ptr(), b.0.0.as_ptr()));
    }

    #[test]
    fn eq_and_hash_work() {
        let rb = RoaringBitmap::from_iter([10, 20]);
        let a = IRoaringBitmap::from(&rb);
        let b = IRoaringBitmap::from(&rb);
        assert_eq!(a, b);
        assert_eq!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn partial_eq_roaringbitmap() {
        let rb = RoaringBitmap::from_iter([1, 2, 3]);
        let ib = IRoaringBitmap::from(&rb);
        assert_eq!(ib, rb);
        assert_eq!(rb, ib);
    }

    /* ---------- clone / drop / ref-counting ---------- */

    #[test]
    fn clone_increments_refcount() {
        let rb = RoaringBitmap::from_iter([1, 2]);
        let a = IRoaringBitmap::from(&rb);
        let gate = INTERNER.lock().unwrap();
        let count = *gate.get(&a.0).unwrap();
        drop(gate);

        let _b = a.clone();
        let gate = INTERNER.lock().unwrap();
        assert_eq!(*gate.get(&a.0).unwrap(), count + 1);
    }

    /* ---------- deref / borrow ---------- */

    #[test]
    fn deref_works() {
        let rb = RoaringBitmap::from_iter([1, 2, 3]);
        let ib = IRoaringBitmap::from(&rb);
        assert_eq!(ib.iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn borrow_trait_works() {
        fn takes_ref(r: &RoaringBitmap) -> usize {
            r.len() as usize
        }
        let ib = IRoaringBitmap::from(RoaringBitmap::from_iter([7, 8, 9]));
        assert_eq!(takes_ref(ib.borrow()), 3);
    }

    /* ---------- default / static singleton ---------- */

    #[test]
    fn default_gives_singleton() {
        let a = IRoaringBitmap::default();
        let b = IRoaringBitmap::default();
        assert!(ptr::eq(a.0.0.as_ptr(), b.0.0.as_ptr()));
        assert!(a.is_empty());
    }

    /* ---------- concurrent access ---------- */

    #[test]
    fn concurrent_intern_is_safe() {
        use std::thread;

        let rb = RoaringBitmap::from_iter(0..1000);
        let mut handles = vec![];

        for _ in 0..8 {
            let rb = rb.clone();
            handles.push(thread::spawn(move || {
                let ib = IRoaringBitmap::from(rb);
                assert_eq!(ib.len(), 1000);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    }

    /// Stress test for many clones
    #[test]
    fn miri_cycle_many_clones() {
        let rb = RoaringBitmap::from_iter(0..100);
        let ib = IRoaringBitmap::from(&rb);
        let mut v = vec![ib.clone(); 100];
        let _ = v.pop();
        v.clear();
    }
}
