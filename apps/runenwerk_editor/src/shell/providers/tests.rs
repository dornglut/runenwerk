use super::*;
use asset::{
    ArtifactCacheKey, ArtifactPayloadKind, AssetArtifactDescriptor, AssetDiagnosticCode,
    AssetDiagnosticRecord, AssetDiagnosticSeverity, AssetKind, AssetRecord, AssetSourceDescriptor,
    SourceHash, asset_artifact_id, asset_id, asset_source_id,
};
use editor_shell::{
    EditorToolSuite, LAYOUT_WORKSPACE_PROFILE_ID, PanelInstanceId, PanelKind,
    ProviderFamilyDefinition, ProviderFamilyId, RUNTIME_DEBUG_WORKSPACE_PROFILE_ID, TabStackId,
    ToolSuiteId, ToolSuiteRegistry, ToolSurfaceDefinition, ToolSurfaceInstanceId,
    ToolSurfacePersistence, ToolSurfaceRole, ToolSurfaceRoute, ToolSurfaceStableKey, UiNodeKind,
    VIEWPORT_SURFACE_DEFINITION_ID, WidgetId, tool_surface_capability_set,
};
use graph::{CyclePolicy, GraphDefinition, GraphId, NodeDefinition, NodeId};
use texture::{
    Ktx2TextureMetadata, TextureDescriptor, TextureDimension, TextureExtent, TexturePixelFormat,
    TextureProductId,
};

fn texture_descriptor(
    product_id: u64,
    dimension: TextureDimension,
    extent: TextureExtent,
) -> TextureDescriptor {
    let descriptor = TextureDescriptor::new(
        TextureProductId::new(product_id),
        format!("texture.{product_id}"),
        dimension,
        extent,
    );
    let mip_count = descriptor.mip_count;
    let descriptor_hash = descriptor.descriptor_hash().to_string();
    descriptor.with_ktx2_metadata(
        Ktx2TextureMetadata::new(
            TexturePixelFormat::Rgba8Unorm,
            mip_count,
            descriptor_hash,
            "1",
        )
        .with_byte_layout(128, [64]),
    )
}

fn texture_payload(descriptor: TextureDescriptor) -> ArtifactPayloadKind {
    ArtifactPayloadKind::TextureProduct {
        descriptor_hash: descriptor.descriptor_hash().to_string(),
        descriptor,
        artifact_uri: None,
    }
}

fn texture_payload_with_uri(
    descriptor: TextureDescriptor,
    artifact_uri: impl Into<String>,
) -> ArtifactPayloadKind {
    ArtifactPayloadKind::TextureProduct {
        descriptor_hash: descriptor.descriptor_hash().to_string(),
        descriptor,
        artifact_uri: Some(artifact_uri.into()),
    }
}

fn generated_texture_payload_with_uri(
    descriptor: TextureDescriptor,
    artifact_uri: impl Into<String>,
) -> ArtifactPayloadKind {
    ArtifactPayloadKind::GeneratedTextureProduct {
        descriptor_hash: descriptor.descriptor_hash().to_string(),
        descriptor,
        artifact_uri: Some(artifact_uri.into()),
    }
}

fn texture_payload_with_hash(
    descriptor: TextureDescriptor,
    descriptor_hash: impl Into<String>,
    artifact_uri: Option<String>,
) -> ArtifactPayloadKind {
    ArtifactPayloadKind::TextureProduct {
        descriptor_hash: descriptor_hash.into(),
        descriptor,
        artifact_uri,
    }
}

fn texture_descriptor_with_byte_length(
    product_id: u64,
    dimension: TextureDimension,
    extent: TextureExtent,
    byte_length: u64,
) -> TextureDescriptor {
    let descriptor = TextureDescriptor::new(
        TextureProductId::new(product_id),
        format!("texture.{product_id}"),
        dimension,
        extent,
    );
    let mip_count = descriptor.mip_count;
    let descriptor_hash = descriptor.descriptor_hash().to_string();
    descriptor.with_ktx2_metadata(
        Ktx2TextureMetadata::new(
            TexturePixelFormat::Rgba8Unorm,
            mip_count,
            descriptor_hash,
            "1",
        )
        .with_byte_layout(
            byte_length,
            [extent.width as u64 * extent.height as u64 * extent.depth as u64 * 4],
        ),
    )
}

fn texture_descriptor_with_mip_count_and_byte_length(
    product_id: u64,
    dimension: TextureDimension,
    extent: TextureExtent,
    mip_count: u32,
    byte_length: u64,
) -> TextureDescriptor {
    let descriptor = TextureDescriptor::new(
        TextureProductId::new(product_id),
        format!("texture.{product_id}"),
        dimension,
        extent,
    )
    .with_mip_count(mip_count);
    let descriptor_hash = descriptor.descriptor_hash().to_string();
    descriptor.with_ktx2_metadata(
        Ktx2TextureMetadata::new(
            TexturePixelFormat::Rgba8Unorm,
            mip_count,
            descriptor_hash,
            "1",
        )
        .with_byte_layout(
            byte_length,
            [extent.width as u64 * extent.height as u64 * extent.depth as u64 * 4],
        ),
    )
}

struct DummyProvider {
    descriptor: SurfaceProviderDescriptor,
    supports: bool,
    support_mode: Option<SurfaceProviderSupportMode>,
    fail: bool,
}

impl EditorSurfaceProvider for DummyProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        self.descriptor.clone()
    }

    fn supports(&self, _request: &SurfaceProviderRequest) -> bool {
        self.support_mode
            .map(SurfaceProviderSupportMode::is_supported)
            .unwrap_or(self.supports)
    }

    fn support_mode(&self, _request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        self.support_mode.unwrap_or({
            if self.supports {
                SurfaceProviderSupportMode::LegacyKind
            } else {
                SurfaceProviderSupportMode::Unsupported
            }
        })
    }

    fn build_frame(
        &self,
        _context: &SurfaceProviderBuildContext<'_>,
        _request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        if self.fail {
            return Err(SurfaceProviderDiagnostic::new(
                "test.provider.failed",
                "provider failed",
            ));
        }
        Ok(ProviderSurfaceFrame {
            title: self.descriptor.label.clone(),
            artifact: SurfacePresentationArtifact::provider(editor_shell::label(
                WidgetId(99),
                "ok",
                ThemeTokens::default().body_small_text_style(FontId(1)),
            )),
            routes: SurfaceRouteTable::empty(),
        })
    }

    fn map_action(
        &self,
        _context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        _action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        Ok(None)
    }
}

fn dummy(id: u64, priority: u16, supports: bool) -> Box<dyn EditorSurfaceProvider> {
    Box::new(DummyProvider {
        descriptor: SurfaceProviderDescriptor::new(
            SurfaceProviderId::try_from_raw(id).unwrap(),
            format!("provider-{id}"),
            SurfaceProviderPriority(priority),
        ),
        supports,
        support_mode: None,
        fail: false,
    })
}

fn dummy_with_support_mode(
    id: u64,
    priority: u16,
    support_mode: SurfaceProviderSupportMode,
) -> Box<dyn EditorSurfaceProvider> {
    Box::new(DummyProvider {
        descriptor: SurfaceProviderDescriptor::new(
            SurfaceProviderId::try_from_raw(id).unwrap(),
            format!("provider-{id}"),
            SurfaceProviderPriority(priority),
        ),
        supports: support_mode.is_supported(),
        support_mode: Some(support_mode),
        fail: false,
    })
}

fn failing(id: u64) -> Box<dyn EditorSurfaceProvider> {
    Box::new(DummyProvider {
        descriptor: SurfaceProviderDescriptor::new(
            SurfaceProviderId::try_from_raw(id).unwrap(),
            "failing",
            SurfaceProviderPriority::DEFAULT,
        ),
        supports: true,
        support_mode: None,
        fail: true,
    })
}

fn request() -> SurfaceProviderRequest {
    let tool_surface_kind = ToolSurfaceKind::Viewport;
    SurfaceProviderRequest {
        workspace_profile_id: LAYOUT_WORKSPACE_PROFILE_ID,
        document_context: SurfaceDocumentContext::Resolved {
            document_id: editor_core::DocumentId(1),
            document_kind: DocumentKind::Scene,
        },
        panel_instance_id: PanelInstanceId::try_from_raw(3).unwrap(),
        tab_stack_id: TabStackId::try_from_raw(3).unwrap(),
        tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(3).unwrap(),
        stable_surface_key: stable_key_for_test(tool_surface_kind),
        provider_family_id: None,
        surface_route: None,
        surface_definition_id: VIEWPORT_SURFACE_DEFINITION_ID,
        capabilities: tool_surface_capability_set(tool_surface_kind),
    }
}

fn request_with_document_context(
    document_context: SurfaceDocumentContext,
    tool_surface_kind: ToolSurfaceKind,
) -> SurfaceProviderRequest {
    SurfaceProviderRequest {
        document_context,
        stable_surface_key: stable_key_for_test(tool_surface_kind),
        surface_definition_id: tool_surface_definition_id(tool_surface_kind),
        capabilities: tool_surface_capability_set(tool_surface_kind),
        ..request()
    }
}

fn request_with_stable_key(
    stable_surface_key: &str,
    tool_surface_kind: ToolSurfaceKind,
) -> SurfaceProviderRequest {
    SurfaceProviderRequest {
        stable_surface_key: ToolSurfaceStableKey::new(stable_surface_key).unwrap(),
        surface_definition_id: tool_surface_definition_id(tool_surface_kind),
        capabilities: tool_surface_capability_set(tool_surface_kind),
        ..request()
    }
}

fn self_authoring_request(tool_surface_kind: ToolSurfaceKind) -> SurfaceProviderRequest {
    SurfaceProviderRequest {
        workspace_profile_id: editor_shell::EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
        document_context: SurfaceDocumentContext::NoActiveDocument,
        panel_instance_id: PanelInstanceId::try_from_raw(10).unwrap(),
        tab_stack_id: TabStackId::try_from_raw(10).unwrap(),
        tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(10).unwrap(),
        stable_surface_key: stable_key_for_test(tool_surface_kind),
        provider_family_id: None,
        surface_route: None,
        surface_definition_id: tool_surface_definition_id(tool_surface_kind),
        capabilities: tool_surface_capability_set(tool_surface_kind),
    }
}

fn m6_material_request(tool_surface_kind: ToolSurfaceKind) -> SurfaceProviderRequest {
    SurfaceProviderRequest {
        workspace_profile_id: editor_shell::MATERIAL_WORKSPACE_PROFILE_ID,
        document_context: SurfaceDocumentContext::Resolved {
            document_id: editor_core::DocumentId(6),
            document_kind: DocumentKind::MaterialGraph,
        },
        panel_instance_id: PanelInstanceId::try_from_raw(20).unwrap(),
        tab_stack_id: TabStackId::try_from_raw(20).unwrap(),
        tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(20).unwrap(),
        stable_surface_key: stable_key_for_test(tool_surface_kind),
        provider_family_id: None,
        surface_route: None,
        surface_definition_id: tool_surface_definition_id(tool_surface_kind),
        capabilities: tool_surface_capability_set(tool_surface_kind),
    }
}

fn stable_key_only_material_request(
    stable_key: &str,
    route: ToolSurfaceRoute,
) -> SurfaceProviderRequest {
    SurfaceProviderRequest {
        workspace_profile_id: editor_shell::MATERIAL_WORKSPACE_PROFILE_ID,
        document_context: SurfaceDocumentContext::Resolved {
            document_id: editor_core::DocumentId(6),
            document_kind: DocumentKind::MaterialGraph,
        },
        panel_instance_id: PanelInstanceId::try_from_raw(20).unwrap(),
        tab_stack_id: TabStackId::try_from_raw(20).unwrap(),
        tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(20).unwrap(),
        stable_surface_key: ToolSurfaceStableKey::new(stable_key).unwrap(),
        provider_family_id: Some(ProviderFamilyId::new("runenwerk.material_lab").unwrap()),
        surface_route: Some(route),
        surface_definition_id: editor_shell::PLACEHOLDER_SURFACE_DEFINITION_ID,
        capabilities: Default::default(),
    }
}

fn m6_texture_request(tool_surface_kind: ToolSurfaceKind) -> SurfaceProviderRequest {
    let document_kind = match tool_surface_kind {
        ToolSurfaceKind::VolumeTextureViewer => DocumentKind::VolumeTexture,
        _ => DocumentKind::ProceduralTexture,
    };
    SurfaceProviderRequest {
        workspace_profile_id: editor_shell::TEXTURE_WORKSPACE_PROFILE_ID,
        document_context: SurfaceDocumentContext::Resolved {
            document_id: editor_core::DocumentId(7),
            document_kind,
        },
        panel_instance_id: PanelInstanceId::try_from_raw(21).unwrap(),
        tab_stack_id: TabStackId::try_from_raw(21).unwrap(),
        tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(21).unwrap(),
        stable_surface_key: stable_key_for_test(tool_surface_kind),
        provider_family_id: None,
        surface_route: None,
        surface_definition_id: tool_surface_definition_id(tool_surface_kind),
        capabilities: tool_surface_capability_set(tool_surface_kind),
    }
}

fn m6_procgen_request(tool_surface_kind: ToolSurfaceKind) -> SurfaceProviderRequest {
    SurfaceProviderRequest {
        workspace_profile_id: editor_shell::PROCGEN_WORKSPACE_PROFILE_ID,
        document_context: SurfaceDocumentContext::Resolved {
            document_id: editor_core::DocumentId(9),
            document_kind: DocumentKind::ProceduralGenerationGraph,
        },
        panel_instance_id: PanelInstanceId::try_from_raw(23).unwrap(),
        tab_stack_id: TabStackId::try_from_raw(23).unwrap(),
        tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(23).unwrap(),
        stable_surface_key: stable_key_for_test(tool_surface_kind),
        provider_family_id: None,
        surface_route: None,
        surface_definition_id: tool_surface_definition_id(tool_surface_kind),
        capabilities: tool_surface_capability_set(tool_surface_kind),
    }
}

fn m6_sdf_request(
    tool_surface_kind: ToolSurfaceKind,
    document_kind: DocumentKind,
) -> SurfaceProviderRequest {
    SurfaceProviderRequest {
        workspace_profile_id: editor_shell::FIELD_WORLD_WORKSPACE_PROFILE_ID,
        document_context: SurfaceDocumentContext::Resolved {
            document_id: editor_core::DocumentId(8),
            document_kind,
        },
        panel_instance_id: PanelInstanceId::try_from_raw(22).unwrap(),
        tab_stack_id: TabStackId::try_from_raw(22).unwrap(),
        tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(22).unwrap(),
        stable_surface_key: stable_key_for_test(tool_surface_kind),
        provider_family_id: None,
        surface_route: None,
        surface_definition_id: tool_surface_definition_id(tool_surface_kind),
        capabilities: tool_surface_capability_set(tool_surface_kind),
    }
}

