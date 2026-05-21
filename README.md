# `bitwise` — Kernel Bitwise Utility Crate

A `no_std`, zero-cost collection of bitwise primitives for x86_64 kernel development in Rust. Every public function is `#[inline(always)]`. There are no heap allocations, no `unsafe` outside of hardware access functions, and no dependencies outside `core`.

---

## Crate Structure

```
bitwise/
├── src/
│   ├── kernel_bitwise/
│   │   ├── bitwise.rs       ← crate root (lib.rs equivalent)
│   │   ├── align.rs         ← address alignment, frame/page arithmetic
│   │   └── endian.rs        ← byte-order conversion and unaligned reads
│   ├── arithmetic/
│   │   ├── bits.rs          ← set/clear/toggle/test, LSB/MSB, popcount
│   │   └── flags.rs         ← FlagRegister type; RFLAGS/CR0/CR4/EFER constants
│   ├── helpers/
│   │   ├── cache.rs         ← cache-line alignment and span arithmetic
│   │   └── paging.rs        ← page table entry encoding and vaddr indexing
│   └── insertion_extraction/
│       ├── dma.rs           ← DMA alignment and scatter-gather descriptors
│       ├── mmio.rs          ← volatile MMIO read/write, MmioBlock
│       └── pio.rs           ← x86 IN/OUT port I/O instructions
├── Cargo.toml
└── README.md
```

### Module Dependency Graph

```
align
 ├── bits
 │    └── flags
 ├── cache
 ├── paging  ←── flags
 ├── dma     ←── align, cache
 ├── mmio    ←── bits
 ├── pio
 └── endian
```

`align` has no dependencies within the crate and is the correct starting point for understanding everything else.

---

## Quick Start

```toml
# Cargo.toml
[dependencies]
bitwise = { path = "../bitwise" }
```

```rust
// Import everything commonly needed in one shot
use bitwise::prelude::*;

// Or import specific modules
use bitwise::{align, bits, mmio, paging};
```

---

## Module Reference

### `align` — Address Alignment and Frame Arithmetic

The bedrock module. All alignment functions take an explicit `align` or `frame_size` parameter — there are no hard-coded page sizes. This means the same function works for 4 KiB standard pages, 2 MiB huge pages, 1 GiB gigantic pages, and arbitrary DMA granularities.

**Alignment requirement**: all `align` and `frame_size` arguments must be powers of two. Debug builds panic on violation; release builds have undefined behavior (matching Linux kernel conventions).

```rust
use bitwise::prelude::*;

// Round a bump-allocator pointer to the next 4 KiB page boundary
let next = align_up(bump_ptr, PAGE_SIZE_4K);

// How many 2 MiB pages does a 7 MiB buffer require?
let count = frames_needed(7 * 1024 * 1024, PAGE_SIZE_2M); // = 4

// Extract the page frame number from a physical address
let pfn = frame_number(0xDEAD_B000, PAGE_SIZE_4K); // = 0xDEAD_B

// Check a physical address is page-aligned before mapping it
assert!(is_aligned(phys, PAGE_SIZE_4K), "frame base not aligned");
```

| Function | Description |
|---|---|
| `align_down(addr, align)` | Round down to nearest multiple of `align` |
| `align_up(addr, align)` | Round up to nearest multiple of `align` |
| `is_aligned(addr, align)` | Test alignment — `O(1)`, single AND |
| `align_offset(addr, align)` | Bytes between `addr` and next boundary |
| `frame_number(phys, frame_size)` | Physical address → frame index |
| `frame_base(frame_num, frame_size)` | Frame index → base physical address |
| `frame_offset(addr, frame_size)` | Byte offset within the containing frame |
| `frames_needed(bytes, frame_size)` | Minimum frames to cover N bytes |
| `fits_in_frame(addr, size, frame_size)` | True if range does not cross a frame boundary |

---

### `bits` — General-Purpose Bit Manipulation

Operates on `u64`. All functions are const-capable and map to single instructions or short sequences on x86_64 with BMI1 enabled (`-C target-feature=+bmi1`).

