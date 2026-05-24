//! File: apps/runenwerk_editor/src/shell/providers/scene/viewport.rs
//! Purpose: Scene viewport provider.

use super::super::*;

pub struct SceneViewportProvider;

impl EditorSurfaceProvider for SceneViewportProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SCENE_VIEWPORT_PROVIDER_ID,
            "Scene Viewport",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        if !matches!(
            request.document_context.resolved_document_kind(),
            Some(DocumentKind::Scene)
        ) {
            return SurfaceProviderSupportMode::Unsupported;
        }
        stable_key_support(request, SCENE_VIEWPORT_SURFACE_KEY)
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let expected_viewport_id = context.viewport_instances.and_then(|instances| {
            instances.viewport_for_tool_surface(request.tool_surface_instance_id)
        });
        let bound_products = context
            .tool_surface_bindings
            .and_then(|bindings| {
                bindings.binding_for_tool_surface(request.tool_surface_instance_id)
            })
            .and_then(|binding| {
                context
                    .viewport_observations
                    .and_then(|observations| observations.frame_for(binding.viewport_id))
            });
        let products = bound_products.or_else(|| {
            expected_viewport_id.and_then(|viewport_id| {
                context
                    .viewport_observations
                    .and_then(|observations| observations.frame_for(viewport_id))
            })
        });
        let tool_state = context.app.viewport_tool_state();
        let viewport_settings = context
            .shell_state
            .workspace_state()
            .tool_surface(request.tool_surface_instance_id)
            .and_then(|surface| surface.viewport_settings);
        let field_visualizer_settings = products
            .map(|products| products.field_visualizer_settings)
            .or_else(|| viewport_settings.map(|settings| settings.field_visualizer_settings))
            .unwrap_or_default();
        let frame = build_viewport_observation_frame(
            products,
            session.viewport_details_visible,
            session.viewport_statistics_visible,
            session.viewport_options_menu_open,
            session.viewport_tools_menu_open,
            session
                .viewport_tool_radial_session
                .map(|radial| radial.anchor_position),
            viewport_settings
                .map(|settings| settings.debug_stage)
                .unwrap_or(editor_viewport::ViewportDebugStage::Scene),
            viewport_settings
                .map(|settings| settings.root_background_opaque)
                .unwrap_or(false),
            field_visualizer_settings,
            context.app.runtime().selected_entity(),
            session.viewport_interaction_state.drag_in_progress(),
            tool_state,
            context.app.runtime().current_scene_reality_version(),
            expected_viewport_id,
            context.frame_metrics,
            context
                .app
                .procgen_runtime()
                .viewport_overlay_status_lines(),
        );
        let view_model = build_viewport_view_model(&frame);
        let root = build_viewport_panel(
            &view_model,
            context.theme,
            request.panel_instance_id,
            Some(request.tool_surface_instance_id),
        );
        let mut routes = SurfaceRouteTable::empty();
        routes.insert(
            surface_widget_id(
                request.tool_surface_instance_id,
                VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                ViewportSurfaceAction::ToggleDetails,
            )),
        );
        routes.insert(
            surface_widget_id(
                request.tool_surface_instance_id,
                VIEWPORT_STATISTICS_TOGGLE_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                ViewportSurfaceAction::ToggleStatistics,
            )),
        );
        routes.insert(
            surface_widget_id(
                request.tool_surface_instance_id,
                VIEWPORT_OPTIONS_BUTTON_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                ViewportSurfaceAction::ToggleOptionsMenu,
            )),
        );
        routes.insert(
            surface_widget_id(
                request.tool_surface_instance_id,
                VIEWPORT_TOOL_RADIAL_BUTTON_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                ViewportSurfaceAction::ToggleToolsMenu,
            )),
        );
        for (index, action) in [
            ViewportSurfaceAction::ActivateSelectTool,
            ViewportSurfaceAction::ActivateTranslateTool,
            ViewportSurfaceAction::ActivateRotateTool,
            ViewportSurfaceAction::ActivateScaleTool,
        ]
        .into_iter()
        .enumerate()
        {
            routes.insert(
                surface_widget_id(
                    request.tool_surface_instance_id,
                    viewport_tool_radial_item_widget_id(index),
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(action)),
            );
        }
        if let Some(viewport_id) = view_model.viewport_id {
            routes.insert(
                surface_widget_id(
                    request.tool_surface_instance_id,
                    VIEWPORT_RESET_CAMERA_WIDGET_ID,
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                    ViewportSurfaceAction::ResetCamera { viewport_id },
                )),
            );
            routes.insert(
                surface_widget_id(
                    request.tool_surface_instance_id,
                    VIEWPORT_ROOT_OPAQUE_TOGGLE_WIDGET_ID,
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                    ViewportSurfaceAction::SetRootBackgroundOpaque {
                        viewport_id,
                        enabled: !view_model.root_background_opaque,
                    },
                )),
            );
            for (index, debug_stage) in editor_viewport::ViewportDebugStage::ALL
                .into_iter()
                .enumerate()
            {
                routes.insert(
                    surface_widget_id(
                        request.tool_surface_instance_id,
                        viewport_debug_stage_button_widget_id(index),
                    ),
                    SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                        ViewportSurfaceAction::SetDebugStage {
                            viewport_id,
                            debug_stage,
                        },
                    )),
                );
            }
            for (index, component) in editor_viewport::ViewportFieldVisualizerComponent::ALL
                .into_iter()
                .enumerate()
            {
                routes.insert(
                    surface_widget_id(
                        request.tool_surface_instance_id,
                        viewport_field_component_button_widget_id(index),
                    ),
                    SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                        ViewportSurfaceAction::PatchFieldVisualizerSettings {
                            viewport_id,
                            patch:
                                editor_viewport::ViewportFieldVisualizerSettingsPatch::SetComponent(
                                    component,
                                ),
                        },
                    )),
                );
            }
            routes.insert(
                surface_widget_id(
                    request.tool_surface_instance_id,
                    VIEWPORT_FIELD_SLICE_DECREMENT_WIDGET_ID,
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                    ViewportSurfaceAction::PatchFieldVisualizerSettings {
                        viewport_id,
                        patch:
                            editor_viewport::ViewportFieldVisualizerSettingsPatch::StepSliceIndex(
                                -1,
                            ),
                    },
                )),
            );
            routes.insert(
                surface_widget_id(
                    request.tool_surface_instance_id,
                    VIEWPORT_FIELD_SLICE_RESET_WIDGET_ID,
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                    ViewportSurfaceAction::PatchFieldVisualizerSettings {
                        viewport_id,
                        patch: editor_viewport::ViewportFieldVisualizerSettingsPatch::SetSliceIndex(
                            0,
                        ),
                    },
                )),
            );
            routes.insert(
                surface_widget_id(
                    request.tool_surface_instance_id,
                    VIEWPORT_FIELD_SLICE_INCREMENT_WIDGET_ID,
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                    ViewportSurfaceAction::PatchFieldVisualizerSettings {
                        viewport_id,
                        patch: editor_viewport::ViewportFieldVisualizerSettingsPatch::StepSliceIndex(
                            1,
                        ),
                    },
                )),
            );
            for (index, color_ramp) in editor_viewport::ViewportFieldVisualizerColorRamp::ALL
                .into_iter()
                .enumerate()
            {
                routes.insert(
                    surface_widget_id(
                        request.tool_surface_instance_id,
                        viewport_field_color_ramp_button_widget_id(index),
                    ),
                    SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                        ViewportSurfaceAction::PatchFieldVisualizerSettings {
                            viewport_id,
                            patch:
                                editor_viewport::ViewportFieldVisualizerSettingsPatch::SetColorRamp(
                                    color_ramp,
                                ),
                        },
                    )),
                );
            }
            for (index, debug_mode) in editor_viewport::ViewportFieldVisualizerDebugMode::ALL
                .into_iter()
                .enumerate()
            {
                routes.insert(
                    surface_widget_id(
                        request.tool_surface_instance_id,
                        viewport_field_debug_mode_button_widget_id(index),
                    ),
                    SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                        ViewportSurfaceAction::PatchFieldVisualizerSettings {
                            viewport_id,
                            patch:
                                editor_viewport::ViewportFieldVisualizerSettingsPatch::SetDebugMode(
                                    debug_mode,
                                ),
                        },
                    )),
                );
            }
        }
        for (index, choice) in view_model.product_choices.iter().enumerate() {
            routes.insert(
                surface_widget_id(
                    request.tool_surface_instance_id,
                    viewport_product_button_widget_id(index),
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::Viewport(
                    ViewportSurfaceAction::SelectProduct {
                        viewport_id: choice.viewport_id,
                        product_id: choice.product_id,
                        enabled: choice.enabled,
                    },
                )),
            );
        }
        Ok(ProviderSurfaceFrame {
            title: "Viewport".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        match action {
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::SelectProduct {
                viewport_id,
                product_id,
                enabled,
            }) if enabled => Ok(Some(editor_domain_proposal(
                request,
                context.projection_epoch,
                EditorDomainMutation::Viewport(ViewportDomainMutation::SelectProduct {
                    viewport_id,
                    product_id,
                }),
            ))),
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::ToggleDetails) => {
                Ok(Some(surface_session_proposal(
                    request,
                    context.projection_epoch,
                    SurfaceSessionMutation::Viewport(ViewportSessionMutation::ToggleDetails),
                )))
            }
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::ToggleStatistics) => {
                Ok(Some(surface_session_proposal(
                    request,
                    context.projection_epoch,
                    SurfaceSessionMutation::Viewport(ViewportSessionMutation::ToggleStatistics),
                )))
            }
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::ToggleOptionsMenu) => {
                Ok(Some(surface_session_proposal(
                    request,
                    context.projection_epoch,
                    SurfaceSessionMutation::Viewport(ViewportSessionMutation::ToggleOptionsMenu),
                )))
            }
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::ToggleToolsMenu) => {
                Ok(Some(surface_session_proposal(
                    request,
                    context.projection_epoch,
                    SurfaceSessionMutation::Viewport(ViewportSessionMutation::ToggleToolsMenu),
                )))
            }
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::ActivateSelectTool) => Ok(Some(
                SurfaceCommandProposal::Shell(ShellCommand::ActivateSelectTool),
            )),
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::ActivateTranslateTool) => Ok(Some(
                SurfaceCommandProposal::Shell(ShellCommand::ActivateTranslateTool),
            )),
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::ActivateRotateTool) => Ok(Some(
                SurfaceCommandProposal::Shell(ShellCommand::ActivateRotateTool),
            )),
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::ActivateScaleTool) => Ok(Some(
                SurfaceCommandProposal::Shell(ShellCommand::ActivateScaleTool),
            )),
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::ResetCamera { viewport_id }) => {
                Ok(Some(editor_domain_proposal(
                    request,
                    context.projection_epoch,
                    EditorDomainMutation::Viewport(ViewportDomainMutation::ResetCamera {
                        viewport_id,
                    }),
                )))
            }
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::SetDebugStage {
                viewport_id,
                debug_stage,
            }) => Ok(Some(editor_domain_proposal(
                request,
                context.projection_epoch,
                EditorDomainMutation::Viewport(ViewportDomainMutation::SetDebugStage {
                    viewport_id,
                    debug_stage,
                }),
            ))),
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::SetRootBackgroundOpaque {
                viewport_id,
                enabled,
            }) => Ok(Some(editor_domain_proposal(
                request,
                context.projection_epoch,
                EditorDomainMutation::Viewport(ViewportDomainMutation::SetRootBackgroundOpaque {
                    viewport_id,
                    enabled,
                }),
            ))),
            SurfaceLocalAction::Viewport(ViewportSurfaceAction::PatchFieldVisualizerSettings {
                viewport_id,
                patch,
            }) => Ok(Some(editor_domain_proposal(
                request,
                context.projection_epoch,
                EditorDomainMutation::Viewport(
                    ViewportDomainMutation::PatchFieldVisualizerSettings { viewport_id, patch },
                ),
            ))),
            _ => Ok(None),
        }
    }
}
