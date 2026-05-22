
use core::option::Option::{self, Some, None};

/// Set bit `n` in `value`. Bit 0 is the least significant.
#[inline(always)]
pub const fn bit_set(value: u64, n: u32) -> u64 {
    debug_assert!(n < 64, "bit index out of range");
    value | (1u64 << n)
}

/// Clear bit `n` in `value`.
#[inline(always)]
pub const fn bit_clear(value: u64, n: u32) -> u64 {
    debug_assert!(n < 64, "bit index out of range");
    value & !(1u64 << n)
}

/// Toggle bit `n` in `value`.
#[inline(always)]
pub const fn bit_toggle(value: u64, n: u32) -> u64 {
    debug_assert!(n < 64, "bit index out of range");
    value ^ (1u64 << n)
}

/// Test whether bit `n` is set. Returns `true` if set.
#[inline(always)]
pub const fn bit_test(value: u64, n: u32) -> bool {
    debug_assert!(n < 64, "bit index out of range");
    (value >> n) & 1 == 1
}

/// Extract a contiguous bit field from bits `[high:low]` inclusive.
///
/// The returned value is right-justified (i.e., shifted to start at bit 0).
///
/// # Example
/// ```no_run
/// # use bitwise::bits::bit_field_get;
/// # let selector: u64 = 0x001B;
/// // Extract the privilege level (bits 13:12) of an x86 segment selector
/// let cpl = bit_field_get(selector, 13, 12);
/// ```
#[inline(always)]
pub const fn bit_field_get(value: u64, high: u32, low: u32) -> u64 {
    debug_assert!(high >= low,  "high must be >= low");
    debug_assert!(high < 64,    "high bit index out of range");
    let width = high - low + 1;
    let mask = if width == 64 { u64::MAX } else { (1u64 << width) - 1 };
    (value >> low) & mask
}

/// Insert `field` into bits `[high:low]` of `value`, leaving all other bits unchanged.
///
/// `field` is treated as right-justified — it will be shifted left by `low` bits
/// before insertion. Bits of `field` above the field width are silently masked off.
#[inline(always)]
pub const fn bit_field_set(value: u64, high: u32, low: u32, field: u64) -> u64 {
    debug_assert!(high >= low, "high must be >= low");
    debug_assert!(high < 64,   "high bit index out of range");
    let width = high - low + 1;
    let mask = if width == 64 { u64::MAX } else { (1u64 << width) - 1 };
    let positioned_mask = mask << low;
    let positioned_field = (field & mask) << low;
    (value & !positioned_mask) | positioned_field
}

/// Return a bitmask with bits `[high:low]` set and all others clear.
#[inline(always)]
pub const fn bit_mask(high: u32, low: u32) -> u64 {
    debug_assert!(high >= low, "high must be >= low");
    debug_assert!(high < 64,   "high bit index out of range");
    let width = high - low + 1;
    let base = if width == 64 { u64::MAX } else { (1u64 << width) - 1 };
    base << low
}

/// Return `true` if `n` is a power of two.
#[inline(always)]
pub const fn is_power_of_two(n: u64) -> bool {
    n != 0 && (n & n.wrapping_sub(1)) == 0
}

/// Round `n` up to the next power of two.
///
/// Returns 1 for input 0. Panics in debug if the result would overflow u64.
#[inline(always)]
pub const fn next_power_of_two(n: u64) -> u64 {
    if n <= 1 { return 1; }
    let leading = (n - 1).leading_zeros();
    debug_assert!(leading > 0, "next_power_of_two would overflow u64");
    1u64 << (64 - leading)
}

/// Isolate the lowest set bit of `n` (also called the least significant set bit).
///
/// Returns 0 if `n == 0`. The result is always a power of two.
/// Maps to a single `BLSI` instruction on x86_64 with BMI1.
#[inline(always)]
pub const fn lowest_set_bit(n: u64) -> u64 {
    n & n.wrapping_neg()
}

/// Clear the lowest set bit of `n`.
///
/// Maps to a single `BLSR` instruction on x86_64 with BMI1.
#[inline(always)]
pub const fn clear_lowest_set_bit(n: u64) -> u64 {
    n & (n - 1)
}

/// Return the index (0-based) of the lowest set bit, or `None` if `n == 0`.
#[inline(always)]
pub const fn lowest_set_bit_index(n: u64) -> Option<u32> {
    if n == 0 { None } else { Some(n.trailing_zeros()) }
}