```rust
use bitwise::prelude::*;

// Isolate the lowest set bit — compiles to BLSI with BMI1
let lsb = lowest_set_bit(bitmap);

// Strip the lowest set bit in a loop over set bits — compiles to BLSR
while bitmap != 0 {
    let bit = lowest_set_bit(bitmap);
    // ... process bit ...
    bitmap = clear_lowest_set_bit(bitmap);
}

// Extract a multi-bit field (right-justified result)
// e.g. bits 13:12 of an x86 segment selector = CPL
let cpl = bit_field_get(selector, 13, 12);

// Insert a value into bits 5:3, leaving the rest unchanged
let updated = bit_field_set(reg, 5, 3, 0b110);
```

| Function | Compiles to (BMI1) | Description |
|---|---|---|
| `bit_set(v, n)` | `BTS` / OR | Set bit n |
| `bit_clear(v, n)` | `BTR` / AND NOT | Clear bit n |
| `bit_toggle(v, n)` | `BTC` / XOR | Toggle bit n |
| `bit_test(v, n)` | `BT` / shift+AND | Test bit n |
| `bit_field_get(v, hi, lo)` | shift + AND | Extract right-justified field |
| `bit_field_set(v, hi, lo, f)` | AND NOT + OR | Insert right-justified field |
| `bit_mask(hi, lo)` | shift | Build contiguous mask |
| `lowest_set_bit(n)` | `BLSI` | Isolate LSB (= `n & -n`) |
| `clear_lowest_set_bit(n)` | `BLSR` | Clear LSB (= `n & (n-1)`) |
| `lowest_set_bit_index(n)` | `TZCNT` | Index of LSB, `Option<u32>` |
| `highest_set_bit_index(n)` | `LZCNT` | Index of MSB = ⌊log₂n⌋, `Option<u32>` |
| `popcount(n)` | `POPCNT` | Count of set bits |
| `is_power_of_two(n)` | AND | Single-bit test |
| `next_power_of_two(n)` | `LZCNT` + shift | Round up to next power of two |

---

### `flags` — Typed Flag Register Interface

`FlagRegister` is a `#[repr(transparent)]` newtype over `u64`. It provides named methods for set/clear/toggle/test operations on flag registers without exposing bare bitwise logic at call sites.

Named constants for every standard x86_64 control register are provided as submodules.

```rust
use bitwise::prelude::*;

// Enable paging and kernel write-protect in CR0
let cr0 = FlagRegister::from_raw(read_cr0());
unsafe { write_cr0(cr0.set(cr0::PG | cr0::WP).raw()); }

// Confirm we are in long mode before proceeding
let efer = FlagRegister::from_raw(rdmsr(0xC000_0080));
assert!(efer.has(efer::LMA));

// Extract the IOPL field (bits 13:12) from RFLAGS
let iopl = (rflags_value & rflags::IOPL) >> 12;
```

**Constant submodules**: `rflags`, `cr0`, `cr4`, `efer`.

---

### `cache` — Cache-Line Arithmetic

False sharing and cache-line splits are among the most insidious performance bugs in kernel code. These functions make the arithmetic explicit.

```rust
use bitwise::prelude::*;

// Pad a per-CPU counter to avoid false sharing with adjacent fields
let padded_size = cache_align_size(core::mem::size_of::<Counter>() as u64);

// Detect a cache-line split before it becomes a benchmark mystery
debug_assert!(
    is_cache_line_contained(field_addr, field_size),
    "hot field crosses a cache line boundary"
);

// How many cache lines does a ring buffer entry touch?
let lines = cache_lines_spanned(entry_addr, entry_size);
```

| Function | Description |
|---|---|
| `cache_align_size(n)` | Round n up to a cache-line multiple |
| `cache_line_of(addr)` | Cache line index containing addr |
| `cache_line_offset(addr)` | Byte offset within the cache line (0..63) |
| `cache_line_start(addr)` | Align addr down to the start of its cache line |
| `cache_lines_spanned(addr, size)` | Number of cache lines touched by a range |
| `is_cache_line_contained(addr, size)` | True if the range does not cross a cache-line boundary |

Constant: `CACHE_LINE_BYTES = 64` (x86_64 Intel/AMD standard).

---

### `paging` — Page Table Entry Encoding

Parameterized by `frame_size` so the same functions work for all three x86_64 page granularities. All PTE flag constants live in the `pte_flags` submodule.

