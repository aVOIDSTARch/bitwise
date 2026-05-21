//! x86_64 Model-Specific Register (MSR) access via `RDMSR` / `WRMSR`.
//!
//! Use [`rdmsr`] and [`wrmsr`] with the constants in [`msr_num`] for named
//! registers. For bit-level definitions of the EFER register see
//! [`crate::flags::efer`].

/// Named MSR addresses (the ECX value passed to RDMSR/WRMSR).
pub mod msr_num {
    /// Extended Feature Enable Register (EFER).
    ///
    /// Controls SCE (syscall), LME/LMA (long mode), NXE (no-execute).
    /// See [`crate::flags::efer`] for individual bit constants.
    pub const EFER:             u32 = 0xC000_0080;

    /// APIC Base Address — holds the LAPIC MMIO base (bits 51:12) and
    /// the APIC global enable flag (bit 11).
    pub const IA32_APIC_BASE:   u32 = 0x0000_001B;

    /// Page Attribute Table — controls memory-type encoding for page table
    /// PAT bits.
    pub const IA32_PAT:         u32 = 0x0000_0277;

    /// FS segment base address (used as TLS base in 64-bit Linux/userspace).
    pub const FS_BASE:          u32 = 0xC000_0100;

    /// GS segment base address (current CPU's per-CPU data pointer in kernels).
    pub const GS_BASE:          u32 = 0xC000_0101;

    /// Kernel GS base — swapped into GS on `SWAPGS`. Stores the kernel
    /// per-CPU base while user GS is live.
    pub const KERNEL_GS_BASE:   u32 = 0xC000_0102;

    /// `SYSCALL` target RIP for 64-bit mode.
    pub const IA32_LSTAR:       u32 = 0xC000_0082;

    /// `SYSCALL` / `SYSRET` segment selectors and RPLs.
    pub const IA32_STAR:        u32 = 0xC000_0081;

    /// `SYSCALL` RFLAGS mask — bits set here are cleared in RFLAGS on entry.
    pub const IA32_FMASK:       u32 = 0xC000_0084;
}

/// Read a 64-bit Model-Specific Register.
///
/// # Safety
/// `ecx` must identify a valid MSR accessible at the current privilege level.
/// Reading an undefined or inaccessible MSR raises `#GP(0)`. Consult the Intel
/// SDM Volume 3B Appendix B for valid MSR ranges for your CPU generation.
#[inline(always)]
pub unsafe fn rdmsr(ecx: u32) -> u64 {
    let eax: u32;
    let edx: u32;
    unsafe {
        core::arch::asm!(
            "rdmsr",
            in("ecx")  ecx,
            out("eax") eax,
            out("edx") edx,
            options(nostack, nomem, preserves_flags)
        );
    }
    ((edx as u64) << 32) | (eax as u64)
}

/// Write a 64-bit value to a Model-Specific Register.
///
/// # Safety
/// `ecx` must identify a valid, writable MSR at the current privilege level.
/// Writing incorrect values to architectural MSRs (e.g., clearing EFER.LME
/// while in long mode, or corrupting STAR selectors before `SYSCALL`) can
/// crash or corrupt the system. Writing to an undefined MSR raises `#GP(0)`.
#[inline(always)]
pub unsafe fn wrmsr(ecx: u32, value: u64) {
    let eax = value as u32;
    let edx = (value >> 32) as u32;
    unsafe {
        core::arch::asm!(
            "wrmsr",
            in("ecx")  ecx,
            in("eax")  eax,
            in("edx")  edx,
            options(nostack, nomem, preserves_flags)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::msr_num::*;

    #[test]
    fn msr_function_signatures() {
        let _rdmsr: unsafe fn(u32) -> u64 = rdmsr;
        let _wrmsr: unsafe fn(u32, u64)   = wrmsr;
    }

    #[test]
    fn msr_num_values_match_intel_sdm() {
        assert_eq!(EFER,           0xC000_0080);
        assert_eq!(IA32_APIC_BASE, 0x0000_001B);
        assert_eq!(IA32_PAT,       0x0000_0277);
        assert_eq!(FS_BASE,        0xC000_0100);
        assert_eq!(GS_BASE,        0xC000_0101);
        assert_eq!(KERNEL_GS_BASE, 0xC000_0102);
        assert_eq!(IA32_LSTAR,     0xC000_0082);
        assert_eq!(IA32_STAR,      0xC000_0081);
        assert_eq!(IA32_FMASK,     0xC000_0084);
    }
}
