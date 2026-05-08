use editor_core::EntityId;
use editor_shell::{
    EntityTableComponentFilter, EntityTableComponentFilterItem, EntityTableHierarchyFilter,
    EntityTableQuery, EntityTableSort, EntityTableSortKey,
};

use crate::editor_runtime::RunenwerkEditorRuntime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTableRow {
    pub entity: EntityId,
    pub display_name: String,
    pub parent: Option<EntityId>,
    pub component_count: usize,
    pub is_selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTablePanelState {
    pub query: EntityTableQuery,
    pub search_query: String,
    pub sort_key: EntityTableSortKey,
    pub sort_ascending: bool,
    pub component_filters: Vec<EntityTableComponentFilterItem>,
    pub rows: Vec<EntityTableRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTablePanelUiState {
    query: EntityTableQuery,
}

impl Default for EntityTablePanelUiState {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityTablePanelUiState {
    pub fn new() -> Self {
        Self {
            query: EntityTableQuery::default(),
        }
    }

    pub fn query(&self) -> &EntityTableQuery {
        &self.query
    }

    pub fn search_query(&self) -> &str {
        &self.query.search_text
    }

    pub fn append_search_text(&mut self, text: &str) {
        self.query.search_text.push_str(text);
    }

    pub fn backspace_search_query(&mut self) {
        self.query.search_text.pop();
    }

    pub fn clear_search_query(&mut self) {
        self.query.search_text.clear();
    }

    pub fn set_search_query(&mut self, query: impl Into<String>) {
        self.query.search_text = query.into();
    }

    pub fn set_selected_only(&mut self, selected_only: bool) {
        self.query.selected_only = selected_only;
    }

    pub fn set_hierarchy_filter(&mut self, filter: EntityTableHierarchyFilter) {
        self.query.hierarchy_filter = filter;
    }

    pub fn set_component_filter(&mut self, filter: EntityTableComponentFilter) {
        self.query.component_filter = filter;
    }

    pub fn toggle_sort(&mut self, sort_key: EntityTableSortKey) {
        if self.query.sort.key == sort_key {
            self.query.sort.ascending = !self.query.sort.ascending;
        } else {
            self.query.sort = EntityTableSort {
                key: sort_key,
                ascending: true,
            };
        }
    }

    pub fn sort_key(&self) -> EntityTableSortKey {
        self.query.sort.key
    }

    pub fn sort_ascending(&self) -> bool {
        self.query.sort.ascending
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityTablePanelCommand {
    SelectEntity { entity: EntityId },
    AppendSearchText { text: String },
    BackspaceSearchQuery,
    ToggleSort { sort_key: EntityTableSortKey },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTablePanelCommandResult {
    pub state: EntityTablePanelState,
}

pub struct EntityTablePanelPresenter;

impl EntityTablePanelPresenter {
    pub fn build_state(
        runtime: &RunenwerkEditorRuntime,
        ui_state: &EntityTablePanelUiState,
    ) -> EntityTablePanelState {
        let selected_entity = runtime.selected_entity();
        let query = ui_state.query().clone();
        let search = query.search_text.trim().to_ascii_lowercase();
        let mut rows = runtime
            .list_scene_entities()
            .into_iter()
            .map(|entity| EntityTableRow {
                component_count: runtime.list_entity_components(entity.id).len(),
                is_selected: selected_entity == Some(entity.id),
                entity: entity.id,
                display_name: entity.display_name,
                parent: entity.parent,
            })
            .filter(|row| query_matches(runtime, row, &query, &search))
            .collect::<Vec<_>>();

        sort_rows(&mut rows, query.sort.key, query.sort.ascending);

        EntityTablePanelState {
            search_query: query.search_text.clone(),
            sort_key: query.sort.key,
            sort_ascending: query.sort.ascending,
            component_filters: component_filter_items(runtime),
            query,
            rows,
        }
    }
}

fn query_matches(
    runtime: &RunenwerkEditorRuntime,
    row: &EntityTableRow,
    query: &EntityTableQuery,
    normalized_search: &str,
) -> bool {
    if query.selected_only && !row.is_selected {
        return false;
    }
    if query.hierarchy_filter == EntityTableHierarchyFilter::RootsOnly && row.parent.is_some() {
        return false;
    }
    match query.component_filter {
        EntityTableComponentFilter::All => {}
        EntityTableComponentFilter::Has(component_type) => {
            if !runtime.entity_has_component(row.entity, component_type) {
                return false;
            }
        }
    }
    search_matches(row, normalized_search)
}

fn search_matches(row: &EntityTableRow, search: &str) -> bool {
    if search.is_empty() {
        return true;
    }
    row.display_name.to_ascii_lowercase().contains(search)
        || row.entity.0.to_string().contains(search)
        || row
            .parent
            .map(|parent| parent.0.to_string().contains(search))
            .unwrap_or(false)
}

fn component_filter_items(runtime: &RunenwerkEditorRuntime) -> Vec<EntityTableComponentFilterItem> {
    let mut items = Vec::with_capacity(runtime.list_registered_component_types().len() + 1);
    items.push(EntityTableComponentFilterItem {
        filter: EntityTableComponentFilter::All,
        label: "All components".to_string(),
    });
    items.extend(runtime.list_registered_component_types().into_iter().map(
        |(component_type, display_name)| EntityTableComponentFilterItem {
            filter: EntityTableComponentFilter::Has(component_type),
            label: display_name,
        },
    ));
    items
}

fn sort_rows(rows: &mut [EntityTableRow], sort_key: EntityTableSortKey, ascending: bool) {
    rows.sort_by(|left, right| {
        let ordering = match sort_key {
            EntityTableSortKey::EntityId => left.entity.cmp(&right.entity),
            EntityTableSortKey::DisplayName => left
                .display_name
                .cmp(&right.display_name)
                .then_with(|| left.entity.cmp(&right.entity)),
            EntityTableSortKey::Parent => left
                .parent
                .cmp(&right.parent)
                .then_with(|| left.display_name.cmp(&right.display_name)),
            EntityTableSortKey::ComponentCount => left
                .component_count
                .cmp(&right.component_count)
                .then_with(|| left.display_name.cmp(&right.display_name)),
        };
        if ascending {
            ordering
        } else {
            ordering.reverse()
        }
    });
}
