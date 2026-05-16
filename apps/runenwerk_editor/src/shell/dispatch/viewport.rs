//! File: apps/runenwerk_editor/src/shell/dispatch/viewport.rs
//! Purpose: Viewport surface command dispatch.

use editor_core::EditorMutationError;
use editor_shell::{
    StructuralCommandTarget, ToolSurfaceKind, ViewportDomainMutation, ViewportSessionMutation,
};
use editor_viewport::{
    ArtifactObservationFrame, ExpressionProductId, ViewportFieldVisualizerSettingsPatch,
    ViewportId, ViewportPresentationState,
};
use ui_surface::{
    ObservationFrame, RatificationAdapter, RatificationDispatchError, RatificationOutcome,
    SessionScopeHandle, SurfaceCapability, SurfaceCapabilitySet, SurfaceIntent, SurfaceIntentKind,
    SurfacePresentationModel, ratify_surface_intent,
};

use crate::editor_app::RunenwerkEditorApp;
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportPresentationStateResource, ViewportRenderStateCommand,
    ViewportRenderStateCommandQueueResource,
};
use crate::shell::RunenwerkEditorShellState;
use crate::shell::dispatch::{
    resolve_surface_command_contract, surface_capability_label, tool_surface_kind_label,
};
use crate::shell::surface_session::ViewportToolRadialSession;

