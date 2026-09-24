use ui_composition::{
    CompositionRootId, CompositionTransactionId, MountedUnitId, PresentationTargetId, RegionId,
};

use super::EditorCompositionRuntime;
use super::{
    EditorCompositionDiagnosticCode as Code, EditorCompositionDiagnosticRecord as Record,
    EditorCompositionDiagnosticStage as Stage, EditorCompositionDiagnosticSubject as Subject,
    EditorCompositionRejection,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorCompositionIdentityAllocator {
    next_target: Option<u64>,
    next_root: Option<u64>,
    next_region: Option<u64>,
    next_mounted_unit: Option<u64>,
    next_transaction: Option<u64>,
    next_panel_instance: Option<u64>,
    next_compatibility_surface: Option<u64>,
    next_viewport_instance: Option<u64>,
    next_compatibility_host: Option<u64>,
    next_tab_stack: Option<u64>,
}

impl EditorCompositionIdentityAllocator {
    pub fn from_runtime(runtime: &EditorCompositionRuntime) -> Self {
        let definition = runtime.composition().definition();
        Self {
            next_target: next_after(definition.targets().iter().map(|value| value.id.raw())),
            next_root: next_after(definition.roots().iter().map(|value| value.id.raw())),
            next_region: next_after(definition.regions().iter().map(|value| value.id.raw())),
            next_mounted_unit: next_after(
                definition
                    .mounted_units()
                    .iter()
                    .map(|value| value.id.raw()),
            ),
            next_transaction: next_after(
                runtime
                    .composition()
                    .applied_transaction_ids()
                    .map(|id| id.raw()),
            ),
            next_panel_instance: next_after(
                runtime
                    .extension()
                    .mounted_units()
                    .iter()
                    .map(|value| value.panel_instance_raw),
            ),
            next_compatibility_surface: next_after(
                runtime
                    .extension()
                    .mounted_units()
                    .iter()
                    .map(|value| value.compatibility_surface_raw),
            ),
            next_viewport_instance: next_after(
                runtime
                    .extension()
                    .mounted_units()
                    .iter()
                    .filter_map(|value| value.viewport_instance_raw),
            ),
            next_compatibility_host: next_after(
                runtime
                    .extension()
                    .regions()
                    .iter()
                    .map(|value| value.compatibility_host_raw),
            ),
            next_tab_stack: next_after(
                runtime
                    .extension()
                    .regions()
                    .iter()
                    .filter_map(|value| value.tab_stack_raw),
            ),
        }
    }

    pub fn allocate_target(&mut self) -> Result<PresentationTargetId, EditorCompositionRejection> {
        take_next(&mut self.next_target, "presentation-target").map(PresentationTargetId::new)
    }

    pub fn allocate_root(&mut self) -> Result<CompositionRootId, EditorCompositionRejection> {
        take_next(&mut self.next_root, "composition-root").map(CompositionRootId::new)
    }

    pub fn allocate_region(&mut self) -> Result<RegionId, EditorCompositionRejection> {
        take_next(&mut self.next_region, "region").map(RegionId::new)
    }

    pub fn allocate_mounted_unit(&mut self) -> Result<MountedUnitId, EditorCompositionRejection> {
        take_next(&mut self.next_mounted_unit, "mounted-unit").map(MountedUnitId::new)
    }

    pub fn allocate_transaction(
        &mut self,
    ) -> Result<CompositionTransactionId, EditorCompositionRejection> {
        take_next(&mut self.next_transaction, "transaction").map(CompositionTransactionId::new)
    }

    pub fn allocate_compatibility_host(&mut self) -> Result<u64, EditorCompositionRejection> {
        take_next(&mut self.next_compatibility_host, "compatibility-host")
    }

    pub fn allocate_panel_instance(&mut self) -> Result<u64, EditorCompositionRejection> {
        take_next(&mut self.next_panel_instance, "panel-instance")
    }

    pub fn allocate_compatibility_surface(&mut self) -> Result<u64, EditorCompositionRejection> {
        take_next(
            &mut self.next_compatibility_surface,
            "compatibility-surface",
        )
    }

    pub fn allocate_viewport_instance(&mut self) -> Result<u64, EditorCompositionRejection> {
        take_next(&mut self.next_viewport_instance, "viewport-instance")
    }

    pub fn allocate_tab_stack(&mut self) -> Result<u64, EditorCompositionRejection> {
        take_next(&mut self.next_tab_stack, "tab-stack")
    }
}

fn next_after(values: impl Iterator<Item = u64>) -> Option<u64> {
    values.max().unwrap_or_default().checked_add(1)
}

fn take_next(
    next: &mut Option<u64>,
    family: &'static str,
) -> Result<u64, EditorCompositionRejection> {
    let current = next.take().filter(|value| *value != 0).ok_or_else(|| {
        EditorCompositionRejection::single(Record::error(
            Code::IdentityExhausted,
            Stage::Transaction,
            Subject::General(format!("editor-composition-{family}-identity")),
            "Promote or reload the layout into a fresh identity space before retrying.",
        ))
    })?;
    *next = current.checked_add(1);
    Ok(current)
}

#[cfg(test)]
mod tests {
    use crate::{
        WorkspaceIdentityAllocator, default_workspace_profile_registry, import_legacy_workspace,
    };

    use super::*;

    #[test]
    fn allocator_starts_after_every_installed_core_identity_family() {
        let profiles = default_workspace_profile_registry();
        let profile = profiles.default_profile().unwrap();
        let mut legacy_ids = WorkspaceIdentityAllocator::new();
        let workspace_id = legacy_ids.allocate_workspace_id();
        let workspace = profile.build_default_workspace_state(workspace_id, &mut legacy_ids);
        let runtime = import_legacy_workspace(profile.id, &workspace).unwrap();
        let definition = runtime.composition().definition();
        let max_target = definition
            .targets()
            .iter()
            .map(|value| value.id)
            .max()
            .unwrap();
        let max_root = definition
            .roots()
            .iter()
            .map(|value| value.id)
            .max()
            .unwrap();
        let max_region = definition
            .regions()
            .iter()
            .map(|value| value.id)
            .max()
            .unwrap();
        let max_unit = definition
            .mounted_units()
            .iter()
            .map(|value| value.id)
            .max()
            .unwrap();

        let mut allocator = EditorCompositionIdentityAllocator::from_runtime(&runtime);

        assert!(allocator.allocate_target().unwrap() > max_target);
        assert!(allocator.allocate_root().unwrap() > max_root);
        assert!(allocator.allocate_region().unwrap() > max_region);
        assert!(allocator.allocate_mounted_unit().unwrap() > max_unit);
    }
}
