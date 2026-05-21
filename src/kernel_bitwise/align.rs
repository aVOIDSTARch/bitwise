/// Round `addr` DOWN to the nearest multiple of `align`.
///
/// Equivalent to `addr - (addr % align)` when `align` is a power of two,
/// but implemented with a single AND and a NOT — no division instruction.
///
/// # Panics (debug)
/// Panics if `align` is zero or not a power of two.
#[inline(always)]
pub const fn align_down(addr: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two(), "align must be a power of two");
    addr & !(align - 1)
}

/// Round `addr` UP to the nearest multiple of `align`.
///
/// If `addr` is already aligned, it is returned unchanged.
///
/// # Panics (debug)
/// Panics if `align` is zero or not a power of two.
#[inline(always)]
pub const fn align_up(addr: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two(), "align must be a power of two");
    align_down(addr.wrapping_add(align - 1), align)
}

/// Returns `true` if `addr` is aligned to `align` bytes.
///
/// Equivalent to `addr % align == 0` for power-of-two alignments,
/// but reduces to a single AND instruction.
#[inline(always)]
pub const fn is_aligned(addr: u64, align: u64) -> bool {
    debug_assert!(align.is_power_of_two(), "align must be a power of two");
    (addr & (align - 1)) == 0
}

/// Returns the number of bytes between `addr` and the next aligned boundary.
///
/// Returns 0 if already aligned. The result is in [0, align).
#[inline(always)]
pub const fn align_offset(addr: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two(), "align must be a power of two");
    addr & (align - 1)
}

/// Compute the frame number for a physical address given a frame size.
///
/// For 4 KiB pages:  frame_number(0x8001_2000, 0x1000) = 0x8_0012
/// For 2 MiB pages:  frame_number(0x4000_0000, 0x20_0000) = 0x200
#[inline(always)]
pub const fn frame_number(phys_addr: u64, frame_size: u64) -> u64 {
    debug_assert!(frame_size.is_power_of_two(), "frame_size must be a power of two");
    phys_addr >> frame_size.trailing_zeros()
}

/// Reconstruct the base physical address from a frame number and frame size.
///
/// Inverse of `frame_number`.
#[inline(always)]
pub const fn frame_base(frame_num: u64, frame_size: u64) -> u64 {
    debug_assert!(frame_size.is_power_of_two(), "frame_size must be a power of two");
    frame_num << frame_size.trailing_zeros()
}

/// Returns the byte offset of `addr` within its containing frame.
///
/// Equivalent to `addr % frame_size` for power-of-two frame sizes.
#[inline(always)]
pub const fn frame_offset(addr: u64, frame_size: u64) -> u64 {
    debug_assert!(frame_size.is_power_of_two(), "frame_size must be a power of two");
    addr & (frame_size - 1)
}

/// Compute how many frames are needed to cover `byte_count` bytes.
///
/// Rounds up: one byte into the second frame counts as two frames.
#[inline(always)]
pub const fn frames_needed(byte_count: u64, frame_size: u64) -> u64 {
    debug_assert!(frame_size.is_power_of_two(), "frame_size must be a power of two");
    // align_up then shift avoids any multiplication
    align_up(byte_count, frame_size) >> frame_size.trailing_zeros()
}

