use super::*;

pub(crate) const SHAPE_CAPSULE: u32 = 2;
pub(crate) const SHAPE_BOX: u32 = 3;
pub(crate) const SHAPE_CYLINDER: u32 = 5;

pub(crate) const OP_ADD_SOLID: u32 = 0;
pub(crate) const OP_SUBTRACT_VOID: u32 = 1;

pub(crate) fn geometry_primitives_from_topology(
    topology: &CavernTopology,
) -> Vec<CavernSdfGeometryPrimitive> {
    let mut out = Vec::with_capacity(topology.rooms.len() + topology.connections.len() + 1);
    let center = [
        (topology.world_bounds.min[0] + topology.world_bounds.max[0]) * 0.5,
        (topology.world_bounds.min[1] + topology.world_bounds.max[1]) * 0.5,
        (topology.world_bounds.min[2] + topology.world_bounds.max[2]) * 0.5,
    ];
    let half_extents = [
        (topology.world_bounds.max[0] - topology.world_bounds.min[0]) * 0.5,
        (topology.world_bounds.max[1] - topology.world_bounds.min[1]) * 0.5,
        (topology.world_bounds.max[2] - topology.world_bounds.min[2]) * 0.5,
    ];
    out.push(CavernSdfGeometryPrimitive {
        shape_kind: SHAPE_BOX,
        op_kind: OP_ADD_SOLID,
        material_class: crate::MATERIAL_CLASS_ROCK,
        material_instance: 0,
        p0: [center[0], center[1], center[2], 0.0],
        p1: [half_extents[0], half_extents[1], half_extents[2], 0.0],
        p2: [0.0; 4],
    });
    for room in &topology.rooms {
        out.push(CavernSdfGeometryPrimitive {
            shape_kind: SHAPE_CYLINDER,
            op_kind: OP_SUBTRACT_VOID,
            material_class: crate::MATERIAL_CLASS_ROCK,
            material_instance: room.id.0 as u32,
            p0: [
                room.center[0],
                room.center[1],
                room.center[2],
                room.radii[0].max(room.radii[2]),
            ],
            p1: [room.radii[1], 0.0, 0.0, 0.0],
            p2: [0.0; 4],
        });
    }
    for connection in &topology.connections {
        out.push(CavernSdfGeometryPrimitive {
            shape_kind: SHAPE_CAPSULE,
            op_kind: OP_SUBTRACT_VOID,
            material_class: crate::MATERIAL_CLASS_ROCK,
            material_instance: 0,
            p0: [
                connection.start[0],
                connection.start[1],
                connection.start[2],
                connection.radius,
            ],
            p1: [connection.end[0], connection.end[1], connection.end[2], 0.0],
            p2: [0.0; 4],
        });
    }
    out
}

pub(super) fn geometry_primitives_from_layout(
    layout: &CavernLayout,
) -> Vec<CavernSdfGeometryPrimitive> {
    let mut out = Vec::with_capacity(layout.rooms.len() + layout.connections.len() + 1);
    let center = [
        (layout.world_bounds[0] + layout.world_bounds[2]) * 0.5,
        2.2,
        (layout.world_bounds[1] + layout.world_bounds[3]) * 0.5,
    ];
    let half_extents = [
        (layout.world_bounds[2] - layout.world_bounds[0]) * 0.5,
        5.8,
        (layout.world_bounds[3] - layout.world_bounds[1]) * 0.5,
    ];
    out.push(CavernSdfGeometryPrimitive {
        shape_kind: SHAPE_BOX,
        op_kind: OP_ADD_SOLID,
        material_class: crate::MATERIAL_CLASS_ROCK,
        material_instance: 0,
        p0: [center[0], center[1], center[2], 0.0],
        p1: [half_extents[0], half_extents[1], half_extents[2], 0.0],
        p2: [0.0; 4],
    });
    for room in &layout.rooms {
        out.push(CavernSdfGeometryPrimitive {
            shape_kind: SHAPE_CYLINDER,
            op_kind: OP_SUBTRACT_VOID,
            material_class: crate::MATERIAL_CLASS_ROCK,
            material_instance: room.id.0 as u32,
            p0: [
                room.center[0],
                2.4,
                room.center[1],
                room.radii[0].max(room.radii[1]),
            ],
            p1: [2.2, 0.0, 0.0, 0.0],
            p2: [0.0; 4],
        });
    }
    for tunnel in &layout.connections {
        out.push(CavernSdfGeometryPrimitive {
            shape_kind: SHAPE_CAPSULE,
            op_kind: OP_SUBTRACT_VOID,
            material_class: crate::MATERIAL_CLASS_ROCK,
            material_instance: 0,
            p0: [tunnel.start[0], 2.2, tunnel.start[1], tunnel.radius],
            p1: [tunnel.end[0], 2.2, tunnel.end[1], 0.0],
            p2: [0.0; 4],
        });
    }
    out
}
