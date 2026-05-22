use core::ptr;

/// Read a `u8` from a memory-mapped register at `addr`.
///
/// # Safety
/// `addr` must be a valid, mapped MMIO address for a `u8`-wide register.
/// The caller is responsible for ensuring the address is correct and the
/// hardware is in an appropriate state.
#[inline(always)]
pub unsafe fn mmio_read8(addr: u64) -> u8 {
    unsafe { ptr::read_volatile(addr as *const u8) }
}

/// Read a `u16` from a memory-mapped register at `addr`.
///
/// # Safety
/// `addr` must be valid, mapped, and naturally aligned to 2 bytes.
#[inline(always)]
pub unsafe fn mmio_read16(addr: u64) -> u16 {
    unsafe { ptr::read_volatile(addr as *const u16) }
}

/// Read a `u32` from a memory-mapped register at `addr`.
///
/// # Safety
/// `addr` must be valid, mapped, and naturally aligned to 4 bytes.
#[inline(always)]
pub unsafe fn mmio_read32(addr: u64) -> u32 {
    unsafe { ptr::read_volatile(addr as *const u32) }
}

/// Read a `u64` from a memory-mapped register at `addr`.
///
/// # Safety
/// `addr` must be valid, mapped, and naturally aligned to 8 bytes.
#[inline(always)]
pub unsafe fn mmio_read64(addr: u64) -> u64 {
    unsafe { ptr::read_volatile(addr as *const u64) }
}

/// Write `value` to a memory-mapped register at `addr`.
///
/// # Safety
/// `addr` must be a valid, mapped MMIO address for a `u8`-wide register.
#[inline(always)]
pub unsafe fn mmio_write8(addr: u64, value: u8) {
    unsafe { ptr::write_volatile(addr as *mut u8, value) }
}

/// # Safety
/// `addr` must be valid, mapped, and naturally aligned to 2 bytes.
#[inline(always)]
pub unsafe fn mmio_write16(addr: u64, value: u16) {
    unsafe { ptr::write_volatile(addr as *mut u16, value) }
}

/// # Safety
/// `addr` must be valid, mapped, and naturally aligned to 4 bytes.
#[inline(always)]
pub unsafe fn mmio_write32(addr: u64, value: u32) {
    unsafe { ptr::write_volatile(addr as *mut u32, value) }
}

/// # Safety
/// `addr` must be valid, mapped, and naturally aligned to 8 bytes.
#[inline(always)]
pub unsafe fn mmio_write64(addr: u64, value: u64) {
    unsafe { ptr::write_volatile(addr as *mut u64, value) }
}

/// Read-modify-write: set bits in `mask` at `addr` (32-bit register).
///
/// Reads the current value, ORs in `mask`, writes back. Not atomic —
/// do not use on registers that require atomic RMW (use hardware-provided
/// atomics or protect with a spinlock).
///
/// # Safety
/// `addr` must be a valid, mapped, 4-byte-aligned MMIO register address.
#[inline(always)]
pub unsafe fn mmio_set_bits32(addr: u64, mask: u32) {
    unsafe {
        let current = mmio_read32(addr);
        mmio_write32(addr, current | mask);
    }
}

/// Read-modify-write: clear bits in `mask` at `addr` (32-bit register).
///
/// # Safety
/// `addr` must be a valid, mapped, 4-byte-aligned MMIO register address.
#[inline(always)]
pub unsafe fn mmio_clear_bits32(addr: u64, mask: u32) {
    unsafe {
        let current = mmio_read32(addr);
        mmio_write32(addr, current & !mask);
    }
}

/// Read-modify-write: update a bit field in a 32-bit MMIO register.
///
/// Clears bits selected by `mask`, then ORs in `value` (pre-positioned).
///
/// # Safety
/// `addr` must be a valid, mapped, 4-byte-aligned MMIO register address.
#[inline(always)]
pub unsafe fn mmio_update_field32(addr: u64, mask: u32, value: u32) {
    unsafe {
        let current = mmio_read32(addr);
        mmio_write32(addr, (current & !mask) | (value & mask));
    }
}

/// Read-modify-write: set bits in `mask` at `addr` (64-bit register).
///
/// # Safety
/// `addr` must be a valid, mapped, 8-byte-aligned MMIO register address.
#[inline(always)]
pub unsafe fn mmio_set_bits64(addr: u64, mask: u64) {
    unsafe {
        let current = mmio_read64(addr);
        mmio_write64(addr, current | mask);
    }
}

/// Read-modify-write: clear bits in `mask` at `addr` (64-bit register).
///
/// # Safety
/// `addr` must be a valid, mapped, 8-byte-aligned MMIO register address.
#[inline(always)]
pub unsafe fn mmio_clear_bits64(addr: u64, mask: u64) {
    unsafe {
        let current = mmio_read64(addr);
        mmio_write64(addr, current & !mask);
    }
}

