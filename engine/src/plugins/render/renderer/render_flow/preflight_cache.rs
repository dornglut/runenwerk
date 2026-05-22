use super::*;
use crate::plugins::render::graph::{
    RenderPreflightValidationConfigResource, RenderPreparedFramePreflightCacheKey,
    RenderPreparedFramePreflightCacheState, RenderPreparedFramePreflightCacheStatus,
    RenderPreparedFramePreflightMode, RenderPreparedFramePreflightReportSource,
    preflight_prepared_render_frame_runtime_guards, prepared_render_frame_preflight_cache_key,
};

#[derive(Debug, Clone)]
pub(crate) struct RendererPreparedFramePreflightCacheEntry {
    key: RenderPreparedFramePreflightCacheKey,
    report: RenderExecutionGraphPreparedReport,
}

impl Renderer {
    pub(crate) fn preflight_prepared_frame(
        &mut self,
        prepared_frame: &PreparedRenderFrame,
        compiled_flows: &[CompiledRenderFlowPlan],
        config: RenderPreflightValidationConfigResource,
    ) -> Result<RenderExecutionGraphPreparedReport> {
        let profile = RenderBackendCapabilityProfile::runtime_default();
        let mode = config.effective_mode();
        let key =
            prepared_render_frame_preflight_cache_key(prepared_frame, compiled_flows, &profile);

        if let Err(error) =
            preflight_prepared_render_frame_runtime_guards(prepared_frame, compiled_flows)
        {
            self.last_preflight_cache_state = RenderPreparedFramePreflightCacheState {
                mode,
                status: RenderPreparedFramePreflightCacheStatus::GuardRejected,
                report_source: RenderPreparedFramePreflightReportSource::RuntimeGuard,
                cache_key: Some(key),
            };
            return Err(anyhow::Error::new(error));
        }

        if mode == RenderPreparedFramePreflightMode::CachedStrict
            && let Some(entry) = &self.preflight_cache
            && entry.key == key
        {
            self.last_preflight_cache_state = RenderPreparedFramePreflightCacheState {
                mode,
                status: RenderPreparedFramePreflightCacheStatus::Hit,
                report_source: RenderPreparedFramePreflightReportSource::CachedReport,
                cache_key: Some(key),
            };
            return Ok(entry.report.clone());
        }

        let status = match mode {
            RenderPreparedFramePreflightMode::StrictEveryFrame => {
                RenderPreparedFramePreflightCacheStatus::StrictMode
            }
            RenderPreparedFramePreflightMode::CachedStrict if self.preflight_cache.is_some() => {
                RenderPreparedFramePreflightCacheStatus::KeyMismatch
            }
            RenderPreparedFramePreflightMode::CachedStrict => {
                RenderPreparedFramePreflightCacheStatus::ColdMiss
            }
        };

        let report = preflight_prepared_render_frame(prepared_frame, compiled_flows, &profile)
            .map_err(anyhow::Error::new)?;
        if mode == RenderPreparedFramePreflightMode::CachedStrict {
            self.preflight_cache = Some(RendererPreparedFramePreflightCacheEntry {
                key: key.clone(),
                report: report.clone(),
            });
        }
        self.last_preflight_cache_state = RenderPreparedFramePreflightCacheState {
            mode,
            status,
            report_source: RenderPreparedFramePreflightReportSource::FullValidation,
            cache_key: Some(key),
        };
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::{
        PreparedFlowInputs, PreparedFlowInvocation, PreparedFlowInvocationId, PreparedFrameContext,
        PreparedFrameContributions, PreparedRenderFrame, PreparedShaderSnapshot,
        PreparedSurfaceInfo, PreparedTargetBinding, PreparedViewFrame,
        RenderDynamicTextureRetention, RenderDynamicTextureTargetDescriptor,
        RenderDynamicTextureTargetKey, RenderFlow, RenderPassId, RenderTextureTargetFormat,
        compile_flow_plan,
    };
    use std::collections::BTreeMap;
    use ui_render_data::ViewportSurfaceBindingRegistry;

    fn prepared_frame(
        compiled: &CompiledRenderFlowPlan,
        inputs: PreparedFlowInputs,
    ) -> PreparedRenderFrame {
        let key = RenderDynamicTextureTargetKey::new("cache.test", "scene");
        let mut bindings = BTreeMap::new();
        bindings.insert(
            "scene_color".to_string(),
            PreparedTargetBinding::DynamicTexture(key.clone()),
        );
        PreparedRenderFrame {
            context: PreparedFrameContext {
                frame_index: 1,
                flow_registry_revision: 1,
                shader_registry_revision: 1,
                prepare_epoch: 1,
            },
            surface: PreparedSurfaceInfo::primary((800, 600)),
            views: vec![PreparedViewFrame::offscreen_product(
                "viewport.cache",
                (320, 180),
            )],
            flows: BTreeMap::new(),
            flow_invocations: vec![PreparedFlowInvocation {
                invocation_id: PreparedFlowInvocationId::new("viewport.cache.scene"),
                flow_id: compiled.flow_id,
                view_id: "viewport.cache".to_string(),
                inputs,
                target_alias_bindings: bindings,
                history_signature: Some("history:cache".to_string()),
            }],
            dynamic_texture_targets: vec![RenderDynamicTextureTargetDescriptor::color_sampled(
                key,
                320,
                180,
                RenderTextureTargetFormat::Rgba8Unorm,
                crate::plugins::render::RenderTextureSampleMode::FilterableFloat,
                RenderDynamicTextureRetention::RetainWhileRequested,
            )],
            dynamic_texture_uploads: Vec::new(),
            product_selections: Vec::new(),
            viewport_surface_bindings: ViewportSurfaceBindingRegistry::default(),
            contributions: PreparedFrameContributions::default(),
            shader: PreparedShaderSnapshot {
                registry_revision: 1,
            },
        }
    }

    fn alias_flow() -> CompiledRenderFlowPlan {
        let flow = RenderFlow::new("cache.alias")
            .with_color_target_alias("scene_color")
            .fullscreen_pass("draw")
            .offscreen_products_only()
            .write_target_alias("scene_color")
            .finish()
            .validate()
            .expect("alias flow should validate");
        compile_flow_plan(&flow).expect("alias flow should compile")
    }

    fn dispatch_flow() -> (CompiledRenderFlowPlan, RenderPassId) {
        let flow = RenderFlow::new("cache.dispatch")
            .with_color_target_alias("scene_color")
            .compute_pass("simulate")
            .dispatch([1, 1, 1])
            .finish()
            .fullscreen_pass("draw")
            .offscreen_products_only()
            .write_target_alias("scene_color")
            .depends_on("simulate")
            .finish()
            .validate()
            .expect("dispatch flow should validate");
        let compiled = compile_flow_plan(&flow).expect("dispatch flow should compile");
        let pass_id = match &compiled.execution.passes[0] {
            CompiledPassExecutionPlan::Compute(value) => value.pass_id,
            _ => panic!("first pass should be compute"),
        };
        (compiled, pass_id)
    }

    #[test]
    fn preflight_cache_reuses_successful_report_for_unchanged_frame_structure() {
        let compiled = alias_flow();
        let frame = prepared_frame(&compiled, PreparedFlowInputs::default());
        let mut renderer = Renderer::new();

        renderer
            .preflight_prepared_frame(
                &frame,
                std::slice::from_ref(&compiled),
                RenderPreflightValidationConfigResource::default(),
            )
            .expect("cold preflight should pass");
        assert_eq!(
            renderer.last_preflight_cache_state().status,
            RenderPreparedFramePreflightCacheStatus::ColdMiss
        );

        renderer
            .preflight_prepared_frame(
                &frame,
                std::slice::from_ref(&compiled),
                RenderPreflightValidationConfigResource::default(),
            )
            .expect("warm preflight should pass");
        assert_eq!(
            renderer.last_preflight_cache_state().status,
            RenderPreparedFramePreflightCacheStatus::Hit
        );
        assert_eq!(
            renderer.last_preflight_cache_state().report_source,
            RenderPreparedFramePreflightReportSource::CachedReport
        );
    }

    #[test]
    fn strict_preflight_config_forces_full_validation_without_cache_hit() {
        let compiled = alias_flow();
        let frame = prepared_frame(&compiled, PreparedFlowInputs::default());
        let mut renderer = Renderer::new();

        renderer
            .preflight_prepared_frame(
                &frame,
                std::slice::from_ref(&compiled),
                RenderPreflightValidationConfigResource::strict_every_frame(),
            )
            .expect("strict preflight should pass");
        renderer
            .preflight_prepared_frame(
                &frame,
                std::slice::from_ref(&compiled),
                RenderPreflightValidationConfigResource::strict_every_frame(),
            )
            .expect("strict preflight should pass again");

        assert_eq!(
            renderer.last_preflight_cache_state().status,
            RenderPreparedFramePreflightCacheStatus::StrictMode
        );
        assert_eq!(
            renderer.last_preflight_cache_state().report_source,
            RenderPreparedFramePreflightReportSource::FullValidation
        );
    }

    #[test]
    fn strict_preflight_env_override_is_pure_and_testable() {
        let config = RenderPreflightValidationConfigResource::default();

        assert_eq!(
            config.effective_mode_for_env(Some("1")),
            RenderPreparedFramePreflightMode::StrictEveryFrame
        );
        assert_eq!(
            config.effective_mode_for_env(Some("off")),
            RenderPreparedFramePreflightMode::CachedStrict
        );
        assert_eq!(
            config.effective_mode_for_env(None::<&str>),
            RenderPreparedFramePreflightMode::CachedStrict
        );
    }

    #[test]
    fn runtime_guard_rejects_invalid_dispatch_even_when_cache_key_matches() {
        let (compiled, pass_id) = dispatch_flow();
        let mut inputs = PreparedFlowInputs::default();
        inputs
            .projected_dispatch_workgroups
            .insert(pass_id, [1, 1, 1]);
        let frame = prepared_frame(&compiled, inputs.clone());
        let mut renderer = Renderer::new();

        renderer
            .preflight_prepared_frame(
                &frame,
                std::slice::from_ref(&compiled),
                RenderPreflightValidationConfigResource::default(),
            )
            .expect("valid dispatch should pass");

        inputs
            .projected_dispatch_workgroups
            .insert(pass_id, [0, 1, 1]);
        let invalid_frame = prepared_frame(&compiled, inputs);
        let error = renderer
            .preflight_prepared_frame(
                &invalid_frame,
                std::slice::from_ref(&compiled),
                RenderPreflightValidationConfigResource::default(),
            )
            .expect_err("zero dispatch should be rejected before cached report reuse");

        assert!(error.to_string().contains("invalid dispatch"));
        assert_eq!(
            renderer.last_preflight_cache_state().status,
            RenderPreparedFramePreflightCacheStatus::GuardRejected
        );
    }
}
