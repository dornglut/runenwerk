/// Application-owned resource policy for the local editor/runtime-preview channel.
///
/// This limit is deliberately not derived from the retired framing width or from RunenNet. Preview
/// products may contain variable-size WorldSDF payloads, so the product protocol needs one explicit
/// finite message ceiling. Products larger than this must be partitioned by Runenwerk product logic
/// rather than expanding transport staging without bound.
pub const PREVIEW_TRANSPORT_MAX_MESSAGE_BYTES: usize = 64 * 1024 * 1024;

/// Maximum number of messages RunenNet delivery may retain for one preview flow/scope.
pub const PREVIEW_TRANSPORT_MAX_PENDING_MESSAGES: usize = 16;

/// Aggregate retained payload budget for one preview delivery flow/scope.
///
/// Keeping this equal to the single-message ceiling guarantees that one maximum-size message can be
/// admitted while preventing multiple maximum-size snapshots from accumulating in delivery state.
pub const PREVIEW_TRANSPORT_MAX_PENDING_PAYLOAD_BYTES: usize = PREVIEW_TRANSPORT_MAX_MESSAGE_BYTES;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_transport_policy_is_finite_and_internally_consistent() {
        assert_eq!(PREVIEW_TRANSPORT_MAX_MESSAGE_BYTES, 64 * 1024 * 1024);
        assert_eq!(PREVIEW_TRANSPORT_MAX_PENDING_MESSAGES, 16);
        assert_eq!(
            PREVIEW_TRANSPORT_MAX_PENDING_PAYLOAD_BYTES,
            PREVIEW_TRANSPORT_MAX_MESSAGE_BYTES
        );
        assert!(PREVIEW_TRANSPORT_MAX_MESSAGE_BYTES < u32::MAX as usize);
    }
}
