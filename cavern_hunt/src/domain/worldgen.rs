use crate::domain::resources::{CavernRunConfig, CavernSeed};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RoomId(pub u16);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomRole {
    Start,
    Combat,
    Loot,
    Fork,
    Elite,
    Extraction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernRoom {
    pub id: RoomId,
    pub role: RoomRole,
    pub center: [f32; 2],
    pub radii: [f32; 2],
    pub spawn_anchor: [f32; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernTunnel {
    pub from: RoomId,
    pub to: RoomId,
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub radius: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernLayout {
    pub rooms: Vec<CavernRoom>,
    pub connections: Vec<CavernTunnel>,
    pub start_room: RoomId,
    pub elite_room: RoomId,
    pub extraction_room: RoomId,
    pub world_bounds: [f32; 4],
}

impl Default for CavernLayout {
    fn default() -> Self {
        Self {
            rooms: Vec::new(),
            connections: Vec::new(),
            start_room: RoomId(0),
            elite_room: RoomId(0),
            extraction_room: RoomId(0),
            world_bounds: [-24.0, -24.0, 24.0, 24.0],
        }
    }
}

impl CavernLayout {
    pub fn generate(seed: CavernSeed, config: &CavernRunConfig) -> Self {
        let mut rng = LayoutRng::new(seed.0);
        let min_rooms = config.room_count_min.max(7);
        let max_rooms = config.room_count_max.max(min_rooms);
        let room_count = min_rooms + rng.range_u8(max_rooms - min_rooms + 1);

        let mut rooms = Vec::new();
        let mut connections = Vec::new();
        let mut next_room = 0u16;

        let main_roles = [
            RoomRole::Start,
            RoomRole::Combat,
            RoomRole::Combat,
            RoomRole::Fork,
            RoomRole::Elite,
            RoomRole::Extraction,
        ];

        let mut previous: Option<CavernRoom> = None;
        let mut fork_room = RoomId(0);
        let mut elite_room = RoomId(0);
        let mut extraction_room = RoomId(0);
        let mut x_cursor = 0.0f32;
        let mut y_cursor = 0.0f32;

        for role in main_roles {
            let radii = match role {
                RoomRole::Start => [5.2, 4.4],
                RoomRole::Combat => [4.8 + rng.range_f32(0.0, 1.4), 4.0 + rng.range_f32(0.0, 1.2)],
                RoomRole::Fork => [6.2, 4.8],
                RoomRole::Elite => [7.0, 5.8],
                RoomRole::Extraction => [5.4, 4.6],
                RoomRole::Loot => [4.2, 3.8],
            };
            let center = if previous.is_some() {
                x_cursor += 8.5 + rng.range_f32(0.0, 2.8);
                y_cursor += rng.range_f32(-2.2, 2.2);
                [x_cursor, y_cursor]
            } else {
                [0.0, 0.0]
            };
            let room = CavernRoom {
                id: RoomId(next_room),
                role,
                center,
                radii,
                spawn_anchor: center,
            };
            if let Some(prev) = &previous {
                connections.push(CavernTunnel {
                    from: prev.id,
                    to: room.id,
                    start: prev.center,
                    end: room.center,
                    radius: 1.75 + rng.range_f32(0.0, 0.35),
                });
            }
            if role == RoomRole::Fork {
                fork_room = room.id;
            } else if role == RoomRole::Elite {
                elite_room = room.id;
            } else if role == RoomRole::Extraction {
                extraction_room = room.id;
            }
            previous = Some(room.clone());
            rooms.push(room);
            next_room = next_room.saturating_add(1);
        }

        let mandatory_loot_parent = rooms
            .iter()
            .find(|room| room.id == fork_room)
            .cloned()
            .expect("fork room exists");
        let loot_sign = if rng.next_bool() { 1.0 } else { -1.0 };
        let loot_center = [
            mandatory_loot_parent.center[0] + rng.range_f32(1.5, 3.5),
            mandatory_loot_parent.center[1] + loot_sign * rng.range_f32(8.0, 10.5),
        ];
        let loot_room = CavernRoom {
            id: RoomId(next_room),
            role: RoomRole::Loot,
            center: loot_center,
            radii: [4.6, 4.0],
            spawn_anchor: loot_center,
        };
        connections.push(CavernTunnel {
            from: mandatory_loot_parent.id,
            to: loot_room.id,
            start: mandatory_loot_parent.center,
            end: loot_room.center,
            radius: 1.55 + rng.range_f32(0.0, 0.25),
        });
        rooms.push(loot_room);
        next_room = next_room.saturating_add(1);

        let mut branch_sign = -loot_sign;
        while rooms.len() < room_count as usize {
            let parent = rooms[rng.range_usize(1, rooms.len() - 2)].clone();
            let role = if rng.next_bool() {
                RoomRole::Combat
            } else {
                RoomRole::Loot
            };
            let center = [
                parent.center[0] + rng.range_f32(2.0, 5.0),
                parent.center[1] + branch_sign * rng.range_f32(7.0, 11.0),
            ];
            let room = CavernRoom {
                id: RoomId(next_room),
                role,
                center,
                radii: [4.0 + rng.range_f32(0.0, 1.5), 3.7 + rng.range_f32(0.0, 1.3)],
                spawn_anchor: center,
            };
            connections.push(CavernTunnel {
                from: parent.id,
                to: room.id,
                start: parent.center,
                end: room.center,
                radius: 1.45 + rng.range_f32(0.0, 0.3),
            });
            rooms.push(room);
            next_room = next_room.saturating_add(1);
            branch_sign *= -1.0;
        }

        let margin = 8.0f32;
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for room in &rooms {
            min_x = min_x.min(room.center[0] - room.radii[0] - margin);
            min_y = min_y.min(room.center[1] - room.radii[1] - margin);
            max_x = max_x.max(room.center[0] + room.radii[0] + margin);
            max_y = max_y.max(room.center[1] + room.radii[1] + margin);
        }

        Self {
            rooms,
            connections,
            start_room: RoomId(0),
            elite_room,
            extraction_room,
            world_bounds: [min_x, min_y, max_x, max_y],
        }
    }

    pub fn room(&self, id: RoomId) -> Option<&CavernRoom> {
        self.rooms.iter().find(|room| room.id == id)
    }

    pub fn room_by_role(&self, role: RoomRole) -> Option<&CavernRoom> {
        self.rooms.iter().find(|room| room.role == role)
    }

    pub fn contains_point(&self, point: [f32; 2], margin: f32) -> bool {
        self.rooms
            .iter()
            .any(|room| point_in_room(point, room, margin))
            || self
                .connections
                .iter()
                .any(|tunnel| point_in_tunnel(point, tunnel, margin))
    }

    pub fn adjacency(&self) -> HashMap<RoomId, HashSet<RoomId>> {
        let mut adjacency = HashMap::<RoomId, HashSet<RoomId>>::new();
        for room in &self.rooms {
            adjacency.entry(room.id).or_default();
        }
        for tunnel in &self.connections {
            adjacency.entry(tunnel.from).or_default().insert(tunnel.to);
            adjacency.entry(tunnel.to).or_default().insert(tunnel.from);
        }
        adjacency
    }
}

fn point_in_room(point: [f32; 2], room: &CavernRoom, margin: f32) -> bool {
    let rx = (room.radii[0] + margin).max(0.1);
    let ry = (room.radii[1] + margin).max(0.1);
    let dx = (point[0] - room.center[0]) / rx;
    let dy = (point[1] - room.center[1]) / ry;
    dx * dx + dy * dy <= 1.0
}

fn point_in_tunnel(point: [f32; 2], tunnel: &CavernTunnel, margin: f32) -> bool {
    let pa = [point[0] - tunnel.start[0], point[1] - tunnel.start[1]];
    let ba = [
        tunnel.end[0] - tunnel.start[0],
        tunnel.end[1] - tunnel.start[1],
    ];
    let denom = (ba[0] * ba[0] + ba[1] * ba[1]).max(0.0001);
    let h = ((pa[0] * ba[0] + pa[1] * ba[1]) / denom).clamp(0.0, 1.0);
    let closest = [tunnel.start[0] + ba[0] * h, tunnel.start[1] + ba[1] * h];
    let dx = point[0] - closest[0];
    let dy = point[1] - closest[1];
    dx * dx + dy * dy <= (tunnel.radius + margin).powi(2)
}

#[derive(Debug, Clone)]
struct LayoutRng {
    state: u64,
}

impl LayoutRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 0
    }

    fn next_f32(&mut self) -> f32 {
        let bits = (self.next_u64() >> 40) as u32;
        bits as f32 / (1u32 << 24) as f32
    }

    fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    fn range_u8(&mut self, upper_exclusive: u8) -> u8 {
        if upper_exclusive <= 1 {
            return 0;
        }
        (self.next_u64() % upper_exclusive as u64) as u8
    }

    fn range_usize(&mut self, start: usize, end_inclusive: usize) -> usize {
        if end_inclusive <= start {
            return start;
        }
        start + (self.next_u64() as usize % (end_inclusive - start + 1))
    }
}