fn asset_request(tool_surface_kind: ToolSurfaceKind) -> SurfaceProviderRequest {
    SurfaceProviderRequest {
        workspace_profile_id: editor_shell::FIELD_WORLD_WORKSPACE_PROFILE_ID,
        document_context: SurfaceDocumentContext::NoActiveDocument,
        panel_instance_id: PanelInstanceId::try_from_raw(30).unwrap(),
        tab_stack_id: TabStackId::try_from_raw(30).unwrap(),
        tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(30).unwrap(),
        stable_surface_key: stable_key_for_test(tool_surface_kind),
        provider_family_id: None,
        surface_route: None,
        surface_definition_id: tool_surface_definition_id(tool_surface_kind),
        capabilities: tool_surface_capability_set(tool_surface_kind),
    }
}

fn inspector_request() -> SurfaceProviderRequest {
    SurfaceProviderRequest {
        workspace_profile_id: RUNTIME_DEBUG_WORKSPACE_PROFILE_ID,
        document_context: SurfaceDocumentContext::NoActiveDocument,
        panel_instance_id: PanelInstanceId::try_from_raw(31).unwrap(),
        tab_stack_id: TabStackId::try_from_raw(31).unwrap(),
        tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(31).unwrap(),
        stable_surface_key: ToolSurfaceStableKey::new(TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY)
            .unwrap(),
        provider_family_id: Some(ProviderFamilyId::new("runenwerk.diagnostics").unwrap()),
        surface_route: Some(ToolSurfaceRoute::ProviderOwnedLocal),
        surface_definition_id: editor_shell::PLACEHOLDER_SURFACE_DEFINITION_ID,
        capabilities: Default::default(),
    }
}

fn stable_key_for_test(tool_surface_kind: ToolSurfaceKind) -> ToolSurfaceStableKey {
    editor_shell::stable_key_for_tool_surface_kind(tool_surface_kind)
        .expect("provider test fixture surface should have a stable key")
}

fn scene_viewport_tool_suite_registry() -> ToolSuiteRegistry {
    let provider_family_id = ProviderFamilyId::new("runenwerk.scene").unwrap();
    ToolSuiteRegistry::new(vec![EditorToolSuite {
        suite_id: ToolSuiteId::new("runenwerk.scene").unwrap(),
        label: "Scene".to_string(),
        provider_families: vec![ProviderFamilyDefinition {
            id: provider_family_id.clone(),
            label: "Scene".to_string(),
        }],
        surfaces: vec![ToolSurfaceDefinition {
            key: ToolSurfaceStableKey::new("runenwerk.scene.viewport").unwrap(),
            label: "Viewport".to_string(),
            role: ToolSurfaceRole::Primary,
            panel_kind: PanelKind::Viewport,
            provider_family: provider_family_id,
            route: ToolSurfaceRoute::ProviderOwnedLocal,
            persistence: ToolSurfacePersistence::StableKey,
            capabilities: ui_surface::SurfaceCapabilitySet::new(true, false, false, false),
            session_retention: ui_surface::SessionRetentionClass::Restorable,
            creation_policy: editor_shell::ToolSurfaceCreationPolicy::SingletonPerWorkspace,
            target_profile_compatibility:
                editor_shell::ToolSurfaceTargetProfileCompatibility::AllProfiles,
            legacy_compatibility_key: None,
        }],
    }])
    .expect("scene viewport fixture should be valid")
}

fn tool_suite_registry_for_provider_family(provider_family_id: &str) -> ToolSuiteRegistry {
    let provider_family_id = ProviderFamilyId::new(provider_family_id).unwrap();
    ToolSuiteRegistry::new(vec![EditorToolSuite {
        suite_id: ToolSuiteId::new(provider_family_id.as_str()).unwrap(),
        label: "Fixture".to_string(),
        provider_families: vec![ProviderFamilyDefinition {
            id: provider_family_id.clone(),
            label: "Fixture".to_string(),
        }],
        surfaces: vec![ToolSurfaceDefinition {
            key: ToolSurfaceStableKey::new("runenwerk.scene.viewport").unwrap(),
            label: "Viewport".to_string(),
            role: ToolSurfaceRole::Primary,
            panel_kind: PanelKind::Viewport,
            provider_family: provider_family_id,
            route: ToolSurfaceRoute::ProviderOwnedLocal,
            persistence: ToolSurfacePersistence::StableKey,
            capabilities: ui_surface::SurfaceCapabilitySet::new(true, true, true, false),
            session_retention: ui_surface::SessionRetentionClass::Restorable,
            creation_policy: editor_shell::ToolSurfaceCreationPolicy::SingletonPerWorkspace,
            target_profile_compatibility:
                editor_shell::ToolSurfaceTargetProfileCompatibility::AllProfiles,
            legacy_compatibility_key: None,
        }],
    }])
    .expect("provider family fixture should be valid")
}

fn provider_family_map(
    registry: &ToolSuiteRegistry,
    provider_family_id: &str,
    provider_ids: &[u64],
) -> ProviderFamilyProviderMap {
    let provider_family_id = ProviderFamilyId::new(provider_family_id).unwrap();
    ProviderFamilyProviderMap::new(
        registry,
        provider_ids
            .iter()
            .copied()
            .map(|id| {
                ProviderFamilyProviderAssignment::new(
                    provider_family_id.clone(),
                    SurfaceProviderId::try_from_raw(id).unwrap(),
                )
            })
            .collect(),
    )
    .expect("provider family map fixture should be valid")
}

fn request_for_provider_family(provider_family_id: &str) -> SurfaceProviderRequest {
    SurfaceProviderRequest {
        provider_family_id: Some(ProviderFamilyId::new(provider_family_id).unwrap()),
        ..request()
    }
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

fn provider_frame_text(frame: &ResolvedSurfaceFrame) -> String {
    format!("{:?}", frame.artifact.root)
}

fn frame_has_product_surface(frame: &ResolvedSurfaceFrame) -> bool {
    fn walk(node: &editor_shell::UiNode) -> bool {
        matches!(node.kind, UiNodeKind::ProductSurface(_)) || node.children.iter().any(walk)
    }
    walk(&frame.artifact.root)
}

fn frame_has_editor_definition_route(frame: &ResolvedSurfaceFrame) -> bool {
    frame.routes.iter().any(|(_, route)| {
        matches!(
            route.action(),
            Some(SurfaceLocalAction::EditorDefinition(_))
        )
    })
}

fn frame_graph_canvas_model(
    frame: &ResolvedSurfaceFrame,
) -> Option<&ui_graph_editor::GraphCanvasViewModel> {
    fn walk(node: &editor_shell::UiNode) -> Option<&ui_graph_editor::GraphCanvasViewModel> {
        if let UiNodeKind::GraphCanvas(canvas) = &node.kind {
            return Some(&canvas.canvas);
        }
        node.children.iter().find_map(walk)
    }
    walk(&frame.artifact.root)
}

fn build_rgba8_ktx2(
    width: u32,
    height: u32,
    depth: u32,
    slice0_texel: [u8; 4],
    slice1_texel: [u8; 4],
) -> Vec<u8> {
    let format = ktx2::Format::R8G8B8A8_UNORM;
    let (basic, type_size) = ktx2::dfd::Basic::from_format(format).expect("rgba8 dfd should build");
    let dfd_block = ktx2::dfd::Block::Basic(basic);
    let dfd_block_bytes = dfd_block.to_vec();
    let dfd_total_size = 4 + dfd_block_bytes.len();
    let level_index_offset = ktx2::Header::LENGTH;
    let dfd_offset = level_index_offset + ktx2::LevelIndex::LENGTH;
    let after_dfd = dfd_offset + dfd_total_size;
    let level_data_offset = after_dfd.div_ceil(4) * 4;
    let texel_count = width as usize * height as usize * depth.max(1) as usize;
    let level_data_size = texel_count * 4;
    let mut bytes = vec![0u8; level_data_offset + level_data_size];

    let header = ktx2::Header {
        format: Some(format),
        type_size,
        pixel_width: width,
        pixel_height: height,
        pixel_depth: if depth > 1 { depth } else { 0 },
        layer_count: 0,
        face_count: 1,
        level_count: 1,
        supercompression_scheme: None,
        index: ktx2::Index {
            dfd_byte_offset: dfd_offset as u32,
            dfd_byte_length: dfd_total_size as u32,
            kvd_byte_offset: 0,
            kvd_byte_length: 0,
            sgd_byte_offset: 0,
            sgd_byte_length: 0,
        },
    };
    bytes[..ktx2::Header::LENGTH].copy_from_slice(&header.as_bytes());
    let index = ktx2::LevelIndex {
        byte_offset: level_data_offset as u64,
        byte_length: level_data_size as u64,
        uncompressed_byte_length: level_data_size as u64,
    };
    bytes[level_index_offset..level_index_offset + ktx2::LevelIndex::LENGTH]
        .copy_from_slice(&index.as_bytes());
    bytes[dfd_offset..dfd_offset + 4].copy_from_slice(&(dfd_total_size as u32).to_le_bytes());
    bytes[dfd_offset + 4..dfd_offset + 4 + dfd_block_bytes.len()].copy_from_slice(&dfd_block_bytes);
    let data = &mut bytes[level_data_offset..level_data_offset + level_data_size];
    let texels_per_slice = width as usize * height as usize;
    for (index, pixel) in data.chunks_exact_mut(4).enumerate() {
        let texel = if index / texels_per_slice == 0 {
            slice0_texel
        } else {
            slice1_texel
        };
        pixel.copy_from_slice(&texel);
    }
    bytes
}

#[test]
fn duplicate_provider_id_is_rejected() {
    let error =
        match EditorSurfaceProviderRegistry::new(vec![dummy(1, 100, true), dummy(1, 90, true)]) {
            Ok(_) => panic!("duplicate ids should be rejected"),
            Err(error) => error,
        };

    assert!(matches!(
        error,
        SurfaceProviderRegistryError::DuplicateProviderId(id) if id == SurfaceProviderId::try_from_raw(1).unwrap()
    ));
}

#[test]
fn mounted_surface_request_includes_advisory_stable_key_when_available() {
    let shell_state = RunenwerkEditorShellState::new();
    let requests = mounted_surface_requests(&shell_state, SurfaceDocumentContext::NoActiveDocument);

    let viewport_request = requests
        .iter()
        .find(|request| request.matches_stable_key(SCENE_VIEWPORT_SURFACE_KEY))
        .expect("default workspace should mount viewport");

    assert_eq!(
        viewport_request.stable_key().as_str(),
        "runenwerk.scene.viewport"
    );
    assert_eq!(viewport_request.provider_family_id, None);
    assert_eq!(viewport_request.surface_route, None);
}

#[test]
fn live_mounted_surface_requests_use_stable_key_authority() {
    let shell_state = RunenwerkEditorShellState::new();
    let requests = mounted_surface_requests(&shell_state, SurfaceDocumentContext::NoActiveDocument);

    let viewport_request = requests
        .iter()
        .find(|request| request.matches_stable_key(SCENE_VIEWPORT_SURFACE_KEY))
        .expect("default workspace should mount viewport by stable key");

    assert_eq!(
        viewport_request.stable_key().as_str(),
        SCENE_VIEWPORT_SURFACE_KEY
    );
}

#[test]
fn live_mounted_surface_requests_include_definition_metadata() {
    let shell_state = RunenwerkEditorShellState::new();
    let requests = mounted_surface_requests(&shell_state, SurfaceDocumentContext::NoActiveDocument);

    let viewport_request = requests
        .iter()
        .find(|request| request.matches_stable_key(SCENE_VIEWPORT_SURFACE_KEY))
        .expect("default workspace should mount viewport by stable key");

    assert_eq!(
        viewport_request.surface_definition_id,
        tool_surface_definition_id(ToolSurfaceKind::Viewport)
    );
}

#[test]
fn mounted_surface_request_enrichment_adds_provider_family_and_route_when_registry_resolves() {
    let shell_state = RunenwerkEditorShellState::new();
    let registry = scene_viewport_tool_suite_registry();

    let requests = mounted_surface_requests_with_registry(
        &shell_state,
        SurfaceDocumentContext::NoActiveDocument,
        Some(registry.surfaces()),
    );

    let viewport_request = requests
        .iter()
        .find(|request| request.matches_stable_key(SCENE_VIEWPORT_SURFACE_KEY))
        .expect("default workspace should mount viewport");

    assert_eq!(
        viewport_request.stable_key().as_str(),
        "runenwerk.scene.viewport"
    );
    assert_eq!(
        viewport_request
            .provider_family_id
            .as_ref()
            .map(ProviderFamilyId::as_str),
        Some("runenwerk.scene")
    );
    assert_eq!(
        viewport_request.surface_route,
        Some(ToolSurfaceRoute::ProviderOwnedLocal)
    );
    assert_eq!(
        viewport_request.capabilities,
        ui_surface::SurfaceCapabilitySet::new(true, false, false, false)
    );
}

#[test]
fn mounted_surface_request_without_registry_matches_legacy_behavior() {
    let shell_state = RunenwerkEditorShellState::new();
    let document_context = SurfaceDocumentContext::NoActiveDocument;

    let legacy_requests = mounted_surface_requests(&shell_state, document_context.clone());
    let explicit_requests =
        mounted_surface_requests_with_registry(&shell_state, document_context, None);

    assert_eq!(legacy_requests, explicit_requests);
    assert!(
        legacy_requests
            .iter()
            .all(|request| request.provider_family_id.is_none() && request.surface_route.is_none())
    );
}

#[test]
fn provider_resolution_unchanged_when_metadata_is_present_but_providers_ignore_it() {
    let registry =
        EditorSurfaceProviderRegistry::new(vec![dummy(1, 200, true), dummy(2, 10, false)])
            .expect("ids are unique");
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let mut enriched_request = request();
    enriched_request.stable_surface_key =
        ToolSurfaceStableKey::new("runenwerk.scene.viewport").unwrap();
    enriched_request.provider_family_id = Some(ProviderFamilyId::new("runenwerk.scene").unwrap());
    enriched_request.surface_route = Some(ToolSurfaceRoute::ProviderOwnedLocal);

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &enriched_request,
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(
        frame.provider_id,
        Some(SurfaceProviderId::try_from_raw(1).unwrap())
    );
}

#[test]
fn provider_family_filtering_limits_candidates_before_supports() {
    let provider_registry =
        EditorSurfaceProviderRegistry::new(vec![dummy(1, 1, true), dummy(2, 200, true)])
            .expect("ids are unique");
    let suite_registry = scene_viewport_tool_suite_registry();
    let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[2]);
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = provider_registry.resolve_frame_with_provider_family_map(
        &context(&app, &shell_state, &theme),
        &request_for_provider_family("runenwerk.scene"),
        &Default::default(),
        Some(&provider_family_map),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(
        frame.provider_id,
        Some(SurfaceProviderId::try_from_raw(2).unwrap())
    );
}

#[test]
fn provider_family_filtering_still_runs_before_stable_key_matching() {
    let provider_registry = EditorSurfaceProviderRegistry::new(vec![
        dummy_with_support_mode(1, 1, SurfaceProviderSupportMode::StableKey),
        dummy_with_support_mode(2, 10, SurfaceProviderSupportMode::StableKey),
    ])
    .expect("ids are unique");
    let suite_registry = scene_viewport_tool_suite_registry();
    let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[2]);
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = provider_registry.resolve_frame_with_provider_family_map(
        &context(&app, &shell_state, &theme),
        &request_for_provider_family("runenwerk.scene"),
        &Default::default(),
        Some(&provider_family_map),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(
        frame.provider_id,
        Some(SurfaceProviderId::try_from_raw(2).unwrap())
    );
}

#[test]
fn provider_family_filtering_preserves_existing_priority_resolution() {
    let provider_registry =
        EditorSurfaceProviderRegistry::new(vec![dummy(1, 200, true), dummy(2, 10, true)])
            .expect("ids are unique");
    let suite_registry = scene_viewport_tool_suite_registry();
    let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[1, 2]);
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = provider_registry.resolve_frame_with_provider_family_map(
        &context(&app, &shell_state, &theme),
        &request_for_provider_family("runenwerk.scene"),
        &Default::default(),
        Some(&provider_family_map),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(
        frame.provider_id,
        Some(SurfaceProviderId::try_from_raw(2).unwrap())
    );
}

#[test]
fn missing_provider_family_assignment_fails_closed() {
    let provider_registry =
        EditorSurfaceProviderRegistry::new(vec![dummy(1, 1, true)]).expect("ids are unique");
    let suite_registry = tool_suite_registry_for_provider_family("runenwerk.scene");
    let provider_family_map = ProviderFamilyProviderMap::new(&suite_registry, Vec::new()).unwrap();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = provider_registry.resolve_frame_with_provider_family_map(
        &context(&app, &shell_state, &theme),
        &request_for_provider_family("runenwerk.scene"),
        &Default::default(),
        Some(&provider_family_map),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
    assert_eq!(frame.provider_id, None);
    assert!(frame.routes.is_empty());
}

#[test]
fn request_without_provider_family_id_keeps_legacy_full_candidate_behavior() {
    let provider_registry =
        EditorSurfaceProviderRegistry::new(vec![dummy(1, 1, true), dummy(2, 200, true)])
            .expect("ids are unique");
    let suite_registry = scene_viewport_tool_suite_registry();
    let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[2]);
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = provider_registry.resolve_frame_with_provider_family_map(
        &context(&app, &shell_state, &theme),
        &request(),
        &Default::default(),
        Some(&provider_family_map),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(
        frame.provider_id,
        Some(SurfaceProviderId::try_from_raw(1).unwrap())
    );
}

#[test]
fn provider_family_filtering_does_not_change_material_graph_provider_resolution() {
    let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let mut request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);
    request.stable_surface_key =
        ToolSurfaceStableKey::new("runenwerk.material_lab.graph_canvas").unwrap();
    request.provider_family_id = Some(ProviderFamilyId::new("runenwerk.material_lab").unwrap());
    request.surface_route = Some(ToolSurfaceRoute::ProviderOwnedGraphCanvas);

    let legacy_frame = host.provider_registry().resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );
    let filtered_frame = host
        .provider_registry()
        .resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
            Some(host.provider_family_provider_map()),
        );

    assert_eq!(filtered_frame.availability, legacy_frame.availability);
    assert_eq!(filtered_frame.provider_id, legacy_frame.provider_id);
    assert_eq!(
        filtered_frame.provider_id,
        Some(SurfaceProviderId::try_from_raw(12).unwrap())
    );
}

