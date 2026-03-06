// Owner: Cavern Hunt Geometry Domain - Edit Commands and Distance Helpers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryEdit {
    pub kind: GeometryEditKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryEditEvent {
    pub revision: GeometryRevision,
    pub edit: GeometryEdit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

fn sd_capsule3(point: [f32; 3], start: [f32; 3], end: [f32; 3], radius: f32) -> f32 {
    let pa = sub3(point, start);
    let ba = sub3(end, start);
    let denom = dot3(ba, ba).max(0.0001);
    let h = (dot3(pa, ba) / denom).clamp(0.0, 1.0);
    let closest = [
        start[0] + ba[0] * h,
        start[1] + ba[1] * h,
        start[2] + ba[2] * h,
    ];
    length3(sub3(point, closest)) - radius
}

fn sd_box3(point: [f32; 3], center: [f32; 3], half_extents: [f32; 3]) -> f32 {
    let q = [
        (point[0] - center[0]).abs() - half_extents[0],
        (point[1] - center[1]).abs() - half_extents[1],
        (point[2] - center[2]).abs() - half_extents[2],
    ];
    let outside = [q[0].max(0.0), q[1].max(0.0), q[2].max(0.0)];
    let outside_len = length3(outside);
    let inside = q[0].max(q[1]).max(q[2]).min(0.0);
    outside_len + inside
}

#[cfg(test)]
mod tests {
    use super::{CavernGeometryGraph, CavernTopology, GeometryOp};
    use crate::domain::{CavernLayout, CavernRunConfig, CavernSeed};

    #[test]
    fn topology_and_graph_are_deterministic_from_layout() {
        let layout = CavernLayout::generate(CavernSeed(7), &CavernRunConfig::default());
        let topology_a = CavernTopology::from_layout(&layout, CavernSeed(7));
        let topology_b = CavernTopology::from_layout(&layout, CavernSeed(7));
        let graph_a = CavernGeometryGraph::from_topology(&topology_a);
        let graph_b = CavernGeometryGraph::from_topology(&topology_b);
        assert_eq!(topology_a, topology_b);
        assert_eq!(graph_a, graph_b);
        assert!(
            graph_a
                .primitives
                .iter()
                .any(|primitive| primitive.op == GeometryOp::SubtractVoid)
        );
    }
}
