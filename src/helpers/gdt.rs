//! x86_64 descriptor table encoding: GDT segment descriptors, IDT gate
//! descriptors, and TSS system descriptors.
//!
//! All encoding functions are `const` and use [`crate::bits::bit_field_set`]
//! for readable, auditable field construction. No inline assembly — fully
//! testable on any host architecture.

// ---------------------------------------------------------------------------
// Segment Selector
// ---------------------------------------------------------------------------

/// A 16-bit x86_64 segment selector: `index[15:3] | TI[2] | RPL[1:0]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    /// The null selector (GDT index 0, RPL 0).
    pub const NULL: Self = Self(0);

    /// Encode a segment selector.
    ///
    /// - `index`: GDT/LDT descriptor index (0–8191).
    /// - `rpl`: Requested Privilege Level (0–3).
    /// - `ldt`: `true` to select from the LDT, `false` for the GDT.
    #[inline(always)]
    pub const fn new(index: u16, rpl: u8, ldt: bool) -> Self {
        debug_assert!(rpl <= 3, "RPL must be 0-3");
        let raw = (index << 3) | ((ldt as u16) << 2) | (rpl as u16 & 0x3);
        Self(raw)
    }

    /// GDT descriptor index (bits 15:3).
    #[inline(always)]
    pub const fn index(self) -> u16 {
        self.0 >> 3
    }

    /// Requested Privilege Level (bits 1:0).
    #[inline(always)]
    pub const fn rpl(self) -> u8 {
        (self.0 & 0x3) as u8
    }

    /// `true` if this selector refers to the LDT rather than the GDT.
    #[inline(always)]
    pub const fn is_ldt(self) -> bool {
        self.0 & 0x4 != 0
    }
}

// ---------------------------------------------------------------------------
// GDT Segment Descriptor (8 bytes)
// ---------------------------------------------------------------------------

/// An x86_64 8-byte GDT segment descriptor.
///
/// In 64-bit mode only code descriptors are meaningful for execution — data
/// and stack descriptors exist for backward compatibility and TSS purposes.
/// Build standard flat descriptors with [`GdtDescriptor::code64_kernel`],
/// [`GdtDescriptor::code64_user`], and [`GdtDescriptor::data64`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct GdtDescriptor(pub u64);

impl GdtDescriptor {
    /// All-zero (null) descriptor — slot 0 of the GDT.
    pub const NULL: Self = Self(0);

    /// 64-bit code segment for ring 0 (DPL=0, L=1, P=1, flat 4 GiB base/limit).
    ///
    /// Type = 0b1010 (code, readable, non-conforming).
    pub const fn code64_kernel() -> Self {
        Self::code64(0)
    }

    /// 64-bit code segment for ring 3 (DPL=3, L=1, P=1, flat).
    pub const fn code64_user() -> Self {
        Self::code64(3)
    }

    /// 64-bit data/stack segment (DPL=0, writable, flat).
    ///
    /// In 64-bit mode limits and bases are not enforced, but the descriptor
    /// must be present and S=1.
    pub const fn data64() -> Self {
        let mut raw: u64 = 0;
        raw = crate::bits::bit_field_set(raw, 15,  0, 0xFFFF);   // limit low
        raw = crate::bits::bit_field_set(raw, 43, 40, 0b0010);   // type: data, writable
        raw = crate::bits::bit_field_set(raw, 44, 44, 1);        // S=1
        raw = crate::bits::bit_field_set(raw, 47, 47, 1);        // P=1
        raw = crate::bits::bit_field_set(raw, 51, 48, 0xF);      // limit high
        raw = crate::bits::bit_field_set(raw, 55, 55, 1);        // G=1
        Self(raw)
    }