#[test]
fn equal_priority_ambiguity_still_fails_closed_after_family_filtering() {
    let provider_registry =
        EditorSurfaceProviderRegistry::new(vec![dummy(1, 100, true), dummy(2, 100, true)])
            .expect("ids are unique");
    let suite_registry = scene_viewport_tool_suite_registry();
    let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[1, 2]);
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = provider_registry.resolve_frame_with_provider_family_map(
        &context(&app, &shell_state, &theme),
        &request_for_provider_family("runenwerk.scene"),
        &Default::default(),
        Some(&provider_family_map),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Ambiguous);
    assert_eq!(frame.provider_id, None);
    assert!(frame.routes.is_empty());
}

#[test]
fn provider_family_filtering_keeps_provider_supports_enum_backed() {
    let provider_registry =
        EditorSurfaceProviderRegistry::new(vec![dummy(1, 100, false)]).expect("ids are unique");
    let suite_registry = scene_viewport_tool_suite_registry();
    let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[1]);
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = provider_registry.resolve_frame_with_provider_family_map(
        &context(&app, &shell_state, &theme),
        &request_for_provider_family("runenwerk.scene"),
        &Default::default(),
        Some(&provider_family_map),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
    assert_eq!(frame.provider_id, None);
}

#[test]
fn material_graph_provider_supports_stable_key_first() {
    let provider = MaterialGraphCanvasProvider;
    let mut request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);
    request.stable_surface_key =
        ToolSurfaceStableKey::new(MATERIAL_GRAPH_CANVAS_SURFACE_KEY).unwrap();

    assert_eq!(
        provider.support_mode(&request),
        SurfaceProviderSupportMode::StableKey
    );
}

#[test]
fn material_graph_provider_rejects_mismatched_stable_key() {
    let provider = MaterialGraphCanvasProvider;
    let mut request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);
    request.stable_surface_key = ToolSurfaceStableKey::new("runenwerk.fixture.other").unwrap();

    assert_eq!(
        provider.support_mode(&request),
        SurfaceProviderSupportMode::Unsupported
    );
}

#[test]
fn material_inspector_provider_supports_stable_key_first() {
    let provider = MaterialInspectorProvider;
    let mut request = m6_material_request(ToolSurfaceKind::MaterialInspector);
    request.stable_surface_key = ToolSurfaceStableKey::new(MATERIAL_INSPECTOR_SURFACE_KEY).unwrap();

    assert_eq!(
        provider.support_mode(&request),
        SurfaceProviderSupportMode::StableKey
    );
}

#[test]
fn material_preview_provider_supports_stable_key_first() {
    let provider = MaterialPreviewProvider;
    let mut request = m6_material_request(ToolSurfaceKind::MaterialPreview);
    request.stable_surface_key = ToolSurfaceStableKey::new(MATERIAL_PREVIEW_SURFACE_KEY).unwrap();

    assert_eq!(
        provider.support_mode(&request),
        SurfaceProviderSupportMode::StableKey
    );
}

#[test]
fn material_lab_providers_do_not_require_legacy_tool_surface_kind() {
    let cases = [
        (
            &MaterialGraphCanvasProvider as &dyn EditorSurfaceProvider,
            stable_key_only_material_request(
                MATERIAL_GRAPH_CANVAS_SURFACE_KEY,
                ToolSurfaceRoute::ProviderOwnedGraphCanvas,
            ),
        ),
        (
            &MaterialInspectorProvider,
            stable_key_only_material_request(
                MATERIAL_INSPECTOR_SURFACE_KEY,
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
        ),
        (
            &MaterialPreviewProvider,
            stable_key_only_material_request(
                MATERIAL_PREVIEW_SURFACE_KEY,
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
        ),
    ];

    for (provider, request) in cases {
        assert_eq!(
            provider.support_mode(&request),
            SurfaceProviderSupportMode::StableKey
        );
        assert!(provider.supports(&request));
    }
}

#[test]
fn material_lab_provider_resolution_uses_stable_keys() {
    let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let cases = [
        (
            stable_key_only_material_request(
                MATERIAL_GRAPH_CANVAS_SURFACE_KEY,
                ToolSurfaceRoute::ProviderOwnedGraphCanvas,
            ),
            MATERIAL_GRAPH_CANVAS_PROVIDER_ID,
        ),
        (
            stable_key_only_material_request(
                MATERIAL_INSPECTOR_SURFACE_KEY,
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
            MATERIAL_INSPECTOR_PROVIDER_ID,
        ),
        (
            stable_key_only_material_request(
                MATERIAL_PREVIEW_SURFACE_KEY,
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
            MATERIAL_PREVIEW_PROVIDER_ID,
        ),
    ];

    for (request, expected_provider_id) in cases {
        let frame = host
            .provider_registry()
            .resolve_frame_with_provider_family_map(
                &context(&app, &shell_state, &theme),
                &request,
                &SurfaceSessionState::default(),
                Some(host.provider_family_provider_map()),
            );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(frame.provider_id, Some(expected_provider_id));
        assert_eq!(frame.stable_surface_key, request.stable_surface_key);
    }
}

#[test]
fn texture_providers_support_registered_stable_keys() {
    let texture_provider = TextureViewerProvider;
    let mut texture_request = m6_texture_request(ToolSurfaceKind::TextureViewer);
    texture_request.stable_surface_key =
        ToolSurfaceStableKey::new(TEXTURE_VIEWER_2D_SURFACE_KEY).unwrap();

    let volume_provider = VolumeTextureViewerProvider;
    let mut volume_request = m6_texture_request(ToolSurfaceKind::VolumeTextureViewer);
    volume_request.stable_surface_key =
        ToolSurfaceStableKey::new(TEXTURE_VIEWER_3D_SURFACE_KEY).unwrap();

    assert_eq!(
        texture_provider.support_mode(&texture_request),
        SurfaceProviderSupportMode::StableKey
    );
    assert_eq!(
        volume_provider.support_mode(&volume_request),
        SurfaceProviderSupportMode::StableKey
    );
}

#[test]
fn asset_providers_support_registered_stable_keys() {
    let asset_provider = AssetBrowserProvider;
    let mut asset_browser_request = asset_request(ToolSurfaceKind::AssetBrowser);
    asset_browser_request.stable_surface_key =
        ToolSurfaceStableKey::new(ASSET_BROWSER_SURFACE_KEY).unwrap();

    let import_provider = ImportInspectorProvider;
    let mut import_request = asset_request(ToolSurfaceKind::ImportInspector);
    import_request.stable_surface_key =
        ToolSurfaceStableKey::new(IMPORT_INSPECTOR_SURFACE_KEY).unwrap();

    assert_eq!(
        asset_provider.support_mode(&asset_browser_request),
        SurfaceProviderSupportMode::StableKey
    );
    assert_eq!(
        import_provider.support_mode(&import_request),
        SurfaceProviderSupportMode::StableKey
    );
}

#[test]
fn console_and_editor_design_providers_support_registered_stable_keys() {
    let console_provider = ConsoleProvider;
    let console_request =
        request_with_stable_key(EDITOR_CONSOLE_SURFACE_KEY, ToolSurfaceKind::Console);

    let self_authoring_provider = SelfAuthoringProvider;
    let mut definition_validation_request =
        self_authoring_request(ToolSurfaceKind::DefinitionValidation);
    definition_validation_request.stable_surface_key =
        ToolSurfaceStableKey::new("runenwerk.editor_design.definition_validation").unwrap();

    assert_eq!(
        console_provider.support_mode(&console_request),
        SurfaceProviderSupportMode::StableKey
    );
    assert_eq!(
        self_authoring_provider.support_mode(&definition_validation_request),
        SurfaceProviderSupportMode::StableKey
    );
}

#[test]
fn field_and_procgen_providers_support_registered_stable_keys() {
    let field_product_provider = FieldProductViewerProvider;
    let mut field_product_request = asset_request(ToolSurfaceKind::FieldProductViewer);
    field_product_request.stable_surface_key =
        ToolSurfaceStableKey::new(FIELD_PRODUCT_VIEWER_SURFACE_KEY).unwrap();

    let sdf_brush_provider = SdfBrushBrowserProvider;
    let mut sdf_brush_request = asset_request(ToolSurfaceKind::SdfBrushBrowser);
    sdf_brush_request.stable_surface_key =
        ToolSurfaceStableKey::new(SDF_BRUSH_BROWSER_SURFACE_KEY).unwrap();

    let field_layer_provider = FieldLayerStackProvider;
    let mut field_layer_request =
        m6_sdf_request(ToolSurfaceKind::FieldLayerStack, DocumentKind::SdfGraph);
    field_layer_request.stable_surface_key =
        ToolSurfaceStableKey::new(FIELD_LAYER_STACK_SURFACE_KEY).unwrap();

    let sdf_graph_provider = SdfGraphCanvasProvider;
    let mut sdf_graph_request =
        m6_sdf_request(ToolSurfaceKind::SdfGraphCanvas, DocumentKind::SdfGraph);
    sdf_graph_request.stable_surface_key =
        ToolSurfaceStableKey::new(SDF_GRAPH_CANVAS_SURFACE_KEY).unwrap();

    let procgen_graph_provider = ProcgenGraphCanvasProvider;
    let mut procgen_graph_request = m6_procgen_request(ToolSurfaceKind::ProcgenGraphCanvas);
    procgen_graph_request.stable_surface_key =
        ToolSurfaceStableKey::new(PROCGEN_GRAPH_CANVAS_SURFACE_KEY).unwrap();

    let procgen_preview_provider = ProcgenPreviewProvider;
    let mut procgen_preview_request = m6_procgen_request(ToolSurfaceKind::ProcgenPreview);
    procgen_preview_request.stable_surface_key =
        ToolSurfaceStableKey::new(PROCGEN_PREVIEW_SURFACE_KEY).unwrap();

    for (provider, request) in [
        (
            &field_product_provider as &dyn EditorSurfaceProvider,
            &field_product_request,
        ),
        (&sdf_brush_provider, &sdf_brush_request),
        (&field_layer_provider, &field_layer_request),
        (&sdf_graph_provider, &sdf_graph_request),
        (&procgen_graph_provider, &procgen_graph_request),
        (&procgen_preview_provider, &procgen_preview_request),
    ] {
        assert_eq!(
            provider.support_mode(request),
            SurfaceProviderSupportMode::StableKey
        );
    }
}

#[test]
fn scene_core_providers_support_registered_stable_keys() {
    let providers = [
        (
            Box::new(SceneOutlinerProvider) as Box<dyn EditorSurfaceProvider>,
            SCENE_OUTLINER_SURFACE_KEY,
            ToolSurfaceKind::Outliner,
        ),
        (
            Box::new(SceneEntityTableProvider) as Box<dyn EditorSurfaceProvider>,
            SCENE_ENTITY_TABLE_SURFACE_KEY,
            ToolSurfaceKind::EntityTable,
        ),
        (
            Box::new(SceneViewportProvider) as Box<dyn EditorSurfaceProvider>,
            SCENE_VIEWPORT_SURFACE_KEY,
            ToolSurfaceKind::Viewport,
        ),
        (
            Box::new(SceneInspectorProvider) as Box<dyn EditorSurfaceProvider>,
            SCENE_INSPECTOR_SURFACE_KEY,
            ToolSurfaceKind::Inspector,
        ),
    ];

    for (provider, stable_key, kind) in providers {
        let request = request_with_stable_key(stable_key, kind);
        assert_eq!(
            provider.support_mode(&request),
            SurfaceProviderSupportMode::StableKey
        );
    }
}

#[test]
fn provider_matching_constants_match_registered_suite_keys() {
    let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
    let constants = [
        &[SCENE_OUTLINER_SURFACE_KEY][..],
        &[SCENE_ENTITY_TABLE_SURFACE_KEY],
        &[SCENE_VIEWPORT_SURFACE_KEY],
        &[SCENE_INSPECTOR_SURFACE_KEY],
        &[EDITOR_CONSOLE_SURFACE_KEY],
        EDITOR_DESIGN_SURFACE_KEYS,
        &[ASSET_BROWSER_SURFACE_KEY],
        &[IMPORT_INSPECTOR_SURFACE_KEY],
        &[FIELD_PRODUCT_VIEWER_SURFACE_KEY],
        &[SDF_BRUSH_BROWSER_SURFACE_KEY],
        &[FIELD_LAYER_STACK_SURFACE_KEY],
        &[SDF_GRAPH_CANVAS_SURFACE_KEY],
        DIAGNOSTICS_SURFACE_KEYS,
        &[MATERIAL_GRAPH_CANVAS_SURFACE_KEY],
        &[MATERIAL_INSPECTOR_SURFACE_KEY],
        &[MATERIAL_PREVIEW_SURFACE_KEY],
        &[TEXTURE_VIEWER_2D_SURFACE_KEY],
        &[TEXTURE_VIEWER_3D_SURFACE_KEY],
        &[PROCGEN_GRAPH_CANVAS_SURFACE_KEY],
        &[PROCGEN_PREVIEW_SURFACE_KEY],
        &[TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY],
    ];

    for stable_key in constants.into_iter().flatten() {
        let stable_key = ToolSurfaceStableKey::new(*stable_key).unwrap();
        assert!(
            host.tool_surface_registry().get(&stable_key).is_some(),
            "provider matching constant should be registered: {}",
            stable_key.as_str()
        );
    }
}

#[test]
fn tool_suite_registry_inspector_provider_is_registered() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();

    assert!(registry.has_provider_id(TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID));
    assert!(
        registry
            .provider_descriptors()
            .any(
                |descriptor| descriptor.id == TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID
                    && descriptor.label == "Tool Suite Registry Inspector"
            )
    );
}

#[test]
fn inspector_provider_supports_stable_key_only() {
    let provider = ToolSuiteRegistryInspectorProvider;
    let request = inspector_request();

    assert_eq!(
        provider.support_mode(&request),
        SurfaceProviderSupportMode::StableKey
    );
    assert!(provider.supports(&request));
}

#[test]
fn inspector_provider_does_not_support_legacy_kind() {
    let provider = ToolSuiteRegistryInspectorProvider;
    let mut request = inspector_request();
    request.stable_surface_key =
        ToolSurfaceStableKey::new("runenwerk.diagnostics.diagnostics").unwrap();

    assert_eq!(
        provider.support_mode(&request),
        SurfaceProviderSupportMode::Unsupported
    );
    assert!(!provider.supports(&request));
}

#[test]
fn inspector_surface_can_be_resolved_by_stable_key() {
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let request = inspector_request();

    let frame = app
        .workbench_host()
        .provider_registry()
        .resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request,
            &SurfaceSessionState::default(),
            Some(app.workbench_host().provider_family_provider_map()),
        );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(
        frame.provider_id,
        Some(TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID)
    );
    assert_eq!(frame.title, "Tool Suite Registry Inspector");
    assert!(frame.routes.is_empty());
}

#[test]
fn inspector_resolution_observation_matches_provider_resolution_for_material_lab() {
    let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let mut request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);
    request.provider_family_id = Some(ProviderFamilyId::new("runenwerk.material_lab").unwrap());
    request.surface_route = Some(ToolSurfaceRoute::ProviderOwnedGraphCanvas);

    let observation = host.provider_registry().observe_resolution_for_request(
        &request,
        host.workspace_profile_registry(),
        Some(host.provider_family_provider_map()),
    );
    let frame = host
        .provider_registry()
        .resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request,
            &SurfaceSessionState::default(),
            Some(host.provider_family_provider_map()),
        );

    assert_eq!(observation.availability, frame.availability);
    assert_eq!(observation.selected_provider_id, frame.provider_id);
    assert!(
        observation
            .candidate_provider_ids
            .contains(&MATERIAL_GRAPH_CANVAS_PROVIDER_ID)
    );
    assert!(observation.support_modes.iter().any(|row| {
        row.provider_id == MATERIAL_GRAPH_CANVAS_PROVIDER_ID
            && row.support_mode == SurfaceProviderSupportMode::StableKey
    }));
}

