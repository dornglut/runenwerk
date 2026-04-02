use crate::{
    CavernGiConfig, CavernGiMode, CavernGiQuality, CavernMaterialQualityConfig,
    CavernMaterialRegistry, CavernMaterialRuntimeState, CavernRenderMode, GiProbeGrid,
    GiProbeUpdateQueue, MATERIAL_CLASS_ROCK, MaterialGraphAssetV1, MaterialProfileAssetV1,
    compile_material_graph,
};
use anyhow::Result;
use engine::plugins::world::WorldAuthorityState;
use engine::prelude::{App, Plugin, PreUpdate, Res, Startup, Time, World, WorldMut};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const MATERIAL_ROOT: &str = "game/assets/materials";
const MATERIAL_GRAPHS_DIR: &str = "game/assets/materials/graphs";
const MATERIAL_PROFILES_DIR: &str = "game/assets/materials/profiles";
const MATERIAL_PROFILE_FALLBACK: &str = "balanced";
const ENV_RENDER_MODE: &str = "CAVERN_RENDER_MODE";
const ENV_GI_MODE: &str = "CAVERN_GI_MODE";
const ENV_GI_QUALITY: &str = "CAVERN_GI_QUALITY";
const ENV_GI_SAMPLE_BUDGET: &str = "CAVERN_GI_SAMPLE_BUDGET";
const ENV_MATERIAL_PROFILE: &str = "CAVERN_MATERIAL_PROFILE";
const ENV_MATERIAL_WATCH: &str = "CAVERN_MATERIAL_WATCH";
const ENV_MATERIAL_POLL_SECS: &str = "CAVERN_MATERIAL_POLL_SECONDS";

pub struct CavernHuntMaterialPlugin;

impl Plugin for CavernHuntMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CavernMaterialRegistry>();
        app.init_resource::<CavernMaterialRuntimeState>();
        app.init_resource::<CavernMaterialQualityConfig>();
        app.init_resource::<GiProbeGrid>();
        app.init_resource::<GiProbeUpdateQueue>();
        app.add_systems(Startup, material_startup_system);
        app.add_systems(PreUpdate, material_hot_reload_system);
        app.add_systems(PreUpdate, gi_probe_scaffold_system);
    }
}

fn material_startup_system(mut world: WorldMut) -> Result<()> {
    configure_quality_from_env(&mut world)?;
    reload_material_assets(&mut world, true)
}

fn material_hot_reload_system(time: Res<Time>, mut world: WorldMut) -> Result<()> {
    let quality = world
        .resource::<CavernMaterialQualityConfig>()
        .cloned()
        .unwrap_or_default();
    if !quality.watch_enabled {
        return Ok(());
    }
    let should_reload = {
        let mut runtime = world.resource_mut::<CavernMaterialRuntimeState>()?;
        runtime.reload_accumulator_seconds += time.delta_seconds.max(0.0);
        if runtime.reload_accumulator_seconds < quality.poll_interval_seconds.max(0.1) {
            false
        } else {
            runtime.reload_accumulator_seconds = 0.0;
            true
        }
    };
    if !should_reload {
        return Ok(());
    }
    reload_material_assets(&mut world, false)
}

fn gi_probe_scaffold_system(mut world: WorldMut) -> Result<()> {
    let quality = world
        .resource::<CavernMaterialQualityConfig>()
        .cloned()
        .unwrap_or_default();
    if quality.gi.mode != CavernGiMode::ProbeGi {
        return Ok(());
    }

    let geometry_revision = world
        .resource::<WorldAuthorityState>()
        .map(|state| state.world_revision.0)
        .unwrap_or_default();
    let mut probe_grid = world.remove_resource::<GiProbeGrid>().unwrap_or_default();
    let mut update_queue = world
        .remove_resource::<GiProbeUpdateQueue>()
        .unwrap_or_default();

    if probe_grid.revision_seen != geometry_revision {
        probe_grid.revision_seen = geometry_revision;
        update_queue.pending_indices.clear();
        update_queue
            .pending_indices
            .extend(0..probe_grid.cells.len().min(u32::MAX as usize) as u32);
    }

    let update_budget = quality.gi.sample_budget.max(1) as usize;
    for index in update_queue.drain_budget(update_budget) {
        if let Some(cell) = probe_grid.cells.get_mut(index as usize) {
            // Placeholder update path: stage confidence warm-up so later
            // irradiance writes can piggyback on the same budgeted queue.
            cell.confidence = (cell.confidence + 0.1).min(1.0);
        }
    }

    world.insert_resource(probe_grid);
    world.insert_resource(update_queue);
    Ok(())
}

