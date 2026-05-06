//! File: apps/runenwerk_editor/src/shell/providers/scene/entity_table.rs
//! Purpose: Scene entity table provider.

use super::super::*;

pub struct SceneEntityTableProvider;

impl EditorSurfaceProvider for SceneEntityTableProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SCENE_ENTITY_TABLE_PROVIDER_ID,
            "Scene Entity Table",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        matches!(
            request.document_context.resolved_document_kind(),
            Some(DocumentKind::Scene)
        ) && request.tool_surface_kind == ToolSurfaceKind::EntityTable
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let state = EntityTablePanelPresenter::build_state(
            context.app.runtime(),
            &session.entity_table_ui_state,
        );
        let view_model = build_entity_table_view_model(&state);
        let root = remap_surface_node_ids(
            build_entity_table_panel(
                &view_model,
                context.theme,
                request.panel_instance_id,
                Some(request.tool_surface_instance_id),
            ),
            request.tool_surface_instance_id,
        );
        let mut routes = SurfaceRouteTable::empty();
        routes.insert(
            remap_widget_id(
                request.tool_surface_instance_id,
                ENTITY_TABLE_LIST_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::SelectEntityTableRow {
                entities: view_model.rows.iter().map(|row| row.entity).collect(),
            }),
        );
        routes.insert(
            remap_widget_id(
                request.tool_surface_instance_id,
                ENTITY_TABLE_SEARCH_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::AppendEntityTableSearchText {
                text: String::new(),
            }),
        );
        for (index, sort_key) in [
            EntityTableSortKey::EntityId,
            EntityTableSortKey::DisplayName,
            EntityTableSortKey::Parent,
            EntityTableSortKey::ComponentCount,
        ]
        .into_iter()
        .enumerate()
        {
            routes.insert(
                remap_widget_id(
                    request.tool_surface_instance_id,
                    entity_table_sort_button_widget_id(index),
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::ToggleEntityTableSort { sort_key }),
            );
        }
        Ok(ProviderSurfaceFrame {
            title: "Entities".to_string(),
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
            SurfaceLocalAction::SelectEntityTableEntity { entity } => {
                Ok(Some(editor_domain_proposal(
                    request,
                    projection_epoch,
                    EditorDomainMutation::SelectEntityTableRow {
                        entities: vec![entity],
                    },
                )))
            }
            SurfaceLocalAction::SelectEntityTableRow { entities } => {
                Ok(Some(editor_domain_proposal(
                    request,
                    projection_epoch,
                    EditorDomainMutation::SelectEntityTableRow { entities },
                )))
            }
            SurfaceLocalAction::AppendEntityTableSearchText { text } => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::AppendEntityTableSearchText { text },
                )))
            }
            SurfaceLocalAction::BackspaceEntityTableSearch => Ok(Some(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::BackspaceEntityTableSearch,
            ))),
            SurfaceLocalAction::ToggleEntityTableSort { sort_key } => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::ToggleEntityTableSort { sort_key },
                )))
            }
            _ => Ok(None),
        }
    }
}
