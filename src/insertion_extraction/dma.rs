use crate::align::{is_aligned, align_up, frames_needed};

/// Standard cache line size on x86_64 Intel/AMD.
/// May differ on other architectures; query CPUID leaf 0x01 EBX[15:8] if uncertain.
pub const CACHE_LINE_SIZE: u64 = 64;

/// Typical IOMMU/DMA page size (matches 4 KiB system page).
pub const DMA_PAGE_SIZE: u64 = 0x1000;

/// Check whether a physical address is suitable as a DMA buffer base.
///
/// `dma_align` is the minimum alignment required by the device (often
/// `CACHE_LINE_SIZE` for coherent DMA, or `DMA_PAGE_SIZE` for scatter-gather).
#[inline(always)]
pub const fn dma_is_aligned(phys_addr: u64, dma_align: u64) -> bool {
    is_aligned(phys_addr, dma_align)
}

/// Compute the aligned DMA buffer start and the wasted bytes before it.
///
/// Given a raw physical address and a required DMA alignment, returns:
/// - `aligned_base`: the first address ≥ `phys_addr` satisfying `dma_align`
/// - `padding_before`: bytes wasted between `phys_addr` and `aligned_base`
///
/// If `phys_addr` is already aligned, `padding_before` is 0.
#[inline(always)]
pub const fn dma_align_buffer(phys_addr: u64, dma_align: u64)
    -> (u64 /* aligned_base */, u64 /* padding_before */)
{
    let aligned = align_up(phys_addr, dma_align);
    (aligned, aligned - phys_addr)
}

/// Round a DMA transfer length up to a multiple of `granularity`.
///
/// Many DMA engines require that the transfer count be a multiple of the
/// device's bus width or burst size (e.g., 4 bytes for 32-bit PCI).
#[inline(always)]
pub const fn dma_round_length(len: u64, granularity: u64) -> u64 {
    align_up(len, granularity)
}

/// Compute the number of scatter-gather entries (segments) needed to describe
/// a buffer given a maximum segment size.
///
/// This assumes worst-case alignment — i.e., the buffer may start at any
/// offset within a segment. Use when you don't yet know the buffer's
/// physical address (e.g., during descriptor ring pre-allocation).
#[inline(always)]
pub const fn dma_sg_segments_needed(len: u64, max_segment_size: u64) -> u64 {
    // +1 for potential split at start/end boundary
    frames_needed(len, max_segment_size) + 1
}

/// Split a physically contiguous buffer into segment descriptors for a
/// scatter-gather list, where each segment may not cross a `segment_boundary`.
///
/// Returns the number of descriptors written into `out`.
///
/// `out` must have capacity ≥ `dma_sg_segments_needed(len, segment_boundary)`.
pub fn dma_build_sg(
    phys_base: u64,
    len: u64,
    segment_boundary: u64,
    out: &mut [(u64, u64)],  // (phys_addr, len) per segment
) -> usize {
    debug_assert!(segment_boundary.is_power_of_two());
    let mut remaining = len;
    let mut addr = phys_base;
    let mut count = 0;

    while remaining > 0 {
        // How many bytes until the next segment boundary?
        let boundary_end = align_up(addr + 1, segment_boundary);
        let chunk = (boundary_end - addr).min(remaining);

        out[count] = (addr, chunk);
        count += 1;

        addr += chunk;
        remaining -= chunk;
    }

    count
}

/// Convert a physical address to a bus address (identity mapping assumed).
///
/// On systems with an IOMMU, the bus address is the IOVA, not the physical
/// address. This stub is the trivial case (one-to-one mapping or no IOMMU).
/// Replace with an IOMMU lookup table in a real driver.
#[inline(always)]
pub const fn phys_to_bus(phys: u64) -> u64 {
    phys  // identity mapping stub
}

