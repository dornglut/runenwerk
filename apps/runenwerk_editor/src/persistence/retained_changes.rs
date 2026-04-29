use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use editor_persistence::{
    RETAINED_CHANGE_LOG_VERSION_V1, RetainedRatifiedChangeLogV1, RetainedRatifiedChangeRecordV1,
    decode_ron, encode_ron_pretty,
};

use crate::editor_runtime::RunenwerkEditorRuntime;

fn retained_change_log_from_runtime(
    runtime: &RunenwerkEditorRuntime,
) -> RetainedRatifiedChangeLogV1 {
    RetainedRatifiedChangeLogV1 {
        version: RETAINED_CHANGE_LOG_VERSION_V1,
        entries: runtime
            .ratified_change_log()
            .iter()
            .map(retained_change_record_from_change)
            .collect(),
    }
}

fn retained_change_record_from_change(
    change: &editor_core::RatifiedChange,
) -> RetainedRatifiedChangeRecordV1 {
    RetainedRatifiedChangeRecordV1 {
        ratification_id: change.ratification_id.0,
        transaction_id: change.transaction.id.0,
        transaction_label: change.transaction.label.clone(),
        causality_id: change.causality_id.0,
        origin: change_origin_label(change.origin).to_string(),
        authority_scope: authority_scope_label(change.authority_scope).to_string(),
        affected_domains: change
            .affected_domains
            .iter()
            .copied()
            .map(meaning_domain_label)
            .map(ToString::to_string)
            .collect(),
        affected_scopes: change.affected_scopes.clone(),
        base_version: change.base_version.0,
        result_version: change.result_version.0,
        semantic_operations: change
            .semantic_operations
            .iter()
            .copied()
            .map(semantic_operation_label)
            .map(ToString::to_string)
            .collect(),
        ratification_class: ratification_class_label(change.ratification_class).to_string(),
        reversibility_class: reversibility_class_label(change.reversibility_class).to_string(),
        retention_hint: retention_hint_label(change.retention_hint).to_string(),
        stability_class: stability_class_label(change.stability_class).to_string(),
        reconciliation_policy: reconciliation_policy_label(change.reconciliation_policy)
            .to_string(),
        propagation_structure: propagation_structure_label(change.propagation_structure)
            .to_string(),
        timestamp_unix_millis: change
            .timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_millis() as u64),
    }
}

pub fn retained_change_log_path_for_scene(scene_path: &Path) -> PathBuf {
    let file_name = scene_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{name}.changes.ron"))
        .unwrap_or_else(|| "scene.changes.ron".to_string());
    scene_path.with_file_name(file_name)
}

pub fn write_retained_change_log(path: &Path, runtime: &RunenwerkEditorRuntime) -> Result<usize> {
    let retained = retained_change_log_from_runtime(runtime);
    let entry_count = retained.entries.len();
    let ron = encode_ron_pretty(&retained).context("failed to encode retained change log")?;
    std::fs::write(path, ron)
        .with_context(|| format!("failed to write retained change log: {}", path.display()))?;
    Ok(entry_count)
}

pub fn read_retained_change_log(path: &Path) -> Result<RetainedRatifiedChangeLogV1> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read retained change log: {}", path.display()))?;
    decode_ron(&source).context("failed to decode retained change log")
}

fn change_origin_label(origin: editor_core::ChangeOrigin) -> &'static str {
    match origin {
        editor_core::ChangeOrigin::EditorShell => "EditorShell",
        editor_core::ChangeOrigin::Shortcut => "Shortcut",
        editor_core::ChangeOrigin::InspectorPanel => "InspectorPanel",
        editor_core::ChangeOrigin::OutlinerPanel => "OutlinerPanel",
        editor_core::ChangeOrigin::EntityTablePanel => "EntityTablePanel",
        editor_core::ChangeOrigin::ToolInteraction => "ToolInteraction",
        editor_core::ChangeOrigin::ViewportInteraction => "ViewportInteraction",
        editor_core::ChangeOrigin::Runtime => "Runtime",
        editor_core::ChangeOrigin::Persistence => "Persistence",
    }
}

fn authority_scope_label(scope: editor_core::AuthorityScope) -> &'static str {
    match scope {
        editor_core::AuthorityScope::LocalEditorSession => "LocalEditorSession",
    }
}

