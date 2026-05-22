//! x86_64 CPUID instruction and CPU feature detection helpers.
//!
//! Use [`cpuid`] / [`cpuid_leaf`] for raw leaf access. Use the `has_*`
//! functions for feature checks — these are safe wrappers that guard against
//! calling unsupported extended leaves.

/// The four output registers of a `CPUID` instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuidResult {
    /// EAX output.
    pub eax: u32,
    /// EBX output.
    pub ebx: u32,
    /// ECX output (also used as a sub-leaf input).
    pub ecx: u32,
    /// EDX output.
    pub edx: u32,
}

/// Execute the `CPUID` instruction with the given leaf (`eax`) and sub-leaf (`ecx`).
///
/// # Safety
/// `CPUID` is a serializing instruction that may behave unexpectedly in some
/// hypervisor environments. Passing a leaf beyond `max_basic_leaf()` or
/// `max_extended_leaf()` may return zeroed or undefined data on some CPUs.
/// Check the maximum supported leaf before calling extended leaves.
#[inline(always)]
pub unsafe fn cpuid(eax: u32, ecx: u32) -> CpuidResult {
    let out_eax: u32;
    let out_ebx: u32;
    let out_ecx: u32;
    let out_edx: u32;
    unsafe {
        // rbx is reserved by LLVM on x86_64; save/restore it around CPUID.
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {tmp:e}, ebx",
            "pop rbx",
            inout("eax") eax => out_eax,
            inout("ecx") ecx => out_ecx,
            tmp = out(reg) out_ebx,
            out("edx") out_edx,
            options(nomem, preserves_flags)
        );
    }
    CpuidResult { eax: out_eax, ebx: out_ebx, ecx: out_ecx, edx: out_edx }
}

/// Execute `CPUID` with the given leaf and sub-leaf 0.
///
/// # Safety
/// See [`cpuid`].
#[inline(always)]
pub unsafe fn cpuid_leaf(eax: u32) -> CpuidResult {
    unsafe { cpuid(eax, 0) }
}

/// Return the maximum basic CPUID leaf supported by this CPU.
///
/// # Safety
/// See [`cpuid`].
#[inline(always)]
pub unsafe fn max_basic_leaf() -> u32 {
    unsafe { cpuid_leaf(0).eax }
}

/// Return the maximum extended CPUID leaf supported by this CPU.
///
/// On all x86_64 CPUs this is at least `0x8000_0000`.
///
/// # Safety
/// See [`cpuid`].
#[inline(always)]
pub unsafe fn max_extended_leaf() -> u32 {
    unsafe { cpuid_leaf(0x8000_0000).eax }
}

// ---------------------------------------------------------------------------
// Feature detection helpers
//
// Exposed as safe `fn` matching the Linux `boot_cpu_has()` convention: kernel
// code universally calls these without `unsafe`. The unsafety is an
// implementation detail — CPUID doesn't modify architectural state and the
// worst a hostile hypervisor can do is return wrong results.
// ---------------------------------------------------------------------------

/// `true` if the CPU supports the No-Execute (NX/XD) bit in page table entries.
///
/// Required before setting `EFER.NXE`. CPUID leaf `0x8000_0001` EDX bit 20.
pub fn has_nx() -> bool {
    if unsafe { max_extended_leaf() } < 0x8000_0001 { return false; }
    unsafe { cpuid(0x8000_0001, 0).edx >> 20 & 1 == 1 }
}

/// `true` if the CPU has a local APIC.
///
/// CPUID leaf `0x0000_0001` EDX bit 9.
pub fn has_apic() -> bool {
    if unsafe { max_basic_leaf() } < 1 { return false; }
    unsafe { cpuid_leaf(0x0000_0001).edx >> 9 & 1 == 1 }
}

/// `true` if the CPU supports SSE.
///
/// CPUID leaf `0x0000_0001` EDX bit 25.
pub fn has_sse() -> bool {
    if unsafe { max_basic_leaf() } < 1 { return false; }
    unsafe { cpuid_leaf(0x0000_0001).edx >> 25 & 1 == 1 }
}

/// `true` if the CPU supports SSE2.
///
/// CPUID leaf `0x0000_0001` EDX bit 26. Guaranteed on all x86_64 CPUs.
pub fn has_sse2() -> bool {
    if unsafe { max_basic_leaf() } < 1 { return false; }
    unsafe { cpuid_leaf(0x0000_0001).edx >> 26 & 1 == 1 }
}

/// `true` if the CPU supports AVX.
///
/// CPUID leaf `0x0000_0001` ECX bit 28. Also requires XSAVE support.
pub fn has_avx() -> bool {
    if unsafe { max_basic_leaf() } < 1 { return false; }
    unsafe { cpuid_leaf(0x0000_0001).ecx >> 28 & 1 == 1 }
}

/// `true` if the CPU supports XSAVE / XRSTOR (required for AVX state saving).
///
/// CPUID leaf `0x0000_0001` ECX bit 26.
pub fn has_xsave() -> bool {
    if unsafe { max_basic_leaf() } < 1 { return false; }
    unsafe { cpuid_leaf(0x0000_0001).ecx >> 26 & 1 == 1 }
}

