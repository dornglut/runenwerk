//! File: apps/runenwerk_editor/src/shell/providers/scene/inspector.rs
//! Purpose: Scene inspector provider.

use super::super::*;

pub struct SceneInspectorProvider;

impl EditorSurfaceProvider for SceneInspectorProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SCENE_INSPECTOR_PROVIDER_ID,
            "Scene Inspector",
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
        stable_key_or_legacy_kind_support(
            request,
            SCENE_INSPECTOR_SURFACE_KEY,
            ToolSurfaceKind::Inspector,
        )
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let panel_model = InspectorPanelPresenter::build_view_model(
            context.app.runtime(),
            &session.inspector_ui_state,
        );
        let frame = build_inspector_observation_frame(
            &panel_model,
            context.app.runtime().current_scene_reality_version(),
        );
        let view_model = build_inspector_view_model(&frame);
        let root = build_inspector_panel(
            &view_model,
            context.theme,
            request.panel_instance_id,
            Some(request.tool_surface_instance_id),
        );
        let mut routes = SurfaceRouteTable::empty();
        for (index, field) in view_model.fields.iter().enumerate() {
            let action = match &field.control {
                InspectorFieldControlKind::BoolToggle { checked } => Some(
                    SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldBool {
                        index,
                        value: *checked,
                    }),
                ),
                InspectorFieldControlKind::IntegerInput { value } => Some(
                    SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldNumber {
                        index,
                        value: *value as f64,
                    }),
                ),
                InspectorFieldControlKind::FloatInput { value } => Some(
                    SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldNumber {
                        index,
                        value: *value,
                    }),
                ),
                InspectorFieldControlKind::TextInput => Some(SurfaceLocalAction::Inspector(
                    InspectorSurfaceAction::EditFieldText {
                        index,
                        text: String::new(),
                    },
                )),
                InspectorFieldControlKind::ReadOnly
                | InspectorFieldControlKind::Group
                | InspectorFieldControlKind::Unsupported => Some(SurfaceLocalAction::Inspector(
                    InspectorSurfaceAction::ActivateField { index },
                )),
                InspectorFieldControlKind::EnumSelect { options, .. } => Some(
                    SurfaceLocalAction::Inspector(InspectorSurfaceAction::SelectFieldEnum {
                        index,
                        options: options.clone(),
                    }),
                ),
            };
            if let Some(action) = action {
                routes.insert(
                    surface_widget_id(
                        request.tool_surface_instance_id,
                        inspector_field_widget_id(index),
                    ),
                    SurfaceLocalRoute::new(action),
                );
            }
            if matches!(field.control, InspectorFieldControlKind::TextInput) {
                routes.insert(
                    surface_widget_id(
                        request.tool_surface_instance_id,
                        inspector_field_focus_widget_id(index),
                    ),
                    SurfaceLocalRoute::new(SurfaceLocalAction::Inspector(
                        InspectorSurfaceAction::FocusField { index },
                    )),
                );
            }
        }
        Ok(ProviderSurfaceFrame {
            title: "Inspector".to_string(),
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
        let projection_epoch = context.projection_epoch;
        match action {
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::ActivateField { index }) => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::Inspector(InspectorSessionMutation::ActivateField {
                        index,
                    }),
                )))
            }
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::FocusField { index }) => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::Inspector(InspectorSessionMutation::FocusField {
                        index,
                    }),
                )))
            }
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::EditFieldText {
                index,
                text,
            }) => Ok(Some(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::Inspector(InspectorSessionMutation::AppendFieldText {
                    index,
                    text,
                }),
            ))),
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::BackspaceFieldText { index }) => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::Inspector(
                        InspectorSessionMutation::BackspaceFieldText { index },
                    ),
                )))
            }
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::CommitFieldText { index }) => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::Inspector(InspectorSessionMutation::CommitFieldText {
                        index,
                    }),
                )))
            }
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::CancelFieldText { index }) => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::Inspector(InspectorSessionMutation::CancelFieldText {
                        index,
                    }),
                )))
            }
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldBool {
                index,
                value,
            }) => Ok(Some(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::Inspector(InspectorSessionMutation::SetFieldBool {
                    index,
                    value,
                }),
            ))),
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldNumber {
                index,
                value,
            }) => Ok(Some(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::Inspector(InspectorSessionMutation::SetFieldNumber {
                    index,
                    value,
                }),
            ))),
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldEnum {
                index,
                value,
            }) => Ok(Some(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::Inspector(InspectorSessionMutation::SetFieldEnum {
                    index,
                    value,
                }),
            ))),
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::SelectFieldEnum { .. }) => {
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}
