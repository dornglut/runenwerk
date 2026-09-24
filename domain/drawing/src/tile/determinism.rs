//! File: domain/drawing/src/tile/determinism.rs
//! Purpose: Local stable hashing for drawing product formation contracts.

#[derive(Debug, Clone, Copy)]
pub(crate) struct StableDrawingHasher {
    state: u64,
}

impl Default for StableDrawingHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl StableDrawingHasher {
    pub(crate) const fn new() -> Self {
        Self {
            state: 0xcbf2_9ce4_8422_2325,
        }
    }

    pub(crate) fn write_str(&mut self, value: &str) {
        self.write_u64(value.len() as u64);
        for byte in value.as_bytes() {
            self.write_u8(*byte);
        }
    }

    pub(crate) fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    pub(crate) fn write_u8(&mut self, value: u8) {
        self.state ^= u64::from(value);
        self.state = self.state.wrapping_mul(0x0000_0100_0000_01b3);
    }

    pub(crate) fn write_u32(&mut self, value: u32) {
        for byte in value.to_le_bytes() {
            self.write_u8(byte);
        }
    }

    pub(crate) fn write_u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.write_u8(byte);
        }
    }

    pub(crate) fn write_i64(&mut self, value: i64) {
        for byte in value.to_le_bytes() {
            self.write_u8(byte);
        }
    }

    pub(crate) fn write_f32(&mut self, value: f32) {
        self.write_u32(value.to_bits());
    }

    pub(crate) fn write_f64(&mut self, value: f64) {
        self.write_u64(value.to_bits());
    }

    pub(crate) fn finish(self) -> u64 {
        non_zero_hash(self.state)
    }
}

pub(crate) fn non_zero_hash(value: u64) -> u64 {
    if value == 0 { 1 } else { value }
}