#[test]
fn inspector_resolution_observation_matches_provider_resolution_for_diagnostics_inspector() {
    let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let request = inspector_request();

    let observation = host.provider_registry().observe_resolution_for_request(
        &request,
        host.workspace_profile_registry(),
        Some(host.provider_family_provider_map()),
    );
    let frame = host
        .provider_registry()
        .resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request,
            &SurfaceSessionState::default(),
            Some(host.provider_family_provider_map()),
        );

    assert_eq!(observation.availability, frame.availability);
    assert_eq!(observation.selected_provider_id, frame.provider_id);
    assert_eq!(
        observation.selected_provider_id,
        Some(TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID)
    );
}

#[test]
fn unresolved_mounted_surface_reports_diagnostic_without_mutation() {
    let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
    let provider_count_before = host.provider_registry().provider_ids().count();
    let assignment_count_before = host.provider_family_provider_map().assignments().len();
    let mut request = request();
    request.stable_surface_key =
        ToolSurfaceStableKey::new("runenwerk.gameplay.graph_canvas").unwrap();
    request.provider_family_id = Some(ProviderFamilyId::new("runenwerk.gameplay").unwrap());
    request.surface_route = Some(ToolSurfaceRoute::ProviderOwnedLocal);

    let observation = host.provider_registry().observe_resolution_for_request(
        &request,
        host.workspace_profile_registry(),
        Some(host.provider_family_provider_map()),
    );

    assert_eq!(
        observation.availability,
        SurfaceProviderAvailability::Unsupported
    );
    assert_eq!(observation.selected_provider_id, None);
    assert!(observation.diagnostic.is_some_and(|diagnostic| {
        diagnostic.code == "editor.surface.unassigned_provider_family"
    }));
    assert_eq!(
        host.provider_registry().provider_ids().count(),
        provider_count_before
    );
    assert_eq!(
        host.provider_family_provider_map().assignments().len(),
        assignment_count_before
    );
}

#[test]
fn no_new_tool_surface_kind_for_inspector() {
    let state_source =
        include_str!("../../../../../domain/editor/editor_shell/src/workspace/state.rs");
    let shell_source = include_str!(
        "../../../../../domain/editor/editor_shell/src/composition/build_editor_shell.rs"
    );

    assert!(!state_source.contains("ToolSuiteRegistryInspector"));
    assert!(!shell_source.contains(TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY));
}

#[test]
fn no_dynamic_plugin_behavior_introduced() {
    let inspector_source = include_str!("tool_suite_registry_inspector.rs");
    let diagnostics_suite_source = include_str!("../tool_suites/diagnostics_tool_suite.rs");

    for source in [inspector_source, diagnostics_suite_source] {
        assert!(!source.contains("runtime::plugin"));
        assert!(!source.contains("dynamic plugin"));
        assert!(!source.contains("PluginMarketplace"));
    }
}

#[test]
fn provider_resolution_prefers_stable_key_support() {
    let provider_registry = EditorSurfaceProviderRegistry::new(vec![
        dummy_with_support_mode(1, 1, SurfaceProviderSupportMode::LegacyKind),
        dummy_with_support_mode(2, 200, SurfaceProviderSupportMode::StableKey),
    ])
    .expect("ids are unique");
    let suite_registry = scene_viewport_tool_suite_registry();
    let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[1, 2]);
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let request = SurfaceProviderRequest {
        stable_surface_key: ToolSurfaceStableKey::new(SCENE_VIEWPORT_SURFACE_KEY).unwrap(),
        provider_family_id: Some(ProviderFamilyId::new("runenwerk.scene").unwrap()),
        ..request()
    };

    let frame = provider_registry.resolve_frame_with_provider_family_map(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
        Some(&provider_family_map),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(
        frame.provider_id,
        Some(SurfaceProviderId::try_from_raw(2).unwrap())
    );
}

#[test]
fn provider_resolution_preserves_legacy_support_when_stable_key_does_not_match() {
    let provider_registry = EditorSurfaceProviderRegistry::new(vec![
        dummy_with_support_mode(1, 200, SurfaceProviderSupportMode::LegacyKind),
        dummy_with_support_mode(2, 10, SurfaceProviderSupportMode::LegacyKind),
    ])
    .expect("ids are unique");
    let suite_registry = scene_viewport_tool_suite_registry();
    let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[1, 2]);
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let mut request = request_for_provider_family("runenwerk.scene");
    request.stable_surface_key = ToolSurfaceStableKey::new("runenwerk.fixture.other").unwrap();

    let frame = provider_registry.resolve_frame_with_provider_family_map(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
        Some(&provider_family_map),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(
        frame.provider_id,
        Some(SurfaceProviderId::try_from_raw(2).unwrap())
    );
}

#[test]
fn provider_resolution_works_without_legacy_kind_for_stable_key_supported_surface() {
    let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let mut request =
        request_with_stable_key(SCENE_VIEWPORT_SURFACE_KEY, ToolSurfaceKind::Viewport);
    request.provider_family_id = Some(ProviderFamilyId::new("runenwerk.scene").unwrap());

    let frame = host
        .provider_registry()
        .resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
            Some(host.provider_family_provider_map()),
        );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(frame.provider_id, Some(SCENE_VIEWPORT_PROVIDER_ID));
}

#[test]
fn provider_resolution_does_not_fall_back_when_stable_key_mismatches() {
    let provider = MaterialGraphCanvasProvider;
    let mut request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);
    request.stable_surface_key = ToolSurfaceStableKey::new("runenwerk.fixture.other").unwrap();

    assert_eq!(
        provider.support_mode(&request),
        SurfaceProviderSupportMode::Unsupported
    );
}

#[test]
fn stable_key_ambiguity_fails_closed() {
    let provider_registry = EditorSurfaceProviderRegistry::new(vec![
        dummy_with_support_mode(1, 100, SurfaceProviderSupportMode::StableKey),
        dummy_with_support_mode(2, 100, SurfaceProviderSupportMode::StableKey),
        dummy_with_support_mode(3, 1, SurfaceProviderSupportMode::LegacyKind),
    ])
    .expect("ids are unique");
    let suite_registry = scene_viewport_tool_suite_registry();
    let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[1, 2, 3]);
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let request = SurfaceProviderRequest {
        stable_surface_key: ToolSurfaceStableKey::new(SCENE_VIEWPORT_SURFACE_KEY).unwrap(),
        provider_family_id: Some(ProviderFamilyId::new("runenwerk.scene").unwrap()),
        ..request()
    };

    let frame = provider_registry.resolve_frame_with_provider_family_map(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
        Some(&provider_family_map),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Ambiguous);
    assert_eq!(frame.provider_id, None);
    assert!(frame.routes.is_empty());
}

#[test]
fn future_placeholder_families_do_not_gain_provider_support() {
    let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let mut request = m6_procgen_request(ToolSurfaceKind::GameplayGraphCanvas);
    request.stable_surface_key =
        ToolSurfaceStableKey::new("runenwerk.gameplay.graph_canvas").unwrap();
    request.provider_family_id = Some(ProviderFamilyId::new("runenwerk.gameplay").unwrap());

    let frame = host
        .provider_registry()
        .resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
            Some(host.provider_family_provider_map()),
        );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
    assert_eq!(frame.provider_id, None);
    assert!(frame.routes.is_empty());
}

#[test]
fn live_mounted_surface_requests_include_provider_family_when_registry_resolves() {
    let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
    let shell_state = RunenwerkEditorShellState::new();
    let requests = mounted_surface_requests_with_registry(
        &shell_state,
        SurfaceDocumentContext::NoActiveDocument,
        Some(host.tool_surface_registry()),
    );

    let viewport_request = requests
        .iter()
        .find(|request| request.matches_stable_key(SCENE_VIEWPORT_SURFACE_KEY))
        .expect("default workspace should mount viewport");

    assert_eq!(
        viewport_request
            .provider_family_id
            .as_ref()
            .map(ProviderFamilyId::as_str),
        Some("runenwerk.scene")
    );
    assert_eq!(
        viewport_request.surface_route,
        Some(ToolSurfaceRoute::ProviderOwnedLocal)
    );
}

#[test]
fn unresolved_registry_surface_request_reports_diagnostic_in_live_frame_path() {
    let app = RunenwerkEditorApp::new();
    let provider_registry =
        EditorSurfaceProviderRegistry::new(vec![dummy(100, 100, true)]).expect("ids are unique");
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame_model = build_editor_shell_frame_model(
        &app,
        &shell_state,
        &provider_registry,
        &theme,
        None,
        None,
        None,
    );

    let viewport_frame = frame_model
        .surfaces
        .values()
        .find(|frame| frame.stable_surface_key.as_str() == SCENE_VIEWPORT_SURFACE_KEY)
        .expect("default workspace should include viewport frame");

    assert_eq!(
        viewport_frame.availability,
        SurfaceProviderAvailability::Unsupported
    );
    assert_eq!(viewport_frame.provider_id, None);
    assert!(viewport_frame.routes.is_empty());
}

#[test]
fn request_contains_stable_identity_definition_and_capabilities() {
    let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
    let shell_state = RunenwerkEditorShellState::new();
    let requests = mounted_surface_requests_with_registry(
        &shell_state,
        SurfaceDocumentContext::NoActiveDocument,
        Some(host.tool_surface_registry()),
    );
    let request = requests
        .iter()
        .find(|request| request.matches_stable_key(SCENE_VIEWPORT_SURFACE_KEY))
        .expect("default workspace should mount viewport");

    assert_eq!(request.stable_key().as_str(), SCENE_VIEWPORT_SURFACE_KEY);
    assert_eq!(
        request.surface_definition_id,
        tool_surface_definition_id(ToolSurfaceKind::Viewport)
    );
    assert_eq!(
        request.capabilities,
        tool_surface_capability_set(ToolSurfaceKind::Viewport)
    );
}

#[test]
fn non_material_providers_ignore_graph_canvas_interaction_by_default() {
    let registry =
        EditorSurfaceProviderRegistry::new(vec![dummy(1, 200, true)]).expect("ids are unique");
    let request = request();
    let proposal = registry
        .map_interaction(
            &SurfaceProviderDispatchContext {
                projection_epoch: 61,
                _marker: std::marker::PhantomData,
            },
            &request,
            SurfaceProviderId::try_from_raw(1).unwrap(),
            SurfaceInteraction::GraphCanvasAction(
                ui_graph_editor::GraphCanvasAction::ClearSelection,
            ),
        )
        .expect("default provider interaction mapper should not fail");

    assert_eq!(proposal, None);
}

#[test]
fn self_authoring_provider_resolves_definition_validation_without_scene_document() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let request = self_authoring_request(ToolSurfaceKind::DefinitionValidation);

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &SurfaceSessionState::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(frame.title, "Definition Validation");
    assert!(!frame.routes.is_empty());
}

