use crate::align::{align_up, align_down};

/// Size of a cache line in bytes on x86_64 Intel/AMD processors.
pub const CACHE_LINE_BYTES: u64 = 64;

/// log₂(`CACHE_LINE_BYTES`) — the right-shift used to convert an address to a cache line index.
pub const CACHE_LINE_BITS:  u64 = 6;

/// Round `size` up to a multiple of the cache line size.
///
/// Use this to pad structures that must not share a cache line
/// with adjacent data (e.g., per-CPU counters, spinlock hot fields).
#[inline(always)]
pub const fn cache_align_size(size: u64) -> u64 {
    align_up(size, CACHE_LINE_BYTES)
}

/// Return the cache line number containing `addr`.
#[inline(always)]
pub const fn cache_line_of(addr: u64) -> u64 {
    addr >> CACHE_LINE_BITS
}

/// Return the offset of `addr` within its cache line (0..63).
#[inline(always)]
pub const fn cache_line_offset(addr: u64) -> u64 {
    addr & (CACHE_LINE_BYTES - 1)
}

/// Return `true` if the range [addr, addr+size) fits in a single cache line.
///
/// A range that crosses a cache-line boundary causes two cache transactions
/// instead of one — a "cache line split" — which is a significant penalty
/// on hot paths.
#[inline(always)]
pub const fn is_cache_line_contained(addr: u64, size: u64) -> bool {
    debug_assert!(size <= CACHE_LINE_BYTES, "size exceeds one cache line");
    cache_line_of(addr) == cache_line_of(addr + size - 1)
}

/// Return the number of cache lines touched by the range [addr, addr+size).
#[inline(always)]
pub const fn cache_lines_spanned(addr: u64, size: u64) -> u64 {
    if size == 0 { return 0; }
    cache_line_of(addr + size - 1) - cache_line_of(addr) + 1
}

/// Align `addr` down to the start of its cache line.
#[inline(always)]
pub const fn cache_line_start(addr: u64) -> u64 {
    align_down(addr, CACHE_LINE_BYTES)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- CACHE_LINE_BYTES / CACHE_LINE_BITS ---

    #[test_case]
    fn constants_are_consistent() {
        assert_eq!(1u64 << CACHE_LINE_BITS, CACHE_LINE_BYTES);
        assert_eq!(CACHE_LINE_BYTES, 64);
    }

    // --- cache_align_size ---

    #[test_case]
    fn cache_align_size_exact_multiple() {
        assert_eq!(cache_align_size(0), 0);
        assert_eq!(cache_align_size(64), 64);
        assert_eq!(cache_align_size(128), 128);
    }

    #[test_case]
    fn cache_align_size_rounds_up() {
        assert_eq!(cache_align_size(1), 64);
        assert_eq!(cache_align_size(63), 64);
        assert_eq!(cache_align_size(65), 128);
        assert_eq!(cache_align_size(127), 128);
    }

    #[test_case]
    fn cache_align_size_result_always_multiple_of_line() {
        for size in [1u64, 7, 31, 33, 63, 64, 65, 100, 127, 128, 200] {
            assert_eq!(cache_align_size(size) % CACHE_LINE_BYTES, 0);
        }
    }

    // --- cache_line_of ---

    #[test_case]
    fn cache_line_of_basic() {
        assert_eq!(cache_line_of(0), 0);
        assert_eq!(cache_line_of(63), 0);
        assert_eq!(cache_line_of(64), 1);
        assert_eq!(cache_line_of(127), 1);
        assert_eq!(cache_line_of(128), 2);
    }

    #[test_case]
    fn cache_line_of_large_address() {
        let addr = 64 * 1000;
        assert_eq!(cache_line_of(addr), 1000);
    }

    // --- cache_line_offset ---

    #[test_case]
    fn cache_line_offset_at_boundary_is_zero() {
        assert_eq!(cache_line_offset(0), 0);
        assert_eq!(cache_line_offset(64), 0);
        assert_eq!(cache_line_offset(128), 0);
    }

    #[test_case]
    fn cache_line_offset_within_line() {
        assert_eq!(cache_line_offset(1), 1);
        assert_eq!(cache_line_offset(63), 63);
        assert_eq!(cache_line_offset(65), 1);
        assert_eq!(cache_line_offset(127), 63);
    }

    #[test_case]
    fn cache_line_offset_range() {
        for addr in 0u64..256 {
            assert!(cache_line_offset(addr) < CACHE_LINE_BYTES);
        }
    }

    // --- cache_line_start ---

    #[test_case]
    fn cache_line_start_at_boundary() {
        assert_eq!(cache_line_start(0), 0);
        assert_eq!(cache_line_start(64), 64);
        assert_eq!(cache_line_start(128), 128);
    }

    #[test_case]
    fn cache_line_start_rounds_down() {
        assert_eq!(cache_line_start(1), 0);
        assert_eq!(cache_line_start(63), 0);
        assert_eq!(cache_line_start(65), 64);
        assert_eq!(cache_line_start(127), 64);
    }

    #[test_case]
    fn cache_line_start_matches_line_of() {
        for addr in [0u64, 1, 33, 63, 64, 65, 100, 127, 128, 200, 255] {
            assert_eq!(cache_line_start(addr), cache_line_of(addr) * CACHE_LINE_BYTES);
        }
    }

    // --- is_cache_line_contained ---

    #[test_case]
    fn contained_entirely_within_one_line() {
        assert!(is_cache_line_contained(0, 1));
        assert!(is_cache_line_contained(0, 64));
        assert!(is_cache_line_contained(64, 1));
        assert!(is_cache_line_contained(64, 64));
        assert!(is_cache_line_contained(10, 20));
    }

    #[test_case]
    fn not_contained_crosses_boundary() {
        assert!(!is_cache_line_contained(63, 2));  // 63..64 crosses line boundary
        assert!(!is_cache_line_contained(1, 64));  // 1..64 crosses at 64
        assert!(!is_cache_line_contained(56, 16)); // 56..71 crosses at 64
    }

    // --- cache_lines_spanned ---

    #[test_case]
    fn zero_size_spans_zero_lines() {
        assert_eq!(cache_lines_spanned(0, 0), 0);
        assert_eq!(cache_lines_spanned(100, 0), 0);
    }

    #[test_case]
    fn single_byte_spans_one_line() {
        assert_eq!(cache_lines_spanned(0, 1), 1);
        assert_eq!(cache_lines_spanned(63, 1), 1);
        assert_eq!(cache_lines_spanned(64, 1), 1);
    }

    #[test_case]
    fn exactly_one_cache_line_spans_one() {
        assert_eq!(cache_lines_spanned(0, 64), 1);
        assert_eq!(cache_lines_spanned(64, 64), 1);
    }

    #[test_case]
    fn crossing_boundary_spans_two() {
        assert_eq!(cache_lines_spanned(63, 2), 2);
        assert_eq!(cache_lines_spanned(1, 64), 2);
        assert_eq!(cache_lines_spanned(56, 16), 2);
    }

    #[test_case]
    fn large_range_spans_correct_lines() {
        // 0..256 = exactly 4 cache lines
        assert_eq!(cache_lines_spanned(0, 256), 4);
        // 1..257: starts at line 0, ends at byte 257 (line 4) → 5 lines
        assert_eq!(cache_lines_spanned(1, 256), 5);
    }
}
