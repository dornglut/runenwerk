use editor_shell::{EditorCompositionRuntime, EditorWindowId, EditorWindowRegistry};
use engine::plugins::render::backend::RenderSurfaceId;
use engine::runtime::NativeWindowId;
use ui_composition::PresentationTargetId;

use crate::shell::EditorCompositionTargetBindingRegistry;

use super::RunenwerkEditorShellState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorWindowPresentationBinding {
    pub native_window_id: NativeWindowId,
    pub render_surface_id: RenderSurfaceId,
}

impl EditorWindowPresentationBinding {
    pub fn primary() -> Self {
        Self {
            native_window_id: NativeWindowId::primary(),
            render_surface_id: RenderSurfaceId::primary(),
        }
    }
}

pub(super) fn reconcile_composition_target_bindings(
    runtime: &EditorCompositionRuntime,
    existing: &EditorCompositionTargetBindingRegistry<EditorWindowPresentationBinding>,
    primary_binding: EditorWindowPresentationBinding,
    created_binding: Option<(PresentationTargetId, EditorWindowPresentationBinding)>,
) -> Result<
    EditorCompositionTargetBindingRegistry<EditorWindowPresentationBinding>,
    editor_shell::EditorCompositionRejection,
> {
    let Some(primary_target) = runtime.composition().definition().targets().first() else {
        return Err(target_binding_rejection(
            "editor composition requires at least one presentation target",
        ));
    };
    let mut bindings = EditorCompositionTargetBindingRegistry::default();
    for target in runtime.composition().definition().targets() {
        let binding = created_binding
            .filter(|(target_id, _)| *target_id == target.id)
            .map(|(_, binding)| binding)
            .or_else(|| existing.binding(target.id).copied())
            .or_else(|| (target.id == primary_target.id).then_some(primary_binding))
            .ok_or_else(|| {
                target_binding_rejection(
                    "every non-primary composition target requires an editor window binding",
                )
            })?;
        bindings.bind(target.id, binding);
    }
    Ok(bindings)
}

pub(super) fn target_binding_rejection(
    message: &'static str,
) -> editor_shell::EditorCompositionRejection {
    editor_shell::EditorCompositionRejection::single(
        editor_shell::EditorCompositionDiagnosticRecord::error(
            editor_shell::EditorCompositionDiagnosticCode::TargetBindingMismatch,
            editor_shell::EditorCompositionDiagnosticStage::Projection,
            editor_shell::EditorCompositionDiagnosticSubject::General(
                "editor-static-target-binding".to_owned(),
            ),
            message,
        ),
    )
}

impl RunenwerkEditorShellState {
    pub fn editor_windows(&self) -> &EditorWindowRegistry {
        &self.editor_windows
    }

    pub fn editor_window_binding(
        &self,
        editor_window_id: EditorWindowId,
    ) -> Option<EditorWindowPresentationBinding> {
        self.editor_window_bindings.get(&editor_window_id).copied()
    }

    pub fn editor_window_for_binding(
        &self,
        binding: EditorWindowPresentationBinding,
    ) -> Option<EditorWindowId> {
        self.editor_window_bindings
            .iter()
            .find_map(|(window_id, candidate)| (*candidate == binding).then_some(*window_id))
    }

    pub fn remove_editor_window_presentation(&mut self, editor_window_id: EditorWindowId) -> bool {
        if self
            .editor_windows
            .remove_window(editor_window_id)
            .is_none()
        {
            return false;
        }
        self.editor_window_bindings.remove(&editor_window_id);
        self.pending_editor_window_presentations
            .retain(|pending| *pending != editor_window_id);
        true
    }

    pub fn composition_target_binding(
        &self,
        target_id: PresentationTargetId,
    ) -> Option<EditorWindowPresentationBinding> {
        self.composition_target_bindings.binding(target_id).copied()
    }

    pub fn composition_target_bindings(
        &self,
    ) -> impl Iterator<
        Item = crate::shell::EditorCompositionTargetBinding<EditorWindowPresentationBinding>,
    > {
        self.composition_target_bindings.iter().map(|entry| {
            crate::shell::EditorCompositionTargetBinding {
                target_id: entry.target_id,
                binding: *entry.binding,
            }
        })
    }

    pub fn bind_editor_window_presentation(
        &mut self,
        editor_window_id: EditorWindowId,
        binding: EditorWindowPresentationBinding,
    ) -> bool {
        if self.editor_windows.record(editor_window_id).is_none() {
            return false;
        }
        self.editor_window_bindings
            .insert(editor_window_id, binding);
        if editor_window_id == self.editor_windows.primary_window_id() {
            let target_ids = self
                .composition_target_bindings
                .iter()
                .map(|entry| entry.target_id)
                .collect::<Vec<_>>();
            for target_id in target_ids {
                self.composition_target_bindings.bind(target_id, binding);
            }
        }
        true
    }

    pub fn bind_composition_target_presentation(
        &mut self,
        target_id: PresentationTargetId,
        editor_window_id: EditorWindowId,
        binding: EditorWindowPresentationBinding,
    ) -> bool {
        if self.editor_windows.record(editor_window_id).is_none()
            || self
                .composition_runtime
                .composition()
                .definition()
                .targets()
                .iter()
                .all(|target| target.id != target_id)
        {
            return false;
        }
        self.editor_window_bindings
            .insert(editor_window_id, binding);
        self.composition_target_bindings.bind(target_id, binding);
        true
    }

    pub fn drain_pending_editor_window_presentations(&mut self) -> Vec<EditorWindowId> {
        std::mem::take(&mut self.pending_editor_window_presentations)
    }

    pub fn open_editor_window_for_active_workspace(&mut self) -> EditorWindowId {
        let editor_window_id = self.identity_allocator.allocate_editor_window_id();
        self.editor_windows
            .open_secondary_window(editor_window_id, self.workspace_id);
        self.pending_editor_window_presentations
            .push(editor_window_id);
        editor_window_id
    }
}