#[test]
fn editor_lab_provider_builds_typed_direct_control_surfaces() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    for surface_kind in [
        ToolSurfaceKind::EditorDesignOutliner,
        ToolSurfaceKind::UiHierarchy,
        ToolSurfaceKind::StyleInspector,
        ToolSurfaceKind::DockLayoutPreview,
        ToolSurfaceKind::DefinitionValidation,
        ToolSurfaceKind::CommandDiff,
    ] {
        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &self_authoring_request(surface_kind),
            &SurfaceSessionState::default(),
        );
        let text = provider_frame_text(&frame);

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert!(text.contains("Editor Lab"), "{surface_kind:?} text: {text}");
        assert!(
            frame_has_editor_definition_route(&frame),
            "{surface_kind:?} should expose direct EditorDefinition routes"
        );
        assert!(!text.contains("Edited in self-authoring"));
        assert!(!text.contains("Retained draft"));
        assert!(!text.contains("Authored Tab"));
    }
}

#[test]
fn ui_designer_workbench_canvas_is_standalone_and_recoverable() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    assert!(
        shell_state
            .self_authoring_mut()
            .select_document_by_str("runenwerk.editor.theme.default")
    );

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &self_authoring_request(ToolSurfaceKind::UiCanvas),
        &SurfaceSessionState::default(),
    );
    let text = provider_frame_text(&frame);

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(frame.title, "UI Designer Workbench");
    assert!(text.contains("UI Designer Workbench"));
    assert!(text.contains("Standalone UI Designer Workbench"));
    assert!(text.contains("Canvas - UI Canvas"));
    assert!(text.contains("Scenario Matrix - Scenario Matrix"));
    assert!(text.contains("Native Evidence - Native Evidence"));
    assert!(text.contains("Selected definition cannot form a retained UI preview"));
    assert!(frame_has_editor_definition_route(&frame));
    assert!(!text.contains("build_self_authoring_control_panel"));
}

#[test]
fn ui_designer_workbench_exposes_v1_catalog_and_source_version_parity() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let state = shell_state.self_authoring();
    let source_version = state
        .selected_source_version_label()
        .expect("default shell state should expose source revision");

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &self_authoring_request(ToolSurfaceKind::UiCanvas),
        &SurfaceSessionState::default(),
    );
    let text = provider_frame_text(&frame);

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert!(text.contains("Token / Recipe Preview - Component Catalog"));
    assert!(text.contains("catalog target profile: editor.workbench"));
    assert!(text.contains("recipe declarations: 2 enabled insertions: 1"));
    assert!(text.contains("Primary Button (editor.pattern.primary_button)"));
    assert!(text.contains(
        "category=editor.product.action target=editor.workbench compatibility=compatible: editor.workbench source=editor.product.design_system slots=direct insert into selected container readiness=product"
    ));
    assert!(text.contains("Insert Primary Button"));
    assert!(
        text.contains(
            "tokens=color.accent, color.foreground, radius.sm, spacing.sm, typography.body"
        )
    );
    assert!(text.contains("Toolbar Command Group (editor.pattern.toolbar_command_group)"));
    assert!(text.contains("readiness=diagnostic"));
    assert!(text.contains("ui.recipe.preview_only_activation"));
    assert!(text.contains("pm003_recipe_catalog_insertion"));
    assert!(text.contains("pm004_product_catalog_rows"));
    assert!(text.contains("pm004_source_version_selection_parity"));
    assert!(
        text.matches(&format!("source version: {source_version}"))
            .count()
            >= 3,
        "catalog, canvas, hierarchy, and inspector panes should share source version: {text}"
    );
    assert_eq!(
        frame
            .routes
            .iter()
            .filter(|(_, route)| matches!(
                route.action(),
                Some(SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::InsertRecipe { recipe_id }
                )) if recipe_id.as_str() == editor_shell::EDITOR_UX_PRIMARY_BUTTON_RECIPE_ID
            ))
            .count(),
        1,
        "only the compatible recipe row should expose an InsertRecipe route: {text}"
    );
    assert!(frame_has_editor_definition_route(&frame));
}

#[test]
fn ui_designer_workbench_catalog_filter_routes_and_filters_structured_rows() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let unfiltered = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &self_authoring_request(ToolSurfaceKind::UiCanvas),
        &SurfaceSessionState::default(),
    );
    assert!(unfiltered.routes.iter().any(|(_, route)| matches!(
        route.action(),
        Some(SurfaceLocalAction::EditorDefinition(
            EditorDefinitionSurfaceAction::SetRecipeCatalogFilter { .. }
        ))
    )));

    crate::shell::dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::SetEditorDefinitionRecipeCatalogFilter {
            query: "toolbar".to_string(),
        },
        None,
        None,
        None,
        None,
    )
    .expect("catalog filter command should dispatch through shell state");

    let filtered = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &self_authoring_request(ToolSurfaceKind::UiCanvas),
        &SurfaceSessionState::default(),
    );
    let text = provider_frame_text(&filtered);

    assert!(text.contains("Filter component catalog"));
    assert!(text.contains("Toolbar Command Group (editor.pattern.toolbar_command_group)"));
    assert!(!text.contains("Primary Button (editor.pattern.primary_button)"));
    assert!(!text.contains("Insert Primary Button"));
}

#[test]
fn ui_designer_workbench_exposes_operation_apply_reload_and_rollback_state() {
    use crate::shell::editor_lab_project::{DefinitionApplyReviewStatus, EditorLabRollbackStatus};
    use editor_definition::{
        EditorLabOperation, EditorLabOperationDiffFamily, EditorLabOperationKind,
        EditorLabOperationStatus,
    };

    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let document_id = shell_state
        .self_authoring()
        .selected_document_id()
        .expect("default shell state should select a UI Designer document")
        .clone();
    let selected_node_id = shell_state
        .self_authoring()
        .selected_ui_node_id()
        .expect("default UI Designer document should expose a text-capable node")
        .to_string();
    let operation_id = shell_state.self_authoring().next_operation_id("pm005_text");

    let report = shell_state
        .self_authoring_mut()
        .apply_editor_lab_operation(EditorLabOperation {
            id: operation_id.clone(),
            document_id: document_id.clone(),
            target_profile: "editor.workbench".to_string(),
            kind: EditorLabOperationKind::SetUiNodeText {
                node_id: selected_node_id.clone(),
                text: "PM005 operation text".to_string(),
            },
            preview_only: false,
            source: Some("pm005.ui_designer_workbench".to_string()),
        })
        .expect("UI Designer text edit should dispatch through EditorLabOperation");
    assert_eq!(report.status, EditorLabOperationStatus::Accepted);
    assert!(report.diff.as_ref().is_some_and(|diff| {
        diff.changes
            .iter()
            .any(|change| change.family == EditorLabOperationDiffFamily::UiAuthoredValue)
    }));

    let rejected = shell_state
        .self_authoring_mut()
        .reject_last_apply_review()
        .expect("apply review should reject without mutating applied state");
    assert_eq!(rejected.status, DefinitionApplyReviewStatus::Rejected);
    assert_eq!(shell_state.self_authoring().applied_count(), 0);

    shell_state
        .self_authoring_mut()
        .apply_selected()
        .expect("selected UI Designer document should apply through a review");
    assert_eq!(
        shell_state
            .self_authoring()
            .last_apply_review()
            .expect("apply should record a review")
            .status,
        DefinitionApplyReviewStatus::Accepted
    );
    assert!(
        shell_state
            .self_authoring()
            .last_applied_document(&document_id)
            .is_some()
    );

    shell_state
        .self_authoring_mut()
        .set_selected_ui_node_text(&selected_node_id, "PM005 dirty draft")
        .expect("draft edits should remain app-owned after apply");
    shell_state
        .self_authoring_mut()
        .reload_selected_from_last_applied()
        .expect("last applied snapshot should reload into the draft");
    let reloaded_preview = shell_state
        .self_authoring()
        .formed_selected_preview(&theme)
        .expect("reloaded applied snapshot should still form a preview");
    assert!(format!("{:?}", reloaded_preview.root).contains("PM005 operation text"));
    assert!(!format!("{:?}", reloaded_preview.root).contains("PM005 dirty draft"));

    shell_state
        .self_authoring_mut()
        .rollback_selected()
        .expect("applied UI Designer document should roll back");
    assert_eq!(
        shell_state
            .self_authoring()
            .last_rollback_record()
            .expect("rollback should record typed recovery state")
            .status,
        EditorLabRollbackStatus::RolledBack
    );
    let history = shell_state.self_authoring().operation_history_snapshot();
    assert_eq!(history.undo_count, 1);
    let source_version = shell_state
        .self_authoring()
        .selected_source_version_label()
        .expect("selected document should expose source revision");

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &self_authoring_request(ToolSurfaceKind::UiCanvas),
        &SurfaceSessionState::default(),
    );
    let text = provider_frame_text(&frame);

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert!(text.contains("pm005_operation_diff_history"));
    assert!(text.contains("pm005_apply_reload_rollback"));
    assert!(text.contains(&format!("source version: {source_version}")));
    assert!(text.contains(&format!("last operation: {operation_id} Accepted")));
    assert!(text.contains("operation diff: UiAuthoredValue"));
    assert!(text.contains("operation history: undo=1 redo=0"));
    assert!(text.contains("apply review:"));
    assert!(text.contains("Accepted diffs="));
    assert!(text.contains("last applied snapshot:"));
    assert!(text.contains("rollback:"));
    assert!(text.contains("RolledBack"));
    assert!(text.contains("Reload last applied"));
    assert!(text.contains("Apply selected definition"));
    assert!(frame_has_editor_definition_route(&frame));
}

#[test]
fn ui_designer_workbench_exposes_pm005_scenario_evidence_and_performance_baselines() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let source_version = shell_state
        .self_authoring()
        .selected_source_version_label()
        .expect("default UI Designer document should expose a source version");

    let packets = shell_state
        .self_authoring_mut()
        .capture_pm005_scenario_evidence_packets(&theme)
        .expect("explicit PM005 scenario evidence capture should validate");
    assert_eq!(packets.len(), 2);
    let editor_packet = packets
        .iter()
        .find(|packet| packet.target_profile() == "editor.workbench")
        .expect("editor.workbench should have runtime evidence");
    assert_eq!(editor_packet.source_version(), source_version);
    assert_eq!(
        editor_packet.scenario_id(),
        "ui-designer.v1-closure.pm005.editor-workbench"
    );
    assert_eq!(editor_packet.performance_baselines().len(), 5);
    assert!(editor_packet.validate_runtime_product_evidence().is_ok());
    assert!(editor_packet.artifacts().iter().all(|artifact| {
        artifact.provenance
            == crate::shell::editor_lab_evidence::EditorLabEvidenceArtifactProvenance::ProductPath
            && artifact.bytes > 0
            && artifact.digest.starts_with("blake3:")
    }));

    let game_packet = packets
        .iter()
        .find(|packet| packet.target_profile() == "game.runtime")
        .expect("game.runtime should have descriptor compatibility evidence");
    assert_eq!(game_packet.source_version(), source_version);
    assert_eq!(
        game_packet.scenario_id(),
        "ui-designer.v1-closure.pm005.game-runtime"
    );
    assert!(!game_packet.is_runtime_product());
    assert!(game_packet.validate_scenario_evidence().is_ok());
    assert!(game_packet.validate_runtime_product_evidence().is_err());
    assert!(
        game_packet
            .fixture_bindings()
            .iter()
            .all(|binding| binding.read_only)
    );
    assert!(
        game_packet
            .intent_descriptors()
            .iter()
            .all(|intent| intent.validated && !intent.executes_runtime_command)
    );

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &self_authoring_request(ToolSurfaceKind::UiCanvas),
        &SurfaceSessionState::default(),
    );
    let text = provider_frame_text(&frame);

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert!(text.contains("pm005_runtime_product_evidence_packet"));
    assert!(text.contains("pm005_performance_baselines"));
    assert!(text.contains("pm005_game_runtime_descriptor_only"));
    assert!(text.contains("evidence packet: ui-designer.v1-closure.pm005.editor-workbench"));
    assert!(text.contains("evidence packet: ui-designer.v1-closure.pm005.game-runtime"));
    assert!(text.contains("kind=runtime-product"));
    assert!(text.contains("kind=descriptor-compatibility"));
    assert!(text.contains("target=editor.workbench"));
    assert!(text.contains("target=game.runtime"));
    assert!(text.contains("document=runenwerk.editor.toolbar"));
    assert!(text.contains(&format!("source={source_version}")));
    assert!(text.contains("capture=ExplicitCommand"));
    assert!(text.contains("freshness=Fresh"));
    assert!(text.contains("evidence artifact: RetainedUiDebug"));
    assert!(text.contains("evidence artifact: UnsupportedCheckReport"));
    assert!(text.contains("unsupported check: concrete game HUD runtime"));
    assert!(text.contains("fixture binding: fixture.game-runtime.safe-area"));
    assert!(text.contains("intent descriptor: intent.game-runtime.open-hud-preview"));
    assert!(text.contains("baseline: Resize"));
    assert!(text.contains("baseline: CanvasInteraction"));
    assert!(text.contains("baseline: CatalogProjection"));
    assert!(text.contains("baseline: DiagnosticsProjection"));
    assert!(text.contains("baseline: FrameBuild"));
    assert!(frame_has_editor_definition_route(&frame));
}

#[test]
fn ambiguous_provider_support_fails_closed_with_zero_routes() {
    let registry =
        EditorSurfaceProviderRegistry::new(vec![dummy(1, 100, true), dummy(2, 100, true)])
            .expect("ids are unique");
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request(),
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Ambiguous);
    assert!(frame.routes.is_empty());
}

#[test]
fn explicit_priority_resolves_deterministically() {
    let registry =
        EditorSurfaceProviderRegistry::new(vec![dummy(1, 200, true), dummy(2, 10, true)])
            .expect("ids are unique");
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request(),
        &Default::default(),
    );

    assert_eq!(
        frame.provider_id,
        Some(SurfaceProviderId::try_from_raw(2).unwrap())
    );
    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
}

#[test]
fn provider_error_artifact_has_diagnostic_and_zero_routes() {
    let registry = EditorSurfaceProviderRegistry::new(vec![failing(1)]).expect("id is unique");
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request(),
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Error);
    assert!(!frame.artifact.diagnostics.is_empty());
    assert!(frame.routes.is_empty());
}

#[test]
fn unsupported_provider_artifact_has_zero_routes() {
    let registry =
        EditorSurfaceProviderRegistry::new(vec![dummy(1, 100, false)]).expect("id is unique");
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request(),
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
    assert!(frame.routes.is_empty());
}

#[test]
fn no_active_document_does_not_resolve_scene_provider() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let request = request_with_document_context(
        SurfaceDocumentContext::NoActiveDocument,
        ToolSurfaceKind::Viewport,
    );

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
    assert!(frame.routes.is_empty());
}

#[test]
fn unresolved_document_returns_diagnostic_without_routes() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let request = request_with_document_context(
        SurfaceDocumentContext::Unresolved {
            document_id: editor_core::DocumentId(99),
        },
        ToolSurfaceKind::Inspector,
    );

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
    assert!(!frame.artifact.diagnostics.is_empty());
    assert!(frame.routes.is_empty());
}

#[test]
fn console_provider_resolves_without_active_scene_document() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let request = request_with_document_context(
        SurfaceDocumentContext::NoActiveDocument,
        ToolSurfaceKind::Console,
    );

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
}

