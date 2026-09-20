// #720: texture product viewers must not borrow active-document authority.
use editor_core::DocumentKind;
use editor_shell::{
    MATERIAL_WORKSPACE_PROFILE_ID, ProviderFamilyId, SCENE_WORKSPACE_PROFILE_ID, ShellCommand,
    SurfaceDocumentContext, SurfaceProviderAvailability, TEXTURE_WORKSPACE_PROFILE_ID,
};
use runenwerk_editor::editor_app::RunenwerkEditorApp;
use runenwerk_editor::shell::{
    EditorSurfaceProviderRegistry, RunenwerkEditorShellState, SurfaceProviderBuildContext,
    SurfaceSessionState, active_document_context, build_editor_shell_frame_model,
    dispatch_shell_command, mounted_surface_requests_with_registry,
};
use ui_theme::ThemeTokens;

const TEXTURE_VIEWER_KEYS: [&str; 2] =
    ["runenwerk.texture.viewer_2d", "runenwerk.texture.viewer_3d"];
const SCENE_AUTHORING_KEYS: [&str; 2] = ["runenwerk.scene.viewport", "runenwerk.scene.outliner"];

fn default_shell(app: &RunenwerkEditorApp) -> RunenwerkEditorShellState {
    let host = app.workbench_host();
    RunenwerkEditorShellState::new_with_workspace_profile_registry_and_tool_surface_registry(
        host.workspace_profile_registry(),
        host.tool_surface_registry(),
    )
    .expect("full editor shell should construct")
}

fn texture_shell(app: &RunenwerkEditorApp) -> RunenwerkEditorShellState {
    let host = app.workbench_host();
    let mut shell = default_shell(app);
    let texture_profile = host
        .workspace_profile(TEXTURE_WORKSPACE_PROFILE_ID)
        .expect("full editor should install Textures workspace");
    shell
        .activate_workspace_profile_ref_with_registry(
            &texture_profile.profile_ref,
            host.workspace_profile_registry(),
            host.tool_surface_registry(),
        )
        .expect("Textures workspace should activate");
    assert_eq!(
        shell.active_workspace_profile_id(),
        TEXTURE_WORKSPACE_PROFILE_ID
    );
    shell
}

#[test]
fn scene_to_textures_command_admits_both_asset_viewers_without_switching_document() {
    let mut app = RunenwerkEditorApp::new();
    assert!(matches!(
        active_document_context(&app),
        SurfaceDocumentContext::Resolved {
            document_kind: DocumentKind::Scene,
            ..
        }
    ));
    let mut shell = default_shell(&app);
    assert_eq!(
        shell.active_workspace_profile_id(),
        SCENE_WORKSPACE_PROFILE_ID
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell),
        ShellCommand::SwitchWorkspaceProfile {
            profile_id: TEXTURE_WORKSPACE_PROFILE_ID,
        },
        None,
        None,
        None,
        None,
    )
    .expect("live workspace switch command should activate Textures");
    assert_eq!(
        shell.active_workspace_profile_id(),
        TEXTURE_WORKSPACE_PROFILE_ID
    );

    let frame = build_editor_shell_frame_model(
        &app,
        &shell,
        &EditorSurfaceProviderRegistry::runenwerk_default(),
        &ThemeTokens::default(),
        None,
        None,
        None,
    );
    for key in TEXTURE_VIEWER_KEYS {
        let surface = frame
            .surfaces
            .values()
            .find(|surface| surface.stable_surface_key.as_str() == key)
            .unwrap_or_else(|| panic!("Textures workspace should mount {key}"));
        assert_eq!(
            surface.availability,
            SurfaceProviderAvailability::Available,
            "{key} must not depend on the unrelated active Scene document"
        );
    }
    assert!(matches!(
        active_document_context(&app),
        SurfaceDocumentContext::Resolved {
            document_kind: DocumentKind::Scene,
            ..
        }
    ));
}