    /// Base address encoded in the descriptor (bits 63:56 + 39:32 + 31:16).
    ///
    /// Returns a `u32` since 64-bit bases require a 16-byte TSS descriptor.
    #[inline(always)]
    pub const fn base(self) -> u32 {
        let low16 = crate::bits::bit_field_get(self.0, 31, 16) as u32;
        let mid8  = crate::bits::bit_field_get(self.0, 39, 32) as u32;
        let high8 = crate::bits::bit_field_get(self.0, 63, 56) as u32;
        low16 | (mid8 << 16) | (high8 << 24)
    }

    /// 20-bit segment limit (bits 51:48 concatenated with bits 15:0).
    #[inline(always)]
    pub const fn limit(self) -> u32 {
        let low16 = crate::bits::bit_field_get(self.0, 15,  0) as u32;
        let high4 = crate::bits::bit_field_get(self.0, 51, 48) as u32;
        low16 | (high4 << 16)
    }

    /// Descriptor Privilege Level (bits 46:45), 0–3.
    #[inline(always)]
    pub const fn dpl(self) -> u8 {
        crate::bits::bit_field_get(self.0, 46, 45) as u8
    }

    /// `true` if the present bit (bit 47) is set.
    #[inline(always)]
    pub const fn is_present(self) -> bool {
        crate::bits::bit_test(self.0, 47)
    }

    /// `true` if this is a long-mode (64-bit) code segment (L bit, bit 53).
    #[inline(always)]
    pub const fn is_long_mode(self) -> bool {
        crate::bits::bit_test(self.0, 53)
    }

    // Internal helper — builds a code64 descriptor for any DPL.
    const fn code64(dpl: u64) -> Self {
        let mut raw: u64 = 0;
        raw = crate::bits::bit_field_set(raw, 15,  0, 0xFFFF);   // limit low
        raw = crate::bits::bit_field_set(raw, 43, 40, 0b1010);   // type: code, readable
        raw = crate::bits::bit_field_set(raw, 44, 44, 1);        // S=1
        raw = crate::bits::bit_field_set(raw, 46, 45, dpl);      // DPL
        raw = crate::bits::bit_field_set(raw, 47, 47, 1);        // P=1
        raw = crate::bits::bit_field_set(raw, 51, 48, 0xF);      // limit high
        raw = crate::bits::bit_field_set(raw, 53, 53, 1);        // L=1 (64-bit)
        raw = crate::bits::bit_field_set(raw, 55, 55, 1);        // G=1
        Self(raw)
    }
}

// ---------------------------------------------------------------------------
// TSS Descriptor (16 bytes = two GDT slots)
// ---------------------------------------------------------------------------

/// A 128-bit system descriptor for a 64-bit Task State Segment.
///
/// Occupies two consecutive 8-byte GDT slots. The type field is set to
/// `0b1001` (64-bit TSS, available).
///
/// Use [`tss_offsets`] for byte offsets into the TSS structure itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TssDescriptor {
    /// Lower 64 bits of the descriptor (placed in the first GDT slot).
    pub low:  u64,
    /// Upper 64 bits of the descriptor (placed in the second GDT slot).
    pub high: u64,
}

impl TssDescriptor {
    /// Encode a TSS descriptor for the given TSS base address and byte limit.
    ///
    /// `limit` is typically `core::mem::size_of::<Tss>() - 1`.
    #[inline(always)]
    pub const fn new(base: u64, limit: u32) -> Self {
        let mut low: u64 = 0;
        low = crate::bits::bit_field_set(low, 15,  0, (limit & 0xFFFF) as u64);  // limit[15:0]
        low = crate::bits::bit_field_set(low, 31, 16, base & 0xFFFF);            // base[15:0]
        low = crate::bits::bit_field_set(low, 39, 32, (base >> 16) & 0xFF);      // base[23:16]
        low = crate::bits::bit_field_set(low, 43, 40, 0b1001);                   // type: 64-bit TSS available
        // S=0 (bit 44): system descriptor — already 0
        // DPL=0 (bits 46:45): already 0
        low = crate::bits::bit_field_set(low, 47, 47, 1);                        // P=1
        low = crate::bits::bit_field_set(low, 51, 48, ((limit >> 16) & 0xF) as u64); // limit[19:16]
        low = crate::bits::bit_field_set(low, 63, 56, (base >> 24) & 0xFF);      // base[31:24]

        // High 64 bits: base[63:32] in bits 31:0; bits 63:32 reserved (0).
        let high = base >> 32;

        Self { low, high }
    }

