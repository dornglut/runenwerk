use super::*;

// Owner: Cavern Hunt SDF Renderer - Tests
#[cfg(test)]
mod tests {
    use super::world_frame_and_geometry::{
        OP_ADD_SOLID, OP_BLOCKER, OP_SUBTRACT_VOID, SHAPE_CAPSULE, SHAPE_CYLINDER, SHAPE_ELLIPSOID,
        SHAPE_SPHERE, geometry_primitives_from_graph, geometry_primitives_from_topology,
    };
    use crate::{CavernGeometryGraph, GeometryPrimitiveShape3, GeometryRevision};

    #[test]
    fn geometry_primitives_from_graph_preserves_ops_and_flattens_splines() {
        let mut graph = CavernGeometryGraph {
            revision: GeometryRevision(1),
            bounds: crate::GeometryBounds3::default(),
            primitives: Vec::new(),
        };
        graph.primitives.push(crate::GeometryPrimitive3 {
            id: crate::GeometryPrimitiveId(1),
            layer: crate::GeometryLayer::Terrain,
            material: crate::GeometryMaterial::Rock,
            op: crate::GeometryOp::AddSolid,
            enabled: true,
            shape: GeometryPrimitiveShape3::Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 5.0,
            },
        });
        graph.primitives.push(crate::GeometryPrimitive3 {
            id: crate::GeometryPrimitiveId(2),
            layer: crate::GeometryLayer::Walkable,
            material: crate::GeometryMaterial::CavernVoid,
            op: crate::GeometryOp::SubtractVoid,
            enabled: true,
            shape: GeometryPrimitiveShape3::TunnelSplineCapsuleChain {
                points: vec![[0.0, 2.0, 0.0], [2.0, 2.0, 0.0], [4.0, 2.0, 0.0]],
                radius: 1.0,
            },
        });
        graph.primitives.push(crate::GeometryPrimitive3 {
            id: crate::GeometryPrimitiveId(3),
            layer: crate::GeometryLayer::Blocker,
            material: crate::GeometryMaterial::Barrier,
            op: crate::GeometryOp::Blocker,
            enabled: true,
            shape: GeometryPrimitiveShape3::Ellipsoid {
                center: [1.0, 1.0, 1.0],
                radii: [0.5, 0.7, 0.9],
            },
        });
        let primitives = geometry_primitives_from_graph(&graph);
        assert_eq!(
            primitives.len(),
            4,
            "spline chain should flatten to two capsules"
        );
        assert_eq!(primitives[0].shape_kind, SHAPE_SPHERE);
        assert_eq!(primitives[0].op_kind, OP_ADD_SOLID);
        assert_eq!(primitives[1].shape_kind, SHAPE_CAPSULE);
        assert_eq!(primitives[1].op_kind, OP_SUBTRACT_VOID);
        assert_eq!(primitives[2].shape_kind, SHAPE_CAPSULE);
        assert_eq!(primitives[2].op_kind, OP_SUBTRACT_VOID);
        assert_eq!(primitives[3].shape_kind, SHAPE_ELLIPSOID);
        assert_eq!(primitives[3].op_kind, OP_BLOCKER);
    }

    #[test]
    fn geometry_primitives_from_topology_keeps_topology_heights() {
        let layout = crate::CavernLayout::generate(
            crate::CavernSeed(1337),
            &crate::CavernRunConfig::default(),
        );
        let topology = crate::CavernTopology::from_layout(&layout, crate::CavernSeed(1337));
        let primitives = geometry_primitives_from_topology(&topology);
        assert!(
            !primitives.is_empty(),
            "topology conversion should produce primitives"
        );
        assert_eq!(
            primitives[0].shape_kind,
            super::world_frame_and_geometry::SHAPE_BOX
        );
        assert_eq!(primitives[0].op_kind, OP_ADD_SOLID);
        let first_room = topology
            .rooms
            .first()
            .expect("topology should contain rooms");
        let first_room_prim = primitives
            .iter()
            .find(|primitive| {
                primitive.shape_kind == SHAPE_CYLINDER && primitive.op_kind == OP_SUBTRACT_VOID
            })
            .expect("expected room cylinder primitive");
        assert_eq!(first_room_prim.p0[1], first_room.center[1]);
        assert_eq!(
            first_room_prim.p0[3],
            first_room.radii[0].max(first_room.radii[2])
        );
        assert_eq!(first_room_prim.p1[0], first_room.radii[1]);
    }
}
