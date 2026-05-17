use super::*;
use crate::plugins::render::{RenderFlowId, RenderPassId};

#[derive(Debug, Clone)]
pub struct EncodedPassEvidence {
    pub dispatch_workgroups: Option<[u32; 3]>,
    pub shader_id: String,
    pub shader_revision: u64,
    pub fallback_used: bool,
    pub pipeline_key: Option<FlowPassPipelineKey>,
}

#[derive(Debug, Clone)]
pub struct EncodedPipelinePass {
    pub dispatch_workgroups: Option<[u32; 3]>,
    pub shader_id: String,
    pub shader_revision: u64,
    pub fallback_used: bool,
    pub pipeline_key: FlowPassPipelineKey,
}

#[derive(Debug, Default)]
pub struct PassResourceTruth {
    pub render_targets: Vec<String>,
    pub sampled_textures: Vec<String>,
    pub storage_textures: Vec<String>,
    pub depth_targets: Vec<String>,
    pub capture_points_available: Vec<RenderCapturePointIdentity>,
}

pub fn collect_pass_resource_truth(
    flow_id: RenderFlowId,
    pass: &CompiledPassExecutionPlan,
    runtime_resources: &FlowRuntimeResources,
) -> PassResourceTruth {
    let pass_id = execution_pass_id(pass);
    let pass_label = pass_id.to_string();
    let mut render_targets = Vec::<String>::new();
    let mut sampled_textures = Vec::<String>::new();
    let mut storage_textures = Vec::<String>::new();
    let mut depth_targets = Vec::<String>::new();
    let mut render_target_seen = BTreeSet::<String>::new();
    let mut sampled_seen = BTreeSet::<String>::new();
    let mut storage_seen = BTreeSet::<String>::new();
    let mut depth_seen = BTreeSet::<String>::new();

    match pass {
        CompiledPassExecutionPlan::Compute(plan) => {
            for entry in &plan.bindings.bind_group.entries {
                match entry {
                    CompiledBindingEntry::SampledTexture { resource } => {
                        push_resolved_resource_id(
                            pass_id,
                            resource,
                            "sampled_texture",
                            runtime_resources,
                            &mut sampled_seen,
                            &mut sampled_textures,
                        );
                    }
                    CompiledBindingEntry::StorageTexture { resource, .. } => {
                        push_resolved_resource_id(
                            pass_id,
                            resource,
                            "storage_texture",
                            runtime_resources,
                            &mut storage_seen,
                            &mut storage_textures,
                        );
                    }
                    _ => {}
                }
            }
        }
        CompiledPassExecutionPlan::Fullscreen(plan) | CompiledPassExecutionPlan::Graphics(plan) => {
            for target in &plan.targets.color_outputs {
                push_resolved_resource_id(
                    pass_id,
                    target,
                    "color_target",
                    runtime_resources,
                    &mut render_target_seen,
                    &mut render_targets,
                );
            }
            if let Some(depth_output) = plan.targets.depth_output.as_ref() {
                push_resolved_resource_id(
                    pass_id,
                    depth_output,
                    "depth_target",
                    runtime_resources,
                    &mut depth_seen,
                    &mut depth_targets,
                );
            }
            for entry in &plan.bindings.bind_group.entries {
                match entry {
                    CompiledBindingEntry::SampledTexture { resource } => {
                        push_resolved_resource_id(
                            pass_id,
                            resource,
                            "sampled_texture",
                            runtime_resources,
                            &mut sampled_seen,
                            &mut sampled_textures,
                        );
                    }
                    CompiledBindingEntry::StorageTexture { resource, .. } => {
                        push_resolved_resource_id(
                            pass_id,
                            resource,
                            "storage_texture",
                            runtime_resources,
                            &mut storage_seen,
                            &mut storage_textures,
                        );
                    }
                    _ => {}
                }
            }
        }
        CompiledPassExecutionPlan::Copy(plan) => {
            if let Some(source) = plan.source.as_ref() {
                push_resolved_resource_id(
                    pass_id,
                    source,
                    "copy_source",
                    runtime_resources,
                    &mut render_target_seen,
                    &mut render_targets,
                );
            }
            if let Some(destination) = plan.destination.as_ref() {
                push_resolved_resource_id(
                    pass_id,
                    destination,
                    "copy_destination",
                    runtime_resources,
                    &mut render_target_seen,
                    &mut render_targets,
                );
            }
        }
        CompiledPassExecutionPlan::Present(plan) => {
            if let Some(source) = plan.source.as_ref() {
                push_resolved_resource_id(
                    pass_id,
                    source,
                    "present_source",
                    runtime_resources,
                    &mut render_target_seen,
                    &mut render_targets,
                );
            }
        }
        CompiledPassExecutionPlan::BuiltinUiComposite(plan) => {
            push_resolved_resource_id(
                pass_id,
                &plan.color_output,
                "ui_composite_color_output",
                runtime_resources,
                &mut render_target_seen,
                &mut render_targets,
            );
        }
    }

    let mut capture_points_available = Vec::<RenderCapturePointIdentity>::new();
    let mut capture_point_seen = BTreeSet::<RenderCapturePointIdentity>::new();
    for resource_id in render_targets
        .iter()
        .chain(sampled_textures.iter())
        .chain(storage_textures.iter())
        .chain(depth_targets.iter())
    {
        let texture_class = runtime_resources
            .capture_texture_class(resource_id.as_str(), CaptureTextureClass::ColorTarget);
        for stage in [CaptureStage::Before, CaptureStage::After] {
            let identity = RenderCapturePointIdentity {
                flow_id: flow_id.to_string(),
                pass_id: pass_label.clone(),
                stage,
                resource_id: resource_id.clone(),
                texture_class,
            };
            if capture_point_seen.insert(identity.clone()) {
                capture_points_available.push(identity);
            }
        }
    }

    PassResourceTruth {
        render_targets,
        sampled_textures,
        storage_textures,
        depth_targets,
        capture_points_available,
    }
}