#[test]
fn material_graph_canvas_provider_resolves_descriptor_surface_with_material_routes() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(700);
    let color_port = material_graph::MaterialValueType::Color.port_type_id();
    let document = material_graph::MaterialGraphDocument::new(
        material_graph::MaterialGraphDocumentId::new(701),
        "provider-source-backed",
        GraphDefinition::new(
            GraphId::new(701),
            "provider-source-backed",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(
                    NodeId::new(3),
                    "pbr.base_color",
                    [graph::PortDefinition::new(
                        graph::PortId::new(30),
                        "color",
                        graph::PortDirection::Output,
                        color_port,
                    )],
                ),
                NodeDefinition::new(
                    NodeId::new(4),
                    "pbr.output",
                    [graph::PortDefinition::new(
                        graph::PortId::new(40),
                        "base_color",
                        graph::PortDirection::Input,
                        color_port,
                    )],
                ),
            ],
            [graph::EdgeDefinition::new(
                graph::EdgeId::new(9),
                graph::PortId::new(30),
                graph::PortId::new(40),
            )],
        ),
        material_graph::MaterialOutputTarget::RenderMaterial,
    );
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "provider.material",
            "Provider Material",
            AssetKind::MaterialGraph,
        ));
    app.material_lab_runtime_mut()
        .set_active_source_document(asset_id, document);
    let request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(frame.provider_id, Some(MATERIAL_GRAPH_CANVAS_PROVIDER_ID));
    assert_eq!(frame.title, "Material Graph Canvas");
    assert!(provider_frame_text(&frame).contains("domain/material_graph remains material truth"));
    let canvas = frame_graph_canvas_model(&frame)
        .expect("material provider frame must contain a real graph canvas node");
    assert_eq!(canvas.nodes.len(), 2);
    assert_eq!(canvas.edges.len(), 1);
    assert_eq!(canvas.canvas_id, ui_graph_editor::GraphCanvasId(701));
    assert!(!frame.routes.is_empty());
}

#[test]
fn material_graph_canvas_view_model_exposes_structured_diagnostics() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    app.material_lab_runtime_mut().record_diagnostic(
        AssetDiagnosticRecord::new(
            AssetDiagnosticCode::RatificationRejected,
            AssetDiagnosticSeverity::Warning,
            "base color input is disconnected",
        )
        .with_subject("material_graph.node:7"),
    );

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_material_request(ToolSurfaceKind::MaterialGraphCanvas),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("material diagnostic [Warning] asset.ratification.rejected"));
    assert!(text.contains("subject=material_graph.node:7"));
    assert!(text.contains("base color input is disconnected"));
}

#[test]
fn material_preview_view_model_reports_preview_status() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_material_request(ToolSurfaceKind::MaterialPreview),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("material preview status [NoSelection]"));
    assert!(text.contains("No material asset selected"));
    assert!(text.contains("last good material preview available: false"));
    assert!(text.contains(
        "material preview scene: unavailable until a material preview product is active"
    ));
    assert!(!frame_has_product_surface(&frame));
}

#[test]
fn material_preview_provider_renders_preview_product_status() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    app.material_lab_runtime_mut()
        .set_active_preview(test_material_preview_product(asset_id(202)));

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_material_request(ToolSurfaceKind::MaterialPreview),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(
        text.contains("material preview product status: active material preview product ready")
    );
    assert!(text.contains("active material product label: material product 30"));
    assert!(text.contains("material preview artifact label: material artifact 32"));
    assert!(text.contains("material preview shader artifact label: shader artifact 33"));
    assert!(
        text.contains("material preview scene shader artifact label: scene shader artifact 34")
    );
    assert!(text.contains("material preview viewport product label: viewport product 10030"));
    assert!(text.contains("material preview scene: rendered GPU product-surface preview"));
    assert!(text.contains(
        "material preview scene target: runenwerk.editor.material_lab.preview_scene:material-product-30-scene"
    ));
    assert!(text.contains(
        "material preview scene bind group identity: engine_ui_product_surface_bind_group:runenwerk.editor.material_lab.preview_scene:material-product-30-scene"
    ));
    assert!(frame_has_product_surface(&frame));
}

#[test]
fn material_preview_provider_renders_preview_scene_product_section() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let product = test_preview_scene_product();
    app.material_lab_runtime_mut()
        .record_preview_scene_product(&product.request_identity(), product.clone())
        .expect("fresh preview scene product should record");

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_material_request(ToolSurfaceKind::MaterialPreview),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("Preview Scene Product"));
    assert!(text.contains("preview scene product status: current preview scene product available"));
    assert!(text.contains("preview scene product mode: scene material table"));
    assert!(text.contains(&format!(
        "preview scene product: {}",
        product.product_identity
    )));
    assert!(text.contains("preview scene product material table: material table provider-table"));
    assert!(
        text.contains("preview scene product resource layout: resource layout provider-layout")
    );
    assert!(
        text.contains("preview scene product shader: shader identity provider-shader-identity")
    );
    assert!(text.contains(
            "preview scene product shader artifact: shader artifact provider-shader-artifact cache provider-shader-cache"
        ));
    assert!(text.contains("preview scene product slots: 1"));
    assert!(text.contains("preview scene product resources: 1"));
}

#[test]
fn material_preview_provider_keeps_first_visible_path_sdf_primitive_material_binding() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_material_request(ToolSurfaceKind::MaterialPreview),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("scene material binding: SDF primitives use scene material slots"));
    assert!(!text.contains("Mesh Preview"));
    assert!(!text.contains("model/mesh preview"));
}

#[test]
fn material_inspector_renders_resource_binding_diagnostics() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    app.material_lab_runtime_mut()
        .set_active_source_document(asset_id(241), material_texture_binding_source_document());

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_material_request(ToolSurfaceKind::MaterialInspector),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("Texture / Resource Bindings"));
    assert!(text.contains("material.resource.unresolved_binding"));
    assert!(text.contains("status=Unresolved"));
}

#[test]
fn material_preview_renders_resource_binding_diagnostics() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    app.material_lab_runtime_mut()
        .set_active_source_document(asset_id(242), material_texture_binding_source_document());

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_material_request(ToolSurfaceKind::MaterialPreview),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("Texture / Resource Bindings"));
    assert!(text.contains("material.resource.unresolved_binding"));
    assert!(text.contains("status=Unresolved"));
}

#[test]
fn provider_string_lines_remain_compatible_during_ml_a() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_material_request(ToolSurfaceKind::MaterialPreview),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("material diagnostics: none (structured)"));
    assert!(text.contains("No active material preview product"));
    assert!(text.contains("No material diagnostics"));
}

fn test_material_preview_product(
    asset_id: asset::AssetId,
) -> crate::material_lab::EditorMaterialPreviewProduct {
    let product = material_graph::FormedMaterialProduct::new(
        material_graph::MaterialProductId::new(30),
        material_graph::MaterialGraphDocumentId::new(31),
        material_graph::MaterialOutputTarget::RenderMaterial,
        material_graph::MaterialCacheKey::new("material-preview-cache"),
    );
    crate::material_lab::EditorMaterialPreviewProduct::new(
        asset_id,
        asset_source_id(22),
        asset_artifact_id(32),
        ArtifactCacheKey::new("artifact-cache"),
        product,
        crate::material_lab::MaterialRendererParameterProfile::RenderMaterial,
        asset_artifact_id(33),
        ArtifactCacheKey::new("shader-cache"),
        ".runenwerk/artifacts/material.wgsl",
        "material-shader",
        asset_artifact_id(34),
        ArtifactCacheKey::new("scene-shader-cache"),
        ".runenwerk/artifacts/scene-material.wgsl",
        "scene-material-shader",
        [],
    )
}

fn test_preview_scene_product() -> crate::material_lab::PreviewSceneProduct {
    let shader = crate::material_lab::PreviewSceneShaderProductRef::new(
        "provider-shader-artifact",
        ArtifactCacheKey::new("provider-shader-cache"),
        "provider-shader-identity",
        ".runenwerk/artifacts/provider-scene-table.wgsl",
        "provider-table",
        "provider-layout",
    );
    crate::material_lab::PreviewSceneProduct::new(
        crate::material_lab::PreviewSceneProductMode::SceneMaterialTable,
        editor_viewport::ExpressionProductId(10030),
        material_graph::MaterialProductId::new(30),
        ArtifactCacheKey::new("provider-active-material-cache"),
        "provider-table",
        "provider-layout",
        shader,
        [crate::material_lab::PreviewSceneMaterialSlot::new(
            0,
            "provider-slot",
            material_graph::MaterialProductId::new(30),
            ArtifactCacheKey::new("provider-material-cache"),
            "provider-scene-shader",
            [crate::material_lab::PreviewSceneResourceSlotMapping::new(
                0, 0,
            )],
        )],
        [crate::material_lab::PreviewSceneResourceSlot::new(
            0,
            "provider-texture-product",
            "Texture2D",
            "2d",
            "rgba8_unorm_srgb|sampled",
            "linear_repeat",
            "provider-texture-artifact",
            ArtifactCacheKey::new("provider-texture-cache"),
        )],
    )
}

fn material_texture_binding_source_document() -> material_graph::MaterialGraphDocument {
    material_graph::MaterialGraphDocument::new(
        material_graph::MaterialGraphDocumentId::new(2401),
        "material-texture-binding",
        GraphDefinition::new(
            GraphId::new(2401),
            "material-texture-binding",
            CyclePolicy::RejectDirectedCycles,
            [NodeDefinition::new(
                NodeId::new(24),
                "texture.sample_2d",
                [],
            )],
            [],
        ),
        material_graph::MaterialOutputTarget::RenderMaterial,
    )
}

#[test]
fn material_provider_actions_map_to_epoch_carrying_shell_commands() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let asset_id = asset_id(101);
    let request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);
    let dispatch_context = SurfaceProviderDispatchContext {
        projection_epoch: 77,
        _marker: std::marker::PhantomData,
    };

    let proposal = registry
        .map_action(
            &dispatch_context,
            &request,
            MATERIAL_GRAPH_CANVAS_PROVIDER_ID,
            SurfaceLocalAction::Material(MaterialSurfaceAction::BuildMaterialPreview { asset_id }),
        )
        .expect("material action should map")
        .expect("material action should produce shell command");

    assert!(matches!(
        proposal,
        SurfaceCommandProposal::Shell(ShellCommand::BuildMaterialPreview {
            asset_id: mapped,
            projection_epoch: 77,
        }) if mapped == asset_id
    ));
}

#[test]
fn procgen_providers_resolve_directly_with_visible_preview_lines() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    assert!(!m6_workspace::is_m6_workspace_surface(
        ToolSurfaceKind::ProcgenGraphCanvas
    ));
    assert!(!m6_workspace::is_m6_workspace_surface(
        ToolSurfaceKind::ProcgenPreview
    ));

    let graph_frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_procgen_request(ToolSurfaceKind::ProcgenGraphCanvas),
        &Default::default(),
    );
    assert_eq!(
        graph_frame.availability,
        SurfaceProviderAvailability::Available
    );
    assert_eq!(
        graph_frame.provider_id,
        Some(PROCGEN_GRAPH_CANVAS_PROVIDER_ID)
    );
    assert!(
        provider_frame_text(&graph_frame)
            .contains("domain-backed Phase 6D bake-capable CPU preview")
    );

    let preview_frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_procgen_request(ToolSurfaceKind::ProcgenPreview),
        &Default::default(),
    );
    assert_eq!(
        preview_frame.availability,
        SurfaceProviderAvailability::Available
    );
    assert_eq!(preview_frame.provider_id, Some(PROCGEN_PREVIEW_PROVIDER_ID));
    let text = provider_frame_text(&preview_frame);
    assert!(text.contains("concrete terrain/material CPU preview"));
    assert!(text.contains("changed_regions=2"));
    assert!(text.contains("procgen field preview products: 0"));
}

#[test]
fn texture_viewer_rejects_descriptor_only_completion() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(60);
    let artifact_id = asset_artifact_id(61);
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "albedo",
            "Albedo",
            AssetKind::Texture2D,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(AssetArtifactDescriptor::new(
            artifact_id,
            asset_id,
            AssetKind::Texture2D,
            texture_payload(texture_descriptor(
                42,
                TextureDimension::Texture2D,
                TextureExtent::new(512, 512, 1),
            )),
            ArtifactCacheKey::new("texture-42"),
        ));
    let request = m6_texture_request(ToolSurfaceKind::TextureViewer);

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(frame.provider_id, Some(TEXTURE_VIEWER_PROVIDER_ID));
    let text = provider_frame_text(&frame);
    assert!(text.contains("preview descriptor: product=42"));
    assert!(text.contains("MissingArtifactUri"));
    assert!(
        !frame_has_product_surface(&frame),
        "descriptor-only texture data must not emit product-surface proof"
    );
    assert!(!frame.routes.is_empty());
}

