use crate::plugins::render::{
    CompiledRenderFlowPlan, PreparedFlowInvocationId, PreparedFlowInvocationRequest,
    PreparedTargetBinding, PreparedViewFrame, RenderDynamicTextureRetention,
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetKey, RenderFlow, RenderFlowId,
    RenderFlowInvocationPolicy, RenderFrameProducerId, RenderPassViewScope,
    RenderResourceDeclaration, RenderTargetAliasKey, RenderTargetAliasKind, RenderTextureSampleMode,
    RenderTextureTargetFormat,
};
use runen_gpu::GpuBindingKey;

pub const FIXED_RESOLUTION_RESOLVE_FLOW_LABEL: &str = "render.fixed_resolution.resolve";
pub const FIXED_RESOLUTION_RESOLVE_PASS_LABEL: &str = "render.fixed_resolution.resolve.pass";
pub const FIXED_RESOLUTION_RESOLVE_SOURCE_ALIAS: &str = "fixed_resolution.source";
pub const FIXED_RESOLUTION_RESOLVE_SHADER_ASSET: &str =
    "assets/shaders/fixed_resolution_resolve.wgsl";

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderFixedResolutionExecutionIdentity {
    target_key: RenderDynamicTextureTargetKey,
    internal_view_id: String,
    fixed_scene_invocation_id: PreparedFlowInvocationId,
    native_scene_invocation_id: PreparedFlowInvocationId,
    resolve_invocation_id: PreparedFlowInvocationId,
}

