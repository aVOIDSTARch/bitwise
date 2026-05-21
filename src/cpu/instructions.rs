//! Thin wrappers around x86_64 CPU instructions that have no safe equivalent
//! in the Rust standard library.
//!
//! All functions are `#[inline(always)]`. Functions that change global CPU
//! state (interrupt enable, power, cache, TLB) are `unsafe`. Pure ordering
//! primitives (`pause`, `mfence`, `lfence`, `sfence`) are safe.

// All functions in this module require x86_64 inline assembly. They compile
// only when the host target is x86_64.

/// Spin-loop hint. Improves power/performance of busy-wait loops.
///
/// Encodes as the `PAUSE` instruction (also serves as `REP NOP`).
/// Safe — no side effects beyond CPU state hints.
#[inline(always)]
pub fn pause() {
    unsafe {
        core::arch::asm!("pause", options(nostack, nomem, preserves_flags));
    }
}

/// Full memory fence — ensures all preceding loads and stores are globally
/// visible before any subsequent memory operation.
///
/// Maps to `MFENCE`. Safe — ordering only, no architectural side effects.
#[inline(always)]
pub fn mfence() {
    unsafe {
        // nomem is intentionally absent: MFENCE is a memory ordering barrier.
        core::arch::asm!("mfence", options(nostack, preserves_flags));
    }
}

/// Load fence — ensures all preceding loads complete before subsequent loads.
///
/// Maps to `LFENCE`. Also serializes instruction fetch on AMD (useful after RDMSR/RDTSC).
#[inline(always)]
pub fn lfence() {
    unsafe {
        core::arch::asm!("lfence", options(nostack, preserves_flags));
    }
}

/// Store fence — ensures all preceding stores are globally visible before
/// subsequent stores.
///
/// Maps to `SFENCE`. Primarily relevant with non-temporal (streaming) stores.
#[inline(always)]
pub fn sfence() {
    unsafe {
        core::arch::asm!("sfence", options(nostack, preserves_flags));
    }
}

/// Halt the CPU until the next interrupt.
///
/// # Safety
/// Must only be called with interrupts enabled (after `sti`) or inside an
/// idle loop that handles interrupt delivery. Halting with interrupts disabled
/// and no NMI pending will freeze the CPU permanently.
#[inline(always)]
pub unsafe fn hlt() {
    unsafe {
        core::arch::asm!("hlt", options(nostack, nomem, preserves_flags));
    }
}

/// Enable hardware interrupts (set RFLAGS.IF).
///
/// # Safety
/// The caller must ensure that all interrupt handlers are installed and that
/// enabling interrupts at this point is safe for the current kernel state.
/// Interrupts may fire immediately after this instruction completes.
#[inline(always)]
pub unsafe fn sti() {
    unsafe {
        // Intentionally omits preserves_flags: STI modifies RFLAGS.IF.
        core::arch::asm!("sti", options(nostack, nomem));
    }
}

/// Disable hardware interrupts (clear RFLAGS.IF).
///
/// # Safety
/// The caller is responsible for re-enabling interrupts (via `sti` or `iretq`)
/// before any point where interrupt delivery is required. Leaving interrupts
/// disabled for too long will prevent timer ticks, IPIs, and device IRQs.
#[inline(always)]
pub unsafe fn cli() {
    unsafe {
        // Intentionally omits preserves_flags: CLI modifies RFLAGS.IF.
        core::arch::asm!("cli", options(nostack, nomem));
    }
}

/// Invalidate the TLB entry for the given virtual address on the current CPU.
///
/// Does not broadcast to other CPUs — the caller must issue IPIs for
/// shootdowns in SMP kernels.
///
/// # Safety
/// Passing an incorrect `vaddr` can corrupt TLB state and cause subsequent
/// faults or silent memory corruption. Must be called at CPL=0.
#[inline(always)]
pub unsafe fn invlpg(vaddr: u64) {
    unsafe {
        core::arch::asm!(
            "invlpg [{v}]",
            v = in(reg) vaddr,
            options(nostack, nomem, preserves_flags)
        );
    }
}

/// Flush the cache line containing the byte at `addr` to memory.
///
/// The line is invalidated in all caches. Subsequent accesses will
/// reload from DRAM. Used for cache maintenance and persistence operations.
///
/// # Safety
/// `addr` must be a valid address. Flushing arbitrary addresses can cause
/// significant performance regressions. Requires CPL=0 or CR4.KL cleared.
#[inline(always)]
pub unsafe fn clflush(addr: u64) {
    unsafe {
        core::arch::asm!(
            "clflush [{a}]",
            a = in(reg) addr,
            options(nostack, preserves_flags)
        );
    }
}

/// Flush and invalidate the cache line at `addr` (weakly ordered).
///
/// Like `clflush` but allows out-of-order completion relative to other
/// `CLFLUSHOPT` operations. Requires the CLFLUSHOPT CPU feature.
///
/// # Safety
/// Same as [`clflush`]. The CPU must support CLFLUSHOPT (check via
/// `cpuid::has_clflushopt()` when that check is available). An `sfence`
/// before the subsequent stores is needed for ordering guarantees.
#[inline(always)]
pub unsafe fn clflushopt(addr: u64) {
    unsafe {
        core::arch::asm!(
            "clflushopt [{a}]",
            a = in(reg) addr,
            options(nostack, preserves_flags)
        );
    }
}

/// Write-back and retain cache ownership for the line at `addr`.
///
/// Unlike `clflush`/`clflushopt`, the line stays valid in the cache after
/// the writeback. Useful for persistent memory (NVDIMM) flush-to-medium.
/// Requires the CLWB CPU feature.
///
/// # Safety
/// Same as [`clflushopt`]. CPU must support CLWB.
#[inline(always)]
pub unsafe fn clwb(addr: u64) {
    unsafe {
        core::arch::asm!(
            "clwb [{a}]",
            a = in(reg) addr,
            options(nostack, preserves_flags)
        );
    }
}

/// Write back all modified cache lines and invalidate all caches.
///
/// Serializing instruction. Very expensive — use only for power management
/// or before entering SMM/hardware sleep states.
///
/// # Safety
/// Must be called at CPL=0. Introduces significant latency.
#[inline(always)]
pub unsafe fn wbinvd() {
    unsafe {
        core::arch::asm!("wbinvd", options(nostack, nomem, preserves_flags));
    }
}

// All functions require x86_64 hardware to execute. Tests verify function
// signatures compile correctly without emitting any asm.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_instruction_signatures() {
        let _pause:    fn()          = pause;
        let _mfence:   fn()          = mfence;
        let _lfence:   fn()          = lfence;
        let _sfence:   fn()          = sfence;
        let _hlt:      unsafe fn()   = hlt;
        let _sti:      unsafe fn()   = sti;
        let _cli:      unsafe fn()   = cli;
        let _invlpg:   unsafe fn(u64) = invlpg;
        let _clflush:  unsafe fn(u64) = clflush;
        let _clflushopt: unsafe fn(u64) = clflushopt;
        let _clwb:     unsafe fn(u64) = clwb;
        let _wbinvd:   unsafe fn()   = wbinvd;
    }
}