fn meaning_domain_label(domain: editor_core::MeaningDomain) -> &'static str {
    match domain {
        editor_core::MeaningDomain::SceneAuthoring => "SceneAuthoring",
    }
}

fn semantic_operation_label(operation: editor_core::SemanticOperation) -> &'static str {
    match operation {
        editor_core::SemanticOperation::SceneCommandApplied => "SceneCommandApplied",
        editor_core::SemanticOperation::SceneTransactionApplied => "SceneTransactionApplied",
        editor_core::SemanticOperation::SceneTransactionUndone => "SceneTransactionUndone",
        editor_core::SemanticOperation::SceneTransactionRedone => "SceneTransactionRedone",
    }
}

fn ratification_class_label(class: editor_core::RatificationClass) -> &'static str {
    match class {
        editor_core::RatificationClass::ImmediateLocal => "ImmediateLocal",
        editor_core::RatificationClass::Authority => "Authority",
        editor_core::RatificationClass::Coordinated => "Coordinated",
        editor_core::RatificationClass::Deferred => "Deferred",
        editor_core::RatificationClass::SessionOnly => "SessionOnly",
    }
}

fn reversibility_class_label(class: editor_core::ReversibilityClass) -> &'static str {
    match class {
        editor_core::ReversibilityClass::Reversible => "Reversible",
        editor_core::ReversibilityClass::Irreversible => "Irreversible",
    }
}

fn retention_hint_label(hint: editor_core::RetentionHint) -> &'static str {
    match hint {
        editor_core::RetentionHint::UndoRedo => "UndoRedo",
        editor_core::RetentionHint::SessionOnly => "SessionOnly",
        editor_core::RetentionHint::Durable => "Durable",
    }
}

fn stability_class_label(class: editor_core::StabilityClass) -> &'static str {
    match class {
        editor_core::StabilityClass::SessionVolatile => "SessionVolatile",
        editor_core::StabilityClass::LocalDurable => "LocalDurable",
    }
}

fn reconciliation_policy_label(policy: editor_core::ReconciliationPolicy) -> &'static str {
    match policy {
        editor_core::ReconciliationPolicy::Forbidden => "Forbidden",
        editor_core::ReconciliationPolicy::RejectOnBaseVersionMismatch => {
            "RejectOnBaseVersionMismatch"
        }
        editor_core::ReconciliationPolicy::LastWriterWinsLocal => "LastWriterWinsLocal",
        editor_core::ReconciliationPolicy::SessionLocalOnly => "SessionLocalOnly",
    }
}

fn propagation_structure_label(structure: editor_core::PropagationStructure) -> &'static str {
    match structure {
        editor_core::PropagationStructure::LocalOnly => "LocalOnly",
        editor_core::PropagationStructure::SessionBroadcast => "SessionBroadcast",
    }
}

#[cfg(test)]
mod tests {
    use editor_core::ChangeOrigin;
    use editor_scene::{SceneCommandIntent, scene_intent_to_command};

    use super::*;
    use crate::editor_runtime::{RunenwerkEditorRuntime, ratify_scene_command_with_transaction_id};

    fn temp_change_log_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        path.push(format!("runenwerk_retained_changes_{nanos}.ron"));
        path
    }

    #[test]
    fn retained_change_log_roundtrip_preserves_entries() {
        let mut runtime = RunenwerkEditorRuntime::new();
        let command_id = runtime.allocate_command_id();
        let transaction_id = runtime.allocate_transaction_id();
        let _ = ratify_scene_command_with_transaction_id(
            &mut runtime,
            "Create Entity",
            scene_intent_to_command(
                command_id,
                SceneCommandIntent::CreateEntity {
                    parent: None,
                    display_name: "Entity".to_string(),
                },
            ),
            transaction_id,
            ChangeOrigin::Runtime,
        )
        .expect("scene command should execute");

        let path = temp_change_log_path();
        let written = write_retained_change_log(&path, &runtime)
            .expect("retained change log should be writable");
        assert_eq!(written, 1);

        let loaded = read_retained_change_log(&path).expect("retained change log should decode");
        assert_eq!(loaded.version, RETAINED_CHANGE_LOG_VERSION_V1);
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].transaction_label, "Create Entity");

        let _ = std::fs::remove_file(path);
    }
}
