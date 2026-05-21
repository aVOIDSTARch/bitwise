#![cfg_attr(not(test), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]   // every unsafe block must justify itself
#![warn(missing_docs)]
#![warn(clippy::missing_safety_doc)]

//! # `bitwise` — Kernel Bitwise Utility Crate
//!
//! A zero-cost, `no_std` collection of bitwise primitives for x86_64 kernel
//! development. Every public function is `#[inline(always)]` and `const`-capable
//! where the operation permits. There are no heap allocations and no dependencies
//! outside `core`.
//!
//! ## Crate Layout
//!
//! ```text
//! bitwise/
//! ├── arithmetic/
//! │   ├── bits.rs          — set/clear/toggle/test, popcount, LSB/MSB utilities
//! │   └── flags.rs         — FlagRegister type; RFLAGS, CR0, CR4, EFER constants
//! ├── cpu/
//! │   ├── cpuid.rs         — CPUID instruction wrapper and feature detection
//! │   ├── instructions.rs  — pause, hlt, cli, sti, mfence, invlpg, clflush
//! │   └── msr.rs           — RDMSR/WRMSR wrappers and MSR address constants
//! ├── helpers/
//! │   ├── bitmap.rs        — BitArray<N> fixed-size bitset for frame allocation
//! │   ├── cache.rs         — cache-line alignment and span arithmetic
//! │   ├── gdt.rs           — GDT/IDT descriptor encoding, SegmentSelector
//! │   └── paging.rs        — page table entry encoding, vaddr index extraction
//! ├── insertion_extraction/
//! │   ├── dma.rs           — DMA buffer alignment and scatter-gather descriptors
//! │   ├── mmio.rs          — volatile MMIO read/write, MmioBlock register view
//! │   └── pio.rs           — x86 IN/OUT port I/O instructions
//! └── kernel_bitwise/
//!     ├── addr.rs          — PhysAddr and VirtAddr typed address wrappers
//!     ├── align.rs         — address alignment, frame/page arithmetic
//!     ├── endian.rs        — big/little endian conversion and unaligned reads
//!     └── bitwise.rs       ← YOU ARE HERE (crate root / lib.rs)
//! ```
//!
//! ## Module Dependency Order
//!
//! ```text
//! align  ←──────────────────────── everything else depends on this
//!   └── addr  (also uses paging)
//!   └── bits
//!         └── flags
//!         └── gdt  (uses bits::bit_field_set)
//!               └── paging, cache
//!                     └── dma, mmio, pio, endian
//! bitmap  ←─────────────────────── standalone, no deps on above
//! cpu/    ←─────────────────────── hardware-only, no deps on above
//!   (msr, cpuid, instructions)
//! ```
//!
//! ## Usage
//!
//! Add to `Cargo.toml`:
//! ```toml
//! [dependencies]
//! bitwise = { path = "../bitwise" }
//! ```
//!
//! Import the prelude for the most commonly needed items:
//! ```rust
//! use bitwise::prelude::*;
//! ```
//!
//! Or import specific modules:
//! ```rust
//! use bitwise::{align, bits, mmio};
//! ```

// ---------------------------------------------------------------------------
// Submodule declarations
//
// The path attributes reflect the actual directory layout on disk.
// Rust's module system resolves `#[path]` relative to the file declaring it.
// ---------------------------------------------------------------------------

// --- kernel_bitwise/ (peer files in the same directory as this root) -------

/// Address alignment and frame/page arithmetic.
///
/// The foundation of the crate. All other modules that deal with physical or
/// virtual addresses depend on the primitives here.
///
/// Key functions: [`align::align_down`], [`align::align_up`],
/// [`align::frame_number`], [`align::frames_needed`].
#[path = "align.rs"]
pub mod align;

/// Byte-order conversion between big-endian, little-endian, and native.
///
/// Covers `u16`, `u32`, and `u64`; includes safe unaligned byte-slice readers
/// for parsing protocol headers and firmware tables without undefined behavior.
#[path = "endian.rs"]
pub mod endian;