/// Return the index (0-based) of the highest set bit, or `None` if `n == 0`.
///
/// Equivalent to floor(log₂(n)). Maps to `BSR` / `LZCNT` on x86_64.
#[inline(always)]
pub const fn highest_set_bit_index(n: u64) -> Option<u32> {
    if n == 0 { None } else { Some(63 - n.leading_zeros()) }
}

/// Count the number of set bits (population count / Hamming weight).
///
/// Maps to the `POPCNT` instruction on x86_64.
#[inline(always)]
pub fn popcount(n: u64) -> u32 {
    n.count_ones()
}

/// Return `true` if `n` has an even number of set bits (even parity).
#[inline(always)]
pub fn even_parity(n: u64) -> bool {
    n.count_ones() % 2 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- bit_set / bit_clear / bit_toggle / bit_test ---

    #[cfg(test)]
    fn bit_set_zero_base() {
        assert_eq!(bit_set(0, 0), 1);
        assert_eq!(bit_set(0, 7), 0x80);
        assert_eq!(bit_set(0, 63), 1u64 << 63);
    }

    #[cfg(test)]
    fn bit_set_already_set_is_idempotent() {
        assert_eq!(bit_set(0xFF, 3), 0xFF);
    }

    #[cfg(test)]
    fn bit_clear_basics() {
        assert_eq!(bit_clear(0xFF, 0), 0xFE);
        assert_eq!(bit_clear(0xFF, 7), 0x7F);
        assert_eq!(bit_clear(0, 5), 0);
    }

    #[cfg(test)]
    fn bit_toggle_roundtrip() {
        for n in 0u32..64 {
            let val = 0u64;
            assert_eq!(bit_toggle(bit_toggle(val, n), n), val);
        }
    }

    #[cfg(test)]
    fn bit_test_set_and_clear() {
        let v = 0b1010_0101u64;
        assert!(bit_test(v, 0));
        assert!(!bit_test(v, 1));
        assert!(bit_test(v, 2));
        assert!(!bit_test(v, 3));
        assert!(bit_test(v, 7));
    }

    // --- bit_field_get ---

    #[cfg(test)]
    fn bit_field_get_low_nibble() {
        assert_eq!(bit_field_get(0xABCD, 3, 0), 0xD);
    }

    #[cfg(test)]
    fn bit_field_get_high_nibble() {
        assert_eq!(bit_field_get(0xABCD, 15, 12), 0xA);
    }

    #[cfg(test)]
    fn bit_field_get_single_bit() {
        assert_eq!(bit_field_get(0b1010, 1, 1), 1);
        assert_eq!(bit_field_get(0b1010, 0, 0), 0);
    }

    #[cfg(test)]
    fn bit_field_get_full_width() {
        assert_eq!(bit_field_get(u64::MAX, 63, 0), u64::MAX);
    }

    #[cfg(test)]
    fn bit_field_get_iopl_example() {
        // IOPL lives in RFLAGS bits 13:12; value 3 = ring 0 only
        let rflags: u64 = 3 << 12;
        assert_eq!(bit_field_get(rflags, 13, 12), 3);
    }

    // --- bit_field_set ---

    #[cfg(test)]
    fn bit_field_set_replaces_field() {
        let v = 0b1111_1111u64;
        assert_eq!(bit_field_set(v, 2, 0, 0b000), 0b1111_1000);
        assert_eq!(bit_field_set(v, 2, 0, 0b101), 0b1111_1101);
    }

    #[cfg(test)]
    fn bit_field_set_preserves_surrounding_bits() {
        let v: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        let result = bit_field_set(v, 7, 4, 0b0000);
        assert_eq!(result & !0xF0, v & !0xF0); // bits outside [7:4] unchanged
        assert_eq!((result >> 4) & 0xF, 0);    // bits [7:4] are 0
    }

    #[cfg(test)]
    fn bit_field_get_set_roundtrip() {
        let original = 0xDEAD_BEEF_1234_5678u64;
        for (hi, lo) in [(3u32, 0u32), (15, 8), (47, 32), (63, 56)] {
            let extracted = bit_field_get(original, hi, lo);
            let width = hi - lo + 1;
            let mask = if width == 64 { u64::MAX } else { (1u64 << width) - 1 };
            assert_eq!(bit_field_get(bit_field_set(original, hi, lo, extracted), hi, lo),
                       extracted & mask);
        }
    }

    // --- bit_mask ---

    #[cfg(test)]
    fn bit_mask_basic() {
        assert_eq!(bit_mask(3, 0), 0xF);
        assert_eq!(bit_mask(7, 4), 0xF0);
        assert_eq!(bit_mask(0, 0), 1);
    }

    #[cfg(test)]
    fn bit_mask_full_width() {
        assert_eq!(bit_mask(63, 0), u64::MAX);
    }

    #[cfg(test)]
    fn bit_mask_matches_field_set_clear() {
        let m = bit_mask(11, 8);
        // mask should equal what bit_field_set inserts with all-ones field
        let ones_in_field = bit_field_set(0, 11, 8, u64::MAX);
        assert_eq!(m, ones_in_field);
    }

    // --- is_power_of_two ---

    #[cfg(test)]
    fn is_power_of_two_true() {
        for exp in 0..64 {
            assert!(is_power_of_two(1u64 << exp), "1<<{exp} should be power-of-two");
        }
    }

    #[cfg(test)]
    fn is_power_of_two_false() {
        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(6));
        assert!(!is_power_of_two(0x1001));
        assert!(!is_power_of_two(u64::MAX));
    }

    // --- next_power_of_two ---

    #[cfg(test)]
    fn next_power_of_two_exact_powers() {
        assert_eq!(next_power_of_two(1), 1);
        assert_eq!(next_power_of_two(2), 2);
        assert_eq!(next_power_of_two(4), 4);
        assert_eq!(next_power_of_two(0x1000), 0x1000);
    }

    #[cfg(test)]
    fn next_power_of_two_rounds_up() {
        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(5), 8);
        assert_eq!(next_power_of_two(0x1001), 0x2000);
        assert_eq!(next_power_of_two(0), 1);
    }

    // --- lowest_set_bit / clear_lowest_set_bit ---

    #[cfg(test)]
    fn lowest_set_bit_isolates_lsb() {
        assert_eq!(lowest_set_bit(0b1010), 0b0010);
        assert_eq!(lowest_set_bit(0b1100), 0b0100);
        assert_eq!(lowest_set_bit(1), 1);
        assert_eq!(lowest_set_bit(u64::MAX), 1);
        assert_eq!(lowest_set_bit(0), 0);
    }

    #[cfg(test)]
    fn clear_lowest_set_bit_strips_lsb() {
        assert_eq!(clear_lowest_set_bit(0b1010), 0b1000);
        assert_eq!(clear_lowest_set_bit(0b1100), 0b1000);
        assert_eq!(clear_lowest_set_bit(1), 0);
    }

    #[cfg(test)]
    fn clear_lowest_set_bit_iteration_matches_popcount() {
        let original = 0b1011_0110_1001_1100u64;
        let expected_count = original.count_ones();
        let mut n = original;
        let mut count = 0u32;
        while n != 0 {
            n = clear_lowest_set_bit(n);
            count += 1;
        }
        assert_eq!(count, expected_count);
    }

    // --- lowest_set_bit_index / highest_set_bit_index ---

    #[cfg(test)]
    fn lowest_set_bit_index_basic() {
        assert_eq!(lowest_set_bit_index(0), None);
        assert_eq!(lowest_set_bit_index(1), Some(0));
        assert_eq!(lowest_set_bit_index(0b1000), Some(3));
        assert_eq!(lowest_set_bit_index(0b1010), Some(1));
        assert_eq!(lowest_set_bit_index(1u64 << 63), Some(63));
    }

    #[cfg(test)]
    fn highest_set_bit_index_basic() {
        assert_eq!(highest_set_bit_index(0), None);
        assert_eq!(highest_set_bit_index(1), Some(0));
        assert_eq!(highest_set_bit_index(0b1000), Some(3));
        assert_eq!(highest_set_bit_index(0b1010), Some(3));
        assert_eq!(highest_set_bit_index(u64::MAX), Some(63));
    }

    #[cfg(test)]
    fn highest_set_bit_index_is_floor_log2() {
        for exp in 0u32..64 {
            assert_eq!(highest_set_bit_index(1u64 << exp), Some(exp));
        }
    }

    // --- popcount / even_parity ---

    #[cfg(test)]
    fn popcount_basic() {
        assert_eq!(popcount(0), 0);
        assert_eq!(popcount(1), 1);
        assert_eq!(popcount(0xFF), 8);
        assert_eq!(popcount(u64::MAX), 64);
        assert_eq!(popcount(0b1011_0110), 5);
    }

    #[cfg(test)]
    fn even_parity_basic() {
        assert!(even_parity(0));           // 0 set bits → even
        assert!(!even_parity(1));          // 1 set bit → odd
        assert!(even_parity(0b11));        // 2 set bits → even
        assert!(!even_parity(0b111));      // 3 set bits → odd
        assert!(even_parity(u64::MAX));    // 64 set bits → even
    }
}
