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
            SCENE_ENTITY_TABLE_SURFACE_KEY,
            ToolSurfaceKind::EntityTable,
        )
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
        let root = build_entity_table_panel(
            &view_model,
            context.theme,
            request.panel_instance_id,
            Some(request.tool_surface_instance_id),
        );
        let mut routes = SurfaceRouteTable::empty();
        routes.insert(
            surface_widget_id(
                request.tool_surface_instance_id,
                ENTITY_TABLE_LIST_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::EntityTable(
                EntityTableSurfaceAction::SelectRow {
                    entities: view_model.rows.iter().map(|row| row.entity).collect(),
                },
            )),
        );
        routes.insert(
            surface_widget_id(
                request.tool_surface_instance_id,
                ENTITY_TABLE_SEARCH_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::EntityTable(
                EntityTableSurfaceAction::AppendSearchText {
                    text: String::new(),
                },
            )),
        );
        routes.insert(
            surface_widget_id(
                request.tool_surface_instance_id,
                ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::EntityTable(
                EntityTableSurfaceAction::ClearSearch,
            )),
        );
        routes.insert(
            surface_widget_id(
                request.tool_surface_instance_id,
                ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::EntityTable(
                EntityTableSurfaceAction::SetSelectedOnly {
                    selected_only: view_model.query.selected_only,
                },
            )),
        );
        routes.insert(
            surface_widget_id(
                request.tool_surface_instance_id,
                ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::EntityTable(
                EntityTableSurfaceAction::SetHierarchyFilter {
                    filter: view_model.query.hierarchy_filter,
                },
            )),
        );
        routes.insert(
            surface_widget_id(
                request.tool_surface_instance_id,
                ENTITY_TABLE_COMPONENT_FILTER_SELECT_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::EntityTable(
                EntityTableSurfaceAction::SelectComponentFilter {
                    filters: view_model
                        .component_filters
                        .iter()
                        .map(|item| item.filter)
                        .collect(),
                },
            )),
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
                surface_widget_id(
                    request.tool_surface_instance_id,
                    entity_table_sort_button_widget_id(index),
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::EntityTable(
                    EntityTableSurfaceAction::ToggleSort { sort_key },
                )),
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
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SelectEntity { entity }) => {
                Ok(Some(editor_domain_proposal(
                    request,
                    projection_epoch,
                    EditorDomainMutation::EntityTable(EntityTableDomainMutation::SelectRow {
                        entities: vec![entity],
                    }),
                )))
            }
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SelectRow { entities }) => {
                Ok(Some(editor_domain_proposal(
                    request,
                    projection_epoch,
                    EditorDomainMutation::EntityTable(EntityTableDomainMutation::SelectRow {
                        entities,
                    }),
                )))
            }
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::AppendSearchText {
                text,
            }) => Ok(Some(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::EntityTable(EntityTableSessionMutation::AppendSearchText {
                    text,
                }),
            ))),
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::BackspaceSearch) => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::EntityTable(
                        EntityTableSessionMutation::BackspaceSearch,
                    ),
                )))
            }
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::ClearSearch) => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::EntityTable(EntityTableSessionMutation::ClearSearch),
                )))
            }
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SetSelectedOnly {
                selected_only,
            }) => Ok(Some(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::EntityTable(EntityTableSessionMutation::SetSelectedOnly {
                    selected_only,
                }),
            ))),
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SetHierarchyFilter {
                filter,
            }) => Ok(Some(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::EntityTable(
                    EntityTableSessionMutation::SetHierarchyFilter { filter },
                ),
            ))),
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SetComponentFilter {
                filter,
            }) => Ok(Some(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::EntityTable(
                    EntityTableSessionMutation::SetComponentFilter { filter },
                ),
            ))),
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SelectComponentFilter {
                ..
            }) => Ok(None),
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::ToggleSort { sort_key }) => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::EntityTable(EntityTableSessionMutation::ToggleSort {
                        sort_key,
                    }),
                )))
            }
            _ => Ok(None),
        }
    }
}