/// Typed physical and virtual address wrappers.
///
/// [`addr::PhysAddr`] enforces that bits 63:52 are zero (x86_64 MAXPHYADDR = 52).
/// [`addr::VirtAddr`] enforces x86_64 canonicality: bits 63:48 must be the
/// sign-extension of bit 47.
///
/// Key types: [`addr::PhysAddr`], [`addr::VirtAddr`].
#[path = "addr.rs"]
pub mod addr;

// --- arithmetic/ -----------------------------------------------------------

/// General-purpose bit manipulation on `u64` values.
///
/// Includes set/clear/toggle/test for single bits, field insertion and
/// extraction for contiguous bit ranges, and numeric properties such as
/// power-of-two testing, next-power-of-two, LSB isolation, and population
/// count.
///
/// Key functions: [`bits::bit_set`], [`bits::bit_field_get`],
/// [`bits::bit_field_set`], [`bits::lowest_set_bit`].
#[path = "../arithmetic/bits.rs"]
pub mod bits;

/// Typed flag register interface and x86_64 hardware register bit definitions.
///
/// Provides the [`flags::FlagRegister`] newtype wrapper and named bit
/// constants for RFLAGS, CR0, CR4, and EFER.  Using named constants instead
/// of bare magic numbers makes architecture-sensitive code auditable and
/// prevents silent bit-position errors.
///
/// Key type: [`flags::FlagRegister`].
/// Key modules: [`flags::rflags`], [`flags::cr0`], [`flags::cr4`], [`flags::efer`].
#[path = "../arithmetic/flags.rs"]
pub mod flags;

// --- helpers/ --------------------------------------------------------------

/// Cache-line alignment and span arithmetic.
///
/// Functions for rounding sizes to cache-line boundaries, detecting cache-line
/// splits, and counting how many cache lines a range touches. Essential for
/// avoiding false sharing in per-CPU data structures and lock-free algorithms.
///
/// Key functions: [`cache::cache_align_size`],
/// [`cache::is_cache_line_contained`], [`cache::cache_lines_spanned`].
#[path = "../helpers/cache.rs"]
pub mod cache;

/// Page table entry encoding, decoding, and virtual address index extraction.
///
/// Parameterized by `frame_size` so the same functions cover 4 KiB, 2 MiB,
/// and 1 GiB mappings. Includes named constants for all standard x86_64 PTE
/// flag bits.
///
/// Key functions: [`paging::pte_encode`], [`paging::pte_phys_addr`],
/// [`paging::vaddr_pt_index`].
/// Key module: [`paging::pte_flags`].
#[path = "../helpers/paging.rs"]
pub mod paging;

/// x86_64 descriptor table encoding: GDT segment descriptors, IDT gate
/// descriptors, and TSS system descriptors.
///
/// All encoding functions are `const` and use [`bits::bit_field_set`] for
/// readable, auditable field construction.
///
/// Key types: [`gdt::SegmentSelector`], [`gdt::SegmentDescriptor`],
/// [`gdt::TssDescriptor`], [`gdt::GateDescriptor`].
#[path = "../helpers/gdt.rs"]
pub mod gdt;

/// Fixed-size bitset backed by a `[u64; N]` array.
///
/// [`bitmap::BitArray<N>`] requires no heap allocation and is designed as the
/// backing data structure for a physical memory frame allocator.
///
/// Key type: [`bitmap::BitArray`].
#[path = "../helpers/bitmap.rs"]
pub mod bitmap;

// --- insertion_extraction/ -------------------------------------------------

