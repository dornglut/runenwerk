use super::*;

pub(crate) const SHAPE_SPHERE: u32 = 0;
pub(crate) const SHAPE_ELLIPSOID: u32 = 1;
pub(crate) const SHAPE_CAPSULE: u32 = 2;
pub(crate) const SHAPE_BOX: u32 = 3;
const SHAPE_ROUNDED_BOX: u32 = 4;
pub(crate) const SHAPE_CYLINDER: u32 = 5;

pub(crate) const OP_ADD_SOLID: u32 = 0;
pub(crate) const OP_SUBTRACT_VOID: u32 = 1;
const OP_MASK_WALKABLE: u32 = 2;
pub(crate) const OP_BLOCKER: u32 = 3;
const OP_HAZARD: u32 = 4;

fn material_class_from_geometry(material: GeometryMaterial) -> u32 {
    match material {
        GeometryMaterial::Rock | GeometryMaterial::CavernVoid => crate::MATERIAL_CLASS_ROCK,
        GeometryMaterial::Barrier => crate::MATERIAL_CLASS_BARRIER,
        GeometryMaterial::Hazard => crate::MATERIAL_CLASS_HAZARD,
        GeometryMaterial::Marker => crate::MATERIAL_CLASS_MARKER,
    }
}

fn op_kind(op: GeometryOp) -> u32 {
    match op {
        GeometryOp::AddSolid => OP_ADD_SOLID,
        GeometryOp::SubtractVoid => OP_SUBTRACT_VOID,
        GeometryOp::MaskWalkable => OP_MASK_WALKABLE,
        GeometryOp::Blocker => OP_BLOCKER,
        GeometryOp::HazardVolume => OP_HAZARD,
    }
}

pub(crate) fn geometry_primitives_from_graph(
    graph: &CavernGeometryGraph,
) -> Vec<CavernSdfGeometryPrimitive> {
    let mut out = Vec::with_capacity(graph.primitives.len());
    for primitive in graph
        .primitives
        .iter()
        .filter(|primitive| primitive.enabled)
    {
        append_shape_primitive(
            &mut out,
            primitive.op,
            &primitive.shape,
            material_class_from_geometry(primitive.material),
            primitive.id.0 as u32,
        );
    }
    out
}

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

fn append_shape_primitive(
    out: &mut Vec<CavernSdfGeometryPrimitive>,
    op: GeometryOp,
    shape: &GeometryPrimitiveShape3,
    material_class: u32,
    material_instance: u32,
) {
    let op_kind = op_kind(op);
    match shape {
        GeometryPrimitiveShape3::Sphere { center, radius } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_SPHERE,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], *radius],
                p1: [0.0; 4],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::Ellipsoid { center, radii } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_ELLIPSOID,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], 0.0],
                p1: [radii[0], radii[1], radii[2], 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::Capsule { start, end, radius } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_CAPSULE,
                op_kind,
                material_class,
                material_instance,
                p0: [start[0], start[1], start[2], *radius],
                p1: [end[0], end[1], end[2], 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::Box {
            center,
            half_extents,
        } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_BOX,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], 0.0],
                p1: [half_extents[0], half_extents[1], half_extents[2], 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::RoundedBox {
            center,
            half_extents,
            radius,
        } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_ROUNDED_BOX,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], *radius],
                p1: [half_extents[0], half_extents[1], half_extents[2], 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::Cylinder {
            center,
            radius,
            half_height,
        } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_CYLINDER,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], *radius],
                p1: [*half_height, 0.0, 0.0, 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::TunnelSplineCapsuleChain { points, radius } => {
            for segment in points.windows(2) {
                out.push(CavernSdfGeometryPrimitive {
                    shape_kind: SHAPE_CAPSULE,
                    op_kind,
                    material_class,
                    material_instance,
                    p0: [segment[0][0], segment[0][1], segment[0][2], *radius],
                    p1: [segment[1][0], segment[1][1], segment[1][2], 0.0],
                    p2: [0.0; 4],
                });
            }
        }
    }
}
