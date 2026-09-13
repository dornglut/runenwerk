//! Private physical carrier helpers for the maintained deterministic evaluator.
//!
//! These helpers centralize the current one-word transport realization without promoting it to
//! renderer-semantic identity or a generic public codec contract.

pub(super) const WORD_BYTES: usize = size_of::<u32>();

pub(super) fn decode_word(bytes: &[u8]) -> u32 {
    debug_assert_eq!(bytes.len(), WORD_BYTES);
    let bytes: [u8; WORD_BYTES] = bytes
        .try_into()
        .expect("maintained deterministic word decoding requires one exact carrier word");
    u32::from_ne_bytes(bytes)
}

pub(super) fn decode_finite_f32(word: u32) -> Option<f32> {
    let value = f32::from_bits(word);
    value.is_finite().then_some(value)
}
