use crate::CavernSeed;
use crate::world::worldgen::{RoomId, RoomRole};
use serde::{Deserialize, Serialize};

mod edits;
mod graph;
mod math;
mod shape_ops;
mod topology;

pub use edits::{GeometryEdit, GeometryEditCommand, GeometryEditEvent};

const DEFAULT_CHAMBER_CENTER_HEIGHT: f32 = 2.4;
const DEFAULT_TUNNEL_CENTER_HEIGHT: f32 = 2.2;
const DEFAULT_CHAMBER_VERTICAL_RADIUS: f32 = 2.2;
const DEFAULT_WORLD_FLOOR_Y: f32 = -1.5;
const DEFAULT_WORLD_CEILING_MARGIN: f32 = 3.5;
pub const CAVERN_GAMEPLAY_HEIGHT: f32 = DEFAULT_TUNNEL_CENTER_HEIGHT;

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ecs::Resource,
)]
pub struct GeometryRevision(pub u64);

impl Default for GeometryRevision {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ecs::Resource,
)]
pub struct GeometryPrimitiveId(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub enum GeometryLayer {
    Terrain,
    Walkable,
    Blocker,
    Hazard,
    RenderOnly,
    NavHint,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub enum GeometryMaterial {
    Rock,
    CavernVoid,
    Barrier,
    Hazard,
    Marker,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub enum GeometryOp {
    AddSolid,
    SubtractVoid,
    MaskWalkable,
    Blocker,
    HazardVolume,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernRoomNode {
    pub id: RoomId,
    pub role: RoomRole,
    pub center: [f32; 3],
    pub radii: [f32; 3],
    pub spawn_anchor: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernConnection {
    pub from: RoomId,
    pub to: RoomId,
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub radius: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub enum GeometryEditKind {
    AddBlocker(GeometryPrimitiveShape3),
    RemovePrimitive(GeometryPrimitiveId),
    EnablePrimitive(GeometryPrimitiveId),
    DisablePrimitive(GeometryPrimitiveId),
    ReplacePrimitive(GeometryPrimitiveId, GeometryPrimitive3),
}

#[cfg(test)]
mod tests;
