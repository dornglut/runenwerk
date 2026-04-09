use chunking::{
    ChunkLoadOrder, ChunkStreamer, ChunkStreamingConfig, ChunkStreamingMode, StreamingFocus,
};
use godot::builtin::{Dictionary, Variant, Vector3};
use godot::classes::{INode, Node};
use godot::prelude::*;
use spatial::GridPartitionConfig;

use crate::bridge::vector3_to_meters;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct ChunkStreamingNode {
    base: Base<Node>,

    chunk_edge_meters: f32,
    region_dim_x: u32,
    region_dim_y: u32,
    region_dim_z: u32,
    fixed_point_scale: i32,

    load_radius_chunks: i32,
    unload_radius_chunks: i32,
    vertical_load_radius_chunks: i32,
    vertical_unload_radius_chunks: i32,

    streamer: Option<ChunkStreamer>,
}

#[godot_api]
impl INode for ChunkStreamingNode {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            chunk_edge_meters: 32.0,
            region_dim_x: 8,
            region_dim_y: 8,
            region_dim_z: 8,
            fixed_point_scale: 1024,
            load_radius_chunks: 4,
            unload_radius_chunks: 6,
            vertical_load_radius_chunks: 1,
            vertical_unload_radius_chunks: 2,
            streamer: None,
        }
    }

    fn ready(&mut self) {
        self.rebuild_streamer();
    }
}

#[godot_api]
impl ChunkStreamingNode {
    #[signal]
    fn chunk_entered(x: i32, y: i32, z: i32);

    #[signal]
    fn chunk_exited(x: i32, y: i32, z: i32);

    #[signal]
    fn active_chunk_count_changed(count: i64);

    #[func]
    pub fn set_chunk_edge_meters(&mut self, value: f32) {
        self.chunk_edge_meters = value.max(1.0);
        self.rebuild_streamer();
    }

    #[func]
    pub fn get_chunk_edge_meters(&self) -> f32 {
        self.chunk_edge_meters
    }

    #[func]
    pub fn set_region_chunk_dims(&mut self, x: i32, y: i32, z: i32) {
        self.region_dim_x = x.max(1) as u32;
        self.region_dim_y = y.max(1) as u32;
        self.region_dim_z = z.max(1) as u32;
        self.rebuild_streamer();
    }

    #[func]
    pub fn set_fixed_point_scale(&mut self, value: i32) {
        self.fixed_point_scale = value.max(1);
        self.rebuild_streamer();
    }

    #[func]
    pub fn set_load_radii(
        &mut self,
        load_radius_chunks: i32,
        unload_radius_chunks: i32,
        vertical_load_radius_chunks: i32,
        vertical_unload_radius_chunks: i32,
    ) {
        self.load_radius_chunks = load_radius_chunks;
        self.unload_radius_chunks = unload_radius_chunks;
        self.vertical_load_radius_chunks = vertical_load_radius_chunks;
        self.vertical_unload_radius_chunks = vertical_unload_radius_chunks;
        self.rebuild_streamer();
    }

    #[func]
    pub fn set_planar_xz_mode(&mut self) {
        if let Some(streamer) = &mut self.streamer {
            let mut config = streamer.config();
            config.mode = ChunkStreamingMode::PlanarXZ;
            streamer.set_config(config);
        }
    }

    #[func]
    pub fn set_volume_3d_mode(&mut self) {
        if let Some(streamer) = &mut self.streamer {
            let mut config = streamer.config();
            config.mode = ChunkStreamingMode::Volume3D;
            streamer.set_config(config);
        }
    }

    #[func]
    pub fn clear_active_chunks(&mut self) {
        if let Some(streamer) = &mut self.streamer {
            streamer.clear();
            self.signals().active_chunk_count_changed().emit(0);
        }
    }

    #[func]
    pub fn active_chunk_count(&self) -> i64 {
        self.streamer
            .as_ref()
            .map(|streamer| streamer.active_chunk_count() as i64)
            .unwrap_or(0)
    }

    #[func]
    pub fn update_focus_from_vector3(&mut self, position: Vector3) {
        let (entered, exited, active_count) = {
            let Some(streamer) = &mut self.streamer else {
                return;
            };

            let diff = streamer.update_focus(StreamingFocus::new(vector3_to_meters(position)));
            let active_count = streamer.active_chunk_count() as i64;

            (diff.entered, diff.exited, active_count)
        };

        for chunk in entered {
            self.signals()
                .chunk_entered()
                .emit(chunk.x, chunk.y, chunk.z);
        }

        for chunk in exited {
            self.signals()
                .chunk_exited()
                .emit(chunk.x, chunk.y, chunk.z);
        }

        self.signals()
            .active_chunk_count_changed()
            .emit(active_count);
    }

    #[func]
    pub fn describe_config(&self) -> Dictionary<Variant, Variant> {
        let mut dict = Dictionary::<Variant, Variant>::new();
        dict.set("chunk_edge_meters", self.chunk_edge_meters);
        dict.set("region_dim_x", self.region_dim_x as i64);
        dict.set("region_dim_y", self.region_dim_y as i64);
        dict.set("region_dim_z", self.region_dim_z as i64);
        dict.set("fixed_point_scale", self.fixed_point_scale);
        dict.set("load_radius_chunks", self.load_radius_chunks);
        dict.set("unload_radius_chunks", self.unload_radius_chunks);
        dict.set(
            "vertical_load_radius_chunks",
            self.vertical_load_radius_chunks,
        );
        dict.set(
            "vertical_unload_radius_chunks",
            self.vertical_unload_radius_chunks,
        );
        dict
    }

    fn rebuild_streamer(&mut self) {
        let partition = GridPartitionConfig {
            chunk_edge_meters: self.chunk_edge_meters.max(1.0),
            region_chunk_dims: [
                self.region_dim_x.max(1),
                self.region_dim_y.max(1),
                self.region_dim_z.max(1),
            ],
            fixed_point_scale: self.fixed_point_scale.max(1),
        };

        let config = ChunkStreamingConfig {
            load_radius_chunks: self.load_radius_chunks,
            unload_radius_chunks: self.unload_radius_chunks,
            vertical_load_radius_chunks: self.vertical_load_radius_chunks,
            vertical_unload_radius_chunks: self.vertical_unload_radius_chunks,
            mode: ChunkStreamingMode::PlanarXZ,
            load_order: ChunkLoadOrder::NearestFirst,
        };

        self.streamer = Some(ChunkStreamer::new(partition, config));
    }
}