    /// Reconstruct the full 64-bit TSS base address from the descriptor.
    #[inline(always)]
    pub const fn base(self) -> u64 {
        let low16  = crate::bits::bit_field_get(self.low, 31, 16);
        let mid8   = crate::bits::bit_field_get(self.low, 39, 32);
        let high8  = crate::bits::bit_field_get(self.low, 63, 56);
        let high32 = self.high & 0xFFFF_FFFF;
        low16 | (mid8 << 16) | (high8 << 24) | (high32 << 32)
    }
}

// ---------------------------------------------------------------------------
// IDT Gate Descriptor (16 bytes)
// ---------------------------------------------------------------------------

/// A 128-bit x86_64 IDT gate descriptor (interrupt or trap gate).
///
/// Use [`IdtGate::interrupt_gate`] for exception/IRQ handlers (IF cleared on
/// entry) and [`IdtGate::trap_gate`] for handlers that must run with interrupts
/// enabled (IF not cleared).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdtGate {
    /// Lower 64 bits.
    pub low:  u64,
    /// Upper 64 bits.
    pub high: u64,
}

impl IdtGate {
    /// A non-present gate — treated as "this vector is not installed" by hardware.
    pub const MISSING: Self = Self { low: 0, high: 0 };

    /// Build an interrupt gate (gate type `0xE`, clears RFLAGS.IF on entry).
    ///
    /// - `handler`: 64-bit handler virtual address.
    /// - `cs`: Code segment selector (typically the kernel CS).
    /// - `dpl`: Descriptor Privilege Level — callers at CPL > DPL raise `#GP`.
    /// - `ist`: Interrupt Stack Table index (0 = disabled, 1–7 = IST entry).
    ///
    /// # Panics (debug)
    /// Panics if `ist > 7` or `dpl > 3`.
    #[inline(always)]
    pub const fn interrupt_gate(handler: u64, cs: SegmentSelector, dpl: u8, ist: u8) -> Self {
        Self::gate(handler, cs, dpl, ist, 0xE)
    }

    /// Build a trap gate (gate type `0xF`, does NOT clear RFLAGS.IF on entry).
    ///
    /// Use for handlers that run with interrupts enabled (e.g., software
    /// exceptions where re-entrancy is safe).
    ///
    /// # Panics (debug)
    /// Same as [`interrupt_gate`].
    #[inline(always)]
    pub const fn trap_gate(handler: u64, cs: SegmentSelector, dpl: u8, ist: u8) -> Self {
        Self::gate(handler, cs, dpl, ist, 0xF)
    }

    /// Reconstruct the 64-bit handler address from the gate descriptor.
    #[inline(always)]
    pub const fn handler_addr(self) -> u64 {
        let low16  = crate::bits::bit_field_get(self.low, 15, 0);
        let mid16  = crate::bits::bit_field_get(self.low, 63, 48);
        let high32 = self.high & 0xFFFF_FFFF;
        low16 | (mid16 << 16) | (high32 << 32)
    }

    /// `true` if the present bit (bit 47 of the low word) is set.
    #[inline(always)]
    pub const fn is_present(self) -> bool {
        crate::bits::bit_test(self.low, 47)
    }

    /// Gate type field (bits 43:40 of the low word).
    ///
    /// `0xE` = interrupt gate, `0xF` = trap gate.
    #[inline(always)]
    pub const fn gate_type(self) -> u8 {
        crate::bits::bit_field_get(self.low, 43, 40) as u8
    }