/// DMA buffer alignment, 32-bit window checking, and scatter-gather list
/// construction.
///
/// Handles the constraints imposed by devices that cannot address above 4 GiB,
/// IOMMU page alignment requirements, and the segment-boundary splits that
/// scatter-gather engines require.
///
/// Key functions: [`dma::dma_align_buffer`], [`dma::dma_build_sg`],
/// [`dma::fits_in_32bit_dma`].
#[path = "../insertion_extraction/dma.rs"]
pub mod dma;

/// Volatile memory-mapped I/O reads and writes.
///
/// All accesses go through `core::ptr::read_volatile` /
/// `core::ptr::write_volatile` — the compiler is explicitly forbidden from
/// caching, reordering, or eliminating them.  Also provides
/// [`mmio::MmioBlock`] for register blocks that follow a `base + offset`
/// addressing scheme.
///
/// Key functions: [`mmio::mmio_read32`], [`mmio::mmio_write32`],
/// [`mmio::mmio_set_bits32`], [`mmio::mmio_update_field32`].
/// Key type: [`mmio::MmioBlock`].
#[path = "../insertion_extraction/mmio.rs"]
pub mod mmio;

/// x86/x86_64 port I/O via `IN` and `OUT` instructions.
///
/// Provides byte, word, and dword variants for both reads and writes, plus
/// read-modify-write helpers for the common case of setting or clearing bits
/// in a legacy device's control register.
///
/// All functions are `unsafe` — executing `IN`/`OUT` at an incorrect port
/// address can crash the system or corrupt device state.
///
/// Key functions: [`pio::inb`], [`pio::outb`], [`pio::pio_set_bits8`].
#[cfg(target_arch = "x86_64")]
#[path = "../insertion_extraction/pio.rs"]
pub mod pio;

// --- cpu/ (x86_64 only — require hardware instructions) --------------------

/// x86_64 Model-Specific Register access via `RDMSR` / `WRMSR`.
///
/// Use [`msr::rdmsr`] and [`msr::wrmsr`] with the constants in [`msr::msr_num`].
/// For bit-level definitions of EFER see [`flags::efer`].
///
/// All functions are `unsafe` — reading/writing MSRs can crash the system.
#[cfg(target_arch = "x86_64")]
#[path = "../cpu/msr.rs"]
pub mod msr;

/// `CPUID` instruction wrapper and CPU feature detection.
///
/// Use [`cpuid::cpuid`] / [`cpuid::cpuid_leaf`] for raw leaf access, or the
/// `has_*` functions (e.g. [`cpuid::has_nx`], [`cpuid::has_apic`]) for guarded
/// feature checks.
///
/// Key type: [`cpuid::CpuidResult`].
#[cfg(target_arch = "x86_64")]
#[path = "../cpu/cpuid.rs"]
pub mod cpuid;

/// Thin wrappers around x86_64 CPU instructions.
///
/// Safe: [`instructions::pause`], [`instructions::mfence`],
/// [`instructions::lfence`], [`instructions::sfence`].
///
/// Unsafe (change global CPU state): [`instructions::hlt`],
/// [`instructions::cli`], [`instructions::sti`], [`instructions::invlpg`],
/// [`instructions::wbinvd`].
#[cfg(target_arch = "x86_64")]
#[path = "../cpu/instructions.rs"]
pub mod instructions;

// ---------------------------------------------------------------------------
// Prelude
//
// Re-exports the items you will reach for on virtually every use site.
// Import with `use bitwise::prelude::*`.
// ---------------------------------------------------------------------------

/// Commonly needed items, importable in bulk.
///
/// ```rust
/// use bitwise::prelude::*;
/// ```
pub mod prelude {
    // Alignment — the absolute baseline
    pub use crate::align::{
        align_down,
        align_up,
        align_offset,
        is_aligned,
        frame_number,
        frame_base,
        frame_offset,
        frames_needed,
        fits_in_frame,
    };

    // Bit manipulation
    pub use crate::bits::{
        bit_set,
        bit_clear,
        bit_toggle,
        bit_test,
        bit_field_get,
        bit_field_set,
        bit_mask,
        is_power_of_two,
        next_power_of_two,
        lowest_set_bit,
        clear_lowest_set_bit,
        lowest_set_bit_index,
        highest_set_bit_index,
        popcount,
        even_parity,
    };

