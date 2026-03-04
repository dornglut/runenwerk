use crate::domain::CavernSeed;
use crate::domain::worldgen::{CavernLayout, CavernRoom, CavernTunnel, RoomId, RoomRole};
use serde::{Deserialize, Serialize};

const DEFAULT_CHAMBER_CENTER_HEIGHT: f32 = 2.4;
const DEFAULT_TUNNEL_CENTER_HEIGHT: f32 = 2.2;
const DEFAULT_CHAMBER_VERTICAL_RADIUS: f32 = 2.2;
const DEFAULT_WORLD_FLOOR_Y: f32 = -1.5;
const DEFAULT_WORLD_CEILING_MARGIN: f32 = 3.5;
pub const CAVERN_GAMEPLAY_HEIGHT: f32 = DEFAULT_TUNNEL_CENTER_HEIGHT;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GeometryRevision(pub u64);

impl Default for GeometryRevision {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GeometryPrimitiveId(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeometryLayer {
    Terrain,
    Walkable,
    Blocker,
    Hazard,
    RenderOnly,
    NavHint,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeometryMaterial {
    Rock,
    CavernVoid,
    Barrier,
    Hazard,
    Marker,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeometryOp {
    AddSolid,
    SubtractVoid,
    MaskWalkable,
    Blocker,
    HazardVolume,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryBounds3 {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Default for GeometryBounds3 {
    fn default() -> Self {
        Self {
            min: [-24.0, DEFAULT_WORLD_FLOOR_Y, -24.0],
            max: [24.0, 8.0, 24.0],
        }
    }
}

impl GeometryBounds3 {
    pub fn from_center_half_extents(center: [f32; 3], half_extents: [f32; 3]) -> Self {
        Self {
            min: [
                center[0] - half_extents[0],
                center[1] - half_extents[1],
                center[2] - half_extents[2],
            ],
            max: [
                center[0] + half_extents[0],
                center[1] + half_extents[1],
                center[2] + half_extents[2],
            ],
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }

    pub fn expanded(&self, padding: f32) -> Self {
        Self {
            min: [
                self.min[0] - padding,
                self.min[1] - padding,
                self.min[2] - padding,
            ],
            max: [
                self.max[0] + padding,
                self.max[1] + padding,
                self.max[2] + padding,
            ],
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min[0] <= other.max[0]
            && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1]
            && self.max[1] >= other.min[1]
            && self.min[2] <= other.max[2]
            && self.max[2] >= other.min[2]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeometryPrimitiveShape3 {
    Sphere {
        center: [f32; 3],
        radius: f32,
    },
    Ellipsoid {
        center: [f32; 3],
        radii: [f32; 3],
    },
    Capsule {
        start: [f32; 3],
        end: [f32; 3],
        radius: f32,
    },
    Box {
        center: [f32; 3],
        half_extents: [f32; 3],
    },
    RoundedBox {
        center: [f32; 3],
        half_extents: [f32; 3],
        radius: f32,
    },
    Cylinder {
        center: [f32; 3],
        radius: f32,
        half_height: f32,
    },
    TunnelSplineCapsuleChain {
        points: Vec<[f32; 3]>,
        radius: f32,
    },
}

impl GeometryPrimitiveShape3 {
    pub fn bounds(&self) -> GeometryBounds3 {
        match self {
            GeometryPrimitiveShape3::Sphere { center, radius } => {
                GeometryBounds3::from_center_half_extents(*center, [*radius, *radius, *radius])
            }
            GeometryPrimitiveShape3::Ellipsoid { center, radii } => {
                GeometryBounds3::from_center_half_extents(*center, *radii)
            }
            GeometryPrimitiveShape3::Capsule { start, end, radius } => GeometryBounds3 {
                min: [
                    start[0].min(end[0]) - *radius,
                    start[1].min(end[1]) - *radius,
                    start[2].min(end[2]) - *radius,
                ],
                max: [
                    start[0].max(end[0]) + *radius,
                    start[1].max(end[1]) + *radius,
                    start[2].max(end[2]) + *radius,
                ],
            },
            GeometryPrimitiveShape3::Box {
                center,
                half_extents,
            } => GeometryBounds3::from_center_half_extents(*center, *half_extents),
            GeometryPrimitiveShape3::RoundedBox {
                center,
                half_extents,
                radius,
            } => GeometryBounds3::from_center_half_extents(
                *center,
                [
                    half_extents[0] + *radius,
                    half_extents[1] + *radius,
                    half_extents[2] + *radius,
                ],
            ),
            GeometryPrimitiveShape3::Cylinder {
                center,
                radius,
                half_height,
            } => {
                GeometryBounds3::from_center_half_extents(*center, [*radius, *half_height, *radius])
            }
            GeometryPrimitiveShape3::TunnelSplineCapsuleChain { points, radius } => {
                let mut bounds = GeometryBounds3::from_center_half_extents(
                    points[0],
                    [*radius, *radius, *radius],
                );
                for point in points.iter().skip(1) {
                    bounds = bounds.union(&GeometryBounds3::from_center_half_extents(
                        *point,
                        [*radius, *radius, *radius],
                    ));
                }
                bounds
            }
        }
    }

    pub fn signed_distance(&self, point: [f32; 3]) -> f32 {
        match self {
            GeometryPrimitiveShape3::Sphere { center, radius } => {
                length3(sub3(point, *center)) - *radius
            }
            GeometryPrimitiveShape3::Ellipsoid { center, radii } => {
                let q = [
                    (point[0] - center[0]) / radii[0].max(0.001),
                    (point[1] - center[1]) / radii[1].max(0.001),
                    (point[2] - center[2]) / radii[2].max(0.001),
                ];
                (length3(q) - 1.0) * radii[0].min(radii[1]).min(radii[2])
            }
            GeometryPrimitiveShape3::Capsule { start, end, radius } => {
                sd_capsule3(point, *start, *end, *radius)
            }
            GeometryPrimitiveShape3::Box {
                center,
                half_extents,
            } => sd_box3(point, *center, *half_extents),
            GeometryPrimitiveShape3::RoundedBox {
                center,
                half_extents,
                radius,
            } => sd_box3(point, *center, *half_extents) - *radius,
            GeometryPrimitiveShape3::Cylinder {
                center,
                radius,
                half_height,
            } => {
                let px = point[0] - center[0];
                let py = (point[1] - center[1]).abs() - *half_height;
                let pz = point[2] - center[2];
                let radial = (px * px + pz * pz).sqrt() - *radius;
                let ax = radial.max(0.0);
                let ay = py.max(0.0);
                radial.max(py).min(0.0) + (ax * ax + ay * ay).sqrt()
            }
            GeometryPrimitiveShape3::TunnelSplineCapsuleChain { points, radius } => {
                let mut distance = f32::INFINITY;
                for window in points.windows(2) {
                    distance = distance.min(sd_capsule3(point, window[0], window[1], *radius));
                }
                distance
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryPrimitive3 {
    pub id: GeometryPrimitiveId,
    pub layer: GeometryLayer,
    pub material: GeometryMaterial,
    pub op: GeometryOp,
    pub enabled: bool,
    pub shape: GeometryPrimitiveShape3,
}

impl GeometryPrimitive3 {
    pub fn bounds(&self) -> GeometryBounds3 {
        self.shape.bounds()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernRoomNode {
    pub id: RoomId,
    pub role: RoomRole,
    pub center: [f32; 3],
    pub radii: [f32; 3],
    pub spawn_anchor: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernConnection {
    pub from: RoomId,
    pub to: RoomId,
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub radius: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernTopology {
    pub seed: CavernSeed,
    pub rooms: Vec<CavernRoomNode>,
    pub connections: Vec<CavernConnection>,
    pub start_room: RoomId,
    pub elite_room: RoomId,
    pub extraction_room: RoomId,
    pub world_bounds: GeometryBounds3,
}

impl Default for CavernTopology {
    fn default() -> Self {
        Self {
            seed: CavernSeed::default(),
            rooms: Vec::new(),
            connections: Vec::new(),
            start_room: RoomId(0),
            elite_room: RoomId(0),
            extraction_room: RoomId(0),
            world_bounds: GeometryBounds3::default(),
        }
    }
}

impl CavernTopology {
    pub fn from_layout(layout: &CavernLayout, seed: CavernSeed) -> Self {
        let rooms = layout
            .rooms
            .iter()
            .map(|room| CavernRoomNode {
                id: room.id,
                role: room.role,
                center: [
                    room.center[0],
                    DEFAULT_CHAMBER_CENTER_HEIGHT,
                    room.center[1],
                ],
                radii: [
                    room.radii[0],
                    DEFAULT_CHAMBER_VERTICAL_RADIUS,
                    room.radii[1],
                ],
                spawn_anchor: [room.spawn_anchor[0], 0.0, room.spawn_anchor[1]],
            })
            .collect::<Vec<_>>();
        let connections = layout
            .connections
            .iter()
            .map(|connection| CavernConnection {
                from: connection.from,
                to: connection.to,
                start: [
                    connection.start[0],
                    DEFAULT_TUNNEL_CENTER_HEIGHT,
                    connection.start[1],
                ],
                end: [
                    connection.end[0],
                    DEFAULT_TUNNEL_CENTER_HEIGHT,
                    connection.end[1],
                ],
                radius: connection.radius,
            })
            .collect::<Vec<_>>();
        Self {
            seed,
            rooms,
            connections,
            start_room: layout.start_room,
            elite_room: layout.elite_room,
            extraction_room: layout.extraction_room,
            world_bounds: GeometryBounds3 {
                min: [
                    layout.world_bounds[0],
                    DEFAULT_WORLD_FLOOR_Y,
                    layout.world_bounds[1],
                ],
                max: [
                    layout.world_bounds[2],
                    DEFAULT_CHAMBER_CENTER_HEIGHT
                        + DEFAULT_CHAMBER_VERTICAL_RADIUS
                        + DEFAULT_WORLD_CEILING_MARGIN,
                    layout.world_bounds[3],
                ],
            },
        }
    }

    pub fn room(&self, id: RoomId) -> Option<&CavernRoomNode> {
        self.rooms.iter().find(|room| room.id == id)
    }

    pub fn to_layout_2d(&self) -> CavernLayout {
        CavernLayout {
            rooms: self
                .rooms
                .iter()
                .map(|room| CavernRoom {
                    id: room.id,
                    role: room.role,
                    center: [room.center[0], room.center[2]],
                    radii: [room.radii[0], room.radii[2]],
                    spawn_anchor: [room.spawn_anchor[0], room.spawn_anchor[2]],
                })
                .collect(),
            connections: self
                .connections
                .iter()
                .map(|connection| CavernTunnel {
                    from: connection.from,
                    to: connection.to,
                    start: [connection.start[0], connection.start[2]],
                    end: [connection.end[0], connection.end[2]],
                    radius: connection.radius,
                })
                .collect(),
            start_room: self.start_room,
            elite_room: self.elite_room,
            extraction_room: self.extraction_room,
            world_bounds: [
                self.world_bounds.min[0],
                self.world_bounds.min[2],
                self.world_bounds.max[0],
                self.world_bounds.max[2],
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernGeometryGraph {
    pub revision: GeometryRevision,
    pub bounds: GeometryBounds3,
    pub primitives: Vec<GeometryPrimitive3>,
}

impl Default for CavernGeometryGraph {
    fn default() -> Self {
        Self {
            revision: GeometryRevision::default(),
            bounds: GeometryBounds3::default(),
            primitives: Vec::new(),
        }
    }
}

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
                shape: GeometryPrimitiveShape3::Ellipsoid {
                    center: room.center,
                    radii: room.radii,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeometryEditKind {
    AddBlocker(GeometryPrimitiveShape3),
    RemovePrimitive(GeometryPrimitiveId),
    EnablePrimitive(GeometryPrimitiveId),
    DisablePrimitive(GeometryPrimitiveId),
    ReplacePrimitive(GeometryPrimitiveId, GeometryPrimitive3),
}

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
