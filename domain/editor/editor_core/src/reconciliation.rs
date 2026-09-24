//! File: domain/editor/editor_core/src/reconciliation.rs
//! Purpose: Reconciliation-policy evaluation contracts for shared ratified change ingress.

use crate::{RatifiedChange, RealityVersion, ReconciliationPolicy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReconciliationRejectReason {
    PolicyForbidden,
    SessionLocalOnly,
    BaseVersionMismatch {
        expected_base_version: RealityVersion,
        current_version: RealityVersion,
    },
    OutOfOrderResultVersion {
        current_version: RealityVersion,
        incoming_result_version: RealityVersion,
    },
}

impl ReconciliationRejectReason {
    pub const fn as_static_str(self) -> &'static str {
        match self {
            Self::PolicyForbidden => "reconciliation policy forbids shared change ingress",
            Self::SessionLocalOnly => {
                "reconciliation policy requires session-local change handling only"
            }
            Self::BaseVersionMismatch { .. } => {
                "incoming shared change base version does not match local reality version"
            }
            Self::OutOfOrderResultVersion { .. } => {
                "incoming shared change result version is older than local reality version"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReconciliationResult {
    Accepted,
    Rejected(ReconciliationRejectReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReconciliationDecision {
    pub policy: ReconciliationPolicy,
    pub current_version: RealityVersion,
    pub incoming_base_version: RealityVersion,
    pub incoming_result_version: RealityVersion,
    pub result: ReconciliationResult,
}

impl ReconciliationDecision {
    pub const fn accepted(
        policy: ReconciliationPolicy,
        current_version: RealityVersion,
        incoming_base_version: RealityVersion,
        incoming_result_version: RealityVersion,
    ) -> Self {
        Self {
            policy,
            current_version,
            incoming_base_version,
            incoming_result_version,
            result: ReconciliationResult::Accepted,
        }
    }

    pub const fn rejected(
        policy: ReconciliationPolicy,
        current_version: RealityVersion,
        incoming_base_version: RealityVersion,
        incoming_result_version: RealityVersion,
        reason: ReconciliationRejectReason,
    ) -> Self {
        Self {
            policy,
            current_version,
            incoming_base_version,
            incoming_result_version,
            result: ReconciliationResult::Rejected(reason),
        }
    }

    pub const fn is_accepted(self) -> bool {
        matches!(self.result, ReconciliationResult::Accepted)
    }

    pub const fn rejection_reason(self) -> Option<ReconciliationRejectReason> {
        match self.result {
            ReconciliationResult::Accepted => None,
            ReconciliationResult::Rejected(reason) => Some(reason),
        }
    }
}

pub fn evaluate_reconciliation(
    incoming: &RatifiedChange,
    current_version: RealityVersion,
) -> ReconciliationDecision {
    let incoming_base_version = incoming.base_version;
    let incoming_result_version = incoming.result_version;
    match incoming.reconciliation_policy {
        ReconciliationPolicy::Forbidden => ReconciliationDecision::rejected(
            incoming.reconciliation_policy,
            current_version,
            incoming_base_version,
            incoming_result_version,
            ReconciliationRejectReason::PolicyForbidden,
        ),
        ReconciliationPolicy::SessionLocalOnly => ReconciliationDecision::rejected(
            incoming.reconciliation_policy,
            current_version,
            incoming_base_version,
            incoming_result_version,
            ReconciliationRejectReason::SessionLocalOnly,
        ),
        ReconciliationPolicy::RejectOnBaseVersionMismatch => {
            if incoming_base_version != current_version {
                return ReconciliationDecision::rejected(
                    incoming.reconciliation_policy,
                    current_version,
                    incoming_base_version,
                    incoming_result_version,
                    ReconciliationRejectReason::BaseVersionMismatch {
                        expected_base_version: incoming_base_version,
                        current_version,
                    },
                );
            }

            ReconciliationDecision::accepted(
                incoming.reconciliation_policy,
                current_version,
                incoming_base_version,
                incoming_result_version,
            )
        }
        ReconciliationPolicy::LastWriterWinsLocal => {
            if incoming_result_version.0 < current_version.0 {
                return ReconciliationDecision::rejected(
                    incoming.reconciliation_policy,
                    current_version,
                    incoming_base_version,
                    incoming_result_version,
                    ReconciliationRejectReason::OutOfOrderResultVersion {
                        current_version,
                        incoming_result_version,
                    },
                );
            }

            ReconciliationDecision::accepted(
                incoming.reconciliation_policy,
                current_version,
                incoming_base_version,
                incoming_result_version,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AuthorityScope, CausalityId, ChangeOrigin, CommandId, CommandMetadata, MeaningDomain,
        PropagationStructure, RatificationClass, RatificationId, ReconciliationPolicy,
        RetentionHint, ReversibilityClass, SemanticOperation, StabilityClass, TransactionId,
        TransactionMetadata,
    };

    fn build_change(
        policy: ReconciliationPolicy,
        base_version: u64,
        result_version: u64,
    ) -> RatifiedChange {
        RatifiedChange {
            ratification_id: RatificationId(1),
            transaction: TransactionMetadata::new(TransactionId(1), "test"),
            causality_id: CausalityId(1),
            origin: ChangeOrigin::Runtime,
            authority_scope: AuthorityScope::LocalEditorSession,
            affected_domains: vec![MeaningDomain::SceneAuthoring],
            affected_scopes: vec!["scene:local".to_string()],
            base_version: RealityVersion(base_version),
            result_version: RealityVersion(result_version),
            command_metadata: vec![CommandMetadata::new(CommandId(1), "test")],
            semantic_operations: vec![SemanticOperation::SceneTransactionApplied],
            ratification_class: RatificationClass::ImmediateLocal,
            reversibility_class: ReversibilityClass::Reversible,
            retention_hint: RetentionHint::UndoRedo,
            stability_class: StabilityClass::LocalDurable,
            reconciliation_policy: policy,
            propagation_structure: PropagationStructure::LocalOnly,
            migration_path: None,
            timestamp: std::time::SystemTime::now(),
        }
    }

    #[test]
    fn reject_on_base_version_mismatch_rejects_when_versions_diverge() {
        let change = build_change(ReconciliationPolicy::RejectOnBaseVersionMismatch, 5, 6);
        let decision = evaluate_reconciliation(&change, RealityVersion(3));
        assert!(!decision.is_accepted());
        assert!(matches!(
            decision.rejection_reason(),
            Some(ReconciliationRejectReason::BaseVersionMismatch { .. })
        ));
    }

    #[test]
    fn reject_on_base_version_mismatch_accepts_when_versions_match() {
        let change = build_change(ReconciliationPolicy::RejectOnBaseVersionMismatch, 3, 4);
        let decision = evaluate_reconciliation(&change, RealityVersion(3));
        assert!(decision.is_accepted());
    }

    #[test]
    fn last_writer_wins_rejects_out_of_order_result_version() {
        let change = build_change(ReconciliationPolicy::LastWriterWinsLocal, 1, 2);
        let decision = evaluate_reconciliation(&change, RealityVersion(5));
        assert!(!decision.is_accepted());
        assert!(matches!(
            decision.rejection_reason(),
            Some(ReconciliationRejectReason::OutOfOrderResultVersion { .. })
        ));
    }
}