pub fn push_resolved_resource_id(
    pass_id: RenderPassId,
    target: &CompiledResourceRef,
    role: &str,
    runtime_resources: &FlowRuntimeResources,
    seen: &mut BTreeSet<String>,
    output: &mut Vec<String>,
) {
    if let Ok(resource_id) = runtime_resources.resolve_resource_key(pass_id, target, role) {
        let resource_id = resource_id.to_string();
        if seen.insert(resource_id.clone()) {
            output.push(resource_id);
        }
    }
}

pub fn execution_pass_kind_name(pass: &CompiledPassExecutionPlan) -> &'static str {
    match pass {
        CompiledPassExecutionPlan::Compute(_) => "compute",
        CompiledPassExecutionPlan::Fullscreen(_) => "fullscreen",
        CompiledPassExecutionPlan::Graphics(_) => "graphics",
        CompiledPassExecutionPlan::Copy(_) => "copy",
        CompiledPassExecutionPlan::Present(_) => "present",
        CompiledPassExecutionPlan::BuiltinUiComposite(_) => "builtin_ui_composite",
    }
}

pub fn execution_flow_pass_kind(pass: &CompiledPassExecutionPlan) -> FlowPassKind {
    match pass {
        CompiledPassExecutionPlan::Compute(_) => FlowPassKind::Compute,
        CompiledPassExecutionPlan::Fullscreen(_) => FlowPassKind::Fullscreen,
        CompiledPassExecutionPlan::Graphics(_) => FlowPassKind::Graphics,
        CompiledPassExecutionPlan::Copy(_) => FlowPassKind::Copy,
        CompiledPassExecutionPlan::Present(_) => FlowPassKind::Present,
        CompiledPassExecutionPlan::BuiltinUiComposite(_) => FlowPassKind::BuiltinUiComposite,
    }
}

pub fn execution_pass_id(pass: &CompiledPassExecutionPlan) -> RenderPassId {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => value.pass_id,
        CompiledPassExecutionPlan::Fullscreen(value) => value.pass_id,
        CompiledPassExecutionPlan::Graphics(value) => value.pass_id,
        CompiledPassExecutionPlan::Copy(value) => value.pass_id,
        CompiledPassExecutionPlan::Present(value) => value.pass_id,
        CompiledPassExecutionPlan::BuiltinUiComposite(value) => value.pass_id,
    }
}

