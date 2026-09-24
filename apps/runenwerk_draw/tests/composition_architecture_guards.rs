use std::fs;
use std::path::{Path, PathBuf};

use drawing::{CanvasCoordinate, CanvasRect};
use runenwerk_draw::app::RunenwerkDrawApp;
use runenwerk_draw::app::composition::{
    DRAWING_CANVAS_UNIT_ID, DrawingCompositionContentState, DrawingCompositionDiagnosticCode,
    DrawingCompositionExtensionV1, DrawingCompositionProjection, DrawingCompositionRuntime,
    DrawingContentRole, DrawingMountedUnitExtensionV1, DrawingTabletPanelProjection,
    DrawingUnavailableProjectionPolicy, select_drawing_content_fallback,
};
use ui_composition::{
    AppProfileId, ContentLiveness, ContentProjectionFallback, MountedUnitDefinition,
    UnavailableContentPolicy,
};
use ui_math::UiSize;

#[test]
fn composition_static_projection_is_caused_by_ratified_structure() {
    let runtime = DrawingCompositionRuntime::builtin().expect("built-in composition should form");
    let content = DrawingCompositionContentState::resolved(&runtime);
    let projection = project(&runtime, &content, UiSize::new(1280.0, 720.0));

    assert_eq!(runtime.composition().definition().targets().len(), 1);
    assert_eq!(runtime.composition().definition().roots().len(), 1);
    assert_eq!(runtime.composition().definition().regions().len(), 7);
    assert_eq!(runtime.composition().definition().mounted_units().len(), 4);
    assert_eq!(projection.mounted_content().len(), 4);
    assert!((projection.top_bar_bounds.height - 36.0).abs() < 0.001);
    assert!(projection.tool_rail_bounds.width > 0.0);
    assert!(projection.support_panel_bounds.width > 0.0);
    assert!(projection.canvas_view.screen_bounds.width > 0.0);

    let narrow = project(&runtime, &content, UiSize::new(640.0, 480.0));
    assert!(narrow.support_panel_bounds.width > 0.0);
    assert_eq!(narrow.mounted_content().len(), 4);
    assert_eq!(
        narrow
            .content_for_role(DrawingContentRole::SupportPanel)
            .expect("support panel should remain mounted")
            .fallback,
        ContentProjectionFallback::ResolvedContent
    );
}

#[test]
fn composition_extension_is_canonical_and_linked_to_the_core_bundle() {
    let runtime = DrawingCompositionRuntime::builtin().expect("built-in composition should form");
    let extension = runtime.extension();
    let canonical = extension.canonical_ron().expect("extension should encode");
    assert_eq!(
        DrawingCompositionExtensionV1::decode_canonical(&canonical)
            .expect("canonical extension should decode"),
        *extension
    );

    let mut reversed_records = extension.mounted_units().to_vec();
    reversed_records.reverse();
    let reversed = DrawingCompositionExtensionV1::new(
        extension.layout_id(),
        extension.core_schema_version(),
        extension.definition_revision(),
        extension.app_profile().clone(),
        reversed_records,
    );
    assert_eq!(
        canonical,
        reversed
            .canonical_ron()
            .expect("insertion order should normalize")
    );

    let bundle = runtime
        .linked_bundle_candidate()
        .expect("core and Draw extension should form one linked bundle");
    assert_eq!(bundle.core().extension_links.len(), 1);
    assert_eq!(bundle.extensions().len(), 1);
    assert_eq!(
        bundle.core().core_payload_digest,
        bundle.extensions()[0].link.core_payload_digest
    );
}

#[test]
fn composition_extension_rejects_incomplete_or_duplicated_role_coverage_atomically() {
    let runtime = DrawingCompositionRuntime::builtin().expect("built-in composition should form");
    let definition = runtime.composition().definition().clone();
    let empty = DrawingCompositionExtensionV1::new(
        definition.id(),
        definition.schema_version(),
        definition.revision(),
        AppProfileId::new("runenwerk.draw").unwrap(),
        Vec::new(),
    );
    let rejection = DrawingCompositionRuntime::new(definition.clone(), empty)
        .expect_err("incomplete extension must reject");
    assert!(rejection.diagnostics().iter().any(|record| {
        record.code() == DrawingCompositionDiagnosticCode::ExtensionCoverageMismatch
    }));

    let mut duplicate = runtime.extension().mounted_units().to_vec();
    duplicate[1].role = duplicate[0].role;
    let duplicate = DrawingCompositionExtensionV1::new(
        definition.id(),
        definition.schema_version(),
        definition.revision(),
        AppProfileId::new("runenwerk.draw").unwrap(),
        duplicate,
    );
    let rejection = DrawingCompositionRuntime::new(definition, duplicate)
        .expect_err("duplicate role must reject");
    assert!(rejection.diagnostics().iter().any(|record| {
        record.code() == DrawingCompositionDiagnosticCode::ExtensionRoleDuplicate
    }));
}