/// Typed MMIO register view — wraps a base address and provides offset-based access.
///
/// Useful for device register blocks where all registers are at `base + offset`.
///
/// ```no_run
/// # use bitwise::mmio::MmioBlock;
/// let uart = MmioBlock::new(0xFEDC_0000);
/// unsafe {
///     uart.write32(0x00, 0x0000_0001);  // enable
///     let status = uart.read32(0x18);   // read status register
/// }
/// ```
pub struct MmioBlock {
    base: u64,
}

impl MmioBlock {
    /// Create a new `MmioBlock` anchored at `base`.
    #[inline(always)]
    pub const fn new(base: u64) -> Self { Self { base } }

    /// # Safety
    /// `self.base + offset` must be a valid, mapped MMIO address.
    #[inline(always)]
    pub unsafe fn read8(&self, offset: u64) -> u8 {
        unsafe { mmio_read8(self.base + offset) }
    }

    /// # Safety
    /// `self.base + offset` must be valid, mapped, and 2-byte aligned.
    #[inline(always)]
    pub unsafe fn read16(&self, offset: u64) -> u16 {
        unsafe { mmio_read16(self.base + offset) }
    }

    /// # Safety
    /// `self.base + offset` must be valid, mapped, and 4-byte aligned.
    #[inline(always)]
    pub unsafe fn read32(&self, offset: u64) -> u32 {
        unsafe { mmio_read32(self.base + offset) }
    }

    /// # Safety
    /// `self.base + offset` must be valid, mapped, and 8-byte aligned.
    #[inline(always)]
    pub unsafe fn read64(&self, offset: u64) -> u64 {
        unsafe { mmio_read64(self.base + offset) }
    }

    /// # Safety
    /// `self.base + offset` must be a valid, mapped MMIO address.
    #[inline(always)]
    pub unsafe fn write8(&self, offset: u64, value: u8) {
        unsafe { mmio_write8(self.base + offset, value) }
    }

    /// # Safety
    /// `self.base + offset` must be valid, mapped, and 2-byte aligned.
    #[inline(always)]
    pub unsafe fn write16(&self, offset: u64, value: u16) {
        unsafe { mmio_write16(self.base + offset, value) }
    }

    /// # Safety
    /// `self.base + offset` must be valid, mapped, and 4-byte aligned.
    #[inline(always)]
    pub unsafe fn write32(&self, offset: u64, value: u32) {
        unsafe { mmio_write32(self.base + offset, value) }
    }

    /// # Safety
    /// `self.base + offset` must be valid, mapped, and 8-byte aligned.
    #[inline(always)]
    pub unsafe fn write64(&self, offset: u64, value: u64) {
        unsafe { mmio_write64(self.base + offset, value) }
    }

    /// # Safety
    /// `self.base + offset` must be valid, mapped, and 4-byte aligned.
    #[inline(always)]
    pub unsafe fn set_bits32(&self, offset: u64, mask: u32) {
        unsafe { mmio_set_bits32(self.base + offset, mask) }
    }