fn configure_quality_from_env(world: &mut World) -> Result<()> {
    let mut quality = world
        .resource::<CavernMaterialQualityConfig>()
        .cloned()
        .unwrap_or_default();
    if let Ok(value) = std::env::var(ENV_MATERIAL_PROFILE) {
        let value = value.trim();
        if !value.is_empty() {
            quality.profile_id = value.to_string();
        }
    }
    if let Ok(value) = std::env::var(ENV_RENDER_MODE) {
        quality.render_mode = parse_render_mode(&value).unwrap_or(quality.render_mode);
    }
    if let Ok(value) = std::env::var(ENV_GI_MODE) {
        quality.gi.mode = parse_gi_mode(&value).unwrap_or(quality.gi.mode);
    }
    if let Ok(value) = std::env::var(ENV_GI_QUALITY) {
        if let Some(parsed) = parse_gi_quality(&value) {
            quality.gi.quality = parsed;
            quality.gi.sample_budget = parsed.default_sample_budget();
        }
    }
    if let Ok(value) = std::env::var(ENV_GI_SAMPLE_BUDGET) {
        if let Ok(parsed) = value.parse::<u32>() {
            quality.gi.sample_budget = parsed.clamp(1, 64);
        }
    }
    if let Ok(value) = std::env::var(ENV_MATERIAL_WATCH) {
        let normalized = value.trim().to_ascii_lowercase();
        quality.watch_enabled = matches!(normalized.as_str(), "1" | "true" | "yes" | "on");
    }
    if let Ok(value) = std::env::var(ENV_MATERIAL_POLL_SECS) {
        if let Ok(parsed) = value.parse::<f32>() {
            quality.poll_interval_seconds = parsed.clamp(0.1, 10.0);
        }
    }
    world.insert_resource(quality);
    Ok(())
}

fn reload_material_assets(world: &mut World, force_profile_defaults: bool) -> Result<()> {
    let mut registry = world
        .remove_resource::<CavernMaterialRegistry>()
        .unwrap_or_default();
    let mut runtime = world
        .remove_resource::<CavernMaterialRuntimeState>()
        .unwrap_or_default();
    let mut quality = world
        .resource::<CavernMaterialQualityConfig>()
        .cloned()
        .unwrap_or_default();
    runtime.clear_diagnostics();

    let previous_programs = runtime.compiled_graphs.clone();
    let previous_class_programs = runtime.class_programs.clone();
    let previous_profile = runtime.active_profile_id.clone();
    let previous_revision = runtime.revision;

    let graph_files = scan_ron_files(Path::new(MATERIAL_GRAPHS_DIR));
    let profile_files = scan_ron_files(Path::new(MATERIAL_PROFILES_DIR));

    if graph_files.is_empty() || profile_files.is_empty() {
        runtime.push_diagnostic(format!(
            "material assets missing under {MATERIAL_ROOT}; expected graphs/ and profiles/"
        ));
    }

    registry.graph_files.clear();
    registry.profile_files.clear();
    registry.graphs.clear();
    registry.profiles.clear();

    for path in graph_files {
        match load_graph_file(&path) {
            Ok(graph) => {
                registry.graph_files.insert(
                    graph.id.clone(),
                    crate::MaterialAssetFileEntry {
                        path: path.clone(),
                        modified: file_modified(&path),
                    },
                );
                registry.graphs.insert(graph.id.clone(), graph);
            }
            Err(err) => runtime.push_diagnostic(err),
        }
    }
    for path in profile_files {
        match load_profile_file(&path) {
            Ok(profile) => {
                registry.profile_files.insert(
                    profile.id.clone(),
                    crate::MaterialAssetFileEntry {
                        path: path.clone(),
                        modified: file_modified(&path),
                    },
                );
                registry.profiles.insert(profile.id.clone(), profile);
            }
            Err(err) => runtime.push_diagnostic(err),
        }
    }

    let mut next_programs = previous_programs.clone();
    for (graph_id, graph) in &registry.graphs {
        match compile_material_graph(graph) {
            Ok(program) => {
                next_programs.insert(graph_id.clone(), program);
            }
            Err(err) => {
                runtime.push_diagnostic(format!(
                    "graph compile failed id={} node={:?}: {}",
                    graph_id, err.node, err.message
                ));
            }
        }
    }
    runtime.compiled_graphs = next_programs;

    let mut selected_profile = select_profile(&registry, &quality.profile_id);
    if selected_profile.is_none() && !registry.profiles.is_empty() {
        selected_profile = select_profile(&registry, MATERIAL_PROFILE_FALLBACK);
        if selected_profile.is_none() {
            selected_profile = registry.profiles.values().next().cloned();
        }
    }
    if let Some(profile) = selected_profile.clone() {
        if force_profile_defaults {
            quality.render_mode = profile.render_mode;
            quality.gi = CavernGiConfig {
                mode: profile.gi_mode,
                quality: profile.gi_quality,
                sample_budget: profile.gi_quality.default_sample_budget(),
            };
        }
        runtime.active_profile_id = Some(profile.id.clone());
        runtime.active_profile = Some(profile.clone());
        runtime.class_programs.clear();
        for (class_id, graph_id) in &profile.class_graphs {
            if let Some(program) = runtime.compiled_graphs.get(graph_id).cloned() {
                runtime.class_programs.insert(*class_id, program);
            } else {
                runtime.push_diagnostic(format!(
                    "profile '{}' references missing graph '{}'",
                    profile.id, graph_id
                ));
            }
        }
    } else {
        runtime.push_diagnostic("no material profile loaded; falling back to legacy shading");
        runtime.active_profile_id = None;
        runtime.active_profile = None;
        runtime.class_programs.clear();
    }

    // Environment flags remain the final override layer so local tuning and
    // emergency fallback do not require editing profile assets.
    apply_quality_overrides_from_env(&mut quality);

    if runtime.class_programs.get(&MATERIAL_CLASS_ROCK).is_none() {
        if let Some(default_rock) = runtime.compiled_graphs.get("rock_terrain").cloned() {
            runtime
                .class_programs
                .insert(MATERIAL_CLASS_ROCK, default_rock);
        }
    }

    if runtime.class_programs != previous_class_programs
        || runtime.active_profile_id != previous_profile
    {
        runtime.bump_revision();
    }
    let changed = runtime.revision != previous_revision;
    if force_profile_defaults || changed {
        if let Some(profile_id) = runtime.active_profile_id.as_deref() {
            tracing::info!(
                profile_id,
                graph_count = runtime.compiled_graphs.len(),
                class_program_count = runtime.class_programs.len(),
                render_mode = ?quality.render_mode,
                gi_mode = ?quality.gi.mode,
                gi_quality = ?quality.gi.quality,
                gi_samples = quality.gi.sample_budget,
                "cavern material runtime ready"
            );
        } else {
            tracing::warn!(
                "cavern material runtime has no active profile; legacy shading remains active"
            );
        }
    }
    if !runtime.diagnostics.is_empty() && (force_profile_defaults || changed) {
        for diagnostic in &runtime.diagnostics {
            tracing::warn!(%diagnostic, "cavern material diagnostic");
        }
    }

    world.insert_resource(registry);
    world.insert_resource(runtime);
    world.insert_resource(quality);
    Ok(())
}

