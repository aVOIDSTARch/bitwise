/// A raw 64-bit flag register value with named bit operations.
///
/// Construct from a `u64`; manipulate with the methods below.
/// The underlying representation is always the raw machine word.
///

use core::{clone::Clone, cmp::{Eq, PartialEq}, fmt::Debug, marker::Copy};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct FlagRegister(pub u64);

impl FlagRegister {
    /// All-zero register value (all flags clear).
    pub const ZERO: Self = Self(0);

    /// Construct from a raw `u64` machine word.
    #[inline(always)]
    pub const fn from_raw(raw: u64) -> Self { Self(raw) }

    /// Return the underlying `u64` machine word.
    #[inline(always)]
    pub const fn raw(self) -> u64 { self.0 }

    /// Test a single flag bit.
    #[inline(always)]
    pub const fn has(self, flag: u64) -> bool {
        (self.0 & flag) == flag
    }

    /// Test any of a set of flag bits.
    #[inline(always)]
    pub const fn has_any(self, flags: u64) -> bool {
        (self.0 & flags) != 0
    }

    /// Set one or more flags (OR).
    #[inline(always)]
    pub const fn set(self, flags: u64) -> Self {
        Self(self.0 | flags)
    }

    /// Clear one or more flags (AND NOT).
    #[inline(always)]
    pub const fn clear(self, flags: u64) -> Self {
        Self(self.0 & !flags)
    }

    /// Toggle one or more flags (XOR).
    #[inline(always)]
    pub const fn toggle(self, flags: u64) -> Self {
        Self(self.0 ^ flags)
    }

    /// Replace a multi-bit field. `mask` selects which bits to replace;
    /// `value` is already positioned (not right-justified).
    #[inline(always)]
    pub const fn replace_field(self, mask: u64, value: u64) -> Self {
        Self((self.0 & !mask) | (value & mask))
    }

    /// Return a new register with only the specified flags preserved.
    #[inline(always)]
    pub const fn isolate(self, mask: u64) -> Self {
        Self(self.0 & mask)
    }
}

/// RFLAGS register bit definitions (Intel Vol. 1 §3.4.3, Vol. 3A §2.3).
pub mod rflags {
    /// Carry Flag — set by arithmetic that produces an unsigned overflow.
    pub const CF:   u64 = 1 << 0;
    /// Parity Flag — set if the low byte of the result has even parity.
    pub const PF:   u64 = 1 << 2;
    /// Auxiliary Carry Flag — carry out of bit 3; used by BCD instructions.
    pub const AF:   u64 = 1 << 4;
    /// Zero Flag — set if the result is zero.
    pub const ZF:   u64 = 1 << 6;
    /// Sign Flag — copy of the most significant bit of the result.
    pub const SF:   u64 = 1 << 7;
    /// Trap Flag — single-step mode; raises a debug exception after each instruction.
    pub const TF:   u64 = 1 << 8;
    /// Interrupt Enable Flag — when set, maskable hardware interrupts are acknowledged.
    pub const IF:   u64 = 1 << 9;
    /// Direction Flag — controls string instruction direction (0 = increment, 1 = decrement).
    pub const DF:   u64 = 1 << 10;
    /// Overflow Flag — set when signed arithmetic overflows.
    pub const OF:   u64 = 1 << 11;
    /// I/O Privilege Level — 2-bit field (bits 13:12); minimum CPL required for I/O instructions.
    pub const IOPL: u64 = 3 << 12;
    /// Nested Task — set when a hardware task switch links the current task to a prior one.
    pub const NT:   u64 = 1 << 14;
    /// Resume Flag — suppresses debug faults on the immediately following instruction.
    pub const RF:   u64 = 1 << 16;
    /// Virtual-8086 Mode — enables V86 mode when set from protected mode.
    pub const VM:   u64 = 1 << 17;
    /// Alignment Check — enables alignment-fault checking when CR0.AM is also set.
    pub const AC:   u64 = 1 << 18;
    /// Virtual Interrupt Flag — virtual copy of IF; used with virtual interrupt delivery.
    pub const VIF:  u64 = 1 << 19;
    /// Virtual Interrupt Pending — set by software to signal a pending virtual interrupt.
    pub const VIP:  u64 = 1 << 20;
    /// CPUID Toggle — if software can toggle this bit, the CPUID instruction is supported.
    pub const ID:   u64 = 1 << 21;
}

