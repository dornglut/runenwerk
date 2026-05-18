use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use editor_persistence::{decode_ron, encode_ron_pretty};
use serde::Deserialize;

use editor_shell::{
    PERSISTED_WORKSPACE_STATE_VERSION_V1, PERSISTED_WORKSPACE_STATE_VERSION_V2,
    PERSISTED_WORKSPACE_STATE_VERSION_V3, PERSISTED_WORKSPACE_STATE_VERSION_V4,
    PERSISTED_WORKSPACE_STATE_VERSION_V5, PersistedWorkspaceStateV1, PersistedWorkspaceStateV2,
    PersistedWorkspaceStateV3, PersistedWorkspaceStateV4, PersistedWorkspaceStateV5,
    WorkspaceProfileId, WorkspaceState, compact_empty_tab_stack_areas,
    default_workspace_profile_registry,
};

#[derive(Debug, Deserialize)]
struct PersistedWorkspaceVersionProbe {
    version: u32,
}

#[derive(Debug)]
pub struct WorkspaceLayoutReadResult {
    pub workspace_state: WorkspaceState,
    pub workspace_profile_id: Option<WorkspaceProfileId>,
    pub layout_template: Option<String>,
    pub layout_template_version: Option<u32>,
    pub last_saved_at_unix_seconds: Option<u64>,
}

const DEFAULT_WORKSPACE_LAYOUT_DIR: &str = "editor-scenes/workspaces";

pub fn default_workspace_layout_dir() -> PathBuf {
    PathBuf::from(DEFAULT_WORKSPACE_LAYOUT_DIR)
}

pub fn workspace_layout_path_for_profile(root: &Path, profile_id: WorkspaceProfileId) -> PathBuf {
    root.join(format!("profile-{}.workspace.ron", profile_id.raw()))
}

pub fn default_workspace_layout_path_for_profile(profile_id: WorkspaceProfileId) -> PathBuf {
    workspace_layout_path_for_profile(&default_workspace_layout_dir(), profile_id)
}

pub fn legacy_workspace_layout_path_for_scene(scene_path: &Path) -> PathBuf {
    let file_name = scene_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{name}.workspace.ron"))
        .unwrap_or_else(|| "scene.workspace.ron".to_string());
    scene_path.with_file_name(file_name)
}

pub fn write_workspace_layout(path: &Path, workspace_state: &WorkspaceState) -> Result<()> {
    write_workspace_layout_with_profile(path, workspace_state, None)
}

pub fn write_workspace_layout_for_profile(
    path: &Path,
    workspace_state: &WorkspaceState,
    profile_id: WorkspaceProfileId,
) -> Result<()> {
    write_workspace_layout_with_profile(path, workspace_state, Some(profile_id))
}

fn write_workspace_layout_with_profile(
    path: &Path,
    workspace_state: &WorkspaceState,
    profile_id: Option<WorkspaceProfileId>,
) -> Result<()> {
    let workspace_state = compact_empty_tab_stack_areas(workspace_state)
        .map_err(|error| anyhow::Error::msg(error.to_string()))
        .context("failed to normalize workspace layout before saving")?;
    let mut persisted = workspace_state
        .to_persisted_v5()
        .map_err(|error| anyhow::Error::msg(error.to_string()))
        .context("failed to form v5 persisted workspace layout")?;
    persisted.workspace_profile_id = profile_id.map(|id| id.raw());
    if let Some(profile_id) = profile_id
        && let Some(profile) = default_workspace_profile_registry().profile(profile_id)
    {
        persisted.layout_template = Some(profile.default_layout_template.contract_id().to_string());
        persisted.layout_template_version =
            Some(profile.default_layout_template.contract_version());
    }
    persisted.last_saved_at_unix_seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs());
    let ron =
        encode_ron_pretty(&persisted).context("failed to encode persisted workspace layout")?;
    std::fs::write(path, ron)
        .with_context(|| format!("failed to write workspace layout: {}", path.display()))
}

