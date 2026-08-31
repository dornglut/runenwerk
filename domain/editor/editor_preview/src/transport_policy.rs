/// Application-owned resource policy for the local editor/runtime-preview channel.
///
/// This is a finite host staging limit, not a WorldSDF/product-format maximum and not a value
/// derived from the retired framing width or from RunenNet. N1 deliberately preserves the current
/// one-command-per-message preview protocol; if a product producer requires logical partitioning,
/// that policy belongs above this transport boundary rather than in RunenNet delivery semantics.
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