/// CR0 register bit definitions (Intel Vol. 3A §2.5).
pub mod cr0 {
    /// Protection Enable — switches the CPU from real mode to protected mode.
    pub const PE:  u64 = 1 << 0;
    /// Monitor Coprocessor — controls WAIT/FWAIT interaction with the TS flag.
    pub const MP:  u64 = 1 << 1;
    /// Emulation — when set, x87 FPU instructions raise #NM (no FPU present).
    pub const EM:  u64 = 1 << 2;
    /// Task Switched — set by hardware on task switch; triggers #NM on x87/SSE use.
    pub const TS:  u64 = 1 << 3;
    /// Extension Type — reads as 1 on all modern CPUs (80387-compatible coprocessor).
    pub const ET:  u64 = 1 << 4;
    /// Numeric Error — enables internal x87 FPU error reporting via IRQ 13.
    pub const NE:  u64 = 1 << 5;
    /// Write Protect — when set, ring-0 code cannot write to read-only user pages.
    pub const WP:  u64 = 1 << 16;
    /// Alignment Mask — enables alignment-fault exceptions when RFLAGS.AC is also set.
    pub const AM:  u64 = 1 << 18;
    /// Not Write-Through — when set, disables write-through caching globally (deprecated).
    pub const NW:  u64 = 1 << 29;
    /// Cache Disable — when set, disables the processor's memory cache globally.
    pub const CD:  u64 = 1 << 30;
    /// Paging Enable — activates paging; CR0.PE must also be set.
    pub const PG:  u64 = 1 << 31;
}

/// CR4 register bit definitions (Intel Vol. 3A §2.6).
pub mod cr4 {
    /// Virtual-8086 Mode Extensions — enables hardware interrupt/exception handling in V86 mode.
    pub const VME:        u64 = 1 << 0;
    /// Protected-Mode Virtual Interrupts — enables virtual interrupt flag in protected mode.
    pub const PVI:        u64 = 1 << 1;
    /// Time Stamp Disable — when set, RDTSC is restricted to CPL=0.
    pub const TSD:        u64 = 1 << 2;
    /// Debugging Extensions — enables I/O breakpoints via DR4/DR5 and debug extensions.
    pub const DE:         u64 = 1 << 3;
    /// Page Size Extension — enables 4 MiB pages in 32-bit (non-PAE) paging mode.
    pub const PSE:        u64 = 1 << 4;
    /// Physical Address Extension — enables 36-bit physical addressing in 32-bit mode.
    pub const PAE:        u64 = 1 << 5;
    /// Machine Check Enable — enables the machine-check exception (#MC) mechanism.
    pub const MCE:        u64 = 1 << 6;
    /// Page Global Enable — allows pages to be marked global (not flushed on CR3 writes).
    pub const PGE:        u64 = 1 << 7;
    /// Performance Counter Enable — allows RDPMC at any privilege level when set.
    pub const PCE:        u64 = 1 << 8;
    /// OS FXSAVE/FXRSTOR Support — required before using SSE instructions.
    pub const OSFXSR:     u64 = 1 << 9;
    /// OS Unmasked SIMD FP Exception Support — enables #XF exceptions for SSE FP errors.
    pub const OSXMMEXCPT: u64 = 1 << 10;
    /// User-Mode Instruction Prevention — blocks SGDT/SLDT/SIDT/SMSW/STR at CPL > 0.
    pub const UMIP:       u64 = 1 << 11;
    /// 57-bit Linear Addresses — enables 5-level paging (57-bit virtual address space).
    pub const LA57:       u64 = 1 << 12;
    /// VMX Enable — enables Virtual Machine Extensions (Intel VT-x).
    pub const VMXE:       u64 = 1 << 13;
    /// SMX Enable — enables Safer Mode Extensions (Intel TXT).
    pub const SMXE:       u64 = 1 << 14;
    /// FSGSBASE Enable — allows RDFSBASE/RDGSBASE/WRFSBASE/WRGSBASE at any CPL.
    pub const FSGSBASE:   u64 = 1 << 16;
    /// PCID Enable — enables process-context identifiers to reduce TLB flush overhead.
    pub const PCIDE:      u64 = 1 << 17;
    /// XSAVE Enable — enables XSAVE/XRSTOR and extended processor state management.
    pub const OSXSAVE:    u64 = 1 << 18;
    /// Supervisor Mode Execution Prevention — prevents ring-0 from executing user-mode pages.
    pub const SMEP:       u64 = 1 << 20;
    /// Supervisor Mode Access Prevention — prevents ring-0 from accessing user-mode pages.
    pub const SMAP:       u64 = 1 << 21;
    /// Protection Key Enable — enables user-space memory protection key enforcement.
    pub const PKE:        u64 = 1 << 22;
    /// Control-flow Enforcement Technology — enables shadow stack and indirect branch tracking.
    pub const CET:        u64 = 1 << 23;
    /// Protection Keys for Supervisor — enables supervisor-mode protection key enforcement.
    pub const PKS:        u64 = 1 << 24;
}

