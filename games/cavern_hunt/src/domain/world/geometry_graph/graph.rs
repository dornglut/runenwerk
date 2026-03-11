use super::{
    CavernGeometryGraph, CavernTopology, GeometryLayer, GeometryMaterial, GeometryOp,
    GeometryPrimitive3, GeometryPrimitiveId, GeometryPrimitiveShape3, GeometryRevision,
};

impl CavernGeometryGraph {
    pub fn from_topology(topology: &CavernTopology) -> Self {
        let mut next_id = 1u64;
        let mut primitives = Vec::new();
        primitives.push(GeometryPrimitive3 {
            id: GeometryPrimitiveId(next_id),
            layer: GeometryLayer::Terrain,
            material: GeometryMaterial::Rock,
            op: GeometryOp::AddSolid,
            enabled: true,
            shape: GeometryPrimitiveShape3::Box {
                center: [
                    (topology.world_bounds.min[0] + topology.world_bounds.max[0]) * 0.5,
                    (topology.world_bounds.min[1] + topology.world_bounds.max[1]) * 0.5,
                    (topology.world_bounds.min[2] + topology.world_bounds.max[2]) * 0.5,
                ],
                half_extents: [
                    (topology.world_bounds.max[0] - topology.world_bounds.min[0]) * 0.5,
                    (topology.world_bounds.max[1] - topology.world_bounds.min[1]) * 0.5,
                    (topology.world_bounds.max[2] - topology.world_bounds.min[2]) * 0.5,
                ],
            },
        });
        next_id += 1;

        for room in &topology.rooms {
            primitives.push(GeometryPrimitive3 {
                id: GeometryPrimitiveId(next_id),
                layer: GeometryLayer::Walkable,
                material: GeometryMaterial::CavernVoid,
                op: GeometryOp::SubtractVoid,
                enabled: true,
                shape: GeometryPrimitiveShape3::Cylinder {
                    center: room.center,
                    radius: room.radii[0].max(room.radii[2]),
                    half_height: room.radii[1],
                },
            });
            next_id += 1;
        }
        for connection in &topology.connections {
            primitives.push(GeometryPrimitive3 {
                id: GeometryPrimitiveId(next_id),
                layer: GeometryLayer::Walkable,
                material: GeometryMaterial::CavernVoid,
                op: GeometryOp::SubtractVoid,
                enabled: true,
                shape: GeometryPrimitiveShape3::Capsule {
                    start: connection.start,
                    end: connection.end,
                    radius: connection.radius,
                },
            });
            next_id += 1;
        }

        Self {
            revision: GeometryRevision::default(),
            bounds: topology.world_bounds,
            primitives,
        }
    }

    pub fn primitive(&self, id: GeometryPrimitiveId) -> Option<&GeometryPrimitive3> {
        self.primitives.iter().find(|primitive| primitive.id == id)
    }

    pub fn primitive_mut(&mut self, id: GeometryPrimitiveId) -> Option<&mut GeometryPrimitive3> {
        self.primitives
            .iter_mut()
            .find(|primitive| primitive.id == id)
    }

    pub fn next_primitive_id(&self) -> GeometryPrimitiveId {
        GeometryPrimitiveId(
            self.primitives
                .iter()
                .map(|primitive| primitive.id.0)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        )
    }
}