pub fn read_workspace_layout(path: &Path) -> Result<WorkspaceState> {
    Ok(read_workspace_layout_with_metadata(path)?.workspace_state)
}

pub fn read_workspace_layout_with_metadata(path: &Path) -> Result<WorkspaceLayoutReadResult> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read workspace layout: {}", path.display()))?;
    let probe: PersistedWorkspaceVersionProbe =
        decode_ron(&source).context("failed to decode persisted workspace layout version")?;
    let (
        workspace_state,
        workspace_profile_id,
        layout_template,
        layout_template_version,
        last_saved_at_unix_seconds,
    ) = match probe.version {
        PERSISTED_WORKSPACE_STATE_VERSION_V1 => {
            let persisted: PersistedWorkspaceStateV1 =
                decode_ron(&source).context("failed to decode v1 persisted workspace layout")?;
            (
                WorkspaceState::from_persisted_v1(persisted),
                None,
                None,
                None,
                None,
            )
        }
        PERSISTED_WORKSPACE_STATE_VERSION_V2 => {
            let persisted: PersistedWorkspaceStateV2 =
                decode_ron(&source).context("failed to decode v2 persisted workspace layout")?;
            (
                WorkspaceState::from_persisted_v2(persisted),
                None,
                None,
                None,
                None,
            )
        }
        PERSISTED_WORKSPACE_STATE_VERSION_V3 => {
            let persisted: PersistedWorkspaceStateV3 =
                decode_ron(&source).context("failed to decode v3 persisted workspace layout")?;
            let workspace_profile_id = persisted
                .workspace_profile_id
                .and_then(|raw| WorkspaceProfileId::try_from_raw(raw).ok());
            let layout_template = persisted.layout_template.clone();
            let layout_template_version = persisted.layout_template_version;
            let last_saved_at_unix_seconds = persisted.last_saved_at_unix_seconds;
            (
                WorkspaceState::from_persisted_v3(persisted),
                workspace_profile_id,
                layout_template,
                layout_template_version,
                last_saved_at_unix_seconds,
            )
        }
        PERSISTED_WORKSPACE_STATE_VERSION_V4 => {
            let persisted: PersistedWorkspaceStateV4 =
                decode_ron(&source).context("failed to decode v4 persisted workspace layout")?;
            let workspace_profile_id = persisted
                .workspace_profile_id
                .and_then(|raw| WorkspaceProfileId::try_from_raw(raw).ok());
            let layout_template = persisted.layout_template.clone();
            let layout_template_version = persisted.layout_template_version;
            let last_saved_at_unix_seconds = persisted.last_saved_at_unix_seconds;
            (
                WorkspaceState::from_persisted_v4(persisted),
                workspace_profile_id,
                layout_template,
                layout_template_version,
                last_saved_at_unix_seconds,
            )
        }
        PERSISTED_WORKSPACE_STATE_VERSION_V5 => {
            let persisted: PersistedWorkspaceStateV5 =
                decode_ron(&source).context("failed to decode v5 persisted workspace layout")?;
            let workspace_profile_id = persisted
                .workspace_profile_id
                .and_then(|raw| WorkspaceProfileId::try_from_raw(raw).ok());
            let layout_template = persisted.layout_template.clone();
            let layout_template_version = persisted.layout_template_version;
            let last_saved_at_unix_seconds = persisted.last_saved_at_unix_seconds;
            (
                WorkspaceState::from_persisted_v5(persisted, None),
                workspace_profile_id,
                layout_template,
                layout_template_version,
                last_saved_at_unix_seconds,
            )
        }
        version => (
            Err(editor_shell::WorkspaceStateError::PersistedVersionUnsupported(version)),
            None,
            None,
            None,
            None,
        ),
    };
    let workspace_state = workspace_state
        .map_err(|error| anyhow::Error::msg(error.to_string()))
        .context("failed to validate persisted workspace layout")?;
    let workspace_state = compact_empty_tab_stack_areas(&workspace_state)
        .map_err(|error| anyhow::Error::msg(error.to_string()))
        .context("failed to normalize persisted workspace layout")?;
    Ok(WorkspaceLayoutReadResult {
        workspace_state,
        workspace_profile_id,
        layout_template,
        layout_template_version,
        last_saved_at_unix_seconds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        editor_app::RunenwerkEditorApp,
        shell::{
            RunenwerkEditorShellState, SurfaceProviderBuildContext, SurfaceSessionState,
            mounted_surface_requests_with_registry,
        },
        shell::tool_suites::{
            MATERIAL_GRAPH_CANVAS_SURFACE_KEY, MATERIAL_INSPECTOR_SURFACE_KEY,
            MATERIAL_PREVIEW_SURFACE_KEY,
        },
    };
    use editor_shell::{SurfaceDocumentContext, SurfaceProviderAvailability};
    use editor_shell::{
        MATERIAL_WORKSPACE_PROFILE_ID, PanelKind, PersistedPanelHostKindV1,
        PersistedPanelHostNodeV1, PersistedPanelInstanceStateV2, PersistedPanelKindV2,
        PersistedTabStackStateV1, PersistedToolSurfaceKindV2, PersistedToolSurfaceMountV1,
        PersistedToolSurfaceStateV2, PersistedWorkspaceStateV2, ToolSurfaceStableKey,
        WorkspaceDefaultToolSurface, WorkspaceIdentityAllocator,
    };
    use std::sync::atomic::{AtomicU64, Ordering};
    use ui_theme::ThemeTokens;

    static TEMP_WORKSPACE_LAYOUT_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_workspace_layout_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let sequence = TEMP_WORKSPACE_LAYOUT_COUNTER.fetch_add(1, Ordering::Relaxed);
        path.push(format!(
            "runenwerk_workspace_layout_{}_{}_{}.ron",
            std::process::id(),
            nanos,
            sequence
        ));
        path
    }

    fn material_lab_workspace() -> WorkspaceState {
        WorkspaceState::from_persisted_v2(PersistedWorkspaceStateV2 {
            version: PERSISTED_WORKSPACE_STATE_VERSION_V2,
            workspace_id: 1,
            root_host_id: 1,
            hosts: vec![PersistedPanelHostNodeV1 {
                id: 1,
                kind: PersistedPanelHostKindV1::TabStackHost { tab_stack_id: 1 },
            }],
            tab_stacks: vec![PersistedTabStackStateV1 {
                id: 1,
                ordered_panels: vec![1, 2, 3],
                active_panel: Some(1),
                locked_tool_surface_kind: Some(PersistedToolSurfaceKindV2::MaterialGraphCanvas),
            }],
            panels: vec![
                PersistedPanelInstanceStateV2 {
                    id: 1,
                    panel_kind: PersistedPanelKindV2::MaterialGraphCanvas,
                    active_tool_surface: Some(1),
                },
                PersistedPanelInstanceStateV2 {
                    id: 2,
                    panel_kind: PersistedPanelKindV2::MaterialInspector,
                    active_tool_surface: Some(2),
                },
                PersistedPanelInstanceStateV2 {
                    id: 3,
                    panel_kind: PersistedPanelKindV2::MaterialPreview,
                    active_tool_surface: Some(3),
                },
            ],
            tool_surfaces: vec![
                PersistedToolSurfaceStateV2 {
                    id: 1,
                    tool_surface_kind: PersistedToolSurfaceKindV2::MaterialGraphCanvas,
                    mount: PersistedToolSurfaceMountV1::Mounted { panel_id: 1 },
                },
                PersistedToolSurfaceStateV2 {
                    id: 2,
                    tool_surface_kind: PersistedToolSurfaceKindV2::MaterialInspector,
                    mount: PersistedToolSurfaceMountV1::Mounted { panel_id: 2 },
                },
                PersistedToolSurfaceStateV2 {
                    id: 3,
                    tool_surface_kind: PersistedToolSurfaceKindV2::MaterialPreview,
                    mount: PersistedToolSurfaceMountV1::Mounted { panel_id: 3 },
                },
            ],
        })
        .expect("material lab persisted fixture should load")
    }

    fn placeholder_workspace() -> WorkspaceState {
        WorkspaceState::from_persisted_v2(PersistedWorkspaceStateV2 {
            version: PERSISTED_WORKSPACE_STATE_VERSION_V2,
            workspace_id: 1,
            root_host_id: 1,
            hosts: vec![PersistedPanelHostNodeV1 {
                id: 1,
                kind: PersistedPanelHostKindV1::TabStackHost { tab_stack_id: 1 },
            }],
            tab_stacks: vec![PersistedTabStackStateV1 {
                id: 1,
                ordered_panels: vec![1],
                active_panel: Some(1),
                locked_tool_surface_kind: None,
            }],
            panels: vec![PersistedPanelInstanceStateV2 {
                id: 1,
                panel_kind: PersistedPanelKindV2::Placeholder,
                active_tool_surface: Some(1),
            }],
            tool_surfaces: vec![PersistedToolSurfaceStateV2 {
                id: 1,
                tool_surface_kind: PersistedToolSurfaceKindV2::Placeholder,
                mount: PersistedToolSurfaceMountV1::Mounted { panel_id: 1 },
            }],
        })
        .expect("unmapped placeholder persisted fixture should load")
    }

    fn stable_key_native_material_lab_default_surfaces() -> Vec<WorkspaceDefaultToolSurface> {
        vec![
            WorkspaceDefaultToolSurface::new_with_panel_kind(
                ToolSurfaceStableKey::new(MATERIAL_GRAPH_CANVAS_SURFACE_KEY).unwrap(),
                PanelKind::MaterialGraphCanvas,
                None,
            ),
            WorkspaceDefaultToolSurface::new_with_panel_kind(
                ToolSurfaceStableKey::new(MATERIAL_INSPECTOR_SURFACE_KEY).unwrap(),
                PanelKind::MaterialInspector,
                None,
            ),
            WorkspaceDefaultToolSurface::new_with_panel_kind(
                ToolSurfaceStableKey::new(MATERIAL_PREVIEW_SURFACE_KEY).unwrap(),
                PanelKind::MaterialPreview,
                None,
            ),
        ]
    }

    fn stable_key_native_material_lab_workspace() -> WorkspaceState {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        WorkspaceState::bootstrap_tool_workspace_layout_with_stable_surfaces(
            workspace_id,
            &mut allocator,
            &stable_key_native_material_lab_default_surfaces(),
        )
        .expect("stable-key-native Material Lab workspace should build")
    }

    fn material_surface_keys(workspace: &WorkspaceState) -> Vec<String> {
        workspace
            .tool_surfaces()
            .filter(|surface| surface.stable_surface_key().as_str().starts_with(
                "runenwerk.material_lab.",
            ))
            .map(|surface| surface.stable_surface_key().as_str().to_string())
            .collect()
    }

    fn tab_stack_shape(workspace: &WorkspaceState) -> Vec<(usize, bool)> {
        workspace
            .tab_stacks()
            .map(|stack| (stack.ordered_panels.len(), stack.active_panel.is_some()))
            .collect()
    }

    fn context<'a>(
        app: &'a RunenwerkEditorApp,
        shell_state: &'a RunenwerkEditorShellState,
        theme: &'a ThemeTokens,
    ) -> SurfaceProviderBuildContext<'a> {
        SurfaceProviderBuildContext {
            app,
            shell_state,
            theme,
            frame_metrics: None,
            viewport_observations: None,
            tool_surface_bindings: None,
            viewport_instances: None,
        }
    }

    fn write_persisted_layout<T: serde::Serialize>(path: &Path, persisted: &T) {
        let ron = encode_ron_pretty(persisted).expect("persisted layout should encode");
        std::fs::write(path, ron).expect("persisted layout should write");
    }

    #[test]
    fn app_workspace_layout_save_writes_v5() {
        let workspace = material_lab_workspace();

        let path = temp_workspace_layout_path();
        write_workspace_layout(&path, &workspace)
            .expect("workspace layout should write successfully");
        let source = std::fs::read_to_string(&path).expect("workspace layout should exist");
        let probe: PersistedWorkspaceVersionProbe =
            decode_ron(&source).expect("workspace layout version should decode");

        assert_eq!(probe.version, PERSISTED_WORKSPACE_STATE_VERSION_V5);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn app_workspace_layout_v5_save_uses_stable_surface_key_primary_identity() {
        let workspace = material_lab_workspace();

        let path = temp_workspace_layout_path();
        write_workspace_layout(&path, &workspace)
            .expect("workspace layout should write successfully");
        let source = std::fs::read_to_string(&path).expect("workspace layout should exist");

        assert!(source.contains("stable_surface_key"));
        assert!(source.contains("legacy_tool_surface_kind"));
        assert!(
            !source
                .lines()
                .map(str::trim_start)
                .any(|line| line.starts_with("tool_surface_kind:")),
            "V5 app save must not serialize tool_surface_kind as primary identity"
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn app_workspace_layout_v5_round_trip_preserves_layout() {
        let workspace = material_lab_workspace();

        let path = temp_workspace_layout_path();
        write_workspace_layout(&path, &workspace)
            .expect("workspace layout should write successfully");
        let loaded = read_workspace_layout(&path).expect("workspace layout should decode");

        assert_eq!(workspace, loaded);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn material_lab_v5_round_trip_preserves_graph_canvas_surface_key() {
        let workspace = stable_key_native_material_lab_workspace();
        let path = temp_workspace_layout_path();

        write_workspace_layout(&path, &workspace)
            .expect("stable-key-native Material Lab workspace should write");
        let loaded = read_workspace_layout(&path)
            .expect("stable-key-native Material Lab workspace should load");

        assert!(
            material_surface_keys(&loaded)
                .iter()
                .any(|key| key == MATERIAL_GRAPH_CANVAS_SURFACE_KEY)
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn material_lab_v5_round_trip_preserves_inspector_surface_key() {
        let workspace = stable_key_native_material_lab_workspace();
        let path = temp_workspace_layout_path();

        write_workspace_layout(&path, &workspace)
            .expect("stable-key-native Material Lab workspace should write");
        let loaded = read_workspace_layout(&path)
            .expect("stable-key-native Material Lab workspace should load");

        assert!(
            material_surface_keys(&loaded)
                .iter()
                .any(|key| key == MATERIAL_INSPECTOR_SURFACE_KEY)
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn material_lab_v5_round_trip_preserves_preview_surface_key() {
        let workspace = stable_key_native_material_lab_workspace();
        let path = temp_workspace_layout_path();

        write_workspace_layout(&path, &workspace)
            .expect("stable-key-native Material Lab workspace should write");
        let loaded = read_workspace_layout(&path)
            .expect("stable-key-native Material Lab workspace should load");

        assert!(
            material_surface_keys(&loaded)
                .iter()
                .any(|key| key == MATERIAL_PREVIEW_SURFACE_KEY)
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn material_lab_v5_round_trip_writes_no_legacy_tool_surface_kind_for_material_surfaces() {
        let workspace = stable_key_native_material_lab_workspace();
        let persisted = workspace
            .to_persisted_v5()
            .expect("stable-key-native Material Lab workspace should form V5");

        for stable_key in [
            MATERIAL_GRAPH_CANVAS_SURFACE_KEY,
            MATERIAL_INSPECTOR_SURFACE_KEY,
            MATERIAL_PREVIEW_SURFACE_KEY,
        ] {
            let persisted_surface = persisted
                .tool_surfaces
                .iter()
                .find(|surface| surface.stable_surface_key == stable_key)
                .expect("Material Lab surface should be persisted by stable key");

            assert_eq!(persisted_surface.legacy_tool_surface_kind, None);
        }
    }

    #[test]
    fn material_lab_v5_round_trip_preserves_tab_stack_layout() {
        let workspace = stable_key_native_material_lab_workspace();
        let expected_shape = tab_stack_shape(&workspace);
        let path = temp_workspace_layout_path();

        write_workspace_layout(&path, &workspace)
            .expect("stable-key-native Material Lab workspace should write");
        let loaded = read_workspace_layout(&path)
            .expect("stable-key-native Material Lab workspace should load");

        assert_eq!(tab_stack_shape(&loaded), expected_shape);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn material_lab_loaded_workspace_resolves_material_providers() {
        let app = RunenwerkEditorApp::new();
        let host = app.workbench_host();
        let workspace = stable_key_native_material_lab_workspace();
        let path = temp_workspace_layout_path();

        write_workspace_layout(&path, &workspace)
            .expect("stable-key-native Material Lab workspace should write");
        let loaded = read_workspace_layout(&path)
            .expect("stable-key-native Material Lab workspace should load");

        let mut shell_state = RunenwerkEditorShellState::new_with_tool_surface_registry(
            host.tool_surface_registry(),
        )
        .expect("shell state should build with hosted registry");
        shell_state.set_active_workspace_profile_id(MATERIAL_WORKSPACE_PROFILE_ID);
        shell_state.replace_workspace_state(loaded);

        let requests = mounted_surface_requests_with_registry(
            &shell_state,
            SurfaceDocumentContext::Resolved {
                document_id: editor_core::DocumentId(6),
                document_kind: editor_core::DocumentKind::MaterialGraph,
            },
            Some(host.tool_surface_registry()),
        );
        let theme = ThemeTokens::default();
        let cases = [
            (MATERIAL_GRAPH_CANVAS_SURFACE_KEY, 12),
            (MATERIAL_INSPECTOR_SURFACE_KEY, 13),
            (MATERIAL_PREVIEW_SURFACE_KEY, 14),
        ];

        for (stable_key, expected_provider_id) in cases {
            let request = requests
                .iter()
                .find(|request| request.stable_key().as_str() == stable_key)
                .expect("Material Lab mounted request should be present");
            let mut stable_only_request = request.clone();
            stable_only_request.legacy_tool_surface_kind = None;

            let frame = host.provider_registry().resolve_frame_with_provider_family_map(
                &context(&app, &shell_state, &theme),
                &stable_only_request,
                &SurfaceSessionState::default(),
                Some(host.provider_family_provider_map()),
            );

            assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
            assert_eq!(
                frame.provider_id.map(|id| id.raw()),
                Some(expected_provider_id)
            );
            assert_eq!(frame.stable_surface_key.as_str(), stable_key);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn workspace_layout_round_trip_preserves_default_profile_shape_after_c3() {
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new_with_tool_surface_registry(
            app.workbench_host().tool_surface_registry(),
        )
        .expect("default profile workspace should build with hosted registry");
        let workspace = shell_state.workspace_state();
        let path = temp_workspace_layout_path();

        write_workspace_layout(&path, workspace).expect("workspace layout should write");
        let loaded = read_workspace_layout(&path).expect("workspace layout should decode");

        assert_eq!(&loaded, workspace);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn workspace_layout_round_trip_still_builds_stable_key_provider_requests() {
        let app = RunenwerkEditorApp::new();
        let mut shell_state = RunenwerkEditorShellState::new_with_tool_surface_registry(
            app.workbench_host().tool_surface_registry(),
        )
        .expect("default profile workspace should build with hosted registry");
        let path = temp_workspace_layout_path();

        write_workspace_layout(&path, shell_state.workspace_state())
            .expect("workspace layout should write");
        let loaded = read_workspace_layout(&path).expect("workspace layout should decode");
        shell_state.replace_workspace_state(loaded);
        let requests = mounted_surface_requests_with_registry(
            &shell_state,
            SurfaceDocumentContext::NoActiveDocument,
            Some(app.workbench_host().tool_surface_registry()),
        );

        assert!(!requests.is_empty());
        assert!(requests.iter().all(|request| {
            app.workbench_host()
                .tool_surface_registry()
                .get(request.stable_key())
                .is_some()
        }));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn app_workspace_layout_v4_still_loads() {
        let workspace = material_lab_workspace();
        let persisted = workspace.to_persisted_v4();

        let path = temp_workspace_layout_path();
        write_persisted_layout(&path, &persisted);
        let loaded = read_workspace_layout(&path).expect("v4 workspace layout should decode");

        assert_eq!(loaded, workspace);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn app_workspace_layout_v5_invalid_stable_key_fails_explicitly() {
        let workspace = material_lab_workspace();
        let mut persisted = workspace
            .to_persisted_v5()
            .expect("material lab workspace should form v5");
        persisted.tool_surfaces[0].stable_surface_key = "Runenwerk.material_lab.graph".to_string();

        let path = temp_workspace_layout_path();
        write_persisted_layout(&path, &persisted);
        let error = read_workspace_layout(&path)
            .expect_err("invalid v5 stable key should fail explicit validation");

        assert!(
            error
                .to_string()
                .contains("failed to validate persisted workspace layout")
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn app_workspace_layout_v5_key_legacy_mismatch_fails_explicitly() {
        let workspace = material_lab_workspace();
        let mut persisted = workspace
            .to_persisted_v5()
            .expect("material lab workspace should form v5");
        persisted.tool_surfaces[0].legacy_tool_surface_kind =
            Some(PersistedToolSurfaceKindV2::MaterialPreview);

        let path = temp_workspace_layout_path();
        write_persisted_layout(&path, &persisted);
        let error = read_workspace_layout(&path)
            .expect_err("v5 stable key and legacy metadata mismatch should fail");

        assert!(
            error
                .to_string()
                .contains("failed to validate persisted workspace layout")
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn app_workspace_layout_v5_persists_placeholder_fallback_surface_explicitly() {
        let workspace = placeholder_workspace();
        let path = temp_workspace_layout_path();

        write_workspace_layout(&path, &workspace)
            .expect("placeholder fallback surface should persist");
        let source = std::fs::read_to_string(&path).expect("workspace layout should exist");

        assert!(source.contains("runenwerk.diagnostics.placeholder"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn profile_layout_path_is_not_derived_from_scene_path() {
        let path = default_workspace_layout_path_for_profile(MATERIAL_WORKSPACE_PROFILE_ID);

        assert_eq!(
            path,
            PathBuf::from("editor-scenes/workspaces/profile-5.workspace.ron")
        );
    }

    #[test]
    fn legacy_scene_layout_path_remains_available_for_load_migration() {
        let path =
            legacy_workspace_layout_path_for_scene(Path::new("editor-scenes/default.scene.ron"));

        assert_eq!(
            path,
            PathBuf::from("editor-scenes/default.scene.ron.workspace.ron")
        );
    }

    #[test]
    fn profile_workspace_layout_roundtrip_preserves_profile_metadata() {
        let workspace = material_lab_workspace();
        let path = temp_workspace_layout_path();

        write_workspace_layout_for_profile(&path, &workspace, MATERIAL_WORKSPACE_PROFILE_ID)
            .expect("workspace layout should write with profile metadata");
        let loaded = read_workspace_layout_with_metadata(&path)
            .expect("workspace layout should decode with profile metadata");

        assert_eq!(loaded.workspace_state, workspace);
        assert_eq!(
            loaded.workspace_profile_id,
            Some(MATERIAL_WORKSPACE_PROFILE_ID)
        );
        assert_eq!(loaded.layout_template.as_deref(), Some("tool-workspace"));
        assert_eq!(loaded.layout_template_version, Some(1));
        assert!(loaded.last_saved_at_unix_seconds.is_some());

        let _ = std::fs::remove_file(path);
    }
}
