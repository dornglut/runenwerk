use editor_core::{
    AuthorityScope, MeaningDomain, PropagationStructure, RatificationClass, RatifiedChange,
    ReconciliationPolicy, RetentionHint, ReversibilityClass, SceneChangeRatificationParams,
    StabilityClass,
};

use crate::editor_runtime::RunenwerkEditorRuntime;

pub(crate) fn ratify_scene_change(
    runtime: &mut RunenwerkEditorRuntime,
    params: SceneChangeRatificationParams,
) -> RatifiedChange {
    let base_version = runtime.current_scene_reality_version();
    let result_version = runtime.advance_scene_reality_version();
    let propagation_structure = match runtime.sharing_policy() {
        editor_core::SharingPolicy::LocalOnly => PropagationStructure::LocalOnly,
        editor_core::SharingPolicy::SessionBroadcast => PropagationStructure::SessionBroadcast,
    };

    RatifiedChange {
        ratification_id: runtime.allocate_ratification_id(),
        transaction: params.transaction,
        causality_id: params
            .causality_id
            .unwrap_or_else(|| runtime.allocate_causality_id()),
        origin: params.origin,
        authority_scope: AuthorityScope::LocalEditorSession,
        affected_domains: vec![MeaningDomain::SceneAuthoring],
        affected_scopes: vec!["scene:local".to_string()],
        base_version,
        result_version,
        command_metadata: params.command_metadata,
        semantic_operations: params.semantic_operations,
        ratification_class: RatificationClass::ImmediateLocal,
        reversibility_class: ReversibilityClass::Reversible,
        retention_hint: RetentionHint::UndoRedo,
        stability_class: StabilityClass::LocalDurable,
        reconciliation_policy: ReconciliationPolicy::RejectOnBaseVersionMismatch,
        propagation_structure,
        migration_path: None,
        timestamp: std::time::SystemTime::now(),
    }
}