/// `true` if the CPU supports PCID (process-context identifiers).
///
/// CPUID leaf `0x0000_0001` ECX bit 17.
pub fn has_pcid() -> bool {
    if unsafe { max_basic_leaf() } < 1 { return false; }
    unsafe { cpuid_leaf(0x0000_0001).ecx >> 17 & 1 == 1 }
}

/// `true` if the CPU supports FSGSBASE instructions (`RDFSBASE`, etc.).
///
/// CPUID leaf `0x0000_0007` (ecx=0) EBX bit 0.
pub fn has_fsgsbase() -> bool {
    if unsafe { max_basic_leaf() } < 7 { return false; }
    unsafe { cpuid(0x0000_0007, 0).ebx & 1 == 1 }
}

/// `true` if the CPU supports INVPCID.
///
/// CPUID leaf `0x0000_0007` (ecx=0) EBX bit 10.
pub fn has_invpcid() -> bool {
    if unsafe { max_basic_leaf() } < 7 { return false; }
    unsafe { cpuid(0x0000_0007, 0).ebx >> 10 & 1 == 1 }
}

/// `true` if the CPU supports SMEP (Supervisor Mode Execution Prevention).
///
/// CPUID leaf `0x0000_0007` (ecx=0) EBX bit 7.
pub fn has_smep() -> bool {
    if unsafe { max_basic_leaf() } < 7 { return false; }
    unsafe { cpuid(0x0000_0007, 0).ebx >> 7 & 1 == 1 }
}

/// `true` if the CPU supports SMAP (Supervisor Mode Access Prevention).
///
/// CPUID leaf `0x0000_0007` (ecx=0) EBX bit 20.
pub fn has_smap() -> bool {
    if unsafe { max_basic_leaf() } < 7 { return false; }
    unsafe { cpuid(0x0000_0007, 0).ebx >> 20 & 1 == 1 }
}

/// `true` if the CPU supports CET Shadow Stack (CET_SS).
///
/// CPUID leaf `0x0000_0007` (ecx=0) ECX bit 7.
pub fn has_cet_ss() -> bool {
    if unsafe { max_basic_leaf() } < 7 { return false; }
    unsafe { cpuid(0x0000_0007, 0).ecx >> 7 & 1 == 1 }
}

/// `true` if the CPU supports CET Indirect Branch Tracking (CET_IBT).
///
/// CPUID leaf `0x0000_0007` (ecx=0) EDX bit 20.
pub fn has_cet_ibt() -> bool {
    if unsafe { max_basic_leaf() } < 7 { return false; }
    unsafe { cpuid(0x0000_0007, 0).edx >> 20 & 1 == 1 }
}

/// `true` if the CPU supports Memory Protection Keys for user pages (PKU).
///
/// CPUID leaf `0x0000_0007` (ecx=0) ECX bit 3.
pub fn has_pku() -> bool {
    if unsafe { max_basic_leaf() } < 7 { return false; }
    unsafe { cpuid(0x0000_0007, 0).ecx >> 3 & 1 == 1 }
}

/// `true` if the CPU supports UMIP (User-Mode Instruction Prevention).
///
/// Prevents `SGDT`, `SIDT`, `SLDT`, `SMSW`, `STR` at CPL > 0.
/// CPUID leaf `0x0000_0007` (ecx=0) ECX bit 2.
pub fn has_umip() -> bool {
    if unsafe { max_basic_leaf() } < 7 { return false; }
    unsafe { cpuid(0x0000_0007, 0).ecx >> 2 & 1 == 1 }
}

/// `true` if the CPU supports 1 GiB gigantic pages.
///
/// CPUID leaf `0x8000_0001` EDX bit 26.
pub fn has_1gb_pages() -> bool {
    if unsafe { max_extended_leaf() } < 0x8000_0001 { return false; }
    unsafe { cpuid(0x8000_0001, 0).edx >> 26 & 1 == 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn cpuid_result_is_copy() {
        let r = CpuidResult { eax: 1, ebx: 2, ecx: 3, edx: 4 };
        let r2 = r;
        assert_eq!(r, r2);
    }

    #[test_case]
    fn cpuid_function_signatures() {
        let _cpuid:             unsafe fn(u32, u32) -> CpuidResult = cpuid;
        let _cpuid_leaf:        unsafe fn(u32) -> CpuidResult      = cpuid_leaf;
        let _max_basic:         unsafe fn() -> u32                 = max_basic_leaf;
        let _max_extended:      unsafe fn() -> u32                 = max_extended_leaf;
    }

    #[test_case]
    fn feature_fn_signatures() {
        let _: fn() -> bool = has_nx;
        let _: fn() -> bool = has_apic;
        let _: fn() -> bool = has_sse;
        let _: fn() -> bool = has_sse2;
        let _: fn() -> bool = has_avx;
        let _: fn() -> bool = has_xsave;
        let _: fn() -> bool = has_pcid;
        let _: fn() -> bool = has_fsgsbase;
        let _: fn() -> bool = has_invpcid;
        let _: fn() -> bool = has_smep;
        let _: fn() -> bool = has_smap;
        let _: fn() -> bool = has_cet_ss;
        let _: fn() -> bool = has_cet_ibt;
        let _: fn() -> bool = has_pku;
        let _: fn() -> bool = has_umip;
        let _: fn() -> bool = has_1gb_pages;
    }
}