pub(crate) fn dispatch_session_mutation(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    mutation: ViewportSessionMutation,
) -> Result<(), EditorMutationError> {
    let Some(surface_contract) =
        resolve_surface_command_contract(shell_state, target, ToolSurfaceKind::Viewport)
    else {
        app.append_console_line(
            "[viewport] session mutation ignored (missing structural tool-surface target)"
                .to_string(),
        );
        return Ok(());
    };
    if surface_contract.tool_surface_kind != ToolSurfaceKind::Viewport {
        app.append_console_line(format!(
            "[viewport] session mutation ignored (surface-kind mismatch): expected=viewport actual={}",
            tool_surface_kind_label(surface_contract.tool_surface_kind),
        ));
        return Ok(());
    }
    let Some(surface_id) = target.active_tool_surface else {
        app.append_console_line(
            "[viewport] session mutation ignored (missing tool-surface session target)".to_string(),
        );
        return Ok(());
    };
    let session = app.surface_sessions_mut().session_mut(surface_id);
    match mutation {
        ViewportSessionMutation::ToggleDetails => {
            session.viewport_details_visible = !session.viewport_details_visible;
        }
        ViewportSessionMutation::ToggleStatistics => {
            session.viewport_statistics_visible = !session.viewport_statistics_visible;
        }
        ViewportSessionMutation::ToggleOptionsMenu => {
            session.viewport_options_menu_open = !session.viewport_options_menu_open;
            if session.viewport_options_menu_open {
                session.viewport_tools_menu_open = false;
                session.viewport_tool_radial_session = None;
            }
        }
        ViewportSessionMutation::ToggleToolsMenu => {
            session.viewport_tools_menu_open = !session.viewport_tools_menu_open;
            if session.viewport_tools_menu_open {
                session.viewport_options_menu_open = false;
                session.viewport_tool_radial_session = None;
            }
        }
        ViewportSessionMutation::OpenToolRadialMenu {
            viewport_id,
            anchor_position,
            opened_by_tab_hold,
        } => {
            session.viewport_options_menu_open = false;
            session.viewport_tools_menu_open = false;
            session.viewport_tool_radial_session = Some(ViewportToolRadialSession {
                tool_surface_id: surface_id,
                viewport_id,
                anchor_position,
                opened_by_tab_hold,
            });
        }
        ViewportSessionMutation::CloseToolRadialMenu => {
            session.viewport_tool_radial_session = None;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_domain_mutation(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    mutation: ViewportDomainMutation,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    viewport_render_commands: Option<&mut ViewportRenderStateCommandQueueResource>,
) -> Result<(), EditorMutationError> {
    match mutation {
        ViewportDomainMutation::SelectProduct {
            viewport_id,
            product_id,
        } => dispatch_select_product(
            app,
            shell_state,
            target,
            viewport_id,
            product_id,
            viewport_presentations,
            viewport_observations,
            tool_surface_bindings,
        ),
        ViewportDomainMutation::ResetCamera { viewport_id } => {
            let Some(resolved_viewport_id) = resolve_viewport_state_command_target(
                app,
                shell_state,
                tool_surface_bindings,
                target,
                viewport_id,
                "camera reset",
            ) else {
                return Ok(());
            };
            let Some(queue) = viewport_render_commands else {
                app.append_console_line(
                    "[viewport] camera reset ignored (missing render command queue)".to_string(),
                );
                return Ok(());
            };
            queue.push(ViewportRenderStateCommand::ResetCamera {
                viewport_id: resolved_viewport_id,
            });
            Ok(())
        }
        ViewportDomainMutation::SetDebugStage {
            viewport_id,
            debug_stage,
        } => {
            let Some(resolved_viewport_id) = resolve_viewport_state_command_target(
                app,
                shell_state,
                tool_surface_bindings,
                target,
                viewport_id,
                "debug stage",
            ) else {
                return Ok(());
            };
            let Some(queue) = viewport_render_commands else {
                app.append_console_line(
                    "[viewport] debug stage ignored (missing render command queue)".to_string(),
                );
                return Ok(());
            };
            queue.push(ViewportRenderStateCommand::SetDebugStage {
                viewport_id: resolved_viewport_id,
                debug_stage,
            });
            Ok(())
        }
        ViewportDomainMutation::SetRootBackgroundOpaque {
            viewport_id,
            enabled,
        } => {
            let Some(resolved_viewport_id) = resolve_viewport_state_command_target(
                app,
                shell_state,
                tool_surface_bindings,
                target,
                viewport_id,
                "root opacity",
            ) else {
                return Ok(());
            };
            let Some(queue) = viewport_render_commands else {
                app.append_console_line(
                    "[viewport] root opacity ignored (missing render command queue)".to_string(),
                );
                return Ok(());
            };
            queue.push(ViewportRenderStateCommand::SetRootBackgroundOpaque {
                viewport_id: resolved_viewport_id,
                enabled,
            });
            Ok(())
        }
        ViewportDomainMutation::PatchFieldVisualizerSettings { viewport_id, patch } => {
            dispatch_patch_field_visualizer_settings(
                app,
                shell_state,
                target,
                viewport_id,
                patch,
                viewport_presentations,
                viewport_observations,
                tool_surface_bindings,
            )
        }
    }
}

fn dispatch_patch_field_visualizer_settings(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    viewport_id: ViewportId,
    patch: ViewportFieldVisualizerSettingsPatch,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
) -> Result<(), EditorMutationError> {
    let Some(resolved_viewport_id) = resolve_viewport_state_command_target(
        app,
        shell_state,
        tool_surface_bindings,
        target,
        viewport_id,
        "field visualizer settings",
    ) else {
        return Ok(());
    };
    let Some(viewport_presentations) = viewport_presentations else {
        app.append_console_line(
            "[viewport] field visualizer patch ignored (missing presentation state)".to_string(),
        );
        return Ok(());
    };
    let mut state = viewport_presentations
        .state_for(resolved_viewport_id)
        .cloned()
        .unwrap_or_else(|| {
            crate::runtime::viewport::initial_presentation_state(resolved_viewport_id)
        });
    let slice_count = viewport_observations
        .and_then(|observations| observations.frame_for(resolved_viewport_id))
        .and_then(|frame| {
            frame
                .available_products
                .iter()
                .find(|descriptor| descriptor.id == state.selected_primary_product_id)
                .and_then(|descriptor| descriptor.channel_layer_slice.as_ref())
                .and_then(|metadata| metadata.slice_count)
        });
    let settings = patch.apply_to(state.field_visualizer_settings, slice_count);
    state.set_field_visualizer_settings(settings);
    viewport_presentations.upsert_state(state);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn dispatch_select_product(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    viewport_id: ViewportId,
    product_id: ExpressionProductId,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
) -> Result<(), EditorMutationError> {
    let (Some(viewport_presentations), Some(viewport_observations), Some(tool_surface_bindings)) = (
        viewport_presentations,
        viewport_observations,
        tool_surface_bindings,
    ) else {
        app.append_console_line(
            "[viewport.binding] product selection ignored (missing runtime binding context)"
                .to_string(),
        );
        return Ok(());
    };

    let resolved_binding = match tool_surface_bindings.resolve_command_target(target, viewport_id) {
        Ok(binding) => binding,
        Err(error) => {
            app.append_console_line(format!(
                "[viewport.binding] product selection ignored: {error}"
            ));
            return Ok(());
        }
    };
    let resolved_viewport_id = resolved_binding.viewport_id;
    let Some(observation_frame) = viewport_observations.frame_for(resolved_viewport_id) else {
        app.append_console_line(format!(
            "[viewport] product selection ignored (missing observation frame): viewport={}",
            resolved_viewport_id.0
        ));
        return Ok(());
    };
    let Some(surface_contract) =
        resolve_surface_command_contract(shell_state, target, ToolSurfaceKind::Viewport)
    else {
        app.append_console_line(
            "[viewport] product selection ignored (missing structural tool-surface target)"
                .to_string(),
        );
        return Ok(());
    };
    if surface_contract.tool_surface_kind != ToolSurfaceKind::Viewport {
        app.append_console_line(format!(
            "[viewport] product selection ignored (surface-kind mismatch): expected=viewport actual={}",
            tool_surface_kind_label(surface_contract.tool_surface_kind),
        ));
        return Ok(());
    }
    let capabilities = surface_contract.capabilities;
    for required_capability in [SurfaceCapability::Observe, SurfaceCapability::Interact] {
        if !capabilities.allows(required_capability) {
            app.append_console_line(format!(
                "[viewport] product selection ignored (missing capability): viewport={} product={} capability={}",
                resolved_viewport_id.0,
                product_id.0,
                surface_capability_label(required_capability),
            ));
            return Ok(());
        }
    }

    let presentation_model = build_viewport_surface_presentation_model(observation_frame);
    if app.debug_logs_enabled() {
        app.append_console_line(format!(
            "[surface.flow] observation viewport={} selected={:?}",
            resolved_viewport_id.0,
            observation_frame
                .selected_primary_product_id
                .map(|value| value.0),
        ));
        app.append_console_line(format!(
            "[surface.flow] presentation viewport={} selectable={:?}",
            resolved_viewport_id.0,
            presentation_model
                .selectable_primary_items()
                .map(|value| value.0)
                .collect::<Vec<_>>(),
        ));
    }
    if !presentation_model.is_primary_selectable(product_id) {
        app.append_console_line(format!(
            "[viewport] product selection ignored (unavailable): viewport={} product={}",
            resolved_viewport_id.0, product_id.0
        ));
        return Ok(());
    }

    let _session_scope = SessionScopeHandle::new(
        surface_contract.surface_instance_id,
        resolved_viewport_id.0,
        surface_contract.retention_class,
    );
    let mut ratification_adapter = ViewportProductSelectionRatificationAdapter::new(
        viewport_presentations,
        resolved_viewport_id,
        capabilities,
    );
    let intent =
        SurfaceIntent::select_primary_item(surface_contract.surface_instance_id, product_id.0);
    if app.debug_logs_enabled() {
        app.append_console_line(format!(
            "[surface.flow] intent viewport={} surface_instance={} kind=SelectPrimaryItem item_id={}",
            resolved_viewport_id.0,
            intent.surface_instance_id.raw(),
            product_id.0,
        ));
    }

    match ratify_surface_intent(&mut ratification_adapter, intent) {
        Ok(RatificationOutcome::Applied) => {
            if app.debug_logs_enabled() {
                app.append_console_line(format!(
                    "[surface.flow] ratification viewport={} outcome=applied",
                    resolved_viewport_id.0
                ));
            }
        }
        Ok(RatificationOutcome::Ignored) => {
            if app.debug_logs_enabled() {
                app.append_console_line(format!(
                    "[surface.flow] ratification viewport={} outcome=ignored",
                    resolved_viewport_id.0
                ));
            }
        }
        Err(RatificationDispatchError::MissingCapability(capability)) => {
            app.append_console_line(format!(
                "[viewport] product selection ignored (missing capability): viewport={} product={} capability={}",
                resolved_viewport_id.0,
                product_id.0,
                surface_capability_label(capability),
            ));
        }
        Err(RatificationDispatchError::Adapter(error)) => return Err(error),
    }
    Ok(())
}

fn resolve_viewport_state_command_target(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&RunenwerkEditorShellState>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    target: StructuralCommandTarget,
    viewport_id: ViewportId,
    command_label: &'static str,
) -> Option<ViewportId> {
    let Some(surface_contract) =
        resolve_surface_command_contract(shell_state, target, ToolSurfaceKind::Viewport)
    else {
        app.append_console_line(format!(
            "[viewport] {command_label} ignored (missing structural tool-surface target)"
        ));
        return None;
    };
    if surface_contract.tool_surface_kind != ToolSurfaceKind::Viewport {
        app.append_console_line(format!(
            "[viewport] {command_label} ignored (surface-kind mismatch): expected=viewport actual={}",
            tool_surface_kind_label(surface_contract.tool_surface_kind),
        ));
        return None;
    }
    for required_capability in [SurfaceCapability::Observe, SurfaceCapability::Interact] {
        if !surface_contract.capabilities.allows(required_capability) {
            app.append_console_line(format!(
                "[viewport] {command_label} ignored (missing capability): viewport={} capability={}",
                viewport_id.0,
                surface_capability_label(required_capability),
            ));
            return None;
        }
    }
    let Some(tool_surface_bindings) = tool_surface_bindings else {
        app.append_console_line(format!(
            "[viewport.binding] {command_label} ignored (missing runtime binding context)"
        ));
        return None;
    };
    match tool_surface_bindings.resolve_command_target(target, viewport_id) {
        Ok(binding) => Some(binding.viewport_id),
        Err(error) => {
            app.append_console_line(format!(
                "[viewport.binding] {command_label} ignored: {error}"
            ));
            None
        }
    }
}

struct ArtifactObservationFrameAdapter<'a> {
    frame: &'a ArtifactObservationFrame,
}

impl ObservationFrame<ExpressionProductId> for ArtifactObservationFrameAdapter<'_> {
    fn selected_primary_item(&self) -> Option<ExpressionProductId> {
        self.frame.selected_primary_product_id
    }

    fn is_item_available(&self, item_id: ExpressionProductId) -> bool {
        self.frame
            .availability_by_product
            .get(&item_id)
            .copied()
            .map(|availability| {
                availability == editor_viewport::ProductAvailabilityState::Available
            })
            .unwrap_or(false)
    }
}

fn build_viewport_surface_presentation_model(
    observation_frame: &ArtifactObservationFrame,
) -> SurfacePresentationModel<ExpressionProductId> {
    let adapter = ArtifactObservationFrameAdapter {
        frame: observation_frame,
    };
    SurfacePresentationModel::from_observation_frame(
        &adapter,
        observation_frame.availability_by_product.keys().copied(),
    )
}

struct ViewportProductSelectionRatificationAdapter<'a> {
    viewport_presentations: &'a mut ViewportPresentationStateResource,
    viewport_id: ViewportId,
    capabilities: SurfaceCapabilitySet,
}

impl<'a> ViewportProductSelectionRatificationAdapter<'a> {
    fn new(
        viewport_presentations: &'a mut ViewportPresentationStateResource,
        viewport_id: ViewportId,
        capabilities: SurfaceCapabilitySet,
    ) -> Self {
        Self {
            viewport_presentations,
            viewport_id,
            capabilities,
        }
    }
}

impl RatificationAdapter for ViewportProductSelectionRatificationAdapter<'_> {
    type Error = EditorMutationError;

    fn has_capability(&self, capability: SurfaceCapability) -> bool {
        self.capabilities.allows(capability)
    }

    fn ratify_intent(&mut self, intent: SurfaceIntent) -> Result<RatificationOutcome, Self::Error> {
        let item_id = match intent.kind {
            SurfaceIntentKind::SelectPrimaryItem { item_id } => item_id,
            SurfaceIntentKind::SelectEntity { .. } | SurfaceIntentKind::ActivateField { .. } => {
                return Ok(RatificationOutcome::Ignored);
            }
        };
        let product_id = ExpressionProductId(item_id);
        if let Some(state) = self.viewport_presentations.state_for_mut(self.viewport_id) {
            state.select_primary_product(product_id);
        } else {
            self.viewport_presentations
                .upsert_state(ViewportPresentationState::new(self.viewport_id, product_id));
        }
        Ok(RatificationOutcome::Applied)
    }
}
