//! A fixed-size bitset backed by a `[u64; N]` array.
//!
//! [`BitArray<N>`] requires no heap allocation — it lives wherever the type is
//! placed (stack, static, or an arena allocation). Designed as the backing
//! data structure for a physical memory frame allocator.

/// A fixed-size bitmap with `N * 64` bits.
///
/// Each bit represents one resource (e.g., a physical page frame). A set bit
/// conventionally means "in use"; a clear bit means "free". Use
/// [`BitArray::alloc_first_free`] to atomically find and claim the first
/// available resource.
///
/// `N` is the number of 64-bit words. Total capacity = `N * 64` bits.
///
/// # Example
/// ```rust
/// # use bitwise::bitmap::BitArray;
/// // Track 512 × 4 KiB frames (2 MiB physical range).
/// let mut map: BitArray<8> = BitArray::new();
/// let frame = map.alloc_first_free().expect("no free frames");
/// assert_eq!(frame, 0);
/// assert!(map.test(0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitArray<const N: usize> {
    words: [u64; N],
}

impl<const N: usize> BitArray<N> {
    /// Create an all-zero (all-free) bitmap.
    pub const fn new() -> Self {
        Self { words: [0u64; N] }
    }

    /// Total bit capacity (`N * 64`).
    pub const fn capacity() -> usize {
        N * 64
    }

    /// Set bit `index` (mark as in-use).
    ///
    /// # Panics (debug)
    /// Panics if `index >= N * 64`.
    #[inline(always)]
    pub fn set(&mut self, index: usize) {
        debug_assert!(index < N * 64, "bitmap index out of range");
        let word = index >> 6;
        let bit  = (index & 63) as u32;
        self.words[word] = crate::bits::bit_set(self.words[word], bit);
    }

    /// Clear bit `index` (mark as free).
    ///
    /// # Panics (debug)
    /// Panics if `index >= N * 64`.
    #[inline(always)]
    pub fn clear(&mut self, index: usize) {
        debug_assert!(index < N * 64, "bitmap index out of range");
        let word = index >> 6;
        let bit  = (index & 63) as u32;
        self.words[word] = crate::bits::bit_clear(self.words[word], bit);
    }

    /// Return `true` if bit `index` is set.
    ///
    /// `const fn` — can be called at compile time.
    #[inline(always)]
    pub const fn test(&self, index: usize) -> bool {
        let word = index >> 6;
        let bit  = (index & 63) as u32;
        crate::bits::bit_test(self.words[word], bit)
    }

    /// Find and set the first clear (free) bit.
    ///
    /// Returns the bit index on success, or `None` if all bits are set.
    /// The bit is atomically claimed — no separate `set` call is needed.
    #[inline]
    pub fn alloc_first_free(&mut self) -> Option<usize> {
        for i in 0..N {
            if self.words[i] != u64::MAX {
                // At least one bit is clear; find it via the LSB of the complement.
                let free_bit = crate::bits::lowest_set_bit_index(!self.words[i]).unwrap();
                let index = i * 64 + free_bit as usize;
                self.words[i] = crate::bits::bit_set(self.words[i], free_bit);
                return Some(index);
            }
        }
        None
    }

    /// Count the number of set (in-use) bits.
    #[inline]
    pub fn count_set(&self) -> usize {
        let mut count = 0usize;
        for i in 0..N {
            count += crate::bits::popcount(self.words[i]) as usize;
        }
        count
    }

    /// Count the number of clear (free) bits.
    #[inline]
    pub fn count_free(&self) -> usize {
        N * 64 - self.count_set()
    }

    /// Set all bits (mark everything as in-use).
    #[inline]
    pub fn set_all(&mut self) {
        for i in 0..N {
            self.words[i] = u64::MAX;
        }
    }

    /// Clear all bits (mark everything as free).
    #[inline]
    pub fn clear_all(&mut self) {
        for i in 0..N {
            self.words[i] = 0;
        }
    }

    /// Iterate over the indices of all set bits in ascending order.
    pub fn iter_set(&self) -> SetBitIter<'_, N> {
        let first_word = if N > 0 { self.words[0] } else { 0 };
        SetBitIter { bitmap: self, word_idx: 0, word_val: first_word }
    }
}

