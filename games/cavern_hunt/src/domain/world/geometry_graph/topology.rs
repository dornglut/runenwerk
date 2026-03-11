use super::{
    CavernConnection, CavernRoomNode, CavernTopology, DEFAULT_CHAMBER_CENTER_HEIGHT,
    DEFAULT_CHAMBER_VERTICAL_RADIUS, DEFAULT_TUNNEL_CENTER_HEIGHT, DEFAULT_WORLD_CEILING_MARGIN,
    DEFAULT_WORLD_FLOOR_Y, GeometryBounds3,
};
use crate::CavernSeed;
use crate::world::worldgen::{CavernLayout, CavernRoom, CavernTunnel, RoomId};

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