    // Internal shared constructor.
    const fn gate(handler: u64, cs: SegmentSelector, dpl: u8, ist: u8, gate_type: u64) -> Self {
        debug_assert!(ist <= 7,  "IST index must be 0-7");
        debug_assert!(dpl <= 3,  "DPL must be 0-3");
        let mut low: u64 = 0;
        low = crate::bits::bit_field_set(low, 15,  0, handler & 0xFFFF);          // handler[15:0]
        low = crate::bits::bit_field_set(low, 31, 16, cs.0 as u64);               // segment selector
        low = crate::bits::bit_field_set(low, 34, 32, ist as u64);                // IST
        low = crate::bits::bit_field_set(low, 43, 40, gate_type);                 // gate type
        low = crate::bits::bit_field_set(low, 46, 45, dpl as u64);               // DPL
        low = crate::bits::bit_field_set(low, 47, 47, 1);                         // P=1
        low = crate::bits::bit_field_set(low, 63, 48, (handler >> 16) & 0xFFFF); // handler[31:16]
        let high = handler >> 32;                                                  // handler[63:32]
        Self { low, high }
    }
}

// ---------------------------------------------------------------------------
// TSS structure byte offsets (Intel Vol. 3A §7.7, Table 7-11)
// ---------------------------------------------------------------------------

/// Byte offsets within the x86_64 Task State Segment structure.
///
/// The kernel is responsible for defining the actual TSS type and placing it
/// in memory; this module provides only the offsets for manual field access
/// when building a TSS without a struct definition.
pub mod tss_offsets {
    /// Offset of RSP0 (ring-0 stack pointer) within the TSS.
    pub const RSP0:      usize = 4;
    /// Offset of RSP1 (ring-1 stack pointer) within the TSS.
    pub const RSP1:      usize = 12;
    /// Offset of RSP2 (ring-2 stack pointer) within the TSS.
    pub const RSP2:      usize = 20;
    /// Interrupt Stack Table entry 1 (IST1).
    pub const IST1:      usize = 36;
    /// IST2.
    pub const IST2:      usize = 44;
    /// IST3.
    pub const IST3:      usize = 52;
    /// IST4.
    pub const IST4:      usize = 60;
    /// IST5.
    pub const IST5:      usize = 68;
    /// IST6.
    pub const IST6:      usize = 76;
    /// IST7.
    pub const IST7:      usize = 84;
    /// Offset of the 16-bit I/O Permission Bitmap base address field.
    pub const IOPB_BASE: usize = 102;
    /// Minimum TSS size in bytes (no IOPB, IOPB base points past end).
    pub const SIZE:      usize = 104;
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SegmentSelector ---

    #[test]
    fn selector_null() {
        assert_eq!(SegmentSelector::NULL.0, 0);
        assert_eq!(SegmentSelector::NULL.index(), 0);
        assert_eq!(SegmentSelector::NULL.rpl(), 0);
        assert!(!SegmentSelector::NULL.is_ldt());
    }

    #[test]
    fn selector_roundtrip() {
        let s = SegmentSelector::new(5, 0, false);
        assert_eq!(s.index(), 5);
        assert_eq!(s.rpl(), 0);
        assert!(!s.is_ldt());
    }

    #[test]
    fn selector_user_cs() {
        // Typical user code selector: index=3, RPL=3
        let s = SegmentSelector::new(3, 3, false);
        assert_eq!(s.index(), 3);
        assert_eq!(s.rpl(), 3);
        assert!(!s.is_ldt());
    }

    #[test]
    fn selector_ldt() {
        let s = SegmentSelector::new(1, 0, true);
        assert!(s.is_ldt());
    }

    // --- GdtDescriptor ---

    #[test]
    fn gdt_null_is_zero() {
        assert_eq!(GdtDescriptor::NULL.0, 0);
    }