```rust
use bitwise::prelude::*;

// Build a present, writable, non-executable 4 KiB PTE
let entry = pte_encode(
    phys_frame_base,
    PAGE_SIZE_4K,
    pte_flags::PRESENT | pte_flags::WRITABLE | pte_flags::NO_EXECUTE,
);

// Extract the physical address from an existing entry
let phys = pte_phys_addr(entry, PAGE_SIZE_4K);

// Get the PML4 index for a virtual address (level 4)
let pml4_idx = vaddr_pt_index(vaddr, 4);  // bits 47:39
let pdpt_idx = vaddr_pt_index(vaddr, 3);  // bits 38:30
let pd_idx   = vaddr_pt_index(vaddr, 2);  // bits 29:21
let pt_idx   = vaddr_pt_index(vaddr, 1);  // bits 20:12

// Mark a page as accessed without disturbing other flags
let updated = pte_set_flags(entry, pte_flags::ACCESSED);
```

**`pte_flags` constants**: `PRESENT`, `WRITABLE`, `USER`, `WRITE_THROUGH`, `CACHE_DISABLE`, `ACCESSED`, `DIRTY`, `HUGE_PAGE`, `GLOBAL`, `NO_EXECUTE`, `AVAIL_MASK`.

---

### `mmio` — Memory-Mapped I/O

All accesses use `core::ptr::read_volatile` / `write_volatile`. The compiler cannot cache, reorder, or eliminate these accesses — this is a hard correctness requirement for device registers, not a performance suggestion.

```rust
use bitwise::prelude::*;

// Direct address access
unsafe {
    mmio_write32(UART_BASE + 0x04, 0x0000_0003); // configure
    let status = mmio_read32(UART_BASE + 0x18);
}

// MmioBlock for register blocks (cleaner for multi-register devices)
let uart = MmioBlock::new(0xFEDC_0000);
unsafe {
    uart.write32(0x00, 0x0001);              // enable
    uart.set_bits32(0x04, 0b0000_0011);     // configure baud
    let status = uart.read32(0x18);
}

// Read-modify-write a field in a control register
unsafe {
    mmio_update_field32(
        GIC_BASE + GICD_CTLR,
        0b11,        // mask: bits 1:0
        0b01,        // value: enable group 0 only
    );
}
```

**Do not** use plain pointer dereferences for MMIO — the compiler is legally permitted to eliminate, merge, or reorder them.

---

### `dma` — DMA Buffer Management

DMA imposes alignment requirements that differ from normal memory allocation. Violating them causes silent data corruption on some hardware and IOMMU faults on others.

```rust
use bitwise::prelude::*;

// Verify a physical buffer is usable by a 32-bit DMA device
if !fits_in_32bit_dma(phys_base, buffer_len) {
    return Err(AllocError::Above32Bit);
}

// Align a buffer start and measure wasted padding
let (aligned_base, padding) = dma_align_buffer(raw_phys, CACHE_LINE_SIZE);

// Build a scatter-gather list for a buffer that may cross segment boundaries
let mut descriptors = [(0u64, 0u64); 16];
let count = dma_build_sg(
    phys_base,
    transfer_len,
    DMA_PAGE_SIZE,   // segment boundary: no segment may cross a page boundary
    &mut descriptors,
);
// submit descriptors[..count] to the device's descriptor ring
```

| Function | Description |
|---|---|
| `dma_is_aligned(phys, align)` | Test DMA alignment |
| `dma_align_buffer(phys, align)` | Returns `(aligned_base, padding_before)` |
| `dma_round_length(len, granularity)` | Round transfer length up to device granularity |
| `dma_build_sg(base, len, boundary, out)` | Build scatter-gather descriptor list |
| `fits_in_32bit_dma(phys, len)` | Check the range fits below 4 GiB |

Constants: `CACHE_LINE_SIZE = 64`, `DMA_PAGE_SIZE = 0x1000`.

---

### `pio` — Port I/O

The `IN` and `OUT` x86 instructions access the separate I/O port address space. Legacy devices (8259 PIC, 8254 PIT, PS/2, serial UART at 0x3F8) require this. All functions are `unsafe` — the caller must ensure CPL=0 (or appropriate IOPL/IOPB).

```rust
use bitwise::prelude::*;

// Read PIC IMR (interrupt mask register)
let imr = unsafe { inb(0x21) };

// Mask IRQ 3 on the primary PIC
unsafe { pio_set_bits8(0x21, 1 << 3); }

// Write to legacy serial port divisor
unsafe {
    outb(0x3FB, 0x80);           // set DLAB
    outw(0x3F8, 12u16);          // divisor low+high: 9600 baud at 1.8432 MHz
    outb(0x3FB, 0x03);           // clear DLAB, 8N1
}
```

