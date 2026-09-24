use std::collections::BTreeMap;

use ui_composition::PresentationTargetId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCompositionTargetBinding<T> {
    pub target_id: PresentationTargetId,
    pub binding: T,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCompositionTargetBindingRegistry<T> {
    bindings: BTreeMap<PresentationTargetId, T>,
}

impl<T> Default for EditorCompositionTargetBindingRegistry<T> {
    fn default() -> Self {
        Self {
            bindings: BTreeMap::new(),
        }
    }
}

impl<T> EditorCompositionTargetBindingRegistry<T> {
    pub fn bind(&mut self, target_id: PresentationTargetId, binding: T) -> Option<T> {
        self.bindings.insert(target_id, binding)
    }

    pub fn binding(&self, target_id: PresentationTargetId) -> Option<&T> {
        self.bindings.get(&target_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = EditorCompositionTargetBinding<&T>> {
        self.bindings
            .iter()
            .map(|(target_id, binding)| EditorCompositionTargetBinding {
                target_id: *target_id,
                binding,
            })
    }

    pub fn unbind(&mut self, target_id: PresentationTargetId) -> Option<T> {
        self.bindings.remove(&target_id)
    }
}