fn fixed_resolution_execution_identity(
    scene_flow_id: RenderFlowId,
) -> RenderFixedResolutionExecutionIdentity {
    let identity = scene_flow_id.to_string();
    RenderFixedResolutionExecutionIdentity {
        target_key: RenderDynamicTextureTargetKey::new(
            "render.fixed_resolution",
            format!("{identity}.color"),
        ),
        internal_view_id: format!("fixed-resolution.{identity}.internal"),
        fixed_scene_invocation_id: PreparedFlowInvocationId::new(format!(
            "fixed-resolution.{identity}.scene"
        )),
        native_scene_invocation_id: PreparedFlowInvocationId::new(format!(
            "fixed-resolution.{identity}.native"
        )),
        resolve_invocation_id: PreparedFlowInvocationId::new(format!(
            "fixed-resolution.{identity}.resolve"
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFixedResolutionExecutionRequest {
    pub producer_id: RenderFrameProducerId,
    pub scene_flow_id: RenderFlowId,
    pub scene_color_alias: RenderTargetAliasKey,
    pub internal_size: (u32, u32),
}

impl RenderFixedResolutionExecutionRequest {
    pub fn new(
        producer_id: RenderFrameProducerId,
        scene_flow_id: RenderFlowId,
        scene_color_alias: RenderTargetAliasKey,
        internal_size: (u32, u32),
    ) -> Self {
        Self {
            producer_id,
            scene_flow_id,
            scene_color_alias,
            internal_size,
        }
    }

    fn prepare(
        &self,
        output_size: (u32, u32),
        resolve_flow_id: RenderFlowId,
    ) -> Result<PreparedFixedResolutionExecution, RenderFixedResolutionExecutionError> {
        validate_extents(self.internal_size, output_size)?;

        let identity = fixed_resolution_execution_identity(self.scene_flow_id);
        let target_key = identity.target_key.clone();
        let view_id = identity.internal_view_id.clone();
        let scene_invocation_id = identity.fixed_scene_invocation_id.clone();
        let resolve_invocation_id = identity.resolve_invocation_id.clone();

        let dynamic_target = RenderDynamicTextureTargetDescriptor::color_sampled(
            target_key.clone(),
            self.internal_size.0,
            self.internal_size.1,
            RenderTextureTargetFormat::Rgba8UnormSrgb,
            RenderTextureSampleMode::FilterableFloat,
            RenderDynamicTextureRetention::RetainWhileRequested,
        );

        let internal_view = PreparedViewFrame::offscreen_product(view_id.clone(), self.internal_size);

        let scene_invocation = PreparedFlowInvocationRequest::new(
            scene_invocation_id,
            self.scene_flow_id,
            view_id,
        )
        .bind_target_alias(
            self.scene_color_alias.as_str(),
            PreparedTargetBinding::DynamicTexture(target_key.clone()),
        )
        .map_err(RenderFixedResolutionExecutionError::AliasBinding)?;

        let resolve_invocation =
            PreparedFlowInvocationRequest::new(resolve_invocation_id, resolve_flow_id, "main")
                .bind_dynamic_texture_alias(FIXED_RESOLUTION_RESOLVE_SOURCE_ALIAS, target_key.clone())
                .map_err(RenderFixedResolutionExecutionError::AliasBinding)?;

        Ok(PreparedFixedResolutionExecution {
            producer_id: self.producer_id,
            output_size,
            internal_size: self.internal_size,
            scene_color_alias: self.scene_color_alias.clone(),
            target_key,
            dynamic_target,
            internal_view,
            scene_invocation,
            resolve_invocation,
            automatic_main_replacement: self.scene_flow_id,
        })
    }

    fn validate_selected_flow(
        &self,
        compiled_flow: &CompiledRenderFlowPlan,
    ) -> Result<(), RenderFixedResolutionExecutionError> {
        if compiled_flow.flow_id != self.scene_flow_id {
            return Err(RenderFixedResolutionExecutionError::SelectedFlowMismatch {
                requested: self.scene_flow_id,
                provided: compiled_flow.flow_id,
            });
        }
        if compiled_flow.invocation_policy != RenderFlowInvocationPolicy::AutomaticMain {
            return Err(
                RenderFixedResolutionExecutionError::SelectedFlowRequiresAutomaticMain {
                    flow_id: self.scene_flow_id,
                },
            );
        }

        let alias_resource_id = compiled_flow
            .resources
            .resources
            .iter()
            .find_map(|resource| match resource {
                RenderResourceDeclaration::TargetAlias(alias)
                    if alias.binding_key() == &self.scene_color_alias
                        && alias.kind() == RenderTargetAliasKind::Color =>
                {
                    Some(alias.id())
                }
                _ => None,
            })
            .ok_or_else(|| RenderFixedResolutionExecutionError::MissingBindableColorAlias {
                flow_id: self.scene_flow_id,
                alias: self.scene_color_alias.clone(),
            })?;

        let mut alias_writer_count = 0usize;
        for pass in &compiled_flow.render_passes {
            let node = pass.node();
            let writes_alias = node.color_outputs.contains(&alias_resource_id)
                || node.copy_destination == Some(alias_resource_id)
                || node.write_textures.contains(&alias_resource_id);
            if !writes_alias {
                continue;
            }
            alias_writer_count += 1;
            if node.view_scope != RenderPassViewScope::AllViews {
                return Err(
                    RenderFixedResolutionExecutionError::SelectedFlowAliasNotDualViewCapable {
                        flow_id: self.scene_flow_id,
                        alias: self.scene_color_alias.clone(),
                    },
                );
            }
        }

        if alias_writer_count == 0 {
            return Err(
                RenderFixedResolutionExecutionError::SelectedFlowAliasNotDualViewCapable {
                    flow_id: self.scene_flow_id,
                    alias: self.scene_color_alias.clone(),
                },
            );
        }

        Ok(())
    }

    fn native_fallback_invocation_against_compiled_flow(
        &self,
        compiled_flow: &CompiledRenderFlowPlan,
    ) -> Option<PreparedFlowInvocationRequest> {
        self.validate_selected_flow(compiled_flow).ok()?;
        let identity = fixed_resolution_execution_identity(self.scene_flow_id);
        Some(
            PreparedFlowInvocationRequest::new(
                identity.native_scene_invocation_id,
                self.scene_flow_id,
                "main",
            )
            .bind_surface_color_alias(self.scene_color_alias.as_str())
            .expect("validated fixed-resolution scene alias must remain valid"),
        )
    }

    pub fn prepare_against_compiled_flow(
        &self,
        output_size: (u32, u32),
        resolve_flow_id: RenderFlowId,
        compiled_flow: &CompiledRenderFlowPlan,
    ) -> Result<PreparedFixedResolutionExecution, RenderFixedResolutionExecutionError> {
        self.validate_selected_flow(compiled_flow)?;
        self.prepare(output_size, resolve_flow_id)
    }

    pub fn admit_against_compiled_flow(
        &self,
        output_size: (u32, u32),
        resolve_flow_id: RenderFlowId,
        compiled_flow: &CompiledRenderFlowPlan,
    ) -> RenderFixedResolutionExecutionAdmission {
        match self.prepare_against_compiled_flow(output_size, resolve_flow_id, compiled_flow) {
            Ok(prepared) => RenderFixedResolutionExecutionAdmission::Fixed(prepared),
            Err(error) => {
                let identity = fixed_resolution_execution_identity(self.scene_flow_id);
                RenderFixedResolutionExecutionAdmission::NativeFallback(
                    RenderFixedResolutionFallback {
                        scene_flow_id: self.scene_flow_id,
                        requested_internal_size: self.internal_size,
                        output_size,
                        reason: error.to_string(),
                        native_scene_invocation: self
                            .native_fallback_invocation_against_compiled_flow(compiled_flow),
                        target_key: identity.target_key,
                        internal_view_id: identity.internal_view_id,
                        fixed_scene_invocation_id: identity.fixed_scene_invocation_id,
                        resolve_invocation_id: identity.resolve_invocation_id,
                    },
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderFixedResolutionExecutionAdmission {
    Fixed(PreparedFixedResolutionExecution),
    NativeFallback(RenderFixedResolutionFallback),
}

impl RenderFixedResolutionExecutionAdmission {
    pub fn native_fallback_active(&self) -> bool {
        matches!(self, Self::NativeFallback(_))
    }

    pub fn fallback_reason(&self) -> Option<&str> {
        match self {
            Self::Fixed(_) => None,
            Self::NativeFallback(fallback) => Some(fallback.reason.as_str()),
        }
    }

    pub fn prepared(&self) -> Option<&PreparedFixedResolutionExecution> {
        match self {
            Self::Fixed(prepared) => Some(prepared),
            Self::NativeFallback(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFixedResolutionFallback {
    pub scene_flow_id: RenderFlowId,
    pub requested_internal_size: (u32, u32),
    pub output_size: (u32, u32),
    pub reason: String,
    pub native_scene_invocation: Option<PreparedFlowInvocationRequest>,
    pub(crate) target_key: RenderDynamicTextureTargetKey,
    pub(crate) internal_view_id: String,
    pub(crate) fixed_scene_invocation_id: PreparedFlowInvocationId,
    pub(crate) resolve_invocation_id: PreparedFlowInvocationId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFixedResolutionExecution {
    pub producer_id: RenderFrameProducerId,
    pub output_size: (u32, u32),
    pub internal_size: (u32, u32),
    pub scene_color_alias: RenderTargetAliasKey,
    pub target_key: RenderDynamicTextureTargetKey,
    pub dynamic_target: RenderDynamicTextureTargetDescriptor,
    pub internal_view: PreparedViewFrame,
    pub scene_invocation: PreparedFlowInvocationRequest,
    pub resolve_invocation: PreparedFlowInvocationRequest,
    pub automatic_main_replacement: RenderFlowId,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderFixedResolutionExecutionError {
    #[error("fixed internal-resolution execution requires nonzero internal and output extents")]
    ZeroExtent,
    #[error(
        "fixed internal extent {internal_width}x{internal_height} exceeds output extent {output_width}x{output_height}"
    )]
    InternalExtentExceedsOutput {
        internal_width: u32,
        internal_height: u32,
        output_width: u32,
        output_height: u32,
    },
    #[error(
        "fixed internal extent {internal_width}x{internal_height} must preserve output aspect {output_width}x{output_height}"
    )]
    AspectMismatch {
        internal_width: u32,
        internal_height: u32,
        output_width: u32,
        output_height: u32,
    },
    #[error(
        "fixed internal-resolution request selects flow {requested:?} but admission inspected flow {provided:?}"
    )]
    SelectedFlowMismatch {
        requested: RenderFlowId,
        provided: RenderFlowId,
    },
    #[error(
        "fixed internal-resolution flow {flow_id:?} must use automatic-main invocation policy"
    )]
    SelectedFlowRequiresAutomaticMain { flow_id: RenderFlowId },
    #[error(
        "fixed internal-resolution flow {flow_id:?} color target alias '{alias}' must be written by at least one pass and every writer must support both main and offscreen views"
    )]
    SelectedFlowAliasNotDualViewCapable {
        flow_id: RenderFlowId,
        alias: RenderTargetAliasKey,
    },
    #[error(
        "fixed internal-resolution flow {flow_id:?} does not expose required color target alias '{alias}'"
    )]
    MissingBindableColorAlias {
        flow_id: RenderFlowId,
        alias: RenderTargetAliasKey,
    },
    #[error("fixed internal-resolution alias binding failed: {0}")]
    AliasBinding(#[source] crate::plugins::render::RenderGpuResourceAdapterError),
}

pub fn fixed_resolution_resolve_flow() -> anyhow::Result<RenderFlow> {
    RenderFlow::new(FIXED_RESOLUTION_RESOLVE_FLOW_LABEL)
        .explicit_invocations_only()
        .with_target_alias(
            FIXED_RESOLUTION_RESOLVE_SOURCE_ALIAS,
            RenderTargetAliasKind::Texture,
        )?
        .with_surface_color()?
        .fullscreen_pass(FIXED_RESOLUTION_RESOLVE_PASS_LABEL)
        .main_surface_only()
        .shader_asset(FIXED_RESOLUTION_RESOLVE_SHADER_ASSET)
        .sample_texture(
            GpuBindingKey::try_new(0, 0)
                .expect("fixed-resolution texture binding must fit GpuBindingKey"),
            GpuBindingKey::try_new(0, 1)
                .expect("fixed-resolution sampler binding must fit GpuBindingKey"),
            FIXED_RESOLUTION_RESOLVE_SOURCE_ALIAS,
        )
        .write_surface_color()?
        .finish()
        .validate()
}

fn validate_extents(
    internal_size: (u32, u32),
    output_size: (u32, u32),
) -> Result<(), RenderFixedResolutionExecutionError> {
    if internal_size.0 == 0
        || internal_size.1 == 0
        || output_size.0 == 0
        || output_size.1 == 0
    {
        return Err(RenderFixedResolutionExecutionError::ZeroExtent);
    }
    if internal_size.0 > output_size.0 || internal_size.1 > output_size.1 {
        return Err(
            RenderFixedResolutionExecutionError::InternalExtentExceedsOutput {
                internal_width: internal_size.0,
                internal_height: internal_size.1,
                output_width: output_size.0,
                output_height: output_size.1,
            },
        );
    }
    if u64::from(internal_size.0) * u64::from(output_size.1)
        != u64::from(internal_size.1) * u64::from(output_size.0)
    {
        return Err(RenderFixedResolutionExecutionError::AspectMismatch {
            internal_width: internal_size.0,
            internal_height: internal_size.1,
            output_width: output_size.0,
            output_height: output_size.1,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn producer(raw: u64) -> RenderFrameProducerId {
        RenderFrameProducerId::try_from_raw(raw).expect("test producer id should be nonzero")
    }

    fn flow(raw: u64) -> RenderFlowId {
        RenderFlowId::try_from_raw(raw).expect("test flow id should be nonzero")
    }

    fn alias() -> RenderTargetAliasKey {
        RenderTargetAliasKey::new("scene_color").expect("test alias should be valid")
    }

    #[test]
    fn fixed_resolution_builder_prepares_internal_target_and_native_resolve() {
        let request =
            RenderFixedResolutionExecutionRequest::new(producer(7), flow(11), alias(), (1280, 720));
        let prepared = request
            .prepare((1920, 1080), flow(12))
            .expect("valid fixed-resolution request should prepare");

        assert_eq!(prepared.internal_size, (1280, 720));
        assert_eq!(prepared.output_size, (1920, 1080));
        assert_eq!(prepared.dynamic_target.width, 1280);
        assert_eq!(prepared.dynamic_target.height, 720);
        assert!(prepared.dynamic_target.usage.color_attachment);
        assert!(prepared.dynamic_target.usage.sampled);
        assert_eq!(prepared.internal_view.target_size_px, (1280, 720));
        assert_eq!(prepared.scene_invocation.flow_id, flow(11));
        assert_ne!(prepared.scene_invocation.view_id, "main");
        assert_eq!(prepared.resolve_invocation.flow_id, flow(12));
        assert_eq!(prepared.resolve_invocation.view_id, "main");
        assert_eq!(prepared.automatic_main_replacement, flow(11));
        assert_eq!(
            prepared
                .scene_invocation
                .target_alias_bindings
                .get(&alias()),
            Some(&PreparedTargetBinding::DynamicTexture(
                prepared.target_key.clone()
            ))
        );
    }

    fn compiled_scene_flow(alias_name: &str) -> CompiledRenderFlowPlan {
        let flow = RenderFlow::new("fixed.test.scene")
            .with_color_target_alias(alias_name)
            .expect("test color alias should be valid")
            .fullscreen_pass("fixed.test.scene.pass")
            .write_target_alias(alias_name)
            .finish()
            .validate()
            .expect("test scene flow should validate");
        crate::plugins::render::compile_flow_plan(&flow)
            .expect("test scene flow should compile")
    }

    #[test]
    fn fixed_resolution_builder_requires_selected_flow_color_alias() {
        let compiled = compiled_scene_flow("scene_color");
        let request = RenderFixedResolutionExecutionRequest::new(
            producer(7),
            compiled.flow_id,
            alias(),
            (1280, 720),
        );
        request
            .prepare_against_compiled_flow((1920, 1080), flow(12), &compiled)
            .expect("declared color alias should admit fixed resolution");

        let missing = RenderFixedResolutionExecutionRequest::new(
            producer(7),
            compiled.flow_id,
            RenderTargetAliasKey::new("other_color").expect("test alias should be valid"),
            (1280, 720),
        );
        assert!(matches!(
            missing.prepare_against_compiled_flow((1920, 1080), flow(12), &compiled),
            Err(RenderFixedResolutionExecutionError::MissingBindableColorAlias { .. })
        ));

        let mismatched_flow =
            RenderFixedResolutionExecutionRequest::new(producer(7), flow(99), alias(), (1280, 720));
        assert!(matches!(
            mismatched_flow.prepare_against_compiled_flow(
                (1920, 1080),
                flow(12),
                &compiled
            ),
            Err(RenderFixedResolutionExecutionError::SelectedFlowMismatch { .. })
        ));
    }

    #[test]
    fn fixed_resolution_requires_automatic_dual_view_scene_flow() {
        let explicit = RenderFlow::new("fixed.test.explicit")
            .explicit_invocations_only()
            .with_color_target_alias("scene_color")
            .expect("test color alias should be valid")
            .fullscreen_pass("fixed.test.explicit.pass")
            .write_target_alias("scene_color")
            .finish()
            .validate()
            .expect("explicit test flow should validate");
        let explicit_compiled =
            crate::plugins::render::compile_flow_plan(&explicit).expect("flow should compile");
        let explicit_request = RenderFixedResolutionExecutionRequest::new(
            producer(7),
            explicit.id(),
            alias(),
            (1280, 720),
        );
        assert!(matches!(
            explicit_request.prepare_against_compiled_flow(
                (1920, 1080),
                flow(12),
                &explicit_compiled
            ),
            Err(RenderFixedResolutionExecutionError::SelectedFlowRequiresAutomaticMain { .. })
        ));

        let main_only = RenderFlow::new("fixed.test.main-only")
            .with_color_target_alias("scene_color")
            .expect("test color alias should be valid")
            .fullscreen_pass("fixed.test.main-only.pass")
            .main_surface_only()
            .write_target_alias("scene_color")
            .finish()
            .validate()
            .expect("main-only test flow should validate");
        let main_only_compiled =
            crate::plugins::render::compile_flow_plan(&main_only).expect("flow should compile");
        let main_only_request = RenderFixedResolutionExecutionRequest::new(
            producer(7),
            main_only.id(),
            alias(),
            (1280, 720),
        );
        assert!(matches!(
            main_only_request.prepare_against_compiled_flow(
                (1920, 1080),
                flow(12),
                &main_only_compiled
            ),
            Err(
                RenderFixedResolutionExecutionError::SelectedFlowAliasNotDualViewCapable { .. }
            )
        ));
    }

    #[test]
    fn fixed_resolution_admission_returns_complete_fixed_parts_or_native_fallback() {
        let compiled = compiled_scene_flow("scene_color");
        let valid = RenderFixedResolutionExecutionRequest::new(
            producer(7),
            compiled.flow_id,
            alias(),
            (1280, 720),
        )
        .admit_against_compiled_flow((1920, 1080), flow(12), &compiled);
        assert!(!valid.native_fallback_active());
        let prepared = valid.prepared().expect("valid admission should retain fixed parts");
        assert_eq!(prepared.internal_size, (1280, 720));
        assert_eq!(prepared.output_size, (1920, 1080));

        let invalid = RenderFixedResolutionExecutionRequest::new(
            producer(7),
            compiled.flow_id,
            alias(),
            (1280, 800),
        )
        .admit_against_compiled_flow((1920, 1080), flow(12), &compiled);
        assert!(invalid.native_fallback_active());
        assert!(
            invalid
                .fallback_reason()
                .is_some_and(|reason| reason.contains("must preserve output aspect"))
        );
        let RenderFixedResolutionExecutionAdmission::NativeFallback(fallback) = &invalid else {
            panic!("invalid fixed request should use native fallback");
        };
        let native = fallback
            .native_scene_invocation
            .as_ref()
            .expect("validated alias-capable scene should retain explicit native fallback");
        assert_eq!(native.flow_id, compiled.flow_id);
        assert_eq!(native.view_id, "main");
        assert_eq!(
            native.target_alias_bindings.get(&alias()),
            Some(&PreparedTargetBinding::SurfaceColor)
        );
        assert!(invalid.prepared().is_none());
    }

    #[test]
    fn fixed_resolution_builder_rejects_zero_mismatched_and_supersampled_extents() {
        let request =
            RenderFixedResolutionExecutionRequest::new(producer(7), flow(11), alias(), (0, 720));
        assert!(matches!(
            request.prepare((1920, 1080), flow(12)),
            Err(RenderFixedResolutionExecutionError::ZeroExtent)
        ));

        let request =
            RenderFixedResolutionExecutionRequest::new(producer(7), flow(11), alias(), (1280, 800));
        assert!(matches!(
            request.prepare((1920, 1080), flow(12)),
            Err(RenderFixedResolutionExecutionError::AspectMismatch { .. })
        ));

        let request =
            RenderFixedResolutionExecutionRequest::new(producer(7), flow(11), alias(), (2560, 1440));
        assert!(matches!(
            request.prepare((1920, 1080), flow(12)),
            Err(RenderFixedResolutionExecutionError::InternalExtentExceedsOutput { .. })
        ));
    }

    #[test]
    fn fixed_resolution_resolve_flow_is_portable_and_alias_driven() {
        let flow = fixed_resolution_resolve_flow().expect("resolve flow should validate");
        assert_eq!(flow.label(), FIXED_RESOLUTION_RESOLVE_FLOW_LABEL);
        assert_eq!(
            flow.invocation_policy(),
            crate::plugins::render::RenderFlowInvocationPolicy::ExplicitOnly
        );
        assert!(flow.resource_id(FIXED_RESOLUTION_RESOLVE_SOURCE_ALIAS).is_some());
    }
}