#[test]
fn composition_content_liveness_preserves_structure_and_uses_ordered_fallbacks() {
    let mut app = RunenwerkDrawApp::new();
    let definition_before = app.composition_runtime().composition().definition().clone();
    let state_revision_before = app.composition_runtime().composition().revision();
    let document_before = app.document().cloned();

    for liveness in [
        ContentLiveness::Resolved,
        ContentLiveness::Missing,
        ContentLiveness::Loading,
        ContentLiveness::Suspended,
        ContentLiveness::Denied,
        ContentLiveness::UnsupportedProfile,
        ContentLiveness::Crashed,
    ] {
        app.set_composition_content_liveness(DRAWING_CANVAS_UNIT_ID, liveness)
            .expect("known mounted-unit liveness should update");
        let projected = app
            .composition_projection()
            .content_for_role(DrawingContentRole::Canvas)
            .expect("canvas should remain structurally projected");
        let expected = if matches!(liveness, ContentLiveness::Resolved) {
            ContentProjectionFallback::ResolvedContent
        } else {
            ContentProjectionFallback::AppProvidedUnavailable
        };
        assert_eq!(projected.fallback, expected);
        assert_eq!(
            app.composition_runtime().composition().definition(),
            &definition_before
        );
        assert_eq!(
            app.composition_runtime().composition().revision(),
            state_revision_before
        );
        assert_eq!(app.document(), document_before.as_ref());
    }

    assert!(
        app.composition_projection()
            .diagnostics()
            .iter()
            .any(|record| {
                record.code() == DrawingCompositionDiagnosticCode::ContentUnavailable
            })
    );
}

#[test]
fn composition_hidden_fallback_requires_both_unit_and_host_permission() {
    let runtime = DrawingCompositionRuntime::builtin().expect("built-in composition should form");
    let source = runtime
        .composition()
        .snapshot()
        .mounted_unit(DRAWING_CANVAS_UNIT_ID)
        .expect("canvas unit should exist");
    let hideable = MountedUnitDefinition::new(
        source.id,
        source.content().clone(),
        source.capabilities().iter().cloned(),
        UnavailableContentPolicy::AllowHide,
    );
    let neutral_only = DrawingMountedUnitExtensionV1 {
        mounted_unit_id: source.id,
        role: DrawingContentRole::Canvas,
        unavailable_projection: DrawingUnavailableProjectionPolicy::NeutralOnly,
    };

    assert_eq!(
        select_drawing_content_fallback(
            &hideable,
            &neutral_only,
            ContentLiveness::Missing,
            false,
            true,
        )
        .unwrap(),
        ContentProjectionFallback::Hidden
    );
    assert!(
        select_drawing_content_fallback(
            &hideable,
            &neutral_only,
            ContentLiveness::Missing,
            false,
            false,
        )
        .is_err()
    );
}

#[test]
fn composition_invalid_target_rejects_without_replacing_last_good_projection() {
    let mut app = RunenwerkDrawApp::new();
    let before = app.composition_projection().clone();
    let rejection = app
        .set_window_size(UiSize::new(f32::NAN, 720.0))
        .expect_err("non-finite target bounds must reject");
    assert!(rejection.diagnostics().iter().any(|record| {
        record.code() == DrawingCompositionDiagnosticCode::ProjectionTargetInvalid
    }));
    assert_eq!(app.composition_projection(), &before);
}

#[test]
fn composition_active_source_has_no_legacy_projection_or_forbidden_owner_imports() {
    let app_root = workspace_root().join("apps/runenwerk_draw/src/app");
    assert!(!app_root.join("workspace.rs").exists());
    let mut sources = Vec::new();
    collect_rust_sources(&app_root, &mut sources);
    let joined = sources
        .iter()
        .map(|path| fs::read_to_string(path).expect("Draw source should be readable"))
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in [
        "DrawingWorkspaceProjection",
        "pub fn workspace(",
        "ui_adaptive_composition",
        "editor_shell",
    ] {
        assert!(
            !joined.contains(forbidden),
            "active Draw source retained {forbidden}"
        );
    }

    let composition_root = app_root.join("composition");
    let mut composition_sources = Vec::new();
    collect_rust_sources(&composition_root, &mut composition_sources);
    let composition_joined = composition_sources
        .iter()
        .map(|path| fs::read_to_string(path).expect("composition source should be readable"))
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in [
        "NativeWindowId",
        "RenderSurfaceId",
        "UiProgram",
        "ui_surface",
    ] {
        assert!(
            !composition_joined.contains(forbidden),
            "Draw composition leaked {forbidden}"
        );
    }

    let state = fs::read_to_string(app_root.join("state.rs")).unwrap();
    assert!(state.contains("composition_runtime: DrawingCompositionRuntime"));
    assert!(state.contains("composition_content: DrawingCompositionContentState"));
    assert!(state.contains("composition_projection: DrawingCompositionProjection"));
}

fn project(
    runtime: &DrawingCompositionRuntime,
    content: &DrawingCompositionContentState,
    size: UiSize,
) -> DrawingCompositionProjection {
    DrawingCompositionProjection::project(
        runtime,
        content,
        size,
        CanvasRect::new(
            CanvasCoordinate::new(0.0, 0.0),
            CanvasCoordinate::new(4096.0, 4096.0),
        ),
        DrawingTabletPanelProjection::default(),
    )
    .expect("built-in composition should project")
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("runenwerk_draw should live under apps/")
        .to_path_buf()
}

fn collect_rust_sources(root: &Path, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("source directory should be readable") {
        let path = entry.expect("source entry should be readable").path();
        if path.is_dir() {
            collect_rust_sources(&path, output);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            output.push(path);
        }
    }
    output.sort();
}
