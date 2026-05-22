use crate::align::frame_offset;

/// x86_64 page table entry flag bits (Intel Vol. 3A §4.5, Table 4-19).
pub mod pte_flags {
    /// Page is present in physical memory; if clear, all other bits are ignored by hardware.
    pub const PRESENT:       u64 = 1 << 0;
    /// Page is writable; if clear, writes fault with #PF (subject to CR0.WP for ring 0).
    pub const WRITABLE:      u64 = 1 << 1;
    /// Page is accessible from user mode (CPL 3); if clear, only ring 0 may access it.
    pub const USER:          u64 = 1 << 2;
    /// Write-through caching for this page; if clear, write-back caching is used.
    pub const WRITE_THROUGH: u64 = 1 << 3;
    /// Caching disabled for this page; all accesses bypass the cache hierarchy.
    pub const CACHE_DISABLE: u64 = 1 << 4;
    /// Set by hardware on any read or write to this page; software must clear it to track usage.
    pub const ACCESSED:      u64 = 1 << 5;
    /// Set by hardware on a write to this page; valid only in PTEs and large-page entries.
    pub const DIRTY:         u64 = 1 << 6;
    /// Large/huge page: 2 MiB when set in a PDE (level 2), 1 GiB when set in a PDPTE (level 3).
    pub const HUGE_PAGE:     u64 = 1 << 7;
    /// Global page — not flushed from the TLB on CR3 writes (requires CR4.PGE).
    pub const GLOBAL:        u64 = 1 << 8;
    /// No-execute — instruction fetches from this page fault with #PF (requires EFER.NXE).
    pub const NO_EXECUTE:    u64 = 1 << 63;

    /// Mask for the 3 OS-available software bits (bits 11:9).
    pub const AVAIL_MASK:    u64 = 0b111 << 9;
    /// Bit position of the OS-available field (bit 9).
    pub const AVAIL_SHIFT:   u32 = 9;

    /// Physical frame number mask for 4 KiB pages (bits 51:12).
    pub const PFN_MASK_4K:   u64 = 0x000F_FFFF_FFFF_F000;
    /// Physical frame number mask for 2 MiB large pages (bits 51:21).
    pub const PFN_MASK_2M:   u64 = 0x000F_FFFF_FFE0_0000;
    /// Physical frame number mask for 1 GiB gigantic pages (bits 51:30).
    pub const PFN_MASK_1G:   u64 = 0x000F_FFFC_0000_0000;
}

/// Build a page table entry from a physical address and flags.
///
/// `phys_addr` must be aligned to `frame_size`. The frame size determines
/// which bits carry the physical address:
///   - 4 KiB: bits 51:12
///   - 2 MiB: bits 51:21 (HUGE_PAGE flag must be set in `flags`)
///   - 1 GiB: bits 51:30 (HUGE_PAGE flag must be set in `flags`)
///
/// The `PRESENT` flag is **not** automatically added — set it in `flags`
/// explicitly. This allows construction of swap entries and guard entries
/// where bit 0 is intentionally clear.
#[inline(always)]
pub const fn pte_encode(phys_addr: u64, frame_size: u64, flags: u64) -> u64 {
    debug_assert!(frame_size.is_power_of_two());
    debug_assert!(phys_addr & (frame_size - 1) == 0, "phys_addr not aligned to frame_size");
    // Physical address bits land at their natural position (no shift needed)
    // since the page offset bits are always 0 for an aligned address.
    phys_addr | flags
}

/// Extract the physical base address from a page table entry.
///
/// `frame_size` must match the size of the mapping described by this entry.
#[inline(always)]
pub const fn pte_phys_addr(entry: u64, frame_size: u64) -> u64 {
    debug_assert!(frame_size.is_power_of_two());
    // Mask off flag bits below the frame offset and the NX bit and reserved
    // bits above bit 51. The result is the physical frame base address.
    let pfn_mask = !((frame_size - 1) | (0xFFF0_0000_0000_0000));
    entry & pfn_mask
}