| Function | Instruction | Description |
|---|---|---|
| `inb(port)` | `IN AL, DX` | Read byte from port |
| `inw(port)` | `IN AX, DX` | Read word from port |
| `inl(port)` | `IN EAX, DX` | Read dword from port |
| `outb(port, v)` | `OUT DX, AL` | Write byte to port |
| `outw(port, v)` | `OUT DX, AX` | Write word to port |
| `outl(port, v)` | `OUT DX, EAX` | Write dword to port |
| `pio_set_bits8(port, mask)` | RMW | Set bits in 8-bit port register |
| `pio_clear_bits8(port, mask)` | RMW | Clear bits in 8-bit port register |

---

### `endian` — Byte-Order Conversion

x86_64 is little-endian. Hardware registers specified by external standards (PCI configuration space, PCIe ECAM, network devices, firmware tables) are often big-endian. Reading them without conversion produces garbage.

```rust
use bitwise::prelude::*;

// Read a big-endian u32 from a PCI config header field
let vendor_device = be32_to_cpu(mmio_read32(pci_base + 0x00));

// Parse a big-endian field from a raw byte buffer (safe, no pointer cast)
let eth_type = read_be32(&packet[12..16]);

// Write a big-endian length field into a protocol header
write_be32(&mut header[4..8], payload_len as u32);
```

---

## Crate-Level Constants

```rust
bitwise::PAGE_SIZE_4K          // 0x0000_1000 — standard 4 KiB page
bitwise::PAGE_SIZE_2M          // 0x0020_0000 — 2 MiB huge page
bitwise::PAGE_SIZE_1G          // 0x4000_0000 — 1 GiB gigantic page
bitwise::PHYS_ADDR_BITS        // 52 — valid physical address bits on x86_64
bitwise::PHYS_ADDR_MASK        // mask for bits 51:0
bitwise::CANONICAL_ADDR_MASK_48 // mask for bits 47:0 (4-level paging)
```

---

## Design Decisions

**`const fn` everywhere possible.** Alignment arithmetic, bit masks, and PTE flags are frequently needed at compile time to initialize static data structures. Every function that does not require runtime state is `const`.

**Explicit `frame_size` parameters, never hard-coded.** Kernels that support huge pages must run the same allocation logic against multiple page sizes. Hard-coding 4096 creates a fork in the code; parameterization does not.

**Volatile-only MMIO, no exceptions.** Plain pointer dereferences of device registers are a correctness bug, not a style issue. The compiler's alias analysis does not understand that a memory location might change between two reads with no intervening write in program text.

**No implicit unsafety propagation.** `#![deny(unsafe_op_in_unsafe_fn)]` is set at the crate root. Every `unsafe` block inside an `unsafe fn` must independently justify its unsafety. This forces the author to be explicit about which operation is actually unsafe rather than laundering it through a function signature.

**BMI1 is not required, but benefits from it.** The code compiles and runs correctly without BMI1. Pass `-C target-feature=+bmi1,+bmi2` to `rustc` (or set `RUSTFLAGS`) if your target CPU supports it (Haswell and later on Intel; Piledriver and later on AMD). LLVM will automatically lower `n & n.wrapping_neg()` to `BLSI`, `n & (n-1)` to `BLSR`, and similar.

---

## Cargo.toml

```toml
[package]
name    = "bitwise"
version = "0.1.0"
edition = "2021"

[dependencies]
# none — core only

[profile.dev]
overflow-checks = true   # catch wrapping arithmetic bugs early

[profile.release]
overflow-checks = false  # release: wrapping is intentional in address arithmetic
opt-level       = 3
lto             = "thin"
```

---

## References

- Intel® 64 and IA-32 Architectures Software Developer's Manual, Volume 3A — Chapters 4 (Paging), 5 (Protection), 10 (APIC)
- Intel® 64 and IA-32 Architectures Software Developer's Manual, Volume 2 — `IN`, `OUT`, `POPCNT`, `LZCNT`, `TZCNT`, `BLSI`, `BLSR`, `BSWAP`
- PCI Local Bus Specification 3.0 — DMA addressing conventions
- Rust Reference, §Inline Assembly — `core::arch::asm!` constraints
- `core::ptr::read_volatile` / `write_volatile` — Rust standard library documentation
