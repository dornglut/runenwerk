use std::collections::BTreeSet;

use editor_core::{ComponentTypeId, EntityId};
use editor_inspector::{InspectorEditValue, InspectorPath};

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorFieldDraft {
    pub entity: EntityId,
    pub component_type: ComponentTypeId,
    pub path: InspectorPath,
    pub value: InspectorEditValue,
}

impl InspectorFieldDraft {
    pub fn new(
        entity: EntityId,
        component_type: ComponentTypeId,
        path: InspectorPath,
        value: InspectorEditValue,
    ) -> Self {
        Self {
            entity,
            component_type,
            path,
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditorInspectorUiState {
    focused_field: Option<InspectorPath>,
    expanded_keys: BTreeSet<String>,
    active_draft: Option<InspectorFieldDraft>,
}

impl EditorInspectorUiState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn focused_field(&self) -> Option<&InspectorPath> {
        self.focused_field.as_ref()
    }

    pub fn active_draft(&self) -> Option<&InspectorFieldDraft> {
        self.active_draft.as_ref()
    }

    pub fn begin_field_edit(
        &mut self,
        entity: EntityId,
        component_type: ComponentTypeId,
        path: InspectorPath,
        value: InspectorEditValue,
    ) {
        self.focused_field = Some(path.clone());
        self.active_draft = Some(InspectorFieldDraft::new(
            entity,
            component_type,
            path,
            value,
        ));
    }

    pub fn update_field_draft(&mut self, value: InspectorEditValue) -> Result<(), &'static str> {
        let draft = self
            .active_draft
            .as_mut()
            .ok_or("no active inspector field draft")?;

        draft.value = value;
        Ok(())
    }

    pub fn clear_draft(&mut self) {
        self.active_draft = None;
    }

    pub fn clear_focus(&mut self) {
        self.focused_field = None;
    }

    pub fn cancel_field_draft(&mut self) {
        self.clear_draft();
        self.clear_focus();
    }

    pub fn take_active_draft(&mut self) -> Option<InspectorFieldDraft> {
        let draft = self.active_draft.take();
        if draft.is_some() {
            self.clear_focus();
        }
        draft
    }

    pub fn set_expanded(&mut self, key: impl Into<String>, is_expanded: bool) {
        let key = key.into();
        if is_expanded {
            self.expanded_keys.insert(key);
        } else {
            self.expanded_keys.remove(&key);
        }
    }

    pub fn toggle_expanded(&mut self, key: impl Into<String>) -> bool {
        let key = key.into();
        if self.expanded_keys.remove(&key) {
            false
        } else {
            self.expanded_keys.insert(key);
            true
        }
    }

    pub fn is_expanded(&self, key: &str) -> bool {
        self.expanded_keys.contains(key)
    }
}
