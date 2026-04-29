//! File: domain/editor/editor_shell/src/view_models/entity_table.rs
//! Purpose: Entity table shell view model.

use editor_core::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityTableSortKey {
    EntityId,
    DisplayName,
    Parent,
    ComponentCount,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTableRowViewModel {
    pub entity: EntityId,
    pub entity_id_label: String,
    pub display_name: String,
    pub parent_label: String,
    pub component_count: usize,
    pub is_selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTableViewModel {
    pub search_query: String,
    pub sort_key: EntityTableSortKey,
    pub sort_ascending: bool,
    pub rows: Vec<EntityTableRowViewModel>,
}

impl Default for EntityTableViewModel {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            sort_key: EntityTableSortKey::DisplayName,
            sort_ascending: true,
            rows: Vec::new(),
        }
    }
}