impl<const N: usize> Default for BitArray<N> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Iterator over set bits
// ---------------------------------------------------------------------------

/// Iterator yielding the indices of all set bits in a [`BitArray`].
pub struct SetBitIter<'a, const N: usize> {
    bitmap:   &'a BitArray<N>,
    word_idx: usize,
    word_val: u64,
}

impl<'a, const N: usize> Iterator for SetBitIter<'a, N> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        loop {
            if self.word_val != 0 {
                let bit = crate::bits::lowest_set_bit_index(self.word_val).unwrap();
                self.word_val = crate::bits::clear_lowest_set_bit(self.word_val);
                return Some(self.word_idx * 64 + bit as usize);
            }
            // Current word exhausted — advance to the next non-zero word.
            self.word_idx += 1;
            if self.word_idx >= N {
                return None;
            }
            self.word_val = self.bitmap.words[self.word_idx];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_all_free() {
        let b: BitArray<4> = BitArray::new();
        assert_eq!(b.count_set(), 0);
        assert_eq!(b.count_free(), 256);
    }

    #[test]
    fn capacity() {
        assert_eq!(BitArray::<1>::capacity(), 64);
        assert_eq!(BitArray::<4>::capacity(), 256);
        assert_eq!(BitArray::<8>::capacity(), 512);
    }

    #[test]
    fn set_clear_test_roundtrip() {
        let mut b: BitArray<2> = BitArray::new();
        for i in [0usize, 1, 63, 64, 127] {
            assert!(!b.test(i));
            b.set(i);
            assert!(b.test(i));
            b.clear(i);
            assert!(!b.test(i));
        }
    }

    #[test]
    fn alloc_first_free_sequential() {
        let mut b: BitArray<1> = BitArray::new();
        for expected in 0..64 {
            assert_eq!(b.alloc_first_free(), Some(expected));
        }
        // All bits full.
        assert_eq!(b.alloc_first_free(), None);
    }

    #[test]
    fn alloc_after_free() {
        let mut b: BitArray<1> = BitArray::new();
        let a = b.alloc_first_free().unwrap(); // 0
        let _ = b.alloc_first_free().unwrap(); // 1
        b.clear(a);
        // Next alloc should reclaim index 0.
        assert_eq!(b.alloc_first_free(), Some(0));
    }

    #[test]
    fn count_set_and_free_consistent() {
        let mut b: BitArray<2> = BitArray::new();
        b.set(3);
        b.set(67);
        assert_eq!(b.count_set(), 2);
        assert_eq!(b.count_free(), BitArray::<2>::capacity() - 2);
    }

    #[test]
    fn set_all_then_clear_all() {
        let mut b: BitArray<2> = BitArray::new();
        b.set_all();
        assert_eq!(b.count_free(), 0);
        b.clear_all();
        assert_eq!(b.count_set(), 0);
    }

    #[test]
    fn iter_set_visits_all_set_bits() {
        let mut b: BitArray<2> = BitArray::new();
        let indices = [0usize, 1, 63, 64, 65, 127];
        for &i in &indices {
            b.set(i);
        }
        let mut collected: [usize; 6] = [0; 6];
        let mut n = 0;
        for idx in b.iter_set() {
            collected[n] = idx;
            n += 1;
        }
        assert_eq!(n, 6);
        assert_eq!(&collected[..], &indices[..]);
    }

    #[test]
    fn iter_set_empty_bitmap() {
        let b: BitArray<2> = BitArray::new();
        assert_eq!(b.iter_set().count(), 0);
    }

    #[test]
    fn iter_set_count_matches_count_set() {
        let mut b: BitArray<4> = BitArray::new();
        for i in (0..256).step_by(7) {
            b.set(i);
        }
        assert_eq!(b.iter_set().count(), b.count_set());
    }

    #[test]
    fn const_test_at_compile_time() {
        const B: BitArray<1> = BitArray::new();
        const _: bool = B.test(0);
    }
}
