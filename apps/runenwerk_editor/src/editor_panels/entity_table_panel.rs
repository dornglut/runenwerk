use editor_core::EntityId;
use editor_shell::EntityTableSortKey;

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
    pub search_query: String,
    pub sort_key: EntityTableSortKey,
    pub sort_ascending: bool,
    pub rows: Vec<EntityTableRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTablePanelUiState {
    search_query: String,
    sort_key: EntityTableSortKey,
    sort_ascending: bool,
}

impl Default for EntityTablePanelUiState {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityTablePanelUiState {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            sort_key: EntityTableSortKey::DisplayName,
            sort_ascending: true,
        }
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn append_search_text(&mut self, text: &str) {
        self.search_query.push_str(text);
    }

    pub fn backspace_search_query(&mut self) {
        self.search_query.pop();
    }

    pub fn set_search_query(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
    }

    pub fn toggle_sort(&mut self, sort_key: EntityTableSortKey) {
        if self.sort_key == sort_key {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_key = sort_key;
            self.sort_ascending = true;
        }
    }

    pub fn sort_key(&self) -> EntityTableSortKey {
        self.sort_key
    }

    pub fn sort_ascending(&self) -> bool {
        self.sort_ascending
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
        let search = ui_state.search_query().trim().to_ascii_lowercase();
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
            .filter(|row| search_matches(row, &search))
            .collect::<Vec<_>>();

        sort_rows(&mut rows, ui_state.sort_key(), ui_state.sort_ascending());

        EntityTablePanelState {
            search_query: ui_state.search_query().to_string(),
            sort_key: ui_state.sort_key(),
            sort_ascending: ui_state.sort_ascending(),
            rows,
        }
    }
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