    #[test]
    fn gdt_code64_kernel_raw_value() {
        // Expected: P=1, DPL=0, S=1, type=0b1010, L=1, G=1, limit=0xFFFFF, base=0
        let raw = GdtDescriptor::code64_kernel().0;
        assert_eq!(raw, 0x00AF_9A00_0000_FFFF);
    }

    #[test]
    fn gdt_code64_user_dpl() {
        let d = GdtDescriptor::code64_user();
        assert_eq!(d.dpl(), 3);
        assert!(d.is_present());
        assert!(d.is_long_mode());
    }

    #[test]
    fn gdt_code64_kernel_fields() {
        let d = GdtDescriptor::code64_kernel();
        assert_eq!(d.base(),    0);
        assert_eq!(d.limit(),   0xFFFFF);
        assert_eq!(d.dpl(),     0);
        assert!(d.is_present());
        assert!(d.is_long_mode());
    }

    #[test]
    fn gdt_data64_fields() {
        let d = GdtDescriptor::data64();
        assert_eq!(d.dpl(), 0);
        assert!(d.is_present());
        assert!(!d.is_long_mode());
    }

    // --- TssDescriptor ---

    #[test]
    fn tss_descriptor_base_roundtrip() {
        let bases = [
            0x0000_0000_1234_5678u64,
            0x0000_FFFF_ABCD_0000u64,
            0x0000_0001_0000_0000u64,
        ];
        for &base in &bases {
            let d = TssDescriptor::new(base, 103);
            assert_eq!(d.base(), base, "base roundtrip failed for {base:#018x}");
        }
    }

    // --- IdtGate ---

    #[test]
    fn idt_gate_missing_is_not_present() {
        assert!(!IdtGate::MISSING.is_present());
    }

    #[test]
    fn idt_interrupt_gate_type() {
        let cs = SegmentSelector::new(1, 0, false);
        let g  = IdtGate::interrupt_gate(0xFFFF_DEAD_BEEF_1234, cs, 0, 0);
        assert_eq!(g.gate_type(), 0xE);
        assert!(g.is_present());
    }

    #[test]
    fn idt_trap_gate_type() {
        let cs = SegmentSelector::new(1, 0, false);
        let g  = IdtGate::trap_gate(0x1234_5678_9ABC_DEF0, cs, 0, 0);
        assert_eq!(g.gate_type(), 0xF);
    }

    #[test]
    fn idt_gate_handler_roundtrip() {
        let handlers = [
            0x0000_0000_0000_1234u64,
            0xFFFF_8000_0000_4567u64,
            0x0000_7FFF_FFFF_F000u64,
        ];
        let cs = SegmentSelector::new(1, 0, false);
        for &h in &handlers {
            let g = IdtGate::interrupt_gate(h, cs, 0, 0);
            assert_eq!(g.handler_addr(), h, "handler roundtrip failed for {h:#018x}");
        }
    }

    #[test]
    fn idt_gate_ist_and_dpl() {
        let cs = SegmentSelector::new(1, 0, false);
        // DPL=3, IST=2
        let g = IdtGate::interrupt_gate(0xFFFF_8000_1234_5000, cs, 3, 2);
        assert_eq!(crate::bits::bit_field_get(g.low, 34, 32), 2); // IST
        assert_eq!(crate::bits::bit_field_get(g.low, 46, 45), 3); // DPL
    }

    // --- tss_offsets ---

    #[test]
    fn tss_offsets_rsp0_at_4() {
        assert_eq!(tss_offsets::RSP0, 4);
    }

    #[test]
    fn tss_offsets_ist_spacing() {
        // IST entries are 8 bytes apart
        assert_eq!(tss_offsets::IST2 - tss_offsets::IST1, 8);
        assert_eq!(tss_offsets::IST7 - tss_offsets::IST1, 48);
    }

    #[test]
    fn tss_offsets_size() {
        assert_eq!(tss_offsets::SIZE, 104);
    }
}
