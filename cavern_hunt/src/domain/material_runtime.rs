use crate::domain::material_graph::{MaterialGraphAssetV1, MaterialProgramV1};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::SystemTime;

pub const MATERIAL_CLASS_ROCK: u32 = 0;
pub const MATERIAL_CLASS_BARRIER: u32 = 1;
pub const MATERIAL_CLASS_HAZARD: u32 = 2;
pub const MATERIAL_CLASS_MARKER: u32 = 3;

pub const CAVERN_RENDER_MODE_LEGACY: u32 = 0;
pub const CAVERN_RENDER_MODE_MATERIAL_GRAPH: u32 = 1;
pub const CAVERN_GI_MODE_OFF: u32 = 0;
pub const CAVERN_GI_MODE_AO_BENT: u32 = 1;
pub const CAVERN_GI_MODE_PROBE: u32 = 2;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CavernRenderMode {
    Legacy,
    MaterialGraph,
}

impl CavernRenderMode {
    pub fn as_gpu_u32(self) -> u32 {
        match self {
            Self::Legacy => CAVERN_RENDER_MODE_LEGACY,
            Self::MaterialGraph => CAVERN_RENDER_MODE_MATERIAL_GRAPH,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CavernGiMode {
    Off,
    AoBentNormal,
    ProbeGi,
}

impl CavernGiMode {
    pub fn as_gpu_u32(self) -> u32 {
        match self {
            Self::Off => CAVERN_GI_MODE_OFF,
            Self::AoBentNormal => CAVERN_GI_MODE_AO_BENT,
            Self::ProbeGi => CAVERN_GI_MODE_PROBE,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CavernGiQuality {
    Low,
    Medium,
    High,
}

impl CavernGiQuality {
    pub fn as_gpu_u32(self) -> u32 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
        }
    }

    pub fn default_sample_budget(self) -> u32 {
        match self {
            Self::Low => 8,
            Self::Medium => 14,
            Self::High => 20,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CavernGiConfig {
    pub mode: CavernGiMode,
    pub quality: CavernGiQuality,
    pub sample_budget: u32,
}

impl Default for CavernGiConfig {
    fn default() -> Self {
        let quality = CavernGiQuality::Medium;
        Self {
            mode: CavernGiMode::AoBentNormal,
            quality,
            sample_budget: quality.default_sample_budget(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CavernMaterialQualityConfig {
    pub profile_id: String,
    pub render_mode: CavernRenderMode,
    pub gi: CavernGiConfig,
    pub watch_enabled: bool,
    pub poll_interval_seconds: f32,
}

impl Default for CavernMaterialQualityConfig {
    fn default() -> Self {
        Self {
            profile_id: "balanced".to_string(),
            render_mode: CavernRenderMode::MaterialGraph,
            gi: CavernGiConfig::default(),
            watch_enabled: true,
            poll_interval_seconds: 0.75,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialProfileAssetV1 {
    pub version: u32,
    pub id: String,
    pub render_mode: CavernRenderMode,
    pub gi_mode: CavernGiMode,
    pub gi_quality: CavernGiQuality,
    pub class_graphs: BTreeMap<u32, String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MaterialAssetFileEntry {
    pub path: PathBuf,
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CavernMaterialRegistry {
    pub graph_files: BTreeMap<String, MaterialAssetFileEntry>,
    pub profile_files: BTreeMap<String, MaterialAssetFileEntry>,
    pub graphs: BTreeMap<String, MaterialGraphAssetV1>,
    pub profiles: BTreeMap<String, MaterialProfileAssetV1>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MaterialProgramHeaderV1 {
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MaterialGpuPayloadV1 {
    pub headers: Vec<MaterialProgramHeaderV1>,
    pub ops: Vec<crate::domain::MaterialOpCodeV1>,
    pub constants: Vec<[f32; 4]>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CavernMaterialRuntimeState {
    pub compiled_graphs: BTreeMap<String, MaterialProgramV1>,
    pub active_profile_id: Option<String>,
    pub active_profile: Option<MaterialProfileAssetV1>,
    pub class_programs: BTreeMap<u32, MaterialProgramV1>,
    pub diagnostics: Vec<String>,
    pub revision: u64,
    pub reload_accumulator_seconds: f32,
}

impl CavernMaterialRuntimeState {
    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    pub fn push_diagnostic(&mut self, message: impl Into<String>) {
        self.diagnostics.push(message.into());
    }

    pub fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }

    pub fn build_gpu_payload(
        &self,
        max_programs: usize,
        max_ops: usize,
        max_constants: usize,
    ) -> MaterialGpuPayloadV1 {
        let mut payload = MaterialGpuPayloadV1::default();
        let mut op_offset = 0_u32;
        let mut const_offset = 0_u32;
        for (class_id, program) in self.class_programs.iter().take(max_programs) {
            if payload.ops.len() >= max_ops || payload.constants.len() >= max_constants {
                break;
            }
            let op_remaining = max_ops.saturating_sub(payload.ops.len());
            let const_remaining = max_constants.saturating_sub(payload.constants.len());
            let op_count = program.ops.len().min(op_remaining);
            let const_count = program.constants.len().min(const_remaining);
            if op_count == 0 {
                continue;
            }
            payload.headers.push(MaterialProgramHeaderV1 {
                class_id: *class_id,
                op_offset,
                op_count: op_count as u32,
                const_offset,
                const_count: const_count as u32,
                base_color_slot: program.outputs.base_color,
                roughness_slot: program.outputs.roughness,
                metallic_slot: program.outputs.metallic,
                normal_perturb_slot: program.outputs.normal_perturb,
                ao_slot: program.outputs.ao,
                emissive_slot: program.outputs.emissive,
            });
            payload
                .ops
                .extend(program.ops.iter().take(op_count).cloned());
            payload
                .constants
                .extend(program.constants.iter().take(const_count).copied());
            op_offset = op_offset.saturating_add(op_count as u32);
            const_offset = const_offset.saturating_add(const_count as u32);
        }
        payload
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GiProbeCell {
    pub sh_coefficients: [[f32; 3]; 9],
    pub visibility: f32,
    pub confidence: f32,
}

impl Default for GiProbeCell {
    fn default() -> Self {
        Self {
            sh_coefficients: [[0.0; 3]; 9],
            visibility: 1.0,
            confidence: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GiProbeGrid {
    pub dimensions: [u32; 3],
    pub origin: [f32; 3],
    pub spacing: f32,
    pub cells: Vec<GiProbeCell>,
    pub revision_seen: u64,
}

impl Default for GiProbeGrid {
    fn default() -> Self {
        let dimensions = [8, 4, 8];
        let cell_count = (dimensions[0] * dimensions[1] * dimensions[2]) as usize;
        Self {
            dimensions,
            origin: [-16.0, 0.0, -16.0],
            spacing: 4.0,
            cells: vec![GiProbeCell::default(); cell_count],
            revision_seen: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GiProbeUpdateQueue {
    pub pending_indices: Vec<u32>,
}

impl GiProbeUpdateQueue {
    pub fn push(&mut self, index: u32) {
        self.pending_indices.push(index);
    }

    pub fn drain_budget(&mut self, budget: usize) -> Vec<u32> {
        let budget = budget.min(self.pending_indices.len());
        self.pending_indices.drain(..budget).collect()
    }
}