/// EFER (Extended Feature Enable Register, MSR `0xC000_0080`) bit definitions.
pub mod efer {
    /// SYSCALL Enable — enables the SYSCALL/SYSRET fast system-call instructions.
    pub const SCE:   u64 = 1 << 0;
    /// Long Mode Enable — enables IA-32e (64-bit) mode; takes effect when CR0.PG is set.
    pub const LME:   u64 = 1 << 8;
    /// Long Mode Active — set by hardware when long mode is active (read-only).
    pub const LMA:   u64 = 1 << 10;
    /// No-Execute Enable — enables the NX page attribute in PTEs (requires PAE paging).
    pub const NXE:   u64 = 1 << 11;
    /// SVM Enable — enables AMD Secure Virtual Machine extensions (AMD-V).
    pub const SVME:  u64 = 1 << 12;
    /// Long Mode Segment Limit Enable — re-enables segment limit checking in 64-bit mode (AMD).
    pub const LMSLE: u64 = 1 << 13;
    /// Fast FXSAVE/FXRSTOR — omits saving legacy x87 FPU state in 64-bit mode (AMD).
    pub const FFXSR: u64 = 1 << 14;
    /// Translation Cache Extension — enables extended TLB tagging (AMD).
    pub const TCE:   u64 = 1 << 15;
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- FlagRegister construction ---

    #[cfg(test)]
    #[test_case]
    fn zero_is_all_zeros() {
        assert_eq!(FlagRegister::ZERO.raw(), 0);
    }

    #[cfg(test)]
    #[test_case]
    fn from_raw_roundtrip() {
        for v in [0u64, 1, 0xFF, u64::MAX, 0xDEAD_BEEF] {
            assert_eq!(FlagRegister::from_raw(v).raw(), v);
        }
    }

    // --- has / has_any ---

    #[cfg(test)]
    #[test_case]
    fn has_single_flag() {
        let r = FlagRegister::from_raw(rflags::ZF | rflags::CF);
        assert!(r.has(rflags::ZF));
        assert!(r.has(rflags::CF));
        assert!(!r.has(rflags::SF));
    }

    #[cfg(test)]
    #[test_case]
    fn has_requires_all_bits() {
        let r = FlagRegister::from_raw(rflags::IOPL);  // bits 13:12, value 3<<12
        assert!(r.has(rflags::IOPL));
        assert!(!r.has(rflags::IOPL | rflags::ZF));  // ZF not set
    }

    #[cfg(test)]
    #[test_case]
    fn has_any_matches_any_bit() {
        let r = FlagRegister::from_raw(rflags::ZF);
        assert!(r.has_any(rflags::ZF | rflags::SF));
        assert!(!r.has_any(rflags::SF | rflags::CF));
    }

    // --- set / clear / toggle ---

    #[cfg(test)]
    #[test_case]
    fn set_adds_bits() {
        let r = FlagRegister::ZERO.set(rflags::IF);
        assert!(r.has(rflags::IF));
        assert_eq!(r.raw(), rflags::IF);
    }

    #[cfg(test)]
    #[test_case]
    fn clear_removes_bits() {
        let r = FlagRegister::from_raw(rflags::IF | rflags::ZF).clear(rflags::IF);
        assert!(!r.has(rflags::IF));
        assert!(r.has(rflags::ZF));
    }

    #[cfg(test)]
    #[test_case]
    fn toggle_flips_bits() {
        let r = FlagRegister::from_raw(rflags::ZF);
        let toggled = r.toggle(rflags::ZF | rflags::CF);
        assert!(!toggled.has(rflags::ZF));
        assert!(toggled.has(rflags::CF));
    }