#[test]
fn texture_viewers_allow_no_active_document_but_preserve_provider_family_guard() {
    let app = RunenwerkEditorApp::new();
    assert_eq!(app.asset_catalog_runtime().selected_asset_id(), None);
    let shell = texture_shell(&app);
    let host = app.workbench_host();
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let theme = ThemeTokens::default();
    let context = SurfaceProviderBuildContext {
        app: &app,
        shell_state: &shell,
        theme: &theme,
        frame_metrics: None,
        viewport_observations: None,
        tool_surface_bindings: None,
        viewport_instances: None,
    };
    let requests = mounted_surface_requests_with_registry(
        &shell,
        SurfaceDocumentContext::NoActiveDocument,
        Some(host.tool_surface_registry()),
    );
    for key in TEXTURE_VIEWER_KEYS {
        let request = requests
            .iter()
            .find(|request| request.matches_stable_key(key))
            .unwrap_or_else(|| panic!("missing mounted viewer {key}"));
        let frame = registry.resolve_frame_with_provider_family_map(
            &context,
            request,
            &SurfaceSessionState::default(),
            Some(host.provider_family_provider_map()),
        );
        assert_eq!(
            frame.availability,
            SurfaceProviderAvailability::Available,
            "{key}"
        );

        let mut unassigned_family = request.clone();
        unassigned_family.provider_family_id =
            Some(ProviderFamilyId::new("runenwerk.test.unassigned").unwrap());
        let frame = registry.resolve_frame_with_provider_family_map(
            &context,
            &unassigned_family,
            &SurfaceSessionState::default(),
            Some(host.provider_family_provider_map()),
        );
        assert_eq!(
            frame.availability,
            SurfaceProviderAvailability::Unsupported,
            "{key}"
        );
    }
}

#[test]
fn scene_authoring_surfaces_remain_denied_in_textures_workspace() {
    let app = RunenwerkEditorApp::new();
    let host = app.workbench_host();
    let scene_shell = default_shell(&app);
    let texture_shell = texture_shell(&app);
    let scene_requests = mounted_surface_requests_with_registry(
        &scene_shell,
        active_document_context(&app),
        Some(host.tool_surface_registry()),
    );
    let theme = ThemeTokens::default();
    let context = SurfaceProviderBuildContext {
        app: &app,
        shell_state: &texture_shell,
        theme: &theme,
        frame_metrics: None,
        viewport_observations: None,
        tool_surface_bindings: None,
        viewport_instances: None,
    };
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();

    for key in SCENE_AUTHORING_KEYS {
        let mut request = scene_requests
            .iter()
            .find(|request| request.matches_stable_key(key))
            .unwrap_or_else(|| panic!("scene workspace should mount {key}"))
            .clone();
        request.workspace_profile_id = TEXTURE_WORKSPACE_PROFILE_ID;
        let frame = registry.resolve_frame_with_provider_family_map(
            &context,
            &request,
            &SurfaceSessionState::default(),
            Some(host.provider_family_provider_map()),
        );
        assert_eq!(
            frame.availability,
            SurfaceProviderAvailability::Unsupported,
            "{key}"
        );
    }
}

#[test]
fn material_workspace_scene_document_behavior_remains_available() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell = default_shell(&app);
    dispatch_shell_command(
        &mut app,
        Some(&mut shell),
        ShellCommand::SwitchWorkspaceProfile {
            profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
        },
        None,
        None,
        None,
        None,
    )
    .expect("live workspace switch command should activate Materials");
    assert_eq!(
        shell.active_workspace_profile_id(),
        MATERIAL_WORKSPACE_PROFILE_ID
    );
    assert!(matches!(
        active_document_context(&app),
        SurfaceDocumentContext::Resolved {
            document_kind: DocumentKind::Scene,
            ..
        }
    ));

    let frame = build_editor_shell_frame_model(
        &app,
        &shell,
        &EditorSurfaceProviderRegistry::runenwerk_default(),
        &ThemeTokens::default(),
        None,
        None,
        None,
    );
    for key in [
        "runenwerk.texture.viewer_2d",
        "runenwerk.material_lab.preview",
    ] {
        let surface = frame
            .surfaces
            .values()
            .find(|surface| surface.stable_surface_key.as_str() == key)
            .unwrap_or_else(|| panic!("Materials workspace should mount {key}"));
        assert_eq!(
            surface.availability,
            SurfaceProviderAvailability::Available,
            "{key}"
        );
    }
}