#[test]
fn texture_viewer_gpu_preview_uses_catalog_residency() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(62);
    let artifact_id = asset_artifact_id(63);
    let bytes = build_rgba8_ktx2(2, 2, 1, [12, 34, 56, 255], [12, 34, 56, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-texture-viewer-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test texture should write");
    let path_string = path.to_string_lossy().to_string();
    let descriptor = texture_descriptor_with_byte_length(
        44,
        TextureDimension::Texture2D,
        TextureExtent::new(2, 2, 1),
        bytes.len() as u64,
    );
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "albedo_gpu",
            "Albedo GPU",
            AssetKind::Texture2D,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(
            AssetArtifactDescriptor::new(
                artifact_id,
                asset_id,
                AssetKind::Texture2D,
                texture_payload_with_uri(descriptor, path_string.clone()),
                ArtifactCacheKey::new("texture-44"),
            )
            .with_artifact_path(path_string.clone()),
        );
    let request = m6_texture_request(ToolSurfaceKind::TextureViewer);

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(frame.provider_id, Some(TEXTURE_VIEWER_PROVIDER_ID));
    let text = provider_frame_text(&frame);
    assert!(frame_has_product_surface(&frame));
    assert!(text.contains("texture viewer: rendered GPU product-surface preview"));
    assert!(text.contains("residency class: engine.material_ktx2_upload"));
    assert!(text.contains("bind group identity: engine_ui_product_surface_bind_group"));
    assert!(text.contains("artifact URI:"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn texture_viewer_gpu_proof_uses_provider_product_surface_path() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(162);
    let artifact_id = asset_artifact_id(163);
    let bytes = build_rgba8_ktx2(2, 2, 1, [108, 210, 162, 255], [108, 210, 162, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-texture-viewer-provider-proof-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test texture should write");
    let path_string = path.to_string_lossy().to_string();
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "wr028_texture_viewer_provider_proof",
            "WR-028 Texture Viewer Provider Proof",
            AssetKind::Texture2D,
        ));
    app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    artifact_id,
                    asset_id,
                    AssetKind::Texture2D,
                    texture_payload_with_uri(
                        texture_descriptor_with_byte_length(
                            9028,
                            TextureDimension::Texture2D,
                            TextureExtent::new(2, 2, 1),
                            bytes.len() as u64,
                        ),
                        "docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/artifacts/fixtures/wr028-texture-viewer-2d.ktx2",
                    ),
                    ArtifactCacheKey::new("wr028-texture-viewer-provider-proof"),
                )
                .with_artifact_path(path_string.clone()),
            );
    app.asset_catalog_runtime_mut().select_asset(Some(asset_id));

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_texture_request(ToolSurfaceKind::TextureViewer),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(frame.provider_id, Some(TEXTURE_VIEWER_PROVIDER_ID));
    assert!(frame_has_product_surface(&frame));
    assert!(text.contains("preview descriptor: product=9028"));
    assert!(text.contains(
        "preview target: runenwerk.editor.texture_preview:texture2d.product9028.mip0.slice0.all"
    ));
    assert!(text.contains("residency class: engine.material_ktx2_upload"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn volume_texture_viewer_gpu_proof_uses_provider_product_surface_path() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(164);
    let artifact_id = asset_artifact_id(165);
    let bytes = build_rgba8_ktx2(2, 2, 2, [153, 103, 173, 255], [153, 103, 173, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-volume-viewer-provider-proof-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test volume texture should write");
    let path_string = path.to_string_lossy().to_string();
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "wr028_volume_viewer_provider_proof",
            "WR-028 Volume Viewer Provider Proof",
            AssetKind::Texture3DVolume,
        ));
    app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    artifact_id,
                    asset_id,
                    AssetKind::Texture3DVolume,
                    generated_texture_payload_with_uri(
                        texture_descriptor_with_byte_length(
                            9029,
                            TextureDimension::Texture3DVolume,
                            TextureExtent::new(2, 2, 2),
                            bytes.len() as u64,
                        ),
                        "docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/artifacts/fixtures/wr028-volume-texture-viewer-3d.ktx2",
                    ),
                    ArtifactCacheKey::new("wr028-volume-viewer-provider-proof"),
                )
                .with_artifact_path(path_string.clone()),
            );
    app.asset_catalog_runtime_mut().select_asset(Some(asset_id));

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_texture_request(ToolSurfaceKind::VolumeTextureViewer),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(frame.provider_id, Some(VOLUME_TEXTURE_VIEWER_PROVIDER_ID));
    assert!(frame_has_product_surface(&frame));
    assert!(text.contains("preview descriptor: product=9029"));
    assert!(text.contains(
        "preview target: runenwerk.editor.texture_preview:texture3d.product9029.mip0.slice0.all"
    ));
    assert!(text.contains("residency class: engine.material_ktx2_upload"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn texture_preview_records_bind_group_identity() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(72);
    let artifact_id = asset_artifact_id(73);
    let bytes = build_rgba8_ktx2(2, 2, 1, [21, 43, 65, 255], [21, 43, 65, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-texture-bind-group-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test texture should write");
    let path_string = path.to_string_lossy().to_string();
    let descriptor = texture_descriptor_with_byte_length(
        78,
        TextureDimension::Texture2D,
        TextureExtent::new(2, 2, 1),
        bytes.len() as u64,
    );
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "bind_group_texture",
            "Bind Group Texture",
            AssetKind::Texture2D,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(
            AssetArtifactDescriptor::new(
                artifact_id,
                asset_id,
                AssetKind::Texture2D,
                texture_payload_with_uri(descriptor, path_string.clone()),
                ArtifactCacheKey::new("texture-78"),
            )
            .with_artifact_path(path_string.clone()),
        );

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_texture_request(ToolSurfaceKind::TextureViewer),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(frame_has_product_surface(&frame));
    assert!(text.contains("sampler identity: min="));
    assert!(text.contains("bind group identity: engine_ui_product_surface_bind_group"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn texture_preview_proof_metadata_has_concrete_descriptor_hash() {
    let mut catalog = asset::AssetCatalog::new();
    let asset_id = asset_id(74);
    let artifact_id = asset_artifact_id(75);
    let bytes = build_rgba8_ktx2(2, 2, 1, [31, 47, 59, 255], [31, 47, 59, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-texture-proof-hash-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test texture should write");
    let path_string = path.to_string_lossy().to_string();
    let descriptor = texture_descriptor_with_byte_length(
        86,
        TextureDimension::Texture2D,
        TextureExtent::new(2, 2, 1),
        bytes.len() as u64,
    );
    let descriptor_hash = descriptor.descriptor_hash().to_string();
    catalog.insert_asset_record(AssetRecord::new(
        asset_id,
        "proof_hash_texture",
        "Proof Hash Texture",
        AssetKind::Texture2D,
    ));
    catalog.insert_artifact(
        AssetArtifactDescriptor::new(
            artifact_id,
            asset_id,
            AssetKind::Texture2D,
            texture_payload_with_uri(descriptor, path_string.clone()),
            ArtifactCacheKey::new("texture-86"),
        )
        .with_artifact_path(path_string.clone()),
    );

    let prepared = crate::texture_preview::prepare_texture_preview(
        &catalog,
        Some(asset_id),
        &crate::texture_preview::TexturePreviewRuntime::default(),
        TextureViewerSurfaceKind::Texture2D,
    )
    .expect("texture preview proof should prepare");

    assert_eq!(prepared.proof.texture_product_id, 86);
    assert_eq!(prepared.proof.descriptor_hash, descriptor_hash);
    assert_eq!(prepared.proof.artifact_uri, path_string);
    assert!(!prepared.proof.descriptor_hash.contains("TextureDescriptor"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn texture_preview_proof_metadata_has_concrete_bind_group_identity() {
    let mut catalog = asset::AssetCatalog::new();
    let asset_id = asset_id(76);
    let artifact_id = asset_artifact_id(77);
    let bytes = build_rgba8_ktx2(2, 2, 1, [41, 67, 83, 255], [41, 67, 83, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-texture-proof-bind-group-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test texture should write");
    let path_string = path.to_string_lossy().to_string();
    catalog.insert_asset_record(AssetRecord::new(
        asset_id,
        "proof_bind_group_texture",
        "Proof Bind Group Texture",
        AssetKind::Texture2D,
    ));
    catalog.insert_artifact(
        AssetArtifactDescriptor::new(
            artifact_id,
            asset_id,
            AssetKind::Texture2D,
            texture_payload_with_uri(
                texture_descriptor_with_byte_length(
                    87,
                    TextureDimension::Texture2D,
                    TextureExtent::new(2, 2, 1),
                    bytes.len() as u64,
                ),
                path_string.clone(),
            ),
            ArtifactCacheKey::new("texture-87"),
        )
        .with_artifact_path(path_string.clone()),
    );

    let prepared = crate::texture_preview::prepare_texture_preview(
        &catalog,
        Some(asset_id),
        &crate::texture_preview::TexturePreviewRuntime::default(),
        TextureViewerSurfaceKind::Texture2D,
    )
    .expect("texture preview proof should prepare");

    assert_eq!(
        prepared.proof.bind_group_identity,
        "engine_ui_product_surface_bind_group:runenwerk.editor.texture_preview:texture2d.product87.mip0.slice0.all"
    );
    assert_eq!(
        prepared.proof.target_key.label(),
        "runenwerk.editor.texture_preview:texture2d.product87.mip0.slice0.all"
    );
    assert_eq!(
        prepared.proof.residency_class,
        "engine.material_ktx2_upload"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn wr028_proof_manifest_rejects_texture_metadata_placeholders() {
    let manifest = include_str!(
        "../../../../../docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron"
    );

    for forbidden in [
        "descriptor_hash_source",
        "<dynamic texture preview target>",
        "manual GPU smoke temp KTX2 artifact",
        "verified inside viewport_gpu_truth_smoke readback assertions",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "WR-028 proof manifest still contains placeholder texture proof metadata: {forbidden}"
        );
    }
    assert!(manifest.contains("texture_product_id: 9028"));
    assert!(manifest.contains("descriptor_hash: \""));
    assert!(manifest.contains("bind_group_identity: \"engine_ui_product_surface_bind_group:"));
    assert!(manifest.contains("residency_class: \"engine.material_ktx2_upload\""));
}

#[test]
fn wr028_proof_manifest_rejects_temp_artifact_paths() {
    let manifest = include_str!(
        "../../../../../docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron"
    );

    for forbidden in [
        "AppData/Local/Temp",
        "std::env::temp_dir",
        "temp://runenwerk",
        "runenwerk-wr021-gpu-proof",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "WR-028 proof manifest must not depend on temp-only proof paths: {forbidden}"
        );
    }
}

#[test]
fn wr028_proof_manifest_links_durable_texture_preview_artifacts() {
    let manifest = include_str!(
        "../../../../../docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron"
    );

    for required in [
        "artifacts/fixtures/wr028-texture-viewer-2d.ktx2",
        "artifacts/fixtures/wr028-volume-texture-viewer-3d.ktx2",
        "artifacts/metadata/wr028-texture2d-proof.ron",
        "artifacts/metadata/wr028-texture3d-proof.ron",
        "texture_viewer_provider_product_surface_path: true",
        "volume_texture_viewer_provider_product_surface_path: true",
    ] {
        assert!(
            manifest.contains(required),
            "WR-028 proof manifest must link durable texture viewer proof artifact: {required}"
        );
    }
}

#[test]
fn texture_preview_records_concrete_catalog_metadata() {
    let manifest = include_str!(
        "../../../../../docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron"
    );

    for required in [
        "texture_product_id: 9028",
        "texture_product_id: 9029",
        "artifact_id: 29028",
        "artifact_id: 29029",
        "descriptor_hash: \"70726f647563745f69643d343a39303238",
        "descriptor_hash: \"70726f647563745f69643d343a39303239",
        "artifact_uri: \"docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/artifacts/fixtures/wr028-texture-viewer-2d.ktx2\"",
        "artifact_uri: \"docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/artifacts/fixtures/wr028-volume-texture-viewer-3d.ktx2\"",
        "preview_target_key: \"runenwerk.editor.texture_preview:texture2d.product9028.mip0.slice0.all\"",
        "preview_target_key: \"runenwerk.editor.texture_preview:texture3d.product9029.mip0.slice0.all\"",
        "capture_hash: \"blake3:483a40eff929a29193ca839ec96a069cc666764b2065c31a4986285bdec97eab\"",
        "capture_hash: \"blake3:917eb702a699db47d62821de87e240a75907e8470e8d5547966e983adcfe8dde\"",
    ] {
        assert!(
            manifest.contains(required),
            "WR-028 proof manifest must record concrete catalog texture metadata: {required}"
        );
    }
}

#[test]
fn texture_viewer_gpu_proof_rejects_direct_temp_resource_bypass() {
    let manifest = include_str!(
        "../../../../../docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron"
    );

    for forbidden in [
        "ResolvedMaterialResource",
        "gpu-truth-texture-70",
        "gpu-truth-texture-71",
        "wr021-material-texture-2d.ktx2",
        "wr021-material-texture-3d.ktx2",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "WR-028 texture viewer proof must not cite the direct material resource bypass: {forbidden}"
        );
    }
}

#[test]
fn texture_preview_uses_selected_catalog_texture_product() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let unselected_asset_id = asset_id(82);
    let selected_asset_id = asset_id(84);
    let bytes = build_rgba8_ktx2(2, 2, 1, [3, 5, 7, 255], [3, 5, 7, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-selected-texture-preview-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test texture should write");
    let path_string = path.to_string_lossy().to_string();
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            unselected_asset_id,
            "unselected_texture",
            "Unselected Texture",
            AssetKind::Texture2D,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(83),
                unselected_asset_id,
                AssetKind::Texture2D,
                texture_payload_with_uri(
                    texture_descriptor_with_byte_length(
                        79,
                        TextureDimension::Texture2D,
                        TextureExtent::new(2, 2, 1),
                        bytes.len() as u64,
                    ),
                    path_string.clone(),
                ),
                ArtifactCacheKey::new("texture-79"),
            )
            .with_artifact_path(path_string.clone()),
        );
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            selected_asset_id,
            "selected_texture",
            "Selected Texture",
            AssetKind::Texture2D,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(85),
                selected_asset_id,
                AssetKind::Texture2D,
                texture_payload_with_uri(
                    texture_descriptor_with_byte_length(
                        80,
                        TextureDimension::Texture2D,
                        TextureExtent::new(2, 2, 1),
                        bytes.len() as u64,
                    ),
                    path_string.clone(),
                ),
                ArtifactCacheKey::new("texture-80"),
            )
            .with_artifact_path(path_string.clone()),
        );
    app.asset_catalog_runtime_mut()
        .select_asset(Some(selected_asset_id));

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_texture_request(ToolSurfaceKind::TextureViewer),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(frame_has_product_surface(&frame));
    assert!(text.contains("preview descriptor: product=80"));
    assert!(!text.contains("preview descriptor: product=79"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn texture_preview_invalid_selected_asset_does_not_fallback() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let valid_asset_id = asset_id(86);
    let selected_asset_id = asset_id(88);
    let bytes = build_rgba8_ktx2(2, 2, 1, [13, 17, 19, 255], [13, 17, 19, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-invalid-selected-texture-preview-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test texture should write");
    let path_string = path.to_string_lossy().to_string();
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            valid_asset_id,
            "valid_fallback_texture",
            "Valid Fallback Texture",
            AssetKind::Texture2D,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(87),
                valid_asset_id,
                AssetKind::Texture2D,
                texture_payload_with_uri(
                    texture_descriptor_with_byte_length(
                        81,
                        TextureDimension::Texture2D,
                        TextureExtent::new(2, 2, 1),
                        bytes.len() as u64,
                    ),
                    path_string.clone(),
                ),
                ArtifactCacheKey::new("texture-81"),
            )
            .with_artifact_path(path_string.clone()),
        );
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            selected_asset_id,
            "invalid_selected_texture",
            "Invalid Selected Texture",
            AssetKind::Texture2D,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(AssetArtifactDescriptor::new(
            asset_artifact_id(89),
            selected_asset_id,
            AssetKind::Texture2D,
            texture_payload(texture_descriptor(
                82,
                TextureDimension::Texture2D,
                TextureExtent::new(2, 2, 1),
            )),
            ArtifactCacheKey::new("texture-82"),
        ));
    app.asset_catalog_runtime_mut()
        .select_asset(Some(selected_asset_id));

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_texture_request(ToolSurfaceKind::TextureViewer),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("preview descriptor: product=82"));
    assert!(text.contains("MissingArtifactUri"));
    assert!(!text.contains("preview descriptor: product=81"));
    assert!(!frame_has_product_surface(&frame));
    let _ = std::fs::remove_file(path);
}

#[test]
fn texture_preview_selected_incompatible_asset_does_not_fallback() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let valid_asset_id = asset_id(86);
    let selected_asset_id = asset_id(88);
    let bytes = build_rgba8_ktx2(2, 2, 1, [13, 17, 19, 255], [13, 17, 19, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-incompatible-selected-texture-preview-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test texture should write");
    let path_string = path.to_string_lossy().to_string();
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            valid_asset_id,
            "valid_fallback_texture",
            "Valid Fallback Texture",
            AssetKind::Texture2D,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(87),
                valid_asset_id,
                AssetKind::Texture2D,
                texture_payload_with_uri(
                    texture_descriptor_with_byte_length(
                        81,
                        TextureDimension::Texture2D,
                        TextureExtent::new(2, 2, 1),
                        bytes.len() as u64,
                    ),
                    path_string.clone(),
                ),
                ArtifactCacheKey::new("texture-81"),
            )
            .with_artifact_path(path_string.clone()),
        );
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            selected_asset_id,
            "selected_volume_texture",
            "Selected Volume Texture",
            AssetKind::Texture3DVolume,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(AssetArtifactDescriptor::new(
            asset_artifact_id(89),
            selected_asset_id,
            AssetKind::Texture3DVolume,
            texture_payload(texture_descriptor(
                82,
                TextureDimension::Texture3DVolume,
                TextureExtent::new(2, 2, 2),
            )),
            ArtifactCacheKey::new("texture-82"),
        ));
    app.asset_catalog_runtime_mut()
        .select_asset(Some(selected_asset_id));

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_texture_request(ToolSurfaceKind::TextureViewer),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("MissingTextureProduct"));
    assert!(text.contains("selected asset 88 has no compatible Texture2D texture product"));
    assert!(!text.contains("preview descriptor: product=81"));
    assert!(!text.contains("preview descriptor: product=82"));
    assert!(!frame_has_product_surface(&frame));
    let _ = std::fs::remove_file(path);
}

#[test]
fn texture_preview_reports_missing_artifact_uri() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(64);
    let artifact_id = asset_artifact_id(65);
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "missing_uri",
            "Missing URI",
            AssetKind::Texture2D,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(AssetArtifactDescriptor::new(
            artifact_id,
            asset_id,
            AssetKind::Texture2D,
            texture_payload(texture_descriptor(
                45,
                TextureDimension::Texture2D,
                TextureExtent::new(2, 2, 1),
            )),
            ArtifactCacheKey::new("texture-45"),
        ));

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_texture_request(ToolSurfaceKind::TextureViewer),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("MissingArtifactUri"));
    assert!(!frame_has_product_surface(&frame));
}

#[test]
fn texture_preview_reports_invalid_descriptor_hash() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(66);
    let artifact_id = asset_artifact_id(67);
    let descriptor =
        texture_descriptor(46, TextureDimension::Texture2D, TextureExtent::new(2, 2, 1));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "bad_hash",
            "Bad Hash",
            AssetKind::Texture2D,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(AssetArtifactDescriptor::new(
            artifact_id,
            asset_id,
            AssetKind::Texture2D,
            texture_payload_with_hash(
                descriptor,
                "not-the-descriptor-hash",
                Some("mem://bad.ktx2".to_string()),
            ),
            ArtifactCacheKey::new("texture-46"),
        ));

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_texture_request(ToolSurfaceKind::TextureViewer),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("InvalidDescriptorHash"));
    assert!(!frame_has_product_surface(&frame));
}

#[test]
fn asset_browser_projects_typed_rows_and_epoch_routes() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(80);
    let source_id = asset_source_id(81);
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(
            AssetRecord::new(asset_id, "field", "Field", AssetKind::SdfGraph)
                .with_primary_source(source_id),
        );
    app.asset_catalog_runtime_mut().catalog_mut().insert_source(
        AssetSourceDescriptor::new(source_id, asset_id, AssetKind::SdfGraph, "assets/field.ron")
            .with_hash(SourceHash::new("sha256", "abc")),
    );
    let request = asset_request(ToolSurfaceKind::AssetBrowser);

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(frame.provider_id, Some(ASSET_BROWSER_PROVIDER_ID));
    let text = provider_frame_text(&frame);
    assert!(text.contains("asset 80 Field"));
    assert!(!frame.routes.is_empty());

    let proposal = registry
        .map_action(
            &SurfaceProviderDispatchContext {
                projection_epoch: 55,
                _marker: std::marker::PhantomData,
            },
            &request,
            ASSET_BROWSER_PROVIDER_ID,
            SurfaceLocalAction::Asset(AssetSurfaceAction::SelectAsset { asset_id }),
        )
        .expect("asset action should map")
        .expect("asset action should produce shell command");
    assert!(matches!(
        proposal,
        SurfaceCommandProposal::Shell(ShellCommand::SelectAsset {
            asset_id: mapped,
            projection_epoch: 55,
        }) if mapped == asset_id
    ));
}

#[test]
fn import_inspector_surfaces_prior_valid_and_routes_reimport() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(90);
    let artifact_id = asset_artifact_id(91);
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "field",
            "Field",
            AssetKind::FormedFieldProduct,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(
            AssetArtifactDescriptor::new(
                artifact_id,
                asset_id,
                AssetKind::FormedFieldProduct,
                ArtifactPayloadKind::FormedFieldProduct {
                    product_id: "field".to_string(),
                },
                ArtifactCacheKey::new("field"),
            )
            .with_artifact_path(".runenwerk/artifacts/field.ron")
            .with_validity(asset::ArtifactValidity::FailedPreserved)
            .with_diagnostic(asset::AssetDiagnosticRecord::error(
                asset::AssetDiagnosticCode::SourceMissing,
                "source missing",
            )),
        );
    app.asset_catalog_runtime_mut().select_asset(Some(asset_id));
    app.asset_catalog_runtime_mut().mark_asset_dirty(asset_id);
    let request = asset_request(ToolSurfaceKind::ImportInspector);

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );

    assert_eq!(frame.provider_id, Some(IMPORT_INSPECTOR_PROVIDER_ID));
    let text = provider_frame_text(&frame);
    assert!(text.contains("preserved artifact 91"));
    assert!(!frame.routes.is_empty());
}

#[test]
fn volume_texture_viewer_slice_mip_channel_controls_affect_preview_request() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(70);
    let artifact_id = asset_artifact_id(71);
    let bytes = build_rgba8_ktx2(2, 2, 2, [1, 2, 3, 255], [8, 9, 10, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-volume-texture-viewer-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test volume texture should write");
    let path_string = path.to_string_lossy().to_string();
    let descriptor = texture_descriptor_with_byte_length(
        77,
        TextureDimension::Texture3DVolume,
        TextureExtent::new(2, 2, 2),
        bytes.len() as u64,
    );
    app.apply_texture_surface_action(TextureSurfaceAction::SetPreviewSlice {
        surface: TextureViewerSurfaceKind::VolumeTexture3D,
        slice_index: 1,
    });
    app.apply_texture_surface_action(TextureSurfaceAction::SetPreviewChannel {
        surface: TextureViewerSurfaceKind::VolumeTexture3D,
        channel: TexturePreviewChannelSelection::G,
    });
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "density_volume",
            "Density Volume",
            AssetKind::Texture3DVolume,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(
            AssetArtifactDescriptor::new(
                artifact_id,
                asset_id,
                AssetKind::Texture3DVolume,
                texture_payload_with_uri(descriptor, path_string.clone()),
                ArtifactCacheKey::new("volume-77"),
            )
            .with_artifact_path(path_string.clone()),
        );
    let request = m6_texture_request(ToolSurfaceKind::VolumeTextureViewer);

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(frame.provider_id, Some(VOLUME_TEXTURE_VIEWER_PROVIDER_ID));
    let text = provider_frame_text(&frame);
    assert!(text.contains("preview descriptor: product=77"));
    assert!(text.contains("selected slice: 1"));
    assert!(text.contains("selected channel: g"));
    assert!(text.contains(
        "preview target: runenwerk.editor.texture_preview:texture3d.product77.mip0.slice1.g"
    ));
    assert!(frame_has_product_surface(&frame));
    assert!(!frame.routes.is_empty());
    let _ = std::fs::remove_file(path);
}

#[test]
fn volume_texture_viewer_slice_changes_preview_request() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(92);
    let bytes = build_rgba8_ktx2(2, 2, 2, [1, 2, 3, 255], [8, 9, 10, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-volume-slice-preview-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test volume texture should write");
    let path_string = path.to_string_lossy().to_string();
    app.apply_texture_surface_action(TextureSurfaceAction::SetPreviewSlice {
        surface: TextureViewerSurfaceKind::VolumeTexture3D,
        slice_index: 1,
    });
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "volume_slice",
            "Volume Slice",
            AssetKind::Texture3DVolume,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(93),
                asset_id,
                AssetKind::Texture3DVolume,
                texture_payload_with_uri(
                    texture_descriptor_with_byte_length(
                        83,
                        TextureDimension::Texture3DVolume,
                        TextureExtent::new(2, 2, 2),
                        bytes.len() as u64,
                    ),
                    path_string.clone(),
                ),
                ArtifactCacheKey::new("volume-83"),
            )
            .with_artifact_path(path_string.clone()),
        );

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_texture_request(ToolSurfaceKind::VolumeTextureViewer),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("selected slice: 1"));
    assert!(text.contains("texture3d.product83.mip0.slice1.all"));
    assert!(frame_has_product_surface(&frame));
    let _ = std::fs::remove_file(path);
}

#[test]
fn volume_texture_viewer_channel_changes_preview_request() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(94);
    let bytes = build_rgba8_ktx2(2, 2, 2, [1, 2, 3, 255], [8, 9, 10, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-volume-channel-preview-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test volume texture should write");
    let path_string = path.to_string_lossy().to_string();
    app.apply_texture_surface_action(TextureSurfaceAction::SetPreviewChannel {
        surface: TextureViewerSurfaceKind::VolumeTexture3D,
        channel: TexturePreviewChannelSelection::B,
    });
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "volume_channel",
            "Volume Channel",
            AssetKind::Texture3DVolume,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(95),
                asset_id,
                AssetKind::Texture3DVolume,
                texture_payload_with_uri(
                    texture_descriptor_with_byte_length(
                        84,
                        TextureDimension::Texture3DVolume,
                        TextureExtent::new(2, 2, 2),
                        bytes.len() as u64,
                    ),
                    path_string.clone(),
                ),
                ArtifactCacheKey::new("volume-84"),
            )
            .with_artifact_path(path_string.clone()),
        );

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_texture_request(ToolSurfaceKind::VolumeTextureViewer),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("selected channel: b"));
    assert!(text.contains("texture3d.product84.mip0.slice0.b"));
    assert!(frame_has_product_surface(&frame));
    let _ = std::fs::remove_file(path);
}

