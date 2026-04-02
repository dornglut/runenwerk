use super::*;

// Owner: Cavern Hunt SDF Renderer - Tests
#[cfg(test)]
mod tests {
    use super::world_frame_and_geometry::{
        OP_ADD_SOLID, OP_SUBTRACT_VOID, SHAPE_CYLINDER, geometry_primitives_from_topology,
    };

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
