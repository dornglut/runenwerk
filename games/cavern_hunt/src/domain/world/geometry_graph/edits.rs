use super::{
    CavernGeometryGraph, GeometryBounds3, GeometryEditKind, GeometryLayer, GeometryMaterial,
    GeometryOp, GeometryPrimitive3, GeometryRevision,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct GeometryEdit {
    pub kind: GeometryEditKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct GeometryEditEvent {
    pub revision: GeometryRevision,
    pub edit: GeometryEdit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct GeometryEditCommand {
    pub edit: GeometryEdit,
}

impl CavernGeometryGraph {
    pub fn apply_edit(&mut self, edit: &GeometryEdit) -> Option<GeometryBounds3> {
        let affected = match &edit.kind {
            GeometryEditKind::AddBlocker(shape) => {
                let primitive = GeometryPrimitive3 {
                    id: self.next_primitive_id(),
                    layer: GeometryLayer::Blocker,
                    material: GeometryMaterial::Barrier,
                    op: GeometryOp::Blocker,
                    enabled: true,
                    shape: shape.clone(),
                };
                let bounds = primitive.bounds();
                self.primitives.push(primitive);
                Some(bounds)
            }
            GeometryEditKind::RemoveBlocker(shape) => {
                let index = self.primitives.iter().position(|primitive| {
                    primitive.layer == GeometryLayer::Blocker && primitive.shape == *shape
                })?;
                let primitive = self.primitives.remove(index);
                Some(primitive.bounds())
            }
            GeometryEditKind::RemovePrimitive(id) => {
                let index = self
                    .primitives
                    .iter()
                    .position(|primitive| primitive.id == *id)?;
                let primitive = self.primitives.remove(index);
                Some(primitive.bounds())
            }
            GeometryEditKind::EnablePrimitive(id) => {
                let primitive = self.primitive_mut(*id)?;
                primitive.enabled = true;
                Some(primitive.bounds())
            }
            GeometryEditKind::DisablePrimitive(id) => {
                let primitive = self.primitive_mut(*id)?;
                primitive.enabled = false;
                Some(primitive.bounds())
            }
            GeometryEditKind::ReplacePrimitive(id, replacement) => {
                let primitive = self.primitive_mut(*id)?;
                let previous = primitive.bounds();
                *primitive = replacement.clone();
                Some(previous.union(&primitive.bounds()))
            }
        };
        if affected.is_some() {
            self.revision.0 = self.revision.0.saturating_add(1);
        }
        affected
    }
}