/// Return `true` if the range [addr, addr + size) is entirely within a single frame.
#[inline(always)]
pub const fn fits_in_frame(addr: u64, size: u64, frame_size: u64) -> bool {
    debug_assert!(frame_size.is_power_of_two(), "frame_size must be a power of two");
    frame_number(addr, frame_size) == frame_number(addr + size - 1, frame_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- align_down ---

    #[test]
    fn align_down_already_aligned() {
        assert_eq!(align_down(0x1000, 0x1000), 0x1000);
        assert_eq!(align_down(0, 8), 0);
    }

    #[test]
    fn align_down_rounds_toward_zero() {
        assert_eq!(align_down(0x1001, 0x1000), 0x1000);
        assert_eq!(align_down(0x1FFF, 0x1000), 0x1000);
        assert_eq!(align_down(0xFFFF, 0x1000), 0xF000);
        assert_eq!(align_down(7, 4), 4);
        assert_eq!(align_down(3, 4), 0);
    }

    #[test]
    fn align_down_large_alignment() {
        assert_eq!(align_down(0x0040_0000, 0x0020_0000), 0x0040_0000);
        assert_eq!(align_down(0x0040_0001, 0x0020_0000), 0x0040_0000);
    }

    // --- align_up ---

    #[test]
    fn align_up_already_aligned() {
        assert_eq!(align_up(0x1000, 0x1000), 0x1000);
        assert_eq!(align_up(0, 0x1000), 0);
    }

    #[test]
    fn align_up_rounds_away_from_zero() {
        assert_eq!(align_up(1, 0x1000), 0x1000);
        assert_eq!(align_up(0x1001, 0x1000), 0x2000);
        assert_eq!(align_up(0x1FFF, 0x1000), 0x2000);
        assert_eq!(align_up(5, 4), 8);
    }

    #[test]
    fn align_up_large_alignment() {
        assert_eq!(align_up(0x0020_0001, 0x0020_0000), 0x0040_0000);
    }

    // --- is_aligned ---

    #[test]
    fn is_aligned_true_cases() {
        assert!(is_aligned(0, 0x1000));
        assert!(is_aligned(0x2000, 0x1000));
        assert!(is_aligned(0x40, 8));
    }

    #[test]
    fn is_aligned_false_cases() {
        assert!(!is_aligned(1, 0x1000));
        assert!(!is_aligned(0x2001, 0x1000));
        assert!(!is_aligned(7, 4));
    }

    // --- align_offset ---

    #[test]
    fn align_offset_at_boundary_is_zero() {
        assert_eq!(align_offset(0x1000, 0x1000), 0);
        assert_eq!(align_offset(0, 64), 0);
    }

    #[test]
    fn align_offset_returns_intra_block_position() {
        assert_eq!(align_offset(0x1001, 0x1000), 1);
        assert_eq!(align_offset(0x1FFF, 0x1000), 0xFFF);
        assert_eq!(align_offset(0x1800, 0x1000), 0x800);
        assert_eq!(align_offset(65, 64), 1);
    }

    // --- frame_number / frame_base roundtrip ---

    #[test]
    fn frame_number_4k() {
        assert_eq!(frame_number(0x0000_0000, 0x1000), 0);
        assert_eq!(frame_number(0x0000_1000, 0x1000), 1);
        assert_eq!(frame_number(0xDEAD_B000, 0x1000), 0xDEAD_B);
        assert_eq!(frame_number(0x0008_0000_0000, 0x1000), 0x80_0000);
    }

    #[test]
    fn frame_number_2m() {
        assert_eq!(frame_number(0x0020_0000, 0x0020_0000), 1);
        assert_eq!(frame_number(0x0040_0000, 0x0020_0000), 2);
    }

    #[test]
    fn frame_base_roundtrip() {
        for &frame_size in &[0x1000u64, 0x0020_0000, 0x4000_0000] {
            for n in [0u64, 1, 7, 511] {
                assert_eq!(frame_number(frame_base(n, frame_size), frame_size), n);
            }
        }
    }

    #[test]
    fn frame_number_base_inverse() {
        let addr = 0xDEAD_B123u64;
        let size = 0x1000u64;
        assert_eq!(
            frame_base(frame_number(addr, size), size),
            align_down(addr, size)
        );
    }

    // --- frame_offset ---

    #[test]
    fn frame_offset_at_base_is_zero() {
        assert_eq!(frame_offset(0x5000, 0x1000), 0);
        assert_eq!(frame_offset(0, 0x1000), 0);
    }

    #[test]
    fn frame_offset_within_frame() {
        assert_eq!(frame_offset(0x5001, 0x1000), 1);
        assert_eq!(frame_offset(0x5FFF, 0x1000), 0xFFF);
        assert_eq!(frame_offset(0x5800, 0x1000), 0x800);
    }

    // --- frames_needed ---

    #[test]
    fn frames_needed_exact_multiple() {
        assert_eq!(frames_needed(0x1000, 0x1000), 1);
        assert_eq!(frames_needed(0x2000, 0x1000), 2);
        assert_eq!(frames_needed(0x0060_0000, 0x0020_0000), 3);
    }

    #[test]
    fn frames_needed_rounds_up() {
        assert_eq!(frames_needed(0x1001, 0x1000), 2);
        assert_eq!(frames_needed(1, 0x1000), 1);
        // README example: 7 MiB needs 4 × 2 MiB frames
        assert_eq!(frames_needed(7 * 1024 * 1024, 0x0020_0000), 4);
    }

    #[test]
    fn frames_needed_zero_bytes() {
        assert_eq!(frames_needed(0, 0x1000), 0);
    }

    // --- fits_in_frame ---

    #[test]
    fn fits_in_frame_entirely_inside() {
        assert!(fits_in_frame(0x1000, 0x100, 0x1000));
        assert!(fits_in_frame(0x1000, 0x1000, 0x1000));
        assert!(fits_in_frame(0x1F00, 0x100, 0x1000));
    }

    #[test]
    fn fits_in_frame_crosses_boundary() {
        assert!(!fits_in_frame(0x1FF0, 0x20, 0x1000));
        assert!(!fits_in_frame(0x0FFF, 0x2, 0x1000));
    }

    #[test]
    fn fits_in_frame_single_byte() {
        assert!(fits_in_frame(0x1FFF, 1, 0x1000));
        assert!(fits_in_frame(0x1000, 1, 0x1000));
    }
}