pub fn execution_pass_feature_id(pass: &CompiledPassExecutionPlan) -> Option<RenderFeatureId> {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => value.feature_id,
        CompiledPassExecutionPlan::Fullscreen(value) => value.feature_id,
        CompiledPassExecutionPlan::Graphics(value) => value.feature_id,
        CompiledPassExecutionPlan::Copy(value) => value.feature_id,
        CompiledPassExecutionPlan::Present(value) => value.feature_id,
        CompiledPassExecutionPlan::BuiltinUiComposite(value) => Some(value.feature_id),
    }
}

pub fn execution_pass_order_index(pass: &CompiledPassExecutionPlan) -> usize {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => value.order_index,
        CompiledPassExecutionPlan::Fullscreen(value) => value.order_index,
        CompiledPassExecutionPlan::Graphics(value) => value.order_index,
        CompiledPassExecutionPlan::Copy(value) => value.order_index,
        CompiledPassExecutionPlan::Present(value) => value.order_index,
        CompiledPassExecutionPlan::BuiltinUiComposite(value) => value.order_index,
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedShaderMaterial<'a> {
    pub source: &'a str,
    pub shader_id: String,
    pub pipeline_identity: String,
    pub revision: u64,
    pub fallback_used: bool,
}

pub fn resolve_shader_material<'a>(
    reference: Option<&RenderShaderReference>,
    shader_registry: &'a ShaderRegistryResource,
    fallback_source: &'a str,
    fallback_identity: &'static str,
) -> ResolvedShaderMaterial<'a> {
    match reference {
        Some(RenderShaderReference::AssetPath(path)) => {
            let revision = shader_registry.revision_for(path);
            ResolvedShaderMaterial {
                source: shader_registry.source_or(path, fallback_source),
                shader_id: path.clone(),
                pipeline_identity: format!("asset:{path}"),
                revision,
                fallback_used: revision == 0,
            }
        }
        Some(RenderShaderReference::MaterialSceneBundle { fallback_asset }) => {
            let revision = shader_registry.revision_for(fallback_asset);
            ResolvedShaderMaterial {
                source: shader_registry.source_or(fallback_asset, fallback_source),
                shader_id: fallback_asset.clone(),
                pipeline_identity: format!("asset:{fallback_asset}"),
                revision,
                fallback_used: revision == 0,
            }
        }
        Some(RenderShaderReference::RegistryHandle(handle)) => {
            let revision = shader_registry.revision_for_handle(*handle);
            ResolvedShaderMaterial {
                source: shader_registry.source_or_handle(*handle, fallback_source),
                shader_id: format!("handle:{}", handle.index()),
                pipeline_identity: format!("handle:{}", handle.index()),
                revision,
                fallback_used: revision == 0,
            }
        }
        None => ResolvedShaderMaterial {
            source: fallback_source,
            shader_id: fallback_identity.to_string(),
            pipeline_identity: fallback_identity.to_string(),
            revision: 0,
            fallback_used: false,
        },
    }
}

pub fn resolve_shader_material_for_packet<'a>(
    reference: Option<&RenderShaderReference>,
    packet: &RendererPreparedPacket,
    shader_registry: &'a ShaderRegistryResource,
    fallback_source: &'a str,
    fallback_identity: &'static str,
) -> ResolvedShaderMaterial<'a> {
    match reference {
        Some(RenderShaderReference::MaterialSceneBundle { fallback_asset }) => {
            if let Some(scene_bundle) = packet
                .prepared_material
                .as_ref()
                .and_then(|material| material.scene_bundle.as_ref())
            {
                let revision = shader_registry.revision_for(scene_bundle.shader_path.as_str());
                return ResolvedShaderMaterial {
                    source: shader_registry
                        .source_or(scene_bundle.shader_path.as_str(), fallback_source),
                    shader_id: scene_bundle.shader_path.clone(),
                    pipeline_identity: format!(
                        "material-scene-bundle:{}:{}:{}",
                        scene_bundle.shader_identity,
                        scene_bundle.material_table_identity,
                        scene_bundle.shader_cache_key
                    ),
                    revision,
                    fallback_used: revision == 0,
                };
            }
            let fallback = resolve_shader_material(
                Some(&RenderShaderReference::AssetPath(fallback_asset.clone())),
                shader_registry,
                fallback_source,
                fallback_identity,
            );
            ResolvedShaderMaterial {
                fallback_used: true,
                ..fallback
            }
        }
        other => {
            resolve_shader_material(other, shader_registry, fallback_source, fallback_identity)
        }
    }
}