    #[cfg(test)]
    #[test_case]
    fn toggle_roundtrip() {
        let original = FlagRegister::from_raw(0xABCD_EF01);
        assert_eq!(original.toggle(u64::MAX).toggle(u64::MAX), original);
    }

    // --- replace_field ---

    #[cfg(test)]
    #[test_case]
    fn replace_field_iopl() {
        let r = FlagRegister::ZERO.replace_field(rflags::IOPL, 2 << 12);
        assert_eq!(r.raw() >> 12 & 0x3, 2);
    }

    #[cfg(test)]
    #[test_case]
    fn replace_field_does_not_touch_other_bits() {
        let r = FlagRegister::from_raw(rflags::IF | rflags::ZF);
        let r2 = r.replace_field(rflags::IOPL, 3 << 12);
        assert!(r2.has(rflags::IF));
        assert!(r2.has(rflags::ZF));
        assert_eq!(r2.raw() >> 12 & 0x3, 3);
    }

    // --- isolate ---

    #[cfg(test)]
    #[test_case]
    fn isolate_keeps_only_masked_bits() {
        let r = FlagRegister::from_raw(rflags::IF | rflags::ZF | rflags::CF);
        let iso = r.isolate(rflags::IF | rflags::ZF);
        assert!(iso.has(rflags::IF));
        assert!(iso.has(rflags::ZF));
        assert!(!iso.has(rflags::CF));
    }

    // --- rflags constant values ---

    #[cfg(test)]
    #[test_case]
    fn rflags_bit_positions() {
        assert_eq!(rflags::CF,   1 << 0);
        assert_eq!(rflags::PF,   1 << 2);
        assert_eq!(rflags::AF,   1 << 4);
        assert_eq!(rflags::ZF,   1 << 6);
        assert_eq!(rflags::SF,   1 << 7);
        assert_eq!(rflags::TF,   1 << 8);
        assert_eq!(rflags::IF,   1 << 9);
        assert_eq!(rflags::DF,   1 << 10);
        assert_eq!(rflags::OF,   1 << 11);
        assert_eq!(rflags::IOPL, 3 << 12);
        assert_eq!(rflags::NT,   1 << 14);
        assert_eq!(rflags::RF,   1 << 16);
        assert_eq!(rflags::VM,   1 << 17);
        assert_eq!(rflags::AC,   1 << 18);
        assert_eq!(rflags::VIF,  1 << 19);
        assert_eq!(rflags::VIP,  1 << 20);
        assert_eq!(rflags::ID,   1 << 21);
    }

    #[cfg(test)]
    #[test_case]
    fn rflags_no_overlapping_single_bit_flags() {
        let single_bits = [
            rflags::CF, rflags::PF, rflags::AF, rflags::ZF,
            rflags::SF, rflags::TF, rflags::IF, rflags::DF,
            rflags::OF, rflags::NT, rflags::RF, rflags::VM,
            rflags::AC, rflags::VIF, rflags::VIP, rflags::ID,
        ];
        for (i, &a) in single_bits.iter().enumerate() {
            for (j, &b) in single_bits.iter().enumerate() {
                if i != j {
                    assert_eq!(a & b, 0, "rflags constants overlap: {:#x} & {:#x}", a, b);
                }
            }
        }
    }

    // --- cr0 constant values ---

    #[cfg(test)]
    #[test_case]
    fn cr0_bit_positions() {
        assert_eq!(cr0::PE,  1 << 0);
        assert_eq!(cr0::MP,  1 << 1);
        assert_eq!(cr0::EM,  1 << 2);
        assert_eq!(cr0::TS,  1 << 3);
        assert_eq!(cr0::ET,  1 << 4);
        assert_eq!(cr0::NE,  1 << 5);
        assert_eq!(cr0::WP,  1 << 16);
        assert_eq!(cr0::AM,  1 << 18);
        assert_eq!(cr0::NW,  1 << 29);
        assert_eq!(cr0::CD,  1 << 30);
        assert_eq!(cr0::PG,  1 << 31);
    }

