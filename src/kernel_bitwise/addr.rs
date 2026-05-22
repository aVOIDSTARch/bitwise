//! Typed physical and virtual address wrappers.
//!
//! [`PhysAddr`] enforces that bits 63:52 are zero (x86_64 MAXPHYADDR = 52).
//! [`VirtAddr`] enforces x86_64 canonicality: bits 63:48 must be the sign-extension
//! of bit 47 (4-level paging, 48-bit virtual address space).

use core::debug_assert;

/// A 52-bit physical address. Bits 63:52 must be zero on x86_64.
///
/// Construct with [`PhysAddr::new`] (debug-checked) or [`PhysAddr::new_truncate`]
/// (silently masks the high bits).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl PhysAddr {
    /// The zero address.
    pub const ZERO: Self = Self(0);

    /// Construct from a raw `u64`.
    ///
    /// # Panics (debug)
    /// Panics if bits 63:52 are non-zero.
    #[inline(always)]
    pub const fn new(addr: u64) -> Self {
        debug_assert!(addr & !crate::PHYS_ADDR_MASK == 0, "PhysAddr: bits 63:52 must be zero");
        Self(addr)
    }

    /// Construct by masking off bits 63:52.
    #[inline(always)]
    pub const fn new_truncate(addr: u64) -> Self {
        Self(addr & crate::PHYS_ADDR_MASK)
    }

    /// Return the raw `u64` value.
    #[inline(always)]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Return `true` if the address is aligned to `align` bytes.
    #[inline(always)]
    pub const fn is_aligned(self, align: u64) -> bool {
        crate::align::is_aligned(self.0, align)
    }

    /// Round the address down to the nearest `align`-byte boundary.
    #[inline(always)]
    pub const fn align_down(self, align: u64) -> Self {
        Self(crate::align::align_down(self.0, align))
    }

    /// Round the address up to the nearest `align`-byte boundary.
    #[inline(always)]
    pub const fn align_up(self, align: u64) -> Self {
        Self(crate::align::align_up(self.0, align))
    }

    /// Return the frame number for the given frame size.
    #[inline(always)]
    pub const fn frame_number(self, frame_size: u64) -> u64 {
        crate::align::frame_number(self.0, frame_size)
    }

    /// Add an offset, returning `None` if the result exceeds 52-bit range.
    #[inline(always)]
    pub const fn checked_add(self, rhs: u64) -> Option<Self> {
        let result = self.0.wrapping_add(rhs);
        if result & !crate::PHYS_ADDR_MASK == 0 {
            Some(Self(result))
        } else {
            None
        }
    }
}

impl core::fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PhysAddr({:#018x})", self.0)
    }
}

impl core::ops::Add<u64> for PhysAddr {
    type Output = PhysAddr;
    #[inline(always)]
    fn add(self, rhs: u64) -> PhysAddr {
        Self::new_truncate(self.0.wrapping_add(rhs))
    }
}

impl core::ops::Sub<u64> for PhysAddr {
    type Output = PhysAddr;
    #[inline(always)]
    fn sub(self, rhs: u64) -> PhysAddr {
        Self(self.0 - rhs)
    }
}

impl core::ops::Sub<PhysAddr> for PhysAddr {
    type Output = u64;
    #[inline(always)]
    fn sub(self, rhs: PhysAddr) -> u64 {
        self.0 - rhs.0
    }
}

// ---------------------------------------------------------------------------