fn apply_quality_overrides_from_env(quality: &mut CavernMaterialQualityConfig) {
    if let Ok(value) = std::env::var(ENV_RENDER_MODE) {
        quality.render_mode = parse_render_mode(&value).unwrap_or(quality.render_mode);
    }
    if let Ok(value) = std::env::var(ENV_GI_MODE) {
        quality.gi.mode = parse_gi_mode(&value).unwrap_or(quality.gi.mode);
    }
    if let Ok(value) = std::env::var(ENV_GI_QUALITY) {
        if let Some(parsed) = parse_gi_quality(&value) {
            quality.gi.quality = parsed;
            quality.gi.sample_budget = parsed.default_sample_budget();
        }
    }
    if let Ok(value) = std::env::var(ENV_GI_SAMPLE_BUDGET) {
        if let Ok(parsed) = value.parse::<u32>() {
            quality.gi.sample_budget = parsed.clamp(1, 64);
        }
    }
}

fn select_profile(
    registry: &CavernMaterialRegistry,
    profile_id: &str,
) -> Option<MaterialProfileAssetV1> {
    let profile_id = profile_id.trim();
    registry.profiles.get(profile_id).cloned()
}

fn scan_ron_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("ron") {
            continue;
        }
        out.push(path);
    }
    out.sort();
    out
}

fn load_graph_file(path: &Path) -> std::result::Result<MaterialGraphAssetV1, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed reading graph {}: {err}", path.display()))?;
    let graph: MaterialGraphAssetV1 = ron::from_str(&raw)
        .map_err(|err| format!("failed parsing graph {}: {err}", path.display()))?;
    Ok(graph)
}

fn load_profile_file(path: &Path) -> std::result::Result<MaterialProfileAssetV1, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed reading profile {}: {err}", path.display()))?;
    let profile: MaterialProfileAssetV1 = ron::from_str(&raw)
        .map_err(|err| format!("failed parsing profile {}: {err}", path.display()))?;
    Ok(profile)
}

fn file_modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}

fn parse_render_mode(value: &str) -> Option<CavernRenderMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "legacy" => Some(CavernRenderMode::Legacy),
        "material_graph" | "materials" | "pbr" => Some(CavernRenderMode::MaterialGraph),
        _ => None,
    }
}

fn parse_gi_mode(value: &str) -> Option<CavernGiMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "off" => Some(CavernGiMode::Off),
        "ao" | "ao_bent" | "aobentnormal" | "ao_bent_normal" => Some(CavernGiMode::AoBentNormal),
        "probes" | "probe" | "ddgi" => Some(CavernGiMode::ProbeGi),
        _ => None,
    }
}

fn parse_gi_quality(value: &str) -> Option<CavernGiQuality> {
    match value.trim().to_ascii_lowercase().as_str() {
        "low" => Some(CavernGiQuality::Low),
        "medium" | "med" => Some(CavernGiQuality::Medium),
        "high" => Some(CavernGiQuality::High),
        _ => None,
    }
}
