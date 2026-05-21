/// Convert a `u16` from big-endian (network) to native (little-endian on x86).
#[inline(always)]
pub const fn be16_to_cpu(x: u16) -> u16 { u16::from_be(x) }

/// Convert a `u32` from big-endian to native.
#[inline(always)]
pub const fn be32_to_cpu(x: u32) -> u32 { u32::from_be(x) }

/// Convert a `u64` from big-endian to native.
#[inline(always)]
pub const fn be64_to_cpu(x: u64) -> u64 { u64::from_be(x) }

/// Convert a `u16` from native (little-endian) to big-endian.
#[inline(always)]
pub const fn cpu_to_be16(x: u16) -> u16 { x.to_be() }

/// Convert a `u32` from native to big-endian.
#[inline(always)]
pub const fn cpu_to_be32(x: u32) -> u32 { x.to_be() }

/// Convert a `u64` from native to big-endian.
#[inline(always)]
pub const fn cpu_to_be64(x: u64) -> u64 { x.to_be() }

/// Read a `u32` from an unaligned byte slice in big-endian order.
///
/// Safe on x86 (which supports unaligned loads) but uses byte-by-byte
/// construction to make the intent explicit and avoid UB from casting.
#[inline(always)]
pub fn read_be32(bytes: &[u8]) -> u32 {
    assert!(bytes.len() >= 4);
    (bytes[0] as u32) << 24
    | (bytes[1] as u32) << 16
    | (bytes[2] as u32) << 8
    | (bytes[3] as u32)
}

/// Write a `u32` to a byte slice in big-endian order.
#[inline(always)]
pub fn write_be32(bytes: &mut [u8], value: u32) {
    assert!(bytes.len() >= 4);
    bytes[0] = (value >> 24) as u8;
    bytes[1] = (value >> 16) as u8;
    bytes[2] = (value >> 8)  as u8;
    bytes[3] = value          as u8;
}

/// Read a `u32` from a byte slice in little-endian order.
#[inline(always)]
pub fn read_le32(bytes: &[u8]) -> u32 {
    assert!(bytes.len() >= 4);
    (bytes[3] as u32) << 24
    | (bytes[2] as u32) << 16
    | (bytes[1] as u32) << 8
    | (bytes[0] as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- be16 roundtrip ---

    #[test]
    fn be16_roundtrip() {
        for v in [0u16, 1, 0x0102, 0xFF00, u16::MAX] {
            assert_eq!(be16_to_cpu(cpu_to_be16(v)), v);
            assert_eq!(cpu_to_be16(be16_to_cpu(v)), v);
        }
    }

    #[test]
    fn cpu_to_be16_puts_msb_first_in_memory() {
        // cpu_to_be16 produces a value whose memory layout (to_ne_bytes) is [MSB, LSB]
        let be = cpu_to_be16(0x0102u16);
        assert_eq!(be.to_ne_bytes(), [0x01, 0x02]);
    }

    // --- be32 roundtrip ---

    #[test]
    fn be32_roundtrip() {
        for v in [0u32, 1, 0x0102_0304, 0xFF00_FF00, u32::MAX] {
            assert_eq!(be32_to_cpu(cpu_to_be32(v)), v);
            assert_eq!(cpu_to_be32(be32_to_cpu(v)), v);
        }
    }

    #[test]
    fn cpu_to_be32_puts_msb_first_in_memory() {
        // Memory layout (to_ne_bytes) should be [MSB, ..., LSB]
        let be = cpu_to_be32(0x1234_5678u32);
        assert_eq!(be.to_ne_bytes(), [0x12, 0x34, 0x56, 0x78]);
    }

    // --- be64 roundtrip ---

    #[test]
    fn be64_roundtrip() {
        for v in [0u64, 1, 0x0102_0304_0506_0708, u64::MAX] {
            assert_eq!(be64_to_cpu(cpu_to_be64(v)), v);
            assert_eq!(cpu_to_be64(be64_to_cpu(v)), v);
        }
    }

    // --- read_be32 known value ---

    #[test]
    fn read_be32_known_bytes() {
        assert_eq!(read_be32(&[0x12, 0x34, 0x56, 0x78]), 0x1234_5678);
        assert_eq!(read_be32(&[0x00, 0x00, 0x00, 0x01]), 1);
        assert_eq!(read_be32(&[0xFF, 0xFF, 0xFF, 0xFF]), u32::MAX);
        assert_eq!(read_be32(&[0x00, 0x00, 0x00, 0x00]), 0);
    }

    #[test]
    fn read_be32_matches_cpu_to_be32() {
        let v = 0xDEAD_BEEFu32;
        // to_ne_bytes gives the memory layout that cpu_to_be32 is meant to produce
        let be_bytes = cpu_to_be32(v).to_ne_bytes();
        assert_eq!(read_be32(&be_bytes), v);
    }

    // --- write_be32 / read_be32 roundtrip ---

    #[test]
    fn write_then_read_be32_roundtrip() {
        for v in [0u32, 1, 0x0102_0304, 0xFF00_FF00, u32::MAX] {
            let mut buf = [0u8; 4];
            write_be32(&mut buf, v);
            assert_eq!(read_be32(&buf), v);
        }
    }

    #[test]
    fn write_be32_byte_order() {
        let mut buf = [0u8; 4];
        write_be32(&mut buf, 0x0A0B_0C0D);
        assert_eq!(buf, [0x0A, 0x0B, 0x0C, 0x0D]);
    }

    // --- read_le32 ---

    #[test]
    fn read_le32_known_bytes() {
        assert_eq!(read_le32(&[0x78, 0x56, 0x34, 0x12]), 0x1234_5678);
        assert_eq!(read_le32(&[0x01, 0x00, 0x00, 0x00]), 1);
        assert_eq!(read_le32(&[0xFF, 0xFF, 0xFF, 0xFF]), u32::MAX);
    }

    #[test]
    fn read_le32_opposite_of_read_be32() {
        let bytes = [0x12u8, 0x34, 0x56, 0x78];
        let be = read_be32(&bytes);
        let le = read_le32(&bytes);
        // BE and LE readings of same bytes should be byte-reversed values of each other
        assert_eq!(be, 0x1234_5678);
        assert_eq!(le, 0x7856_3412);
        assert_eq!(be.swap_bytes(), le);
    }

    // --- longer-than-4 slices are accepted ---

    #[test]
    fn read_functions_accept_longer_slices() {
        let buf = [0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06];
        assert_eq!(read_be32(&buf), 0x0102_0304);
        assert_eq!(read_le32(&buf), 0x0403_0201);
    }
}