/// A canonical 64-bit virtual address (4-level paging, 48-bit address space).
///
/// Bits 63:48 must be copies of bit 47 — either all zero (user space,
/// `0x0000_0000_0000_0000`..`0x0000_7FFF_FFFF_FFFF`) or all one (kernel space,
/// `0xFFFF_8000_0000_0000`..`0xFFFF_FFFF_FFFF_FFFF`).
///
/// Construct with [`VirtAddr::new`] (returns `Option`) or
/// [`VirtAddr::new_truncate`] (sign-extends bit 47 silently).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    /// The zero address (canonical user-space null).
    pub const ZERO: Self = Self(0);

    /// Return `true` if `addr` satisfies the x86_64 canonicality requirement.
    ///
    /// Bits 63:48 must equal bit 47 (all-zeros or all-ones).
    #[inline(always)]
    pub const fn is_canonical(addr: u64) -> bool {
        let top = addr >> 47;
        top == 0 || top == (u64::MAX >> 47)
    }

    /// Construct from a raw `u64`, returning `None` if it is not canonical.
    #[inline(always)]
    pub const fn new(addr: u64) -> Option<Self> {
        if Self::is_canonical(addr) {
            Some(Self(addr))
        } else {
            None
        }
    }

    /// Construct by sign-extending bit 47 of `addr` into bits 63:48.
    ///
    /// Any junk in bits 63:48 of the input is discarded.
    #[inline(always)]
    pub const fn new_truncate(addr: u64) -> Self {
        // Shift left 16 to put bit 47 at bit 63, cast to i64 for arithmetic
        // right shift, shift back — this sign-extends bit 47 into bits 63:48.
        Self(((addr << 16) as i64 >> 16) as u64)
    }

    /// Construct without canonicality validation.
    ///
    /// # Safety
    /// `addr` must satisfy [`VirtAddr::is_canonical`]. Passing a non-canonical
    /// address will produce an address that causes #GP on use.
    #[inline(always)]
    pub const unsafe fn new_unchecked(addr: u64) -> Self {
        debug_assert!(Self::is_canonical(addr), "VirtAddr: address is not canonical");
        Self(addr)
    }

    /// Return the raw `u64` value.
    #[inline(always)]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Return `true` if the address is aligned to `align` bytes.
    #[inline(always)]
    pub const fn is_aligned(self, align: u64) -> bool {
        crate::align::is_aligned(self.0, align)
    }

    /// Round down to the nearest `align`-byte boundary, preserving canonicality.
    #[inline(always)]
    pub const fn align_down(self, align: u64) -> Self {
        Self(crate::align::align_down(self.0, align))
    }

    /// Round up to the nearest `align`-byte boundary.
    #[inline(always)]
    pub const fn align_up(self, align: u64) -> Self {
        Self(crate::align::align_up(self.0, align))
    }

    /// Extract the page table index for the given level (1–4 for 4-level paging).
    ///
    /// Returns a value in `0..=511`.
    #[inline(always)]
    pub const fn pt_index(self, level: u32) -> u64 {
        crate::paging::vaddr_pt_index(self.0, level)
    }

    /// Return the byte offset within the page of the given size.
    #[inline(always)]
    pub const fn page_offset(self, frame_size: u64) -> u64 {
        crate::paging::vaddr_page_offset(self.0, frame_size)
    }
}

impl core::fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtAddr({:#018x})", self.0)
    }
}

impl core::ops::Add<u64> for VirtAddr {
    type Output = VirtAddr;
    /// Add an offset, sign-extending the result at bit 47.
    #[inline(always)]
    fn add(self, rhs: u64) -> VirtAddr {
        VirtAddr::new_truncate(self.0.wrapping_add(rhs))
    }
}

impl core::ops::Sub<u64> for VirtAddr {
    type Output = VirtAddr;
    /// Subtract an offset, sign-extending the result at bit 47.
    #[inline(always)]
    fn sub(self, rhs: u64) -> VirtAddr {
        VirtAddr::new_truncate(self.0.wrapping_sub(rhs))
    }
}

