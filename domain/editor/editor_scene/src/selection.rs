//! Scene-owned selection addresses and explicit sharing context.

use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::time::SystemTime;

use editor_core::{ChangeOrigin, ComponentTypeId, EntityId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SceneSelectionScope(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SceneSelectionTarget {
    Entity(EntityId),
    Component {
        entity: EntityId,
        component_type: ComponentTypeId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SceneSelectionAddress {
    scope: SceneSelectionScope,
    target: SceneSelectionTarget,
}

impl SceneSelectionAddress {
    pub fn entity(scope: SceneSelectionScope, entity: EntityId) -> Self {
        Self {
            scope,
            target: SceneSelectionTarget::Entity(entity),
        }
    }

    pub fn component(
        scope: SceneSelectionScope,
        entity: EntityId,
        component_type: ComponentTypeId,
    ) -> Self {
        Self {
            scope,
            target: SceneSelectionTarget::Component {
                entity,
                component_type,
            },
        }
    }

    pub fn scope(&self) -> SceneSelectionScope {
        self.scope
    }

    pub fn target(&self) -> &SceneSelectionTarget {
        &self.target
    }

    pub fn entity_id(&self) -> EntityId {
        match self.target {
            SceneSelectionTarget::Entity(entity)
            | SceneSelectionTarget::Component { entity, .. } => entity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneSelectionError {
    StaleAddress {
        expected: SceneSelectionScope,
        actual: SceneSelectionScope,
    },
    AddressNotSelected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneSelectionContext {
    scope: SceneSelectionScope,
    items: BTreeSet<SceneSelectionAddress>,
    primary: Option<SceneSelectionAddress>,
}

impl SceneSelectionContext {
    pub fn new(scope: SceneSelectionScope) -> Self {
        Self {
            scope,
            items: BTreeSet::new(),
            primary: None,
        }
    }

    pub fn scope(&self) -> SceneSelectionScope {
        self.scope
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn primary(&self) -> Option<&SceneSelectionAddress> {
        self.primary.as_ref()
    }

    pub fn contains(&self, address: &SceneSelectionAddress) -> bool {
        address.scope() == self.scope && self.items.contains(address)
    }

    pub fn iter(&self) -> impl Iterator<Item = &SceneSelectionAddress> {
        self.items.iter()
    }

    pub fn clear(&mut self) -> bool {
        let changed = !self.items.is_empty();
        self.items.clear();
        self.primary = None;
        changed
    }

    pub fn set_single(
        &mut self,
        address: SceneSelectionAddress,
    ) -> Result<(), SceneSelectionError> {
        self.validate_scope(&address)?;
        self.items.clear();
        self.items.insert(address.clone());
        self.primary = Some(address);
        Ok(())
    }

    pub fn add(&mut self, address: SceneSelectionAddress) -> Result<(), SceneSelectionError> {
        self.validate_scope(&address)?;
        let was_empty = self.items.is_empty();
        self.items.insert(address.clone());
        if was_empty || self.primary.is_none() {
            self.primary = Some(address);
        }
        Ok(())
    }

    pub fn remove(&mut self, address: &SceneSelectionAddress) -> bool {
        let removed = self.items.remove(address);
        if removed && self.primary.as_ref() == Some(address) {
            self.primary = self.items.iter().next().cloned();
        }
        removed
    }

    pub fn set_primary(
        &mut self,
        address: &SceneSelectionAddress,
    ) -> Result<(), SceneSelectionError> {
        self.validate_scope(address)?;
        if !self.items.contains(address) {
            return Err(SceneSelectionError::AddressNotSelected);
        }
        self.primary = Some(address.clone());
        Ok(())
    }

    fn validate_scope(&self, address: &SceneSelectionAddress) -> Result<(), SceneSelectionError> {
        if address.scope() != self.scope {
            return Err(SceneSelectionError::StaleAddress {
                expected: self.scope,
                actual: address.scope(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneSelectionChangeKind {
    SetSingle { address: SceneSelectionAddress },
    Cleared,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneSelectionChange {
    pub origin: ChangeOrigin,
    pub kind: SceneSelectionChangeKind,
    pub timestamp: SystemTime,
}

impl SceneSelectionChange {
    pub fn new(origin: ChangeOrigin, kind: SceneSelectionChangeKind) -> Self {
        Self {
            origin,
            kind,
            timestamp: SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SceneSelectionChangeLog {
    entries: VecDeque<SceneSelectionChange>,
    max_entries: usize,
}

impl Default for SceneSelectionChangeLog {
    fn default() -> Self {
        Self::with_capacity(512)
    }
}

impl SceneSelectionChangeLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: max_entries.max(1),
        }
    }

    pub fn push(&mut self, change: SceneSelectionChange) {
        if self.entries.len() == self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(change);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn last(&self) -> Option<&SceneSelectionChange> {
        self.entries.back()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SceneSelectionChange> {
        self.entries.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_address_is_rejected_by_a_new_selection_scope() {
        let old = SceneSelectionContext::new(SceneSelectionScope(1));
        let old_address = SceneSelectionAddress::entity(old.scope(), EntityId(1));
        let mut current = SceneSelectionContext::new(SceneSelectionScope(2));

        let error = current
            .set_single(old_address)
            .expect_err("an address from a prior scene scope must be rejected");

        assert_eq!(
            error,
            SceneSelectionError::StaleAddress {
                expected: SceneSelectionScope(2),
                actual: SceneSelectionScope(1),
            }
        );
        assert!(current.is_empty());
    }

    #[test]
    fn selection_context_tracks_one_primary_address_and_multiple_items() {
        let mut context = SceneSelectionContext::new(SceneSelectionScope(1));
        let entity = SceneSelectionAddress::entity(context.scope(), EntityId(1));
        let component =
            SceneSelectionAddress::component(context.scope(), EntityId(1), ComponentTypeId(2));

        context
            .add(entity.clone())
            .expect("current-scope entity should be accepted");
        context
            .add(component.clone())
            .expect("current-scope component should be accepted");
        context
            .set_primary(&component)
            .expect("selected address should be eligible as primary");

        assert_eq!(context.len(), 2);
        assert_eq!(context.primary(), Some(&component));
    }
}