/// Check whether a page table entry is present (bit 0 set).
#[inline(always)]
pub const fn pte_is_present(entry: u64) -> bool {
    entry & pte_flags::PRESENT != 0
}

/// Check whether an entry is a large/huge page (bit 7 set at PDPTE or PDE level).
#[inline(always)]
pub const fn pte_is_huge(entry: u64) -> bool {
    entry & pte_flags::HUGE_PAGE != 0
}

/// Set one or more flags in an existing entry, preserving the physical address.
#[inline(always)]
pub const fn pte_set_flags(entry: u64, flags: u64) -> u64 {
    entry | flags
}

/// Clear one or more flags in an existing entry, preserving the physical address.
#[inline(always)]
pub const fn pte_clear_flags(entry: u64, flags: u64) -> u64 {
    entry & !flags
}

/// Read the OS-available software bits (bits 11:9) from an entry.
#[inline(always)]
pub const fn pte_avail_bits(entry: u64) -> u64 {
    (entry & pte_flags::AVAIL_MASK) >> pte_flags::AVAIL_SHIFT
}

/// Write the OS-available software bits (bits 11:9) into an entry.
#[inline(always)]
pub const fn pte_set_avail_bits(entry: u64, bits: u64) -> u64 {
    debug_assert!(bits <= 0b111, "only 3 avail bits exist");
    (entry & !pte_flags::AVAIL_MASK) | ((bits << pte_flags::AVAIL_SHIFT) & pte_flags::AVAIL_MASK)
}

/// Compute the index into a page table level from a virtual address.
///
/// On x86_64 4-level paging:
/// - Level 4 (PML4):  bits 47:39
/// - Level 3 (PDPT):  bits 38:30
/// - Level 2 (PD):    bits 29:21
/// - Level 1 (PT):    bits 20:12
/// - Page offset:     bits 11:0
///
/// `level` is 1..=4 matching the above. The returned index is 0..=511.
#[inline(always)]
pub const fn vaddr_pt_index(vaddr: u64, level: u32) -> u64 {
    debug_assert!(level >= 1 && level <= 4, "level must be 1..=4");
    let shift = 12 + (level - 1) * 9;
    (vaddr >> shift) & 0x1FF
}