impl core::ops::Sub<VirtAddr> for VirtAddr {
    type Output = u64;
    /// Return the byte distance between two virtual addresses.
    #[inline(always)]
    fn sub(self, rhs: VirtAddr) -> u64 {
        self.0.wrapping_sub(rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- PhysAddr ---

    #[test_case]
    fn phys_new_valid() {
        let p = PhysAddr::new(0x0000_1234_5678_9000);
        assert_eq!(p.as_u64(), 0x0000_1234_5678_9000);
    }

    #[test_case]
    fn phys_new_truncate_strips_high_bits() {
        let p = PhysAddr::new_truncate(0xFFFF_1234_5678_9000);
        assert_eq!(p.as_u64(), 0x000F_1234_5678_9000);
    }

    #[test_case]
    fn phys_zero() {
        assert_eq!(PhysAddr::ZERO.as_u64(), 0);
    }

    #[test_case]
    fn phys_align_roundtrip() {
        let p = PhysAddr::new(0x0000_0000_1234_5ABC);
        assert_eq!(p.align_down(0x1000).as_u64(), 0x0000_0000_1234_5000);
        assert_eq!(p.align_up(0x1000).as_u64(),   0x0000_0000_1234_6000);
    }

    #[test_case]
    fn phys_is_aligned() {
        assert!( PhysAddr::new(0x0000_0000_0002_0000).is_aligned(0x1000));
        assert!(!PhysAddr::new(0x0000_0000_0002_0001).is_aligned(0x1000));
    }

    #[test_case]
    fn phys_frame_number_4k() {
        assert_eq!(PhysAddr::new(0x0000_0000_0002_3000).frame_number(0x1000), 0x23);
    }

    #[test_case]
    fn phys_checked_add_in_range() {
        let p = PhysAddr::new(0x0000_0000_0010_0000);
        assert_eq!(p.checked_add(0x1000).unwrap().as_u64(), 0x0000_0000_0010_1000);
    }

    #[test_case]
    fn phys_checked_add_overflow_returns_none() {
        let p = PhysAddr::new(crate::PHYS_ADDR_MASK);
        assert!(p.checked_add(1).is_none());
    }

    #[test_case]
    fn phys_add_sub_roundtrip() {
        let p = PhysAddr::new(0x0000_0000_0010_0000);
        assert_eq!((p + 0x5000 - 0x5000).as_u64(), p.as_u64());
    }

    #[test_case]
    fn phys_sub_phys() {
        let a = PhysAddr::new(0x0000_0000_0020_0000);
        let b = PhysAddr::new(0x0000_0000_0010_0000);
        assert_eq!(a - b, 0x0010_0000);
    }

    // --- VirtAddr ---

    #[test_case]
    fn virt_is_canonical_user() {
        assert!(VirtAddr::is_canonical(0x0000_0000_0000_0000));
        assert!(VirtAddr::is_canonical(0x0000_7FFF_FFFF_FFFF));
    }

    #[test_case]
    fn virt_is_canonical_kernel() {
        assert!(VirtAddr::is_canonical(0xFFFF_8000_0000_0000));
        assert!(VirtAddr::is_canonical(0xFFFF_FFFF_FFFF_FFFF));
    }

    #[test_case]
    fn virt_is_not_canonical() {
        assert!(!VirtAddr::is_canonical(0x0000_8000_0000_0000));
        assert!(!VirtAddr::is_canonical(0x8000_0000_0000_0000));
        assert!(!VirtAddr::is_canonical(0x0001_0000_0000_0000));
    }

    #[test_case]
    fn virt_new_some_and_none() {
        assert!(VirtAddr::new(0x0000_0000_1234_5000).is_some());
        assert!(VirtAddr::new(0xFFFF_FFFF_FFFF_F000).is_some());
        assert!(VirtAddr::new(0x0000_8000_0000_0000).is_none());
    }

    #[test_case]
    fn virt_new_truncate_sign_extends_user() {
        let v = VirtAddr::new_truncate(0xABCD_0000_0000_1000);
        // lower 48 bits = 0x0000_0000_1000, bit 47 = 0 → upper bits 0
        assert_eq!(v.as_u64(), 0x0000_0000_0000_1000);
    }

    #[test_case]
    fn virt_new_truncate_sign_extends_kernel() {
        let v = VirtAddr::new_truncate(0x0000_8000_0000_0000);
        // bit 47 of input = 1 → upper bits all 1
        assert_eq!(v.as_u64(), 0xFFFF_8000_0000_0000);
    }

    #[test_case]
    fn virt_align_down_page() {
        let v = VirtAddr::new(0xFFFF_FFFF_FFFF_F800).unwrap();
        assert_eq!(v.align_down(0x1000).as_u64(), 0xFFFF_FFFF_FFFF_F000);
    }

    #[test_case]
    fn virt_pt_index_level1() {
        // vaddr bits [20:12] for level 1
        let v = VirtAddr::new(0x0000_0000_0020_1000).unwrap();
        assert_eq!(v.pt_index(1), 1);
    }

    #[test_case]
    fn virt_page_offset_4k() {
        let v = VirtAddr::new(0x0000_0000_0000_1ABC).unwrap();
        assert_eq!(v.page_offset(0x1000), 0xABC);
    }

    #[test_case]
    fn virt_add_preserves_canonicality() {
        let v = VirtAddr::new(0xFFFF_FFFF_FFFF_F000).unwrap();
        let v2 = v + 0x1000;
        assert!(VirtAddr::is_canonical(v2.as_u64()));
    }

    #[test_case]
    fn virt_sub_virt() {
        let a = VirtAddr::new(0x0000_0000_0002_0000).unwrap();
        let b = VirtAddr::new(0x0000_0000_0001_0000).unwrap();
        assert_eq!(a - b, 0x0001_0000);
    }
}
