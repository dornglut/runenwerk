use super::model_manager::{ModelMaterial, ModelMesh, ModelMeshVertex};
use super::world_compute::WorldRenderFrame;
use gridcore::{CellType, Grid2d, MeshType};
use std::collections::BTreeMap;

const LOGIC_RESOLUTION: usize = 8;
const CACHE_MARGIN_CHUNKS: i32 = 2;
const TILE_GAP_RATIO: f32 = 0.10;
const TILE_SURFACE_Y: f32 = 0.005;

#[derive(Debug, Default)]
struct MeshBuilder {
    vertices: Vec<ModelMeshVertex>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    fn push_quad(&mut self, center_x: f32, center_z: f32, half: f32) {
        let base = self.vertices.len() as u32;
        self.vertices.push(ModelMeshVertex {
            position: [center_x - half, TILE_SURFACE_Y, center_z - half],
            uv: [0.0, 0.0],
        });
        self.vertices.push(ModelMeshVertex {
            position: [center_x + half, TILE_SURFACE_Y, center_z - half],
            uv: [1.0, 0.0],
        });
        self.vertices.push(ModelMeshVertex {
            position: [center_x + half, TILE_SURFACE_Y, center_z + half],
            uv: [1.0, 1.0],
        });
        self.vertices.push(ModelMeshVertex {
            position: [center_x - half, TILE_SURFACE_Y, center_z + half],
            uv: [0.0, 1.0],
        });
        // CCW viewed from +Y
        self.indices
            .extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
    }
}

#[derive(Debug, Default)]
pub struct ChunkMesher {
    cache: BTreeMap<(i32, i32), Vec<ModelMesh>>,
}

impl ChunkMesher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_template_mesh_name(name: &str) -> bool {
        name_matches(name, "corner_col")
            || name_matches(name, "diagonal_corner_col")
            || name_matches(name, "edge_col")
            || name_matches(name, "full_col")
            || name_matches(name, "T_col")
    }

    pub fn visible_chunk_meshes(&mut self, frame: &WorldRenderFrame) -> Vec<ModelMesh> {
        if !frame.infinite_world || frame.chunk_load_radius == 0 || frame.chunk_size <= 0.0 {
            return Vec::new();
        }

        let player = frame
            .agents
            .iter()
            .find(|a| a.team == 0)
            .or_else(|| frame.agents.first())
            .map(|a| (a.x, a.y))
            .unwrap_or((0.0, 0.0));
        let chunk_size = frame.chunk_size.max(4.0);
        let center_cx = (player.0 / chunk_size).floor() as i32;
        let center_cy = (player.1 / chunk_size).floor() as i32;
        let radius = frame.chunk_load_radius as i32;

        let mut out = Vec::new();
        for cy in (center_cy - radius)..=(center_cy + radius) {
            for cx in (center_cx - radius)..=(center_cx + radius) {
                let key = (cx, cy);
                if !self.cache.contains_key(&key) {
                    let meshes = generate_chunk_meshes(cx, cy, chunk_size);
                    self.cache.insert(key, meshes);
                }
                if let Some(cached) = self.cache.get(&key) {
                    out.extend(cached.iter().cloned());
                }
            }
        }

        let keep_radius = radius + CACHE_MARGIN_CHUNKS;
        self.cache.retain(|(cx, cy), _| {
            (cx - center_cx).abs() <= keep_radius && (cy - center_cy).abs() <= keep_radius
        });

        out
    }
}

fn generate_chunk_meshes(chunk_x: i32, chunk_y: i32, chunk_size: f32) -> Vec<ModelMesh> {
    let mut grid = Grid2d::<CellType>::new(LOGIC_RESOLUTION, LOGIC_RESOLUTION);
    for y in 0..LOGIC_RESOLUTION {
        for x in 0..LOGIC_RESOLUTION {
            let wx = chunk_x * LOGIC_RESOLUTION as i32 + x as i32;
            let wy = chunk_y * LOGIC_RESOLUTION as i32 + y as i32;
            let occupancy = hash2_u32(wx, wy) > 0.58;
            grid.cells[y * LOGIC_RESOLUTION + x] = if occupancy {
                CellType::Wall
            } else {
                CellType::Empty
            };
        }
    }

    let dual_res = (LOGIC_RESOLUTION + 1) as f32;
    let tile_world = chunk_size / dual_res.max(1.0);
    let half = tile_world * 0.5 * (1.0 - TILE_GAP_RATIO).max(0.1);
    let chunk_origin_x = chunk_x as f32 * chunk_size;
    let chunk_origin_z = chunk_y as f32 * chunk_size;

    let mut corner = MeshBuilder::default();
    let mut diagonal = MeshBuilder::default();
    let mut edge = MeshBuilder::default();
    let mut t = MeshBuilder::default();
    let mut full = MeshBuilder::default();

    let tiles = grid.mesh_tiles();
    for (y, row) in tiles.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            if !tile.logic {
                continue;
            }
            let center_x = chunk_origin_x + ((x as f32 + 0.5) * tile_world);
            let center_z = chunk_origin_z + ((y as f32 + 0.5) * tile_world);
            match tile.visual.kind {
                MeshType::Corner => corner.push_quad(center_x, center_z, half),
                MeshType::Diagonal => diagonal.push_quad(center_x, center_z, half),
                MeshType::Edge => edge.push_quad(center_x, center_z, half),
                MeshType::T => t.push_quad(center_x, center_z, half),
                MeshType::Full => full.push_quad(center_x, center_z, half),
                MeshType::Empty | MeshType::Debug => {}
            }
        }
    }

    let mut out = Vec::new();
    push_mesh_if_any(
        &mut out,
        corner,
        [0.24, 0.70, 0.34, 1.0],
        format!("chunk_{chunk_x}_{chunk_y}_corner"),
    );
    push_mesh_if_any(
        &mut out,
        diagonal,
        [0.30, 0.60, 0.38, 1.0],
        format!("chunk_{chunk_x}_{chunk_y}_diag"),
    );
    push_mesh_if_any(
        &mut out,
        edge,
        [0.20, 0.78, 0.30, 1.0],
        format!("chunk_{chunk_x}_{chunk_y}_edge"),
    );
    push_mesh_if_any(
        &mut out,
        t,
        [0.28, 0.66, 0.30, 1.0],
        format!("chunk_{chunk_x}_{chunk_y}_t"),
    );
    push_mesh_if_any(
        &mut out,
        full,
        [0.16, 0.84, 0.24, 1.0],
        format!("chunk_{chunk_x}_{chunk_y}_full"),
    );
    out
}

fn push_mesh_if_any(out: &mut Vec<ModelMesh>, builder: MeshBuilder, color: [f32; 4], name: String) {
    if builder.vertices.is_empty() || builder.indices.is_empty() {
        return;
    }
    out.push(ModelMesh {
        name,
        vertices: builder.vertices,
        indices: builder.indices,
        material: ModelMaterial {
            base_color_factor: color,
            base_color_texture: None,
            nearest_sampling: false,
        },
    });
}

fn hash2_u32(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B9) ^ (y as u32).wrapping_mul(0x85EB_CA6B);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    (h as f32) / (u32::MAX as f32)
}

fn name_matches(name: &str, base: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let base = base.to_ascii_lowercase();
    lower == base
        || lower.starts_with(&format!("{base}."))
        || lower.starts_with(&format!("{base}_"))
}
