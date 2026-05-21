/// Read a byte from x86 I/O port `port`.
///
/// # Safety
/// Executing `IN` at the wrong port can crash the system, corrupt device
/// state, or cause non-maskable interrupts. Caller must ensure the port
/// is valid and the current privilege level permits port access (CPL=0
/// or IOPL ≥ CPL, or the I/O Permission Bitmap grants access).
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

/// Read a word (u16) from I/O port `port`.
///
/// # Safety
/// See [`inb`] safety requirements.
#[inline(always)]
pub unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    unsafe {
        core::arch::asm!(
            "in ax, dx",
            in("dx") port,
            out("ax") value,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

/// Read a dword (u32) from I/O port `port`.
///
/// # Safety
/// See [`inb`] safety requirements.
#[inline(always)]
pub unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    unsafe {
        core::arch::asm!(
            "in eax, dx",
            in("dx") port,
            out("eax") value,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

/// Write a byte to I/O port `port`.
///
/// # Safety
/// See [`inb`] safety requirements.
#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Write a word to I/O port `port`.
///
/// # Safety
/// See [`inb`] safety requirements.
#[inline(always)]
pub unsafe fn outw(port: u16, value: u16) {
    unsafe {
        core::arch::asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Write a dword to I/O port `port`.
///
/// # Safety
/// See [`inb`] safety requirements.
#[inline(always)]
pub unsafe fn outl(port: u16, value: u32) {
    unsafe {
        core::arch::asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Read-modify-write: set bits in a port I/O register (8-bit).
///
/// # Safety
/// See [`inb`] safety requirements.
#[inline(always)]
pub unsafe fn pio_set_bits8(port: u16, mask: u8) {
    unsafe { outb(port, inb(port) | mask) };
}

/// Read-modify-write: clear bits in a port I/O register (8-bit).
///
/// # Safety
/// See [`inb`] safety requirements.
#[inline(always)]
pub unsafe fn pio_clear_bits8(port: u16, mask: u8) {
    unsafe { outb(port, inb(port) & !mask) };
}

// Port I/O functions require real hardware to execute. These tests only
// verify that the function signatures compile and that all variants exist.
// Actual IN/OUT behavior is verified by running on x86_64 hardware or in an
// emulator (QEMU) with appropriate privilege level.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pio_functions_have_correct_signatures() {
        // Verify that each function can be referenced — this catches regressions
        // in signature (wrong type, removed function, etc.) without executing
        // any IN/OUT instruction.
        let _inb:  unsafe fn(u16) -> u8        = inb;
        let _inw:  unsafe fn(u16) -> u16       = inw;
        let _inl:  unsafe fn(u16) -> u32       = inl;
        let _outb: unsafe fn(u16, u8)          = outb;
        let _outw: unsafe fn(u16, u16)         = outw;
        let _outl: unsafe fn(u16, u32)         = outl;
        let _set:  unsafe fn(u16, u8)          = pio_set_bits8;
        let _clr:  unsafe fn(u16, u8)          = pio_clear_bits8;
    }
}
