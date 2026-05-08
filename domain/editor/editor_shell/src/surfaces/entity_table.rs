//! File: domain/editor/editor_shell/src/surfaces/entity_table.rs
//! Purpose: Entity table surface workflow contracts.

use editor_core::{ComponentTypeId, EntityId};

use crate::EntityTableSortKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityTableSort {
    pub key: EntityTableSortKey,
    pub ascending: bool,
}

impl Default for EntityTableSort {
    fn default() -> Self {
        Self {
            key: EntityTableSortKey::DisplayName,
            ascending: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntityTableHierarchyFilter {
    #[default]
    All,
    RootsOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntityTableComponentFilter {
    #[default]
    All,
    Has(ComponentTypeId),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EntityTableQuery {
    pub search_text: String,
    pub selected_only: bool,
    pub hierarchy_filter: EntityTableHierarchyFilter,
    pub component_filter: EntityTableComponentFilter,
    pub sort: EntityTableSort,
}

impl EntityTableQuery {
    pub fn with_sort(mut self, key: EntityTableSortKey, ascending: bool) -> Self {
        self.sort = EntityTableSort { key, ascending };
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EntityTableFilter {
    pub selected_only: bool,
    pub hierarchy_filter: EntityTableHierarchyFilter,
    pub component_filter: EntityTableComponentFilter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTableComponentFilterItem {
    pub filter: EntityTableComponentFilter,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityTableSurfaceAction {
    SelectEntity {
        entity: EntityId,
    },
    SelectRow {
        entities: Vec<EntityId>,
    },
    AppendSearchText {
        text: String,
    },
    BackspaceSearch,
    ClearSearch,
    SetSelectedOnly {
        selected_only: bool,
    },
    SetHierarchyFilter {
        filter: EntityTableHierarchyFilter,
    },
    SetComponentFilter {
        filter: EntityTableComponentFilter,
    },
    SelectComponentFilter {
        filters: Vec<EntityTableComponentFilter>,
    },
    ToggleSort {
        sort_key: EntityTableSortKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityTableSessionMutation {
    AppendSearchText { text: String },
    BackspaceSearch,
    ClearSearch,
    SetSelectedOnly { selected_only: bool },
    SetHierarchyFilter { filter: EntityTableHierarchyFilter },
    SetComponentFilter { filter: EntityTableComponentFilter },
    ToggleSort { sort_key: EntityTableSortKey },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityTableDomainMutation {
    SelectRow { entities: Vec<EntityId> },
}
