// Owner: Cavern Hunt Domain Resources - Replication, Networking, and SDF Frame State
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AdaptiveSmoothingState {
    pub target_delay_ms: f32,
    pub effective_delay_ms: f32,
    pub last_rtt_ms: f32,
    pub jitter_ms: f32,
    pub samples: u64,
}

impl Default for AdaptiveSmoothingState {
    fn default() -> Self {
        Self {
            target_delay_ms: 80.0,
            effective_delay_ms: 80.0,
            last_rtt_ms: 0.0,
            jitter_ms: 0.0,
            samples: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct CorrectionStats {
    pub small_corrections: u64,
    pub medium_corrections: u64,
    pub large_corrections: u64,
    pub hard_snaps: u64,
    pub total_distance: f32,
    pub last_distance: f32,
    pub ema_distance: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ReplicationRuntimeMetrics {
    pub bytes_sent_last_tick: u64,
    pub bytes_sent_total: u64,
    pub load_shed_level_last_tick: u8,
    pub bytes_sent_geometry_last_tick: u64,
    pub bytes_sent_keyframe_last_tick: u64,
    pub bytes_sent_patch_last_tick: u64,
    pub bytes_sent_player_ops_last_tick: u64,
    pub bytes_sent_enemy_ops_last_tick: u64,
    pub bytes_sent_projectile_ops_last_tick: u64,
    pub bytes_sent_pickup_ops_last_tick: u64,
    pub bytes_sent_extraction_ops_last_tick: u64,
    pub patch_player_ops_last_tick: u64,
    pub patch_enemy_ops_last_tick: u64,
    pub patch_projectile_ops_last_tick: u64,
    pub patch_pickup_ops_last_tick: u64,
    pub patch_extraction_ops_last_tick: u64,
    pub dropped_enemy_ops_last_tick: u64,
    pub dropped_projectile_ops_last_tick: u64,
    pub dropped_pickup_ops_last_tick: u64,
    pub dropped_extraction_ops_last_tick: u64,
    pub dropped_enemy_ops_total: u64,
    pub dropped_projectile_ops_total: u64,
    pub dropped_pickup_ops_total: u64,
    pub dropped_extraction_ops_total: u64,
    pub bytes_received_last_frame: u64,
    pub bytes_received_total: u64,
    pub keyframes_received_last_frame: u64,
    pub patches_received_last_frame: u64,
    pub patches_applied_last_frame: u64,
    pub patches_skipped_base_mismatch_last_frame: u64,
    pub patches_stale_ignored_last_frame: u64,
    pub patch_apply_micros_last: u64,
    pub patch_apply_micros_total: u64,
    pub keyframes_applied: u64,
    pub patches_applied: u64,
    pub full_world_restores: u64,
    pub smoothing_samples_last_frame: u64,
    pub smoothing_error_mean_last_frame: f32,
    pub smoothing_error_max_last_frame: f32,
    pub smoothing_alpha_mean_last_frame: f32,
    pub smoothing_extrapolation_ms_last_frame: f32,
    pub local_correction_distance_last: f32,
    pub local_correction_hard_snaps_total: u64,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplicationBudgetConfig {
    pub enemy_ops_per_patch_level0: usize,
    pub enemy_ops_per_patch_level1: usize,
    pub enemy_ops_per_patch_level2: usize,
    pub projectile_ops_per_patch_level0: usize,
    pub projectile_ops_per_patch_level1: usize,
    pub projectile_ops_per_patch_level2: usize,
    pub pickup_ops_per_patch_level0: usize,
    pub pickup_ops_per_patch_level1: usize,
    pub pickup_ops_per_patch_level2: usize,
    pub extraction_ops_per_patch_level0: usize,
    pub extraction_ops_per_patch_level1: usize,
    pub extraction_ops_per_patch_level2: usize,
}

impl Default for ReplicationBudgetConfig {
    fn default() -> Self {
        Self {
            enemy_ops_per_patch_level0: 128,
            enemy_ops_per_patch_level1: 72,
            enemy_ops_per_patch_level2: 36,
            projectile_ops_per_patch_level0: 256,
            projectile_ops_per_patch_level1: 128,
            projectile_ops_per_patch_level2: 64,
            pickup_ops_per_patch_level0: 64,
            pickup_ops_per_patch_level1: 32,
            pickup_ops_per_patch_level2: 16,
            extraction_ops_per_patch_level0: 16,
            extraction_ops_per_patch_level1: 8,
            extraction_ops_per_patch_level2: 4,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplicationCadenceConfig {
    pub patch_emit_interval_level0: u64,
    pub patch_emit_interval_level1: u64,
    pub patch_emit_interval_level2: u64,
    pub enemy_patch_interval_level0: u64,
    pub enemy_patch_interval_level1: u64,
    pub enemy_patch_interval_level2: u64,
    pub projectile_patch_interval_level0: u64,
    pub projectile_patch_interval_level1: u64,
    pub projectile_patch_interval_level2: u64,
    pub pickup_patch_interval_level0: u64,
    pub pickup_patch_interval_level1: u64,
    pub pickup_patch_interval_level2: u64,
    pub extraction_patch_interval_level0: u64,
    pub extraction_patch_interval_level1: u64,
    pub extraction_patch_interval_level2: u64,
}

impl Default for ReplicationCadenceConfig {
    fn default() -> Self {
        Self {
            patch_emit_interval_level0: 1,
            patch_emit_interval_level1: 2,
            patch_emit_interval_level2: 3,
            enemy_patch_interval_level0: 1,
            enemy_patch_interval_level1: 2,
            enemy_patch_interval_level2: 4,
            projectile_patch_interval_level0: 1,
            projectile_patch_interval_level1: 2,
            projectile_patch_interval_level2: 3,
            pickup_patch_interval_level0: 4,
            pickup_patch_interval_level1: 6,
            pickup_patch_interval_level2: 10,
            extraction_patch_interval_level0: 1,
            extraction_patch_interval_level1: 1,
            extraction_patch_interval_level2: 2,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplicationLoadShedConfig {
    pub bytes_threshold_level1: u64,
    pub bytes_threshold_level2: u64,
    pub dropped_ops_threshold_level1: u64,
    pub dropped_ops_threshold_level2: u64,
    pub connections_force_level1_at_or_above: usize,
    pub connections_force_level2_bytes_threshold: u64,
}

impl Default for ReplicationLoadShedConfig {
    fn default() -> Self {
        Self {
            bytes_threshold_level1: 60_000,
            bytes_threshold_level2: 100_000,
            dropped_ops_threshold_level1: 1,
            dropped_ops_threshold_level2: 24,
            connections_force_level1_at_or_above: 3,
            connections_force_level2_bytes_threshold: 45_000,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplicationKeyframeConfig {
    pub interval_ticks: u64,
    pub emit_on_cursor_mismatch: bool,
    pub emit_on_reconnect: bool,
}

impl Default for ReplicationKeyframeConfig {
    fn default() -> Self {
        Self {
            interval_ticks: 60,
            emit_on_cursor_mismatch: true,
            emit_on_reconnect: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CavernServerControlMap {
    pub by_player_id: BTreeMap<u32, CavernControlState>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CavernServerAppliedInputTickMap {
    pub by_player_id: BTreeMap<u32, SimulationTick>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CavernPlayerOwnershipState {
    pub by_connection_id: BTreeMap<u64, u32>,
}

impl CavernPlayerOwnershipState {
    pub fn retain_active_connections<I>(&mut self, active_connections: I)
    where
        I: IntoIterator<Item = u64>,
    {
        let active_connections = active_connections
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        self.by_connection_id
            .retain(|connection_id, _| active_connections.contains(connection_id));
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CavernGeometryRuntimeState {
    pub extraction_seal_primitive: Option<GeometryPrimitiveId>,
    pub edit_events: Vec<GeometryEditEvent>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfAgent {
    pub pos: [f32; 2],
    pub radius: f32,
    pub health_ratio: f32,
    pub team: u32,
    pub kind: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfGeometryPrimitive {
    pub shape_kind: u32,
    pub op_kind: u32,
    pub material_class: u32,
    pub material_instance: u32,
    pub p0: [f32; 4],
    pub p1: [f32; 4],
    pub p2: [f32; 4],
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfMaterialProgramHeader {
    pub class_id: u32,
    pub op_offset: u32,
    pub op_count: u32,
    pub const_offset: u32,
    pub const_count: u32,
    pub base_color_slot: u32,
    pub roughness_slot: u32,
    pub metallic_slot: u32,
    pub normal_perturb_slot: u32,
    pub ao_slot: u32,
    pub emissive_slot: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfMaterialOp {
    pub op: u32,
    pub dst: u32,
    pub src_a: u32,
    pub src_b: u32,
    pub src_c: u32,
    pub const_idx: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CavernSdfWorldFrame {
    pub world_bounds: [f32; 4],
    pub floor_height: f32,
    pub rock_height: f32,
    pub camera: CavernCameraState,
    pub render_mode: u32,
    pub gi_mode: u32,
    pub gi_quality: u32,
    pub gi_sample_budget: u32,
    pub material_program_headers: Vec<CavernSdfMaterialProgramHeader>,
    pub material_ops: Vec<CavernSdfMaterialOp>,
    pub material_constants: Vec<[f32; 4]>,
    pub geometry_primitives: Vec<CavernSdfGeometryPrimitive>,
    pub agents: Vec<CavernSdfAgent>,
}

impl Default for CavernSdfWorldFrame {
    fn default() -> Self {
        Self {
            world_bounds: [-24.0, -24.0, 24.0, 24.0],
            floor_height: 0.0,
            rock_height: 3.8,
            camera: CavernCameraState::default(),
            render_mode: crate::domain::CAVERN_RENDER_MODE_MATERIAL_GRAPH,
            gi_mode: crate::domain::CAVERN_GI_MODE_AO_BENT,
            gi_quality: 1,
            gi_sample_budget: 14,
            material_program_headers: Vec::new(),
            material_ops: Vec::new(),
            material_constants: Vec::new(),
            geometry_primitives: Vec::new(),
            agents: Vec::new(),
        }
    }
}