pub fn hash_bind_group_layout_entries(entries: &[BindGroupLayoutEntry]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for entry in entries {
        entry.binding.hash(&mut hasher);
        entry.visibility.bits().hash(&mut hasher);
        format!("{:?}", entry.ty).hash(&mut hasher);
    }
    hasher.finish()
}

pub fn hash_view_signature(view_id: &str, surface_size: (u32, u32)) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    view_id.hash(&mut hasher);
    surface_size.hash(&mut hasher);
    hasher.finish()
}

pub fn material_specialization_fragment_hash(
    packet: &RendererPreparedPacket,
    pass_feature_id: Option<RenderFeatureId>,
) -> u64 {
    if pass_feature_id != Some(crate::plugins::render::features::MATERIAL_RENDER_FEATURE_ID) {
        return 0;
    }

    pass_feature_id
        .and_then(|feature_id| packet.feature_runtime_signatures.get(&feature_id).copied())
        .unwrap_or_default()
}

pub fn feature_runtime_version(
    packet: &RendererPreparedPacket,
    pass_feature_id: Option<RenderFeatureId>,
) -> u64 {
    let Some(feature_id) = pass_feature_id else {
        return 0;
    };

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    feature_id.hash(&mut hasher);
    if let Some(gate) = packet.feature_gates.get(&feature_id) {
        gate.status.hash(&mut hasher);
        gate.fallback_policy.hash(&mut hasher);
    }
    packet
        .feature_runtime_signatures
        .get(&feature_id)
        .copied()
        .unwrap_or_default()
        .hash(&mut hasher);
    hasher.finish()
}