/// Check whether a physical address range fits within a 32-bit DMA window
/// (required for devices that cannot address above 4 GiB without IOMMU remapping).
#[inline(always)]
pub const fn fits_in_32bit_dma(phys: u64, len: u64) -> bool {
    phys.saturating_add(len) <= 0x1_0000_0000
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- dma_is_aligned ---

    #[test]
    fn dma_is_aligned_power_of_two() {
        assert!(dma_is_aligned(0, CACHE_LINE_SIZE));
        assert!(dma_is_aligned(64, CACHE_LINE_SIZE));
        assert!(dma_is_aligned(0x1000, DMA_PAGE_SIZE));
        assert!(!dma_is_aligned(1, CACHE_LINE_SIZE));
        assert!(!dma_is_aligned(0x1001, DMA_PAGE_SIZE));
    }

    // --- dma_align_buffer ---

    #[test]
    fn dma_align_buffer_already_aligned() {
        let (base, pad) = dma_align_buffer(0x1000, DMA_PAGE_SIZE);
        assert_eq!(base, 0x1000);
        assert_eq!(pad, 0);
    }

    #[test]
    fn dma_align_buffer_unaligned() {
        let (base, pad) = dma_align_buffer(0x1001, DMA_PAGE_SIZE);
        assert_eq!(base, 0x2000);
        assert_eq!(pad, 0x2000 - 0x1001);
    }

    #[test]
    fn dma_align_buffer_pad_plus_base_equals_next_boundary() {
        for phys in [0u64, 1, 63, 64, 100, 0x1FFF, 0x2000] {
            let align = CACHE_LINE_SIZE;
            let (base, pad) = dma_align_buffer(phys, align);
            assert_eq!(phys + pad, base);
            assert_eq!(base % align, 0);
        }
    }

    // --- dma_round_length ---

    #[test]
    fn dma_round_length_exact_multiple() {
        assert_eq!(dma_round_length(0, 4), 0);
        assert_eq!(dma_round_length(4, 4), 4);
        assert_eq!(dma_round_length(64, 64), 64);
    }

    #[test]
    fn dma_round_length_rounds_up() {
        assert_eq!(dma_round_length(1, 4), 4);
        assert_eq!(dma_round_length(3, 4), 4);
        assert_eq!(dma_round_length(5, 4), 8);
        assert_eq!(dma_round_length(65, 64), 128);
    }

    // --- dma_build_sg ---

    #[test]
    fn dma_build_sg_aligned_fits_in_one_segment() {
        let mut out = [(0u64, 0u64); 4];
        // Buffer starts at segment boundary, fits exactly
        let count = dma_build_sg(0x0000, 0x1000, 0x1000, &mut out);
        assert_eq!(count, 1);
        assert_eq!(out[0], (0x0000, 0x1000));
    }

    #[test]
    fn dma_build_sg_crosses_one_boundary() {
        let mut out = [(0u64, 0u64); 4];
        // Buffer at 0x0800, length 0x1000 → crosses boundary at 0x1000
        let count = dma_build_sg(0x0800, 0x1000, 0x1000, &mut out);
        assert_eq!(count, 2);
        assert_eq!(out[0].0, 0x0800);
        assert_eq!(out[0].1, 0x0800);  // 0x1000 - 0x0800
        assert_eq!(out[1].0, 0x1000);
        assert_eq!(out[1].1, 0x0800);  // remaining
    }

    #[test]
    fn dma_build_sg_total_length_preserved() {
        let mut out = [(0u64, 0u64); 8];
        let phys = 0x0500u64;
        let len  = 0x3000u64;
        let seg  = 0x1000u64;
        let count = dma_build_sg(phys, len, seg, &mut out);
        let total: u64 = out[..count].iter().map(|&(_, l)| l).sum();
        assert_eq!(total, len);
    }

    #[test]
    fn dma_build_sg_each_segment_stays_within_boundary() {
        let mut out = [(0u64, 0u64); 8];
        let phys = 0x0F00u64;
        let len  = 0x4000u64;
        let seg  = 0x1000u64;
        let count = dma_build_sg(phys, len, seg, &mut out);
        for &(addr, chunk_len) in &out[..count] {
            // start and (end - 1) must lie in the same segment
            let start_seg = addr / seg;
            let end_seg   = (addr + chunk_len - 1) / seg;
            assert_eq!(start_seg, end_seg,
                "segment [{:#x}, +{:#x}) crosses boundary", addr, chunk_len);
        }
    }

    #[test]
    fn dma_build_sg_zero_length() {
        let mut out = [(0u64, 0u64); 4];
        let count = dma_build_sg(0x1000, 0, 0x1000, &mut out);
        assert_eq!(count, 0);
    }

    // --- fits_in_32bit_dma ---

    #[test]
    fn fits_in_32bit_dma_true_cases() {
        assert!(fits_in_32bit_dma(0, 0x1_0000_0000));
        assert!(fits_in_32bit_dma(0, 0));
        assert!(fits_in_32bit_dma(0xFFFF_F000, 0x1000));
        assert!(fits_in_32bit_dma(0x1000, 0x1000));
    }

    #[test]
    fn fits_in_32bit_dma_false_cases() {
        assert!(!fits_in_32bit_dma(0xFFFF_F000, 0x2000)); // 0xFFFF_F000 + 0x2000 = 0x1_0000_F000
        assert!(!fits_in_32bit_dma(0x1_0000_0000, 1));
        assert!(!fits_in_32bit_dma(0x1_0000_0001, 0));
    }

    // --- phys_to_bus identity ---

    #[test]
    fn phys_to_bus_identity_mapping() {
        for addr in [0u64, 0x1000, 0xDEAD_BEEF, u64::MAX] {
            assert_eq!(phys_to_bus(addr), addr);
        }
    }

    // --- CACHE_LINE_SIZE / DMA_PAGE_SIZE constants ---

    #[test]
    fn constants_are_powers_of_two() {
        assert!(CACHE_LINE_SIZE.is_power_of_two());
        assert!(DMA_PAGE_SIZE.is_power_of_two());
        assert_eq!(CACHE_LINE_SIZE, 64);
        assert_eq!(DMA_PAGE_SIZE, 0x1000);
    }
}
