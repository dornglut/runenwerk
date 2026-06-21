use ui_composition::CompositionTransactionId;
use ui_composition::{CompositionPolicies, CompositionTransaction, StateRevision};

use super::{
    EditorCompositionDiagnosticCode as Code, EditorCompositionDiagnosticRecord as Record,
    EditorCompositionDiagnosticStage as Stage, EditorCompositionDiagnosticSubject as Subject,
    EditorCompositionExtensionV1, EditorCompositionProjectionArtifact, EditorCompositionRejection,
    EditorCompositionRuntime, project_editor_composition,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCompositionChangeSet {
    expected_revision: StateRevision,
    core: CompositionTransaction,
    extension: EditorCompositionExtensionV1,
}

impl EditorCompositionChangeSet {
    pub fn new(
        expected_revision: StateRevision,
        core: CompositionTransaction,
        extension: EditorCompositionExtensionV1,
    ) -> Self {
        Self {
            expected_revision,
            core,
            extension,
        }
    }

    pub const fn expected_revision(&self) -> StateRevision {
        self.expected_revision
    }

    pub const fn core(&self) -> &CompositionTransaction {
        &self.core
    }

    pub const fn extension(&self) -> &EditorCompositionExtensionV1 {
        &self.extension
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedEditorCompositionCommit {
    source_revision: StateRevision,
    runtime: EditorCompositionRuntime,
    projection: EditorCompositionProjectionArtifact,
}

impl PreparedEditorCompositionCommit {
    pub const fn source_revision(&self) -> StateRevision {
        self.source_revision
    }

    pub fn projection(&self) -> &EditorCompositionProjectionArtifact {
        &self.projection
    }

    pub fn candidate_runtime(&self) -> &EditorCompositionRuntime {
        &self.runtime
    }
}

impl EditorCompositionRuntime {
    pub fn prepare_change(
        &self,
        change: EditorCompositionChangeSet,
        policies: CompositionPolicies<'_>,
    ) -> Result<PreparedEditorCompositionCommit, EditorCompositionRejection> {
        let source_revision = self.composition().revision();
        if change.expected_revision != source_revision
            || change.core.expected_revision() != source_revision
        {
            return Err(stale_rejection(
                self.composition().definition().id().raw(),
                source_revision,
                change.expected_revision,
            ));
        }

        let mut candidate_state = self.composition().clone();
        candidate_state
            .transact(change.core, policies)
            .map_err(|rejection| core_rejection(source_revision, &rejection))?;
        let mut candidate_runtime = Self::install(candidate_state, change.extension)?;
        candidate_runtime.extension_history = self.extension_history.clone();
        candidate_runtime
            .extension_history
            .record(self.extension.clone(), candidate_runtime.extension.clone());
        let projection = project_editor_composition(&candidate_runtime)?;
        Ok(PreparedEditorCompositionCommit {
            source_revision,
            runtime: candidate_runtime,
            projection,
        })
    }

    pub fn commit_prepared(
        &mut self,
        prepared: PreparedEditorCompositionCommit,
    ) -> Result<EditorCompositionProjectionArtifact, EditorCompositionRejection> {
        let current = self.composition().revision();
        if current != prepared.source_revision {
            return Err(stale_rejection(
                self.composition().definition().id().raw(),
                current,
                prepared.source_revision,
            ));
        }
        *self = prepared.runtime;
        Ok(prepared.projection)
    }

    pub fn undo_structural(
        &mut self,
        transaction_id: CompositionTransactionId,
        policies: CompositionPolicies<'_>,
    ) -> Result<EditorCompositionProjectionArtifact, EditorCompositionRejection> {
        let Some(extension) = self.extension_history.undo_extension().cloned() else {
            return Err(history_unavailable(
                self.composition.definition().id().raw(),
                "undo",
            ));
        };
        let mut candidate = self.clone();
        candidate
            .composition
            .undo(transaction_id, policies)
            .map_err(|rejection| core_rejection(self.composition.revision(), &rejection))?;
        candidate.extension = extension.relinked_to_definition(
            candidate.composition.definition().id(),
            candidate.composition.definition().revision(),
        );
        candidate
            .extension
            .validate_against(&candidate.composition)?;
        candidate.extension_history.commit_undo();
        let projection = project_editor_composition(&candidate)?;
        *self = candidate;
        Ok(projection)
    }

    pub fn redo_structural(
        &mut self,
        transaction_id: CompositionTransactionId,
        policies: CompositionPolicies<'_>,
    ) -> Result<EditorCompositionProjectionArtifact, EditorCompositionRejection> {
        let Some(extension) = self.extension_history.redo_extension().cloned() else {
            return Err(history_unavailable(
                self.composition.definition().id().raw(),
                "redo",
            ));
        };
        let mut candidate = self.clone();
        candidate
            .composition
            .redo(transaction_id, policies)
            .map_err(|rejection| core_rejection(self.composition.revision(), &rejection))?;
        candidate.extension = extension.relinked_to_definition(
            candidate.composition.definition().id(),
            candidate.composition.definition().revision(),
        );
        candidate
            .extension
            .validate_against(&candidate.composition)?;
        candidate.extension_history.commit_redo();
        let projection = project_editor_composition(&candidate)?;
        *self = candidate;
        Ok(projection)
    }
}

fn history_unavailable(layout_raw: u64, operation: &'static str) -> EditorCompositionRejection {
    EditorCompositionRejection::single(Record::error(
        Code::HistoryUnavailable,
        Stage::Transaction,
        Subject::Layout(layout_raw),
        "Apply structural undo or redo only when a paired core and editor-extension entry exists.",
    ).with_context("operation", operation))
}

fn core_rejection(
    source_revision: StateRevision,
    rejection: &ui_composition::CompositionRejection,
) -> EditorCompositionRejection {
    let codes = rejection
        .diagnostics()
        .iter()
        .map(|record| record.code().as_str())
        .collect::<Vec<_>>()
        .join(",");
    EditorCompositionRejection::single(
        Record::error(
            Code::TransactionRejected,
            Stage::Transaction,
            Subject::General("editor-composition-transaction".to_owned()),
            "Correct the rejected core transaction before retrying the editor composition change.",
        )
        .with_context("source_revision", source_revision.raw().to_string())
        .with_context("core_diagnostic_codes", codes),
    )
}

fn stale_rejection(
    layout_raw: u64,
    current: StateRevision,
    expected: StateRevision,
) -> EditorCompositionRejection {
    EditorCompositionRejection::single(
        Record::error(
            Code::StalePreparedChange,
            Stage::Transaction,
            Subject::Layout(layout_raw),
            "Rebuild and reauthorize the editor composition change against the current revision.",
        )
        .with_context("current_revision", current.raw().to_string())
        .with_context("expected_revision", expected.raw().to_string()),
    )
}

#[cfg(test)]
mod tests {
    use ui_composition::{
        CompositionCapabilityPolicy, CompositionLifecyclePolicy, CompositionPolicyDecision,
        CompositionSnapshot, CompositionTargetPolicy, CompositionTransactionId, DefinitionRevision,
        RegionKind, SplitFraction,
    };

    use crate::{
        WorkspaceIdentityAllocator, default_workspace_profile_registry, import_legacy_workspace,
    };

    use super::*;

    struct Allow;

    impl CompositionLifecyclePolicy for Allow {
        fn evaluate(
            &self,
            _: CompositionSnapshot<'_>,
            _: &CompositionTransaction,
        ) -> CompositionPolicyDecision {
            CompositionPolicyDecision::Accepted
        }
    }

    impl CompositionCapabilityPolicy for Allow {
        fn evaluate(
            &self,
            _: CompositionSnapshot<'_>,
            _: &CompositionTransaction,
        ) -> CompositionPolicyDecision {
            CompositionPolicyDecision::Accepted
        }
    }

    impl CompositionTargetPolicy for Allow {
        fn evaluate(
            &self,
            _: CompositionSnapshot<'_>,
            _: &CompositionTransaction,
        ) -> CompositionPolicyDecision {
            CompositionPolicyDecision::Accepted
        }
    }

    fn policies(allow: &Allow) -> CompositionPolicies<'_> {
        CompositionPolicies {
            lifecycle: allow,
            capability: allow,
            target: allow,
        }
    }

    fn runtime() -> EditorCompositionRuntime {
        let profiles = default_workspace_profile_registry();
        let profile = profiles.default_profile().unwrap();
        let mut ids = WorkspaceIdentityAllocator::new();
        let workspace_id = ids.allocate_workspace_id();
        let workspace = profile.build_default_workspace_state(workspace_id, &mut ids);
        import_legacy_workspace(profile.id, &workspace).unwrap()
    }

    fn resize_change(
        runtime: &EditorCompositionRuntime,
        transaction_raw: u64,
        basis_points: u16,
    ) -> EditorCompositionChangeSet {
        let split = runtime
            .composition()
            .definition()
            .regions()
            .iter()
            .find(|region| matches!(region.kind, RegionKind::Split { .. }))
            .expect("default editor composition should contain a split");
        let revision = runtime.composition().revision();
        let transaction = CompositionTransaction::new(
            CompositionTransactionId::new(transaction_raw),
            revision,
            vec![ui_composition::CompositionCommand::resize_split(
                split.id,
                SplitFraction::try_new(basis_points).unwrap(),
            )],
        );
        let extension = runtime.extension().relinked_to_definition(
            runtime.composition().definition().id(),
            DefinitionRevision::new(revision.raw() + 1),
        );
        EditorCompositionChangeSet::new(revision, transaction, extension)
    }

    #[test]
    fn prepare_is_side_effect_free_and_commit_swaps_core_extension_and_projection_together() {
        let mut runtime = runtime();
        let before = runtime.clone();
        let allow = Allow;

        let prepared = runtime
            .prepare_change(resize_change(&runtime, 9_001, 5_200), policies(&allow))
            .unwrap();

        assert_eq!(runtime, before);
        let projection = runtime.commit_prepared(prepared).unwrap();
        assert_eq!(runtime.composition().revision(), StateRevision::new(2));
        assert_eq!(
            runtime.extension().definition_revision(),
            DefinitionRevision::new(2)
        );
        assert_eq!(
            projection.regions.len(),
            runtime.composition().definition().regions().len()
        );
    }

    #[test]
    fn stale_prepared_commit_rejects_without_replacing_current_runtime() {
        let mut runtime = runtime();
        let allow = Allow;
        let first = runtime
            .prepare_change(resize_change(&runtime, 9_002, 5_100), policies(&allow))
            .unwrap();
        let stale = runtime
            .prepare_change(resize_change(&runtime, 9_003, 5_300), policies(&allow))
            .unwrap();
        runtime.commit_prepared(first).unwrap();
        let before_stale = runtime.clone();

        let rejection = runtime.commit_prepared(stale).unwrap_err();

        assert_eq!(runtime, before_stale);
        assert!(
            rejection
                .diagnostics()
                .iter()
                .any(|record| record.code() == Code::StalePreparedChange)
        );
    }

    #[test]
    fn structural_undo_and_redo_restore_paired_core_and_extension_revisions() {
        let mut runtime = runtime();
        let allow = Allow;
        let original_extension = runtime.extension().clone();
        let prepared = runtime
            .prepare_change(resize_change(&runtime, 9_010, 5_700), policies(&allow))
            .unwrap();
        runtime.commit_prepared(prepared).unwrap();

        runtime
            .undo_structural(CompositionTransactionId::new(9_011), policies(&allow))
            .unwrap();
        assert_eq!(runtime.composition().revision(), StateRevision::new(3));
        assert_eq!(
            runtime.extension().mounted_units(),
            original_extension.mounted_units()
        );
        assert_eq!(
            runtime.extension().definition_revision(),
            runtime.composition().definition().revision()
        );

        runtime
            .redo_structural(CompositionTransactionId::new(9_012), policies(&allow))
            .unwrap();
        assert_eq!(runtime.composition().revision(), StateRevision::new(4));
        assert_eq!(
            runtime.extension().definition_revision(),
            runtime.composition().definition().revision()
        );
    }
}
