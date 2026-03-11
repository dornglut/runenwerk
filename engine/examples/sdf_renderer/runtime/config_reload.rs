// Owner: SDF Renderer Example - Config Load and Hot Reload
use crate::*;

pub(crate) fn apply_sdf_params(state: &mut SdfWorldState, params: &SdfParamsConfig) {
    let _ = (
        params.world_scene_label.as_str(),
        params.overlay_scene_label.as_str(),
        params.render_mesh_overlay,
    );
    state.world_bounds = params.world_bounds;
    state.camera_target = params.camera.target;
    state.camera_yaw = params.camera.yaw;
    state.camera_pitch = params.camera.pitch;
    state.camera_distance = params.camera.distance;
    state.camera_pitch_min = params.camera.pitch_min;
    state.camera_pitch_max = params.camera.pitch_max;
    state.camera_distance_min = params.camera.distance_min;
    state.camera_distance_max = params.camera.distance_max;
    state.camera_fov_y = params.camera.fov_y_radians;
    state.world_paused = params.world_paused;
    state.debug_view_mode = params.debug_view_mode;
    state.display_fit_mode = params.display.fit_mode.as_shader_mode();
    state.display_target_aspect = params.display.target_aspect.max(0.0);
    state.display_render_scale = params.display.render_scale.clamp(0.25, 4.0);
    state.display_bar_color = params.display.bar_color;
}

pub(crate) fn maybe_reload_sdf_params(
    state: &mut SdfWorldState,
    runtime_config: &mut SdfRuntimeConfigState,
) {
    let latest_path = find_config_path(PARAMS_CONFIG_FILE);
    let latest_modified = file_modified(&latest_path);
    let changed = latest_path != runtime_config.params_config_path
        || latest_modified != runtime_config.params_config_modified;
    if !changed {
        return;
    }

    runtime_config.params_config_path = latest_path;
    runtime_config.params_config_modified = latest_modified;

    match try_load_config::<SdfParamsConfig>(PARAMS_CONFIG_FILE) {
        Ok(params) => {
            apply_sdf_params(state, &params);
            runtime_config.controls = params.controls;
            tracing::info!(
                config_path = runtime_config.params_config_path.display().to_string(),
                display_fit_mode = state.display_fit_mode,
                display_target_aspect = state.display_target_aspect,
                display_render_scale = state.display_render_scale,
                "sdf params hot reloaded"
            );
        }
        Err(err) => {
            tracing::error!(
                config_path = runtime_config.params_config_path.display().to_string(),
                ?err,
                "sdf params hot reload failed; keeping previous runtime values"
            );
        }
    }
}

pub(crate) fn load_config_with_default<T>(file_name: &str) -> T
where
    T: DeserializeOwned + Default,
{
    let config_path = find_config_path(file_name);
    let raw = match fs::read_to_string(&config_path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!(
                config = file_name,
                path = config_path.display().to_string(),
                "sdf config file missing; using built-in defaults"
            );
            return T::default();
        }
        Err(err) => {
            tracing::error!(
                config = file_name,
                path = config_path.display().to_string(),
                ?err,
                "sdf config file read failed; using built-in defaults"
            );
            return T::default();
        }
    };

    match ron::from_str::<T>(&raw) {
        Ok(parsed) => parsed,
        Err(err) => {
            tracing::error!(
                config = file_name,
                path = config_path.display().to_string(),
                ?err,
                "sdf config parse failed; using built-in defaults"
            );
            T::default()
        }
    }
}

pub(crate) fn try_load_config<T>(file_name: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let config_path = find_config_path(file_name);
    let raw = fs::read_to_string(&config_path).map_err(|err| {
        anyhow!(
            "read failed for '{}': {} ({})",
            file_name,
            config_path.display(),
            err
        )
    })?;
    ron::from_str::<T>(&raw).map_err(|err| {
        anyhow!(
            "parse failed for '{}': {} ({})",
            file_name,
            config_path.display(),
            err
        )
    })
}

pub(crate) fn find_config_path(file_name: &str) -> PathBuf {
    let primary = Path::new(SDF_ASSETS_DIR_PRIMARY).join(file_name);
    if primary.exists() {
        return primary;
    }
    let fallback = Path::new(SDF_ASSETS_DIR_FALLBACK).join(file_name);
    if fallback.exists() {
        return fallback;
    }
    primary
}

pub(crate) fn file_modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}