    // Flag registers
    pub use crate::flags::FlagRegister;
    pub use crate::flags::{rflags, cr0, cr4, efer};

    // Cache
    pub use crate::cache::{
        cache_align_size,
        cache_line_of,
        cache_line_offset,
        cache_line_start,
        cache_lines_spanned,
        is_cache_line_contained,
        CACHE_LINE_BYTES,
    };

    // Paging
    pub use crate::paging::{
        pte_encode,
        pte_phys_addr,
        pte_is_present,
        pte_is_huge,
        pte_set_flags,
        pte_clear_flags,
        pte_avail_bits,
        pte_set_avail_bits,
        vaddr_pt_index,
        vaddr_page_offset,
        pte_flags,
    };

    // MMIO
    pub use crate::mmio::{
        mmio_read8,
        mmio_read16,
        mmio_read32,
        mmio_read64,
        mmio_write8,
        mmio_write16,
        mmio_write32,
        mmio_write64,
        mmio_set_bits32,
        mmio_clear_bits32,
        mmio_update_field32,
        mmio_set_bits64,
        mmio_clear_bits64,
        MmioBlock,
    };

    // DMA
    pub use crate::dma::{
        dma_is_aligned,
        dma_align_buffer,
        dma_round_length,
        dma_build_sg,
        fits_in_32bit_dma,
        CACHE_LINE_SIZE,
        DMA_PAGE_SIZE,
    };

    // Port I/O (x86_64 only — requires IN/OUT instructions)
    #[cfg(target_arch = "x86_64")]
    pub use crate::pio::{
        inb, inw, inl,
        outb, outw, outl,
        pio_set_bits8,
        pio_clear_bits8,
    };

    // Endian
    pub use crate::endian::{
        be16_to_cpu, be32_to_cpu, be64_to_cpu,
        cpu_to_be16, cpu_to_be32, cpu_to_be64,
        read_be32, write_be32, read_le32,
    };

    // Typed addresses
    pub use crate::addr::{PhysAddr, VirtAddr};

    // GDT/IDT descriptors
    pub use crate::gdt::SegmentSelector;

    // Bitmap
    pub use crate::bitmap::BitArray;

    // CPU instructions (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    pub use crate::instructions::{pause, hlt, cli, sti, mfence, lfence, sfence};

    // MSR (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    pub use crate::msr::{rdmsr, wrmsr};

    // CPUID (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    pub use crate::cpuid::{CpuidResult, cpuid_leaf, max_basic_leaf};
}

// ---------------------------------------------------------------------------
// Crate-level constants
// ---------------------------------------------------------------------------

/// Standard 4 KiB page size. The most common frame size on x86_64.
pub const PAGE_SIZE_4K: u64 = 0x0000_1000;

/// 2 MiB huge page size. Requires PSE or PAE+PSE in CR4.
pub const PAGE_SIZE_2M: u64 = 0x0020_0000;

/// 1 GiB gigantic page size. Requires 1GB page support (CPUID leaf 0x8000_0001 EDX bit 26).
pub const PAGE_SIZE_1G: u64 = 0x4000_0000;

/// x86_64 physical address bits in use (52 bits; bits 63:52 reserved).
pub const PHYS_ADDR_BITS: u32 = 52;

/// Mask for valid physical address bits.
pub const PHYS_ADDR_MASK: u64 = (1u64 << PHYS_ADDR_BITS) - 1;

/// x86_64 canonical virtual address mask (48-bit addressing, 4-level paging).
/// Bits 63:48 must be copies of bit 47. Use 57-bit mask for 5-level paging (LA57).
pub const CANONICAL_ADDR_MASK_48: u64 = 0x0000_FFFF_FFFF_FFFF;