#[test]
fn volume_texture_viewer_mip_request_is_diagnosed_when_unresident() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let asset_id = asset_id(96);
    let bytes = build_rgba8_ktx2(2, 2, 2, [1, 2, 3, 255], [8, 9, 10, 255]);
    let path = std::env::temp_dir().join(format!(
        "runenwerk-volume-mip-preview-{}.ktx2",
        std::process::id()
    ));
    std::fs::write(&path, &bytes).expect("test volume texture should write");
    let path_string = path.to_string_lossy().to_string();
    app.apply_texture_surface_action(TextureSurfaceAction::SetPreviewMip {
        surface: TextureViewerSurfaceKind::VolumeTexture3D,
        mip_level: 1,
    });
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_asset_record(AssetRecord::new(
            asset_id,
            "volume_mip",
            "Volume Mip",
            AssetKind::Texture3DVolume,
        ));
    app.asset_catalog_runtime_mut()
        .catalog_mut()
        .insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(97),
                asset_id,
                AssetKind::Texture3DVolume,
                texture_payload_with_uri(
                    texture_descriptor_with_mip_count_and_byte_length(
                        85,
                        TextureDimension::Texture3DVolume,
                        TextureExtent::new(2, 2, 2),
                        2,
                        bytes.len() as u64,
                    ),
                    path_string.clone(),
                ),
                ArtifactCacheKey::new("volume-85"),
            )
            .with_artifact_path(path_string.clone()),
        );

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &m6_texture_request(ToolSurfaceKind::VolumeTextureViewer),
        &Default::default(),
    );

    let text = provider_frame_text(&frame);
    assert!(text.contains("preview descriptor: product=85 mip=1"));
    assert!(text.contains("FailedUpload"));
    assert!(text.contains("selected mip 1 is unsupported"));
    assert!(!frame_has_product_surface(&frame));
    let _ = std::fs::remove_file(path);
}

#[test]
fn sdf_field_layer_stack_provider_resolves_before_m6_fallback_with_routes() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let mut app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let layer_id = app.sdf_operation_workspace().document().layers()[0].id;
    app.sdf_operation_workspace_mut()
        .apply_command(
            editor_scene::SdfOperationCommandIntent::AddPrimitiveOperation {
                layer_id,
                display_name: "Sphere Add".to_string(),
                primitive: editor_scene::SdfPrimitiveSpec::new(
                    editor_scene::SdfPrimitiveKind::Sphere,
                    editor_scene::SdfBooleanIntent::Add,
                ),
                material_channel: 2,
            },
        )
        .expect("SDF command should apply");
    let request = m6_sdf_request(ToolSurfaceKind::FieldLayerStack, DocumentKind::SdfGraph);

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(frame.provider_id, Some(FIELD_LAYER_STACK_PROVIDER_ID));
    let text = provider_frame_text(&frame);
    assert!(text.contains("lowered world_ops records: 1"));
    assert!(text.contains("commit eligible: true"));
    assert!(!frame.routes.is_empty());
}

#[test]
fn field_layer_stack_actions_map_to_sdf_domain_proposals() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let layer_id = app.sdf_operation_workspace().document().layers()[0].id;
    let request = m6_sdf_request(ToolSurfaceKind::FieldLayerStack, DocumentKind::SdfGraph);
    let dispatch_context = SurfaceProviderDispatchContext {
        projection_epoch: 44,
        _marker: std::marker::PhantomData,
    };

    let proposal = registry
        .map_action(
            &dispatch_context,
            &request,
            FIELD_LAYER_STACK_PROVIDER_ID,
            SurfaceLocalAction::SdfOperation(
                editor_shell::SdfOperationSurfaceAction::ApplyCommand {
                    intent: editor_scene::SdfOperationCommandIntent::SetLayerEnabled {
                        layer_id,
                        enabled: false,
                    },
                },
            ),
        )
        .expect("provider should map action")
        .expect("action should produce proposal");

    match proposal {
        SurfaceCommandProposal::EditorDomain(proposal) => {
            assert_eq!(proposal.projection_epoch, 44);
            assert!(matches!(
                proposal.mutation,
                EditorDomainMutation::SdfOperation(
                    editor_shell::SdfOperationDomainMutation::ApplyCommand { .. }
                )
            ));
        }
        _ => panic!("SDF field action should map to an editor domain proposal"),
    }
}

#[test]
fn sdf_graph_canvas_provider_is_descriptor_first_and_command_backed() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let request = m6_sdf_request(ToolSurfaceKind::SdfGraphCanvas, DocumentKind::SdfGraph);

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    assert_eq!(frame.provider_id, Some(SDF_GRAPH_CANVAS_PROVIDER_ID));
    assert!(provider_frame_text(&frame).contains("canvas/session state is not SDF graph truth"));
    assert!(provider_frame_text(&frame).contains("graph can lower: false"));
    assert!(!frame.routes.is_empty());
}

#[test]
fn sdf_surfaces_fail_closed_for_incompatible_document_kind() {
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    let theme = ThemeTokens::default();
    let request = m6_sdf_request(
        ToolSurfaceKind::FieldLayerStack,
        DocumentKind::MaterialGraph,
    );

    let frame = registry.resolve_frame(
        &context(&app, &shell_state, &theme),
        &request,
        &Default::default(),
    );

    assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
    assert!(frame.routes.is_empty());
}