    #[cfg(test)]
    #[test_case]
    fn cr0_no_overlapping_flags() {
        let flags = [
            cr0::PE, cr0::MP, cr0::EM, cr0::TS, cr0::ET,
            cr0::NE, cr0::WP, cr0::AM, cr0::NW, cr0::CD, cr0::PG,
        ];
        for (i, &a) in flags.iter().enumerate() {
            for (j, &b) in flags.iter().enumerate() {
                if i != j {
                    assert_eq!(a & b, 0, "cr0 constants overlap: {:#x} & {:#x}", a, b);
                }
            }
        }
    }

    // --- cr4 constant values ---

    #[cfg(test)]
    #[test_case]
    fn cr4_bit_positions() {
        assert_eq!(cr4::VME,        1 << 0);
        assert_eq!(cr4::PVI,        1 << 1);
        assert_eq!(cr4::TSD,        1 << 2);
        assert_eq!(cr4::DE,         1 << 3);
        assert_eq!(cr4::PSE,        1 << 4);
        assert_eq!(cr4::PAE,        1 << 5);
        assert_eq!(cr4::MCE,        1 << 6);
        assert_eq!(cr4::PGE,        1 << 7);
        assert_eq!(cr4::PCE,        1 << 8);
        assert_eq!(cr4::OSFXSR,     1 << 9);
        assert_eq!(cr4::OSXMMEXCPT, 1 << 10);
        assert_eq!(cr4::UMIP,       1 << 11);
        assert_eq!(cr4::LA57,       1 << 12);
        assert_eq!(cr4::VMXE,       1 << 13);
        assert_eq!(cr4::SMXE,       1 << 14);
        assert_eq!(cr4::FSGSBASE,   1 << 16);
        assert_eq!(cr4::PCIDE,      1 << 17);
        assert_eq!(cr4::OSXSAVE,    1 << 18);
        assert_eq!(cr4::SMEP,       1 << 20);
        assert_eq!(cr4::SMAP,       1 << 21);
        assert_eq!(cr4::PKE,        1 << 22);
        assert_eq!(cr4::CET,        1 << 23);
        assert_eq!(cr4::PKS,        1 << 24);
    }

    // --- efer constant values ---

    #[cfg(test)]
    #[test_case]
    fn efer_bit_positions() {
        assert_eq!(efer::SCE,   1 << 0);
        assert_eq!(efer::LME,   1 << 8);
        assert_eq!(efer::LMA,   1 << 10);
        assert_eq!(efer::NXE,   1 << 11);
        assert_eq!(efer::SVME,  1 << 12);
        assert_eq!(efer::LMSLE, 1 << 13);
        assert_eq!(efer::FFXSR, 1 << 14);
        assert_eq!(efer::TCE,   1 << 15);
    }

    #[cfg(test)]
    #[test_case]
    fn efer_no_overlapping_flags() {
        let flags = [
            efer::SCE, efer::LME, efer::LMA, efer::NXE,
            efer::SVME, efer::LMSLE, efer::FFXSR, efer::TCE,
        ];
        for (i, &a) in flags.iter().enumerate() {
            for (j, &b) in flags.iter().enumerate() {
                if i != j {
                    assert_eq!(a & b, 0, "efer constants overlap: {:#x} & {:#x}", a, b);
                }
            }
        }
    }

    // --- FlagRegister as a typed wrapper over hardware constants ---

    #[cfg(test)]
    #[test_case]
    fn flag_register_typical_long_mode_efer() {
        // A typical EFER value on boot: SCE + LME + LMA + NXE
        let efer_val = FlagRegister::ZERO
            .set(efer::SCE)
            .set(efer::LME)
            .set(efer::LMA)
            .set(efer::NXE);
        assert!(efer_val.has(efer::LMA));
        assert!(efer_val.has(efer::NXE));
        assert!(!efer_val.has(efer::SVME));
        assert_eq!(efer_val.raw(), efer::SCE | efer::LME | efer::LMA | efer::NXE);
    }

    #[cfg(test)]
    #[test_case]
    fn flag_register_cr0_paging_enable() {
        let cr0_val = FlagRegister::from_raw(cr0::PE | cr0::NE | cr0::WP | cr0::PG);
        assert!(cr0_val.has(cr0::PG));
        assert!(cr0_val.has(cr0::PE));
        assert!(!cr0_val.has(cr0::EM));
        let no_pg = cr0_val.clear(cr0::PG);
        assert!(!no_pg.has(cr0::PG));
        assert!(no_pg.has(cr0::PE));
    }
}