pub fn compiled_storage_access_to_storage_texture_access(
    access: CompiledStorageAccess,
) -> StorageTextureAccess {
    match access {
        CompiledStorageAccess::ReadOnly => StorageTextureAccess::ReadOnly,
        CompiledStorageAccess::ReadWrite => StorageTextureAccess::ReadWrite,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::features::{
        MATERIAL_RENDER_FEATURE_ID, UI_RENDER_FEATURE_ID, WORLD_DRAW_RENDER_FEATURE_ID,
    };
    use crate::plugins::render::{CompiledPresentExecutionPlan, CompiledViewMask};

    fn packet_with_feature_gate(
        feature_id: RenderFeatureId,
        gate: FeatureExecutionGate,
    ) -> RendererPreparedPacket {
        let mut feature_gates = BTreeMap::new();
        feature_gates.insert(feature_id, gate);
        let mut feature_runtime_signatures = BTreeMap::new();
        feature_runtime_signatures.insert(feature_id, 1);
        RendererPreparedPacket {
            surface_format: TextureFormat::Rgba8Unorm,
            surface_size: (1, 1),
            view_id: "main".to_string(),
            feature_gates,
            feature_runtime_signatures,
            prepared_material: None,
            prepared_material_gpu_resources: None,
            prepared_ui: UiPreparedDraws::default(),
            viewport_surface_bindings: Default::default(),
            prepare_timings: RendererFrameTimings::default(),
        }
    }

    #[test]
    fn ui_feature_gate_skips_when_missing_and_policy_is_skip() {
        let renderer = Renderer::new();
        let packet = packet_with_feature_gate(
            UI_RENDER_FEATURE_ID,
            FeatureExecutionGate {
                status: FeatureContributionStatus::Missing,
                fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            },
        );
        let action = renderer
            .resolve_feature_pass_action(
                UI_RENDER_FEATURE_ID,
                RenderPassId::try_from_raw(1).unwrap(),
                &packet,
            )
            .expect("skip policy should not error");
        assert_eq!(action, FeaturePassAction::Skip);
    }

    #[test]
    fn ui_feature_gate_fails_when_missing_and_policy_is_fail_frame() {
        let renderer = Renderer::new();
        let packet = packet_with_feature_gate(
            UI_RENDER_FEATURE_ID,
            FeatureExecutionGate {
                status: FeatureContributionStatus::Missing,
                fallback_policy: FeatureFallbackPolicy::FailFrame,
            },
        );
        assert!(
            renderer
                .resolve_feature_pass_action(
                    UI_RENDER_FEATURE_ID,
                    RenderPassId::try_from_raw(1).unwrap(),
                    &packet
                )
                .is_err(),
            "missing + fail-frame should produce an execution error"
        );
    }

    #[test]
    fn generic_feature_gate_applies_to_non_ui_passes() {
        let renderer = Renderer::new();
        let packet = packet_with_feature_gate(
            WORLD_DRAW_RENDER_FEATURE_ID,
            FeatureExecutionGate {
                status: FeatureContributionStatus::Missing,
                fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            },
        );

        let action = renderer
            .resolve_feature_pass_action(
                WORLD_DRAW_RENDER_FEATURE_ID,
                RenderPassId::try_from_raw(2).unwrap(),
                &packet,
            )
            .expect("skip policy should not error");
        assert_eq!(action, FeaturePassAction::Skip);
    }

    #[test]
    fn feature_runtime_version_changes_when_runtime_signature_changes() {
        let mut packet = packet_with_feature_gate(
            WORLD_DRAW_RENDER_FEATURE_ID,
            FeatureExecutionGate {
                status: FeatureContributionStatus::Ready,
                fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            },
        );
        let base = feature_runtime_version(&packet, Some(WORLD_DRAW_RENDER_FEATURE_ID));
        packet
            .feature_runtime_signatures
            .insert(WORLD_DRAW_RENDER_FEATURE_ID, 99);
        let changed = feature_runtime_version(&packet, Some(WORLD_DRAW_RENDER_FEATURE_ID));
        assert_ne!(base, changed);
    }

    #[test]
    fn material_specialization_hash_uses_material_feature_signature() {
        let mut packet = packet_with_feature_gate(
            MATERIAL_RENDER_FEATURE_ID,
            FeatureExecutionGate {
                status: FeatureContributionStatus::Ready,
                fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            },
        );
        packet
            .feature_runtime_signatures
            .insert(MATERIAL_RENDER_FEATURE_ID, 1234);
        assert_eq!(
            material_specialization_fragment_hash(&packet, Some(MATERIAL_RENDER_FEATURE_ID)),
            1234
        );
        assert_eq!(
            material_specialization_fragment_hash(&packet, Some(WORLD_DRAW_RENDER_FEATURE_ID)),
            0
        );
    }

    #[test]
    fn material_scene_bundle_missing_from_packet_is_a_forbidden_fallback() {
        let packet = packet_with_feature_gate(
            MATERIAL_RENDER_FEATURE_ID,
            FeatureExecutionGate {
                status: FeatureContributionStatus::Ready,
                fallback_policy: FeatureFallbackPolicy::FailFrame,
            },
        );
        let shader_registry = ShaderRegistryResource::new();
        let shader = resolve_shader_material_for_packet(
            Some(&RenderShaderReference::MaterialSceneBundle {
                fallback_asset: "assets/shaders/editor_viewport_scene_product.wgsl".to_string(),
            }),
            &packet,
            &shader_registry,
            "fallback shader",
            "builtin:graphics",
        );

        assert!(
            shader.fallback_used,
            "material scene passes must not silently use the fallback scene shader when no generated scene bundle is prepared"
        );
    }

    #[test]
    fn pass_view_mask_filters_non_matching_views() {
        let renderer = Renderer::new();
        let mut explicit = BTreeSet::new();
        explicit.insert("main".to_string());
        let pass = CompiledPassExecutionPlan::Present(CompiledPresentExecutionPlan {
            pass_id: RenderPassId::try_from_raw(1).unwrap(),
            order_index: 0,
            feature_id: None,
            view_mask: CompiledViewMask::Explicit(explicit),
            source: None,
        });

        assert!(renderer.pass_targets_active_view(
            &pass,
            "main",
            crate::plugins::render::PreparedViewKind::MainSurface
        ));
        assert!(!renderer.pass_targets_active_view(
            &pass,
            "minimap",
            crate::plugins::render::PreparedViewKind::OffscreenProduct
        ));
    }
}