/// Compute the page offset within the final frame from a virtual address
/// and frame size.
#[inline(always)]
pub const fn vaddr_page_offset(vaddr: u64, frame_size: u64) -> u64 {
    frame_offset(vaddr, frame_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::align::align_down;

    // --- pte_flags constants ---

    #[cfg(test)]
    fn pte_flag_bit_positions() {
        assert_eq!(pte_flags::PRESENT,       1 << 0);
        assert_eq!(pte_flags::WRITABLE,      1 << 1);
        assert_eq!(pte_flags::USER,          1 << 2);
        assert_eq!(pte_flags::WRITE_THROUGH, 1 << 3);
        assert_eq!(pte_flags::CACHE_DISABLE, 1 << 4);
        assert_eq!(pte_flags::ACCESSED,      1 << 5);
        assert_eq!(pte_flags::DIRTY,         1 << 6);
        assert_eq!(pte_flags::HUGE_PAGE,     1 << 7);
        assert_eq!(pte_flags::GLOBAL,        1 << 8);
        assert_eq!(pte_flags::NO_EXECUTE,    1 << 63);
        assert_eq!(pte_flags::AVAIL_SHIFT,   9);
        assert_eq!(pte_flags::AVAIL_MASK,    0b111 << 9);
    }

    #[cfg(test)]
    fn pfn_masks_cover_correct_bits() {
        // 4K: bits 51:12
        assert_eq!(pte_flags::PFN_MASK_4K, 0x000F_FFFF_FFFF_F000);
        // 2M: bits 51:21
        assert_eq!(pte_flags::PFN_MASK_2M, 0x000F_FFFF_FFE0_0000);
        // 1G: bits 51:30
        assert_eq!(pte_flags::PFN_MASK_1G, 0x000F_FFFC_0000_0000);
    }

    #[cfg(test)]
    fn pfn_masks_are_disjoint_from_low_flag_bits() {
        let low_flags = pte_flags::PRESENT | pte_flags::WRITABLE | pte_flags::USER
            | pte_flags::WRITE_THROUGH | pte_flags::CACHE_DISABLE
            | pte_flags::ACCESSED | pte_flags::DIRTY | pte_flags::HUGE_PAGE
            | pte_flags::GLOBAL | pte_flags::AVAIL_MASK;
        assert_eq!(pte_flags::PFN_MASK_4K & low_flags, 0);
        assert_eq!(pte_flags::PFN_MASK_2M & low_flags, 0);
        assert_eq!(pte_flags::PFN_MASK_1G & low_flags, 0);
    }

    // --- pte_encode / pte_phys_addr roundtrip ---

    #[cfg(test)]
    fn pte_encode_decode_4k() {
        let phys = 0x0001_2000u64;  // 4K-aligned
        let flags = pte_flags::PRESENT | pte_flags::WRITABLE;
        let entry = pte_encode(phys, 0x1000, flags);
        assert_eq!(pte_phys_addr(entry, 0x1000), phys);
        assert!(pte_is_present(entry));
    }

    #[cfg(test)]
    fn pte_encode_decode_2m() {
        let phys = 0x0020_0000u64;  // 2M-aligned
        let flags = pte_flags::PRESENT | pte_flags::HUGE_PAGE;
        let entry = pte_encode(phys, 0x0020_0000, flags);
        assert_eq!(pte_phys_addr(entry, 0x0020_0000), phys);
        assert!(pte_is_huge(entry));
    }

    #[cfg(test)]
    fn pte_encode_decode_1g() {
        let phys = 0x4000_0000u64;  // 1G-aligned
        let flags = pte_flags::PRESENT | pte_flags::HUGE_PAGE;
        let entry = pte_encode(phys, 0x4000_0000, flags);
        assert_eq!(pte_phys_addr(entry, 0x4000_0000), phys);
    }

    #[cfg(test)]
    fn pte_encode_zero_flags_keeps_addr() {
        let phys = 0x5000u64;
        let entry = pte_encode(phys, 0x1000, 0);
        assert_eq!(pte_phys_addr(entry, 0x1000), phys);
        assert!(!pte_is_present(entry));
    }

   #[cfg(test)]
    fn pte_phys_addr_strips_nx_bit() {
        let phys = 0x0001_2000u64;
        let entry = pte_encode(phys, 0x1000, pte_flags::PRESENT | pte_flags::NO_EXECUTE);
        assert_eq!(pte_phys_addr(entry, 0x1000), phys);
    }

    // --- pte_set_flags / pte_clear_flags ---

   #[cfg(test)]
    fn set_and_clear_flags() {
        let entry = pte_encode(0x3000, 0x1000, pte_flags::PRESENT);
        let with_nx = pte_set_flags(entry, pte_flags::NO_EXECUTE);
        assert_eq!(with_nx & pte_flags::NO_EXECUTE, pte_flags::NO_EXECUTE);
        let without_nx = pte_clear_flags(with_nx, pte_flags::NO_EXECUTE);
        assert_eq!(without_nx & pte_flags::NO_EXECUTE, 0);
        // Physical address preserved through both operations
        assert_eq!(pte_phys_addr(without_nx, 0x1000), 0x3000);
    }

    // --- pte_avail_bits roundtrip ---

    #[cfg(test)]
    fn avail_bits_roundtrip_all_values() {
        let base = pte_encode(0x4000, 0x1000, pte_flags::PRESENT);
        for bits in 0u64..=7 {
            let entry = pte_set_avail_bits(base, bits);
            assert_eq!(pte_avail_bits(entry), bits);
            // Physical address must not be disturbed
            assert_eq!(pte_phys_addr(entry, 0x1000), 0x4000);
        }
    }

    #[cfg(test)]
    fn avail_bits_do_not_leak_into_pfn() {
        let base = pte_encode(0x5000, 0x1000, 0);
        let entry = pte_set_avail_bits(base, 0b111);
        assert_eq!(pte_phys_addr(entry, 0x1000), 0x5000);
    }

    // --- pte_is_present / pte_is_huge ---

    #[cfg(test)]
    fn is_present_and_huge_flags() {
        let absent = 0u64;
        assert!(!pte_is_present(absent));
        assert!(!pte_is_huge(absent));

        let present = pte_flags::PRESENT;
        assert!(pte_is_present(present));
        assert!(!pte_is_huge(present));

        let huge = pte_flags::PRESENT | pte_flags::HUGE_PAGE;
        assert!(pte_is_present(huge));
        assert!(pte_is_huge(huge));
    }

    // --- vaddr_pt_index ---

    #[cfg(test)]
    fn vaddr_pt_index_reference_values() {
        // Virtual address with known index values in each level:
        // Build a vaddr where level-1 idx = 1, level-2 idx = 2, level-3 idx = 3, level-4 idx = 4
        let vaddr: u64 = (4u64 << 39) | (3u64 << 30) | (2u64 << 21) | (1u64 << 12);
        assert_eq!(vaddr_pt_index(vaddr, 1), 1);
        assert_eq!(vaddr_pt_index(vaddr, 2), 2);
        assert_eq!(vaddr_pt_index(vaddr, 3), 3);
        assert_eq!(vaddr_pt_index(vaddr, 4), 4);
    }

    #[cfg(test)]
    fn vaddr_pt_index_max_value_is_511() {
        // 9 bits set in each field → 511
        let vaddr: u64 = (0x1FFu64 << 39) | (0x1FFu64 << 30) | (0x1FFu64 << 21) | (0x1FFu64 << 12);
        assert_eq!(vaddr_pt_index(vaddr, 1), 511);
        assert_eq!(vaddr_pt_index(vaddr, 2), 511);
        assert_eq!(vaddr_pt_index(vaddr, 3), 511);
        assert_eq!(vaddr_pt_index(vaddr, 4), 511);
    }

   #[cfg(test)]
    fn vaddr_pt_index_zero_address() {
        for level in 1u32..=4 {
            assert_eq!(vaddr_pt_index(0, level), 0);
        }
    }

   #[cfg(test)]
    fn vaddr_pt_index_matches_bit_shift_reference() {
        let vaddr = 0xFFFF_8000_0020_3000u64;
        // Level 1: bits 20:12
        assert_eq!(vaddr_pt_index(vaddr, 1), (vaddr >> 12) & 0x1FF);
        // Level 2: bits 29:21
        assert_eq!(vaddr_pt_index(vaddr, 2), (vaddr >> 21) & 0x1FF);
        // Level 3: bits 38:30
        assert_eq!(vaddr_pt_index(vaddr, 3), (vaddr >> 30) & 0x1FF);
        // Level 4: bits 47:39
        assert_eq!(vaddr_pt_index(vaddr, 4), (vaddr >> 39) & 0x1FF);
    }

    // --- vaddr_page_offset ---

    #[cfg(test)]
    fn vaddr_page_offset_4k() {
        assert_eq!(vaddr_page_offset(0x1000, 0x1000), 0);
        assert_eq!(vaddr_page_offset(0x1ABC, 0x1000), 0xABC);
        assert_eq!(vaddr_page_offset(0x1FFF, 0x1000), 0xFFF);
    }

    #[cfg(test)]
    fn vaddr_page_offset_2m() {
        let addr = 0x0020_0500u64;
        assert_eq!(vaddr_page_offset(addr, 0x0020_0000), 0x500);
    }

   #[cfg(test)]
    fn vaddr_page_offset_with_pte_roundtrip() {
        // pte_phys_addr(pte_encode(base, size, flags)) + vaddr_page_offset(vaddr, size) == vaddr
        let phys_base = 0x0040_0000u64;
        let vaddr     = 0x0040_0ABCu64;
        let size      = 0x0020_0000u64;
        let base      = align_down(vaddr, size);
        assert_eq!(base, phys_base);
        let offset = vaddr_page_offset(vaddr, size);
        assert_eq!(phys_base + offset, vaddr);
    }
}