    /// # Safety
    /// `self.base + offset` must be valid, mapped, and 4-byte aligned.
    #[inline(always)]
    pub unsafe fn clear_bits32(&self, offset: u64, mask: u32) {
        unsafe { mmio_clear_bits32(self.base + offset, mask) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Return the address of a local mut variable as a u64 for use as a
    // fake MMIO address. Only valid for the lifetime of the binding.
    fn addr_of_mut<T>(r: &mut T) -> u64 {
        r as *mut T as u64
    }

    // --- mmio_read8 / mmio_write8 ---

    #[test_case]
    fn read_write_u8() {
        let mut storage: u8 = 0;
        let addr = addr_of_mut(&mut storage);
        unsafe { mmio_write8(addr, 0xAB) };
        assert_eq!(unsafe { mmio_read8(addr) }, 0xAB);
    }

    // --- mmio_read16 / mmio_write16 ---

    #[test_case]
    fn read_write_u16() {
        let mut storage: u16 = 0;
        let addr = addr_of_mut(&mut storage);
        unsafe { mmio_write16(addr, 0xBEEF) };
        assert_eq!(unsafe { mmio_read16(addr) }, 0xBEEF);
    }

    // --- mmio_read32 / mmio_write32 ---

    #[test_case]
    fn read_write_u32_roundtrip() {
        let mut storage: u32 = 0;
        let addr = addr_of_mut(&mut storage);
        for v in [0u32, 1, 0xFF00_FF00, u32::MAX] {
            unsafe { mmio_write32(addr, v) };
            assert_eq!(unsafe { mmio_read32(addr) }, v);
        }
    }

    // --- mmio_read64 / mmio_write64 ---

    #[test_case]
    fn read_write_u64_roundtrip() {
        let mut storage: u64 = 0;
        let addr = addr_of_mut(&mut storage);
        for v in [0u64, 1, 0xDEAD_BEEF_1234_5678, u64::MAX] {
            unsafe { mmio_write64(addr, v) };
            assert_eq!(unsafe { mmio_read64(addr) }, v);
        }
    }

    // --- mmio_set_bits32 ---

    #[test_case]
    fn set_bits32_ors_into_existing() {
        let mut storage: u32 = 0x0000_00F0;
        let addr = addr_of_mut(&mut storage);
        unsafe { mmio_set_bits32(addr, 0x0000_000F) };
        assert_eq!(unsafe { mmio_read32(addr) }, 0x0000_00FF);
    }

    #[test_case]
    fn set_bits32_idempotent() {
        let mut storage: u32 = 0xFFFF_FFFF;
        let addr = addr_of_mut(&mut storage);
        unsafe { mmio_set_bits32(addr, 0x1234_5678) };
        assert_eq!(unsafe { mmio_read32(addr) }, 0xFFFF_FFFF);
    }

    // --- mmio_clear_bits32 ---

    #[test_case]
    fn clear_bits32_removes_bits() {
        let mut storage: u32 = 0x0000_00FF;
        let addr = addr_of_mut(&mut storage);
        unsafe { mmio_clear_bits32(addr, 0x0000_000F) };
        assert_eq!(unsafe { mmio_read32(addr) }, 0x0000_00F0);
    }

    #[test_case]
    fn clear_bits32_idempotent_on_zero() {
        let mut storage: u32 = 0;
        let addr = addr_of_mut(&mut storage);
        unsafe { mmio_clear_bits32(addr, 0xFFFF_FFFF) };
        assert_eq!(unsafe { mmio_read32(addr) }, 0);
    }

    // --- mmio_update_field32 ---

    #[test_case]
    fn update_field32_replaces_masked_bits() {
        let mut storage: u32 = 0xFFFF_FFFF;
        let addr = addr_of_mut(&mut storage);
        // Clear bits [7:4] and set them to 0b0101
        let mask: u32 = 0x0000_00F0;
        let value: u32 = 0x0000_0050;  // 0b0101 shifted to bits [7:4]
        unsafe { mmio_update_field32(addr, mask, value) };
        let result = unsafe { mmio_read32(addr) };
        assert_eq!(result & mask, value);
        assert_eq!(result & !mask, 0xFFFF_FF0F);  // rest unchanged
    }

    // --- mmio_set_bits64 / mmio_clear_bits64 ---

    #[test_case]
    fn set_and_clear_bits64() {
        let mut storage: u64 = 0;
        let addr = addr_of_mut(&mut storage);
        unsafe { mmio_set_bits64(addr, 1 << 63) };
        assert_eq!(unsafe { mmio_read64(addr) }, 1 << 63);
        unsafe { mmio_clear_bits64(addr, 1 << 63) };
        assert_eq!(unsafe { mmio_read64(addr) }, 0);
    }

    // --- MmioBlock ---

    #[test_case]
    fn mmio_block_read_write32() {
        let mut regs = [0u32; 4];
        let base = addr_of_mut(&mut regs[0]);
        let block = MmioBlock::new(base);
        unsafe { block.write32(4, 0xDEAD_BEEF) };
        assert_eq!(unsafe { block.read32(4) }, 0xDEAD_BEEF);
        // Offset 0 must be untouched
        assert_eq!(unsafe { block.read32(0) }, 0);
    }

    #[test_case]
    fn mmio_block_set_and_clear_bits32() {
        let mut storage: u32 = 0x00FF_00FF;
        let base = addr_of_mut(&mut storage);
        let block = MmioBlock::new(base);
        unsafe { block.set_bits32(0, 0xFF00_FF00) };
        assert_eq!(unsafe { block.read32(0) }, 0xFFFF_FFFF);
        unsafe { block.clear_bits32(0, 0x0F0F_0F0F) };
        assert_eq!(unsafe { block.read32(0) }, 0xF0F0_F0F0);
    }

    #[test_case]
    fn mmio_block_read_write8() {
        let mut storage: u8 = 0;
        let base = addr_of_mut(&mut storage);
        let block = MmioBlock::new(base);
        unsafe { block.write8(0, 0x7E) };
        assert_eq!(unsafe { block.read8(0) }, 0x7E);
    }

    #[test_case]
    fn mmio_block_read_write64() {
        let mut storage: u64 = 0;
        let base = addr_of_mut(&mut storage);
        let block = MmioBlock::new(base);
        unsafe { block.write64(0, u64::MAX) };
        assert_eq!(unsafe { block.read64(0) }, u64::MAX);
    }
}
