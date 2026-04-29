use editor_shell::{EntityTableRowViewModel, EntityTableViewModel};

use crate::editor_panels::EntityTablePanelState;

pub fn build_entity_table_view_model(state: &EntityTablePanelState) -> EntityTableViewModel {
    EntityTableViewModel {
        search_query: state.search_query.clone(),
        sort_key: state.sort_key,
        sort_ascending: state.sort_ascending,
        rows: state
            .rows
            .iter()
            .map(|row| EntityTableRowViewModel {
                entity: row.entity,
                entity_id_label: row.entity.0.to_string(),
                display_name: row.display_name.clone(),
                parent_label: row
                    .parent
                    .map(|parent| parent.0.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                component_count: row.component_count,
                is_selected: row.is_selected,
            })
            .collect(),
    }
}
