use super::*;

use engine::plugins::ActionState;
use engine::plugins::render::inspect::{
    CaptureStage, CaptureTextureClass, RenderCaptureSelector, RenderTextureDiffRequest,
};
use engine::plugins::render::{GpuParams, GpuUniform, RenderDeterministicFiniteEvaluationExtent};
use runen_gpu::GpuBindingKey;
use runen_input::PhysicalKeyIdentity;
use std::path::PathBuf;

pub(super) const RL2_COMPARISON_FLOW_ID: &str = "runenwerk.render_lab.rl2.compare";
pub(super) const RL2_COMPARISON_VISUALIZE_A_PASS_ID: &str =
    "runenwerk.render_lab.rl2.compare.visualize_a";
pub(super) const RL2_COMPARISON_VISUALIZE_B_PASS_ID: &str =
    "runenwerk.render_lab.rl2.compare.visualize_b";
pub(super) const RL2_COMPARISON_PASS_ID: &str = "runenwerk.render_lab.rl2.compare.compose";
pub(super) const RL2_COMPARISON_PRESENT_ID: &str = "runenwerk.render_lab.rl2.compare.present";
pub(super) const RL2_COMPARISON_RAW_A_ALIAS: &str = "rl2.compare.raw_a";
pub(super) const RL2_COMPARISON_RAW_B_ALIAS: &str = "rl2.compare.raw_b";
pub(super) const RL2_COMPARISON_DISPLAY_A_ALIAS: &str = "rl2.compare.display_a";
pub(super) const RL2_COMPARISON_DISPLAY_B_ALIAS: &str = "rl2.compare.display_b";

const RL2_COMPARISON_TARGET_NAMESPACE: &str = "runenwerk.render_lab.rl2.compare";
const RL2_COMPARISON_RAW_A_TARGET_ID: &str = "raw_a";
const RL2_COMPARISON_RAW_B_TARGET_ID: &str = "raw_b";
const RL2_COMPARISON_DISPLAY_A_TARGET_ID: &str = "display_a";
const RL2_COMPARISON_DISPLAY_B_TARGET_ID: &str = "display_b";

const RL2_COMPARISON_PRODUCER_A_ID: u64 = 6893;
const RL2_COMPARISON_PRODUCER_B_ID: u64 = 6894;
const RL2_COMPARISON_PRODUCT_PRODUCER_ID: u64 = 6895;

const ACTION_COMPARISON_SPLIT: &str = "render_lab.compare.split";
const ACTION_COMPARISON_FULL_A: &str = "render_lab.compare.full_a";
const ACTION_COMPARISON_FULL_B: &str = "render_lab.compare.full_b";
const ACTION_COMPARISON_FLIP: &str = "render_lab.compare.flip";

const COMPARISON_DIVIDER_HIT_FRACTION: f32 = 0.00625;
const COMPARISON_DIVIDER_HIT_MIN_PX: f32 = 8.0;
const COMPARISON_DIVIDER_HIT_MAX_PX: f32 = 24.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenderLabComparisonMode {
    Split,
    FullA,
    FullB,
}

impl RenderLabComparisonMode {
    const fn uniform_code(self) -> f32 {
        match self {
            Self::Split => 0.0,
            Self::FullA => 1.0,
            Self::FullB => 2.0,
        }
    }

    const fn flipped(self) -> Self {
        match self {
            Self::Split | Self::FullB => Self::FullA,
            Self::FullA => Self::FullB,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, runen_ecs::Resource)]
pub(super) struct RenderLabComparisonState {
    active: bool,
    candidate_size_px: (u32, u32),
    mode: RenderLabComparisonMode,
    divider_normalized: f32,
    divider_drag_active: bool,
    divider_drag_captured_this_frame: bool,
}

impl Default for RenderLabComparisonState {
    fn default() -> Self {
        Self {
            active: false,
            candidate_size_px: (1, 1),
            mode: RenderLabComparisonMode::Split,
            divider_normalized: 0.5,
            divider_drag_active: false,
            divider_drag_captured_this_frame: false,
        }
    }
}

#[derive(Debug, Clone, Default, runen_ecs::Resource)]
pub(super) struct RenderLabComparisonEvidenceConfig {
    output_root: Option<PathBuf>,
    submitted_frame_limit: Option<usize>,
    capture_selectors: Vec<RenderCaptureSelector>,
    texture_diff: Option<RenderTextureDiffRequest>,
    completed: bool,
}

impl RenderLabComparisonEvidenceConfig {
    pub(super) fn bounded(
        output_root: impl Into<PathBuf>,
        submitted_frame_limit: usize,
        flow: &RenderFlow,
    ) -> Result<Self> {
        if submitted_frame_limit == 0 {
            bail!("Render Lab comparison evidence requires a positive submitted-frame limit");
        }
        let flow_id = flow.id().to_string();
        let selector = |pass_label: &str, resource_id: &str| -> Result<RenderCaptureSelector> {
            let pass_id = flow.pass_id(pass_label).ok_or_else(|| {
                anyhow::anyhow!(
                    "comparison evidence pass '{pass_label}' is absent from flow '{}'",
                    flow.label()
                )
            })?;
            Ok(RenderCaptureSelector {
                flow_id: Some(flow_id.clone()),
                pass_id: Some(pass_id.to_string()),
                stage: CaptureStage::After,
                resource_id: resource_id.to_string(),
                texture_class: CaptureTextureClass::ColorTarget,
            })
        };
        let left = selector(
            RL2_COMPARISON_VISUALIZE_A_PASS_ID,
            RL2_COMPARISON_DISPLAY_A_ALIAS,
        )?;
        let right = selector(
            RL2_COMPARISON_VISUALIZE_B_PASS_ID,
            RL2_COMPARISON_DISPLAY_B_ALIAS,
        )?;
        Ok(Self {
            output_root: Some(output_root.into()),
            submitted_frame_limit: Some(submitted_frame_limit),
            capture_selectors: vec![left.clone(), right.clone()],
            texture_diff: Some(RenderTextureDiffRequest::new(
                "runenwerk.render_lab.rl2.compare.a_vs_b",
                left,
                right,
            )),
            completed: false,
        })
    }

    pub(super) fn is_active(&self) -> bool {
        self.output_root.is_some()
    }

    pub(super) fn capture_should_arm(&self, completed_submissions: usize) -> bool {
        self.is_active()
            && !self.completed
            && self
                .submitted_frame_limit
                .is_some_and(|limit| completed_submissions.saturating_add(1) == limit)
    }

    pub(super) fn awaiting_capture_completion(&self, completed_submissions: usize) -> bool {
        self.is_active()
            && !self.completed
            && self
                .submitted_frame_limit
                .is_some_and(|limit| completed_submissions >= limit)
    }

    pub(super) fn submitted_frame_limit(&self) -> Option<usize> {
        self.submitted_frame_limit
    }

    pub(super) fn capture_selectors(&self) -> &[RenderCaptureSelector] {
        &self.capture_selectors
    }

    pub(super) fn texture_diff(&self) -> Option<&RenderTextureDiffRequest> {
        self.texture_diff.as_ref()
    }

    pub(super) fn artifact_output_dir(&self) -> Option<PathBuf> {
        self.output_root
            .as_deref()
            .map(|root| root.join("captures"))
    }

    pub(super) fn output_root(&self) -> Option<&std::path::Path> {
        self.output_root.as_deref()
    }

    pub(super) fn mark_completed(&mut self) {
        self.completed = true;
    }
}

impl RenderLabComparisonState {
    pub(super) fn active(candidate_size_px: (u32, u32)) -> Self {
        Self {
            active: true,
            candidate_size_px,
            ..Self::default()
        }
    }

    pub(super) const fn is_active(&self) -> bool {
        self.active
    }

    pub(super) const fn candidate_size_px(&self) -> (u32, u32) {
        self.candidate_size_px
    }

    pub(super) const fn suppress_left_camera_drag(&self) -> bool {
        self.active && (self.divider_drag_active || self.divider_drag_captured_this_frame)
    }

    fn params(&self) -> RenderLabComparisonParams {
        RenderLabComparisonParams {
            control: [
                self.mode.uniform_code(),
                self.divider_normalized.clamp(0.0, 1.0),
                0.0,
                0.0,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RenderLabComparisonParams {
    control: [f32; 4],
}

impl GpuParams for RenderLabComparisonParams {
    type Raw = [f32; 4];

    fn to_gpu(&self) -> Self::Raw {
        self.control
    }
}

impl GpuUniform for RenderLabComparisonParams {}

pub(super) struct RenderLabComparisonPublication {
    pub producer_a: engine::plugins::render::RenderFrameProducerId,
    pub producer_b: engine::plugins::render::RenderFrameProducerId,
    pub product_producer: engine::plugins::render::RenderFrameProducerId,
    pub raw_a_target: RenderDynamicTextureTargetDescriptor,
    pub raw_b_target: RenderDynamicTextureTargetDescriptor,
    pub display_a_target: RenderDynamicTextureTargetDescriptor,
    pub display_b_target: RenderDynamicTextureTargetDescriptor,
    pub invocation: PreparedFlowInvocationRequest,
    pub contribution_a: RenderDeterministicFrameContribution,
    pub contribution_b: RenderDeterministicFrameContribution,
}

pub(super) fn install_render_lab_comparison_bindings(app: &mut App) {
    use engine::prelude::AppActionBindingsExt;

    app.add_input_bindings([
        (ACTION_COMPARISON_SPLIT, PhysicalKeyIdentity::code("F6")),
        (ACTION_COMPARISON_FULL_A, PhysicalKeyIdentity::code("F7")),
        (ACTION_COMPARISON_FULL_B, PhysicalKeyIdentity::code("F8")),
        (ACTION_COMPARISON_FLIP, PhysicalKeyIdentity::code("F9")),
    ]);
}

pub(super) fn update_render_lab_comparison_system(
    input: Res<InputState>,
    actions: Res<ActionState>,
    presentation: Res<engine::PrimaryPresentationMetricsResource>,
    mut state: ResMut<RenderLabComparisonState>,
) {
    if !state.active {
        return;
    }

    state.divider_drag_captured_this_frame = state.divider_drag_active;

    if actions.action_pressed(ACTION_COMPARISON_SPLIT) {
        state.mode = RenderLabComparisonMode::Split;
        state.divider_drag_active = false;
    }
    if actions.action_pressed(ACTION_COMPARISON_FULL_A) {
        state.mode = RenderLabComparisonMode::FullA;
        state.divider_drag_active = false;
    }
    if actions.action_pressed(ACTION_COMPARISON_FULL_B) {
        state.mode = RenderLabComparisonMode::FullB;
        state.divider_drag_active = false;
    }
    if actions.action_pressed(ACTION_COMPARISON_FLIP) {
        state.mode = state.mode.flipped();
        state.divider_drag_active = false;
    }

    if state.mode == RenderLabComparisonMode::Split {
        apply_divider_pointer_input(&mut state, &input, presentation.size_px().0);
    } else {
        state.divider_drag_active = false;
    }
}

fn apply_divider_pointer_input(
    state: &mut RenderLabComparisonState,
    input: &InputState,
    width_px: u32,
) {
    let width = width_px.max(1) as f32;
    let hit_band = (width * COMPARISON_DIVIDER_HIT_FRACTION)
        .clamp(COMPARISON_DIVIDER_HIT_MIN_PX, COMPARISON_DIVIDER_HIT_MAX_PX);
    let motions = input.mouse_motion_samples();
    let transitions = input.mouse_button_transitions();
    let mut transition_index = 0usize;

    for motion_index in 0..=motions.len() {
        while transitions
            .get(transition_index)
            .is_some_and(|transition| transition.motion_sample_index == motion_index)
        {
            let transition = transitions[transition_index];
            if transition.is_left_pressed() && !state.divider_drag_active {
                let divider_x = state.divider_normalized * width;
                if (transition.position.0 - divider_x).abs() <= hit_band {
                    state.divider_drag_active = true;
                    state.divider_drag_captured_this_frame = true;
                    state.divider_normalized = normalized_divider_x(transition.position.0, width);
                }
            } else if transition.is_left_released() {
                state.divider_drag_active = false;
            }
            transition_index += 1;
        }

        if state.divider_drag_active
            && let Some(motion) = motions.get(motion_index)
        {
            state.divider_drag_captured_this_frame = true;
            state.divider_normalized = normalized_divider_x(motion.position.0, width);
        }
    }

    if state.divider_drag_active && motions.is_empty() && input.mouse_delta.0 != 0.0 {
        state.divider_normalized =
            (state.divider_normalized + input.mouse_delta.0 / width).clamp(0.0, 1.0);
    }
}

fn normalized_divider_x(x: f32, width: f32) -> f32 {
    (x / width.max(1.0)).clamp(0.0, 1.0)
}

pub(super) fn render_lab_comparison_flow() -> Result<RenderFlow> {
    RenderFlow::new(RL2_COMPARISON_FLOW_ID)
        .explicit_invocations_only()
        .with_state::<RenderLabComparisonState>()
        .with_target_alias(RL2_COMPARISON_RAW_A_ALIAS, RenderTargetAliasKind::Texture)?
        .with_target_alias(RL2_COMPARISON_RAW_B_ALIAS, RenderTargetAliasKind::Texture)?
        .with_color_target_alias(RL2_COMPARISON_DISPLAY_A_ALIAS)?
        .with_color_target_alias(RL2_COMPARISON_DISPLAY_B_ALIAS)?
        .with_surface_color()?
        .fullscreen_pass(RL2_COMPARISON_VISUALIZE_A_PASS_ID)
        .main_surface_only()
        .shader_asset("assets/shaders/runenwerk_render_lab_radiance.wgsl")
        .sample_texture_load(GpuBindingKey::try_new(0, 0)?, RL2_COMPARISON_RAW_A_ALIAS)
        .clear_color([0.0, 0.0, 0.0, 1.0])
        .write_target_alias(RL2_COMPARISON_DISPLAY_A_ALIAS)
        .finish()
        .fullscreen_pass(RL2_COMPARISON_VISUALIZE_B_PASS_ID)
        .main_surface_only()
        .shader_asset("assets/shaders/runenwerk_render_lab_radiance.wgsl")
        .sample_texture_load(GpuBindingKey::try_new(0, 0)?, RL2_COMPARISON_RAW_B_ALIAS)
        .clear_color([0.0, 0.0, 0.0, 1.0])
        .write_target_alias(RL2_COMPARISON_DISPLAY_B_ALIAS)
        .finish()
        .fullscreen_pass(RL2_COMPARISON_PASS_ID)
        .main_surface_only()
        .shader_asset("assets/shaders/runenwerk_render_lab_comparison.wgsl")
        .uniform_from_state(
            GpuBindingKey::try_new(0, 0)?,
            RenderLabComparisonState::params,
        )?
        .sample_texture_load(
            GpuBindingKey::try_new(0, 1)?,
            RL2_COMPARISON_DISPLAY_A_ALIAS,
        )
        .sample_texture_load(
            GpuBindingKey::try_new(0, 2)?,
            RL2_COMPARISON_DISPLAY_B_ALIAS,
        )
        .clear_color([0.0, 0.0, 0.0, 1.0])
        .write_surface_color()?
        .finish()
        .present_pass(RL2_COMPARISON_PRESENT_ID)?
        .main_surface_only()
        .surface_color()?
        .finish()
        .validate()
}

pub(super) fn build_render_lab_comparison_publication(
    camera: &RenderLabCamera,
    flow_id: engine::plugins::render::RenderFlowId,
    output_size_px: (u32, u32),
    candidate_size_px: (u32, u32),
) -> Result<RenderLabComparisonPublication> {
    validate_comparison_extents(output_size_px, candidate_size_px)?;

    let producer_a =
        engine::plugins::render::RenderFrameProducerId::try_from_raw(RL2_COMPARISON_PRODUCER_A_ID)
            .expect("comparison producer A id is non-zero");
    let producer_b =
        engine::plugins::render::RenderFrameProducerId::try_from_raw(RL2_COMPARISON_PRODUCER_B_ID)
            .expect("comparison producer B id is non-zero");
    let product_producer = engine::plugins::render::RenderFrameProducerId::try_from_raw(
        RL2_COMPARISON_PRODUCT_PRODUCER_ID,
    )
    .expect("comparison product producer id is non-zero");

    let raw_a_key = RenderDynamicTextureTargetKey::new(
        RL2_COMPARISON_TARGET_NAMESPACE,
        RL2_COMPARISON_RAW_A_TARGET_ID,
    );
    let raw_b_key = RenderDynamicTextureTargetKey::new(
        RL2_COMPARISON_TARGET_NAMESPACE,
        RL2_COMPARISON_RAW_B_TARGET_ID,
    );
    let display_a_key = RenderDynamicTextureTargetKey::new(
        RL2_COMPARISON_TARGET_NAMESPACE,
        RL2_COMPARISON_DISPLAY_A_TARGET_ID,
    );
    let display_b_key = RenderDynamicTextureTargetKey::new(
        RL2_COMPARISON_TARGET_NAMESPACE,
        RL2_COMPARISON_DISPLAY_B_TARGET_ID,
    );

    let raw_target = |key: RenderDynamicTextureTargetKey| {
        RenderDynamicTextureTargetDescriptor::new(
            key,
            output_size_px.0,
            output_size_px.1,
            RenderTextureTargetFormat::R32Float,
            RenderTextureTargetUsage {
                color_attachment: false,
                depth_attachment: false,
                sampled: true,
                storage: false,
                copy_src: false,
                copy_dst: true,
            },
            RenderTextureSampleMode::NonFilterableFloat,
            RenderDynamicTextureRetention::RetainWhileRequested,
        )
    };
    let display_target = |key: RenderDynamicTextureTargetKey| {
        RenderDynamicTextureTargetDescriptor::color_sampled(
            key,
            output_size_px.0,
            output_size_px.1,
            RenderTextureTargetFormat::Rgba8UnormSrgb,
            RenderTextureSampleMode::NonFilterableFloat,
            RenderDynamicTextureRetention::RetainWhileRequested,
        )
    };

    let fixture = founding_footprint_fixture_with_observation_and_extent(
        camera.observation_to_scene(),
        output_size_px.0,
        output_size_px.1,
    )?;
    let evaluation_a =
        RenderDeterministicFiniteEvaluationExtent::new(output_size_px.0, output_size_px.1)
            .ok_or_else(|| anyhow::anyhow!("comparison P100 evaluation extent must be non-zero"))?;
    let evaluation_b =
        RenderDeterministicFiniteEvaluationExtent::new(candidate_size_px.0, candidate_size_px.1)
            .ok_or_else(|| {
                anyhow::anyhow!("comparison candidate evaluation extent must be non-zero")
            })?;

    let contribution_a = RenderDeterministicFrameContribution {
        producer_id: producer_a,
        render_surface_id: RenderSurfaceId::primary(),
        scene: fixture.scene.clone(),
        request: fixture.request.clone(),
        semantic_inputs: fixture.semantic_inputs.clone(),
        availability: fixture.availability.clone(),
        output_index: 0,
        target_key: raw_a_key.clone(),
        finite_evaluation_extent: Some(evaluation_a),
    };
    let contribution_b = RenderDeterministicFrameContribution {
        producer_id: producer_b,
        render_surface_id: RenderSurfaceId::primary(),
        scene: fixture.scene,
        request: fixture.request,
        semantic_inputs: fixture.semantic_inputs,
        availability: fixture.availability,
        output_index: 0,
        target_key: raw_b_key.clone(),
        finite_evaluation_extent: Some(evaluation_b),
    };

    let invocation = PreparedFlowInvocationRequest::new(
        format!("{RL2_COMPARISON_FLOW_ID}.main"),
        flow_id,
        "main",
    )
    .bind_dynamic_texture_alias(RL2_COMPARISON_RAW_A_ALIAS, raw_a_key.clone())?
    .bind_dynamic_texture_alias(RL2_COMPARISON_RAW_B_ALIAS, raw_b_key.clone())?
    .bind_dynamic_texture_alias(RL2_COMPARISON_DISPLAY_A_ALIAS, display_a_key.clone())?
    .bind_dynamic_texture_alias(RL2_COMPARISON_DISPLAY_B_ALIAS, display_b_key.clone())?;

    Ok(RenderLabComparisonPublication {
        producer_a,
        producer_b,
        product_producer,
        raw_a_target: raw_target(raw_a_key),
        raw_b_target: raw_target(raw_b_key),
        display_a_target: display_target(display_a_key),
        display_b_target: display_target(display_b_key),
        invocation,
        contribution_a,
        contribution_b,
    })
}

pub(super) fn stage_render_lab_comparison_publication(
    targets: &mut RenderDynamicTextureTargetRequestRegistryResource,
    frame_requests: &mut PreparedRenderFrameRequestResource,
    contributions: &mut RenderDeterministicFrameContributionResource,
    publication: RenderLabComparisonPublication,
) -> Result<()> {
    let RenderLabComparisonPublication {
        producer_a,
        producer_b,
        product_producer,
        raw_a_target,
        raw_b_target,
        display_a_target,
        display_b_target,
        invocation,
        contribution_a,
        contribution_b,
    } = publication;

    if contribution_a.render_surface_id != RenderSurfaceId::primary()
        || contribution_b.render_surface_id != RenderSurfaceId::primary()
    {
        bail!("Render Lab comparison supports exactly the primary render surface");
    }
    if contribution_a.producer_id != producer_a || contribution_b.producer_id != producer_b {
        bail!("Render Lab comparison contribution producer identity changed during staging");
    }

    let mut staged_targets = targets.clone();
    staged_targets.replace_surface_contribution(
        producer_a,
        RenderSurfaceId::primary(),
        [raw_a_target],
    )?;
    staged_targets.replace_surface_contribution(
        producer_b,
        RenderSurfaceId::primary(),
        [raw_b_target],
    )?;
    staged_targets.replace_surface_contribution(
        product_producer,
        RenderSurfaceId::primary(),
        [display_a_target, display_b_target],
    )?;

    let mut staged_frame_requests = frame_requests.clone();
    staged_frame_requests.replace_surface_contribution_with_automatic_main_replacements(
        product_producer,
        RenderSurfaceId::primary(),
        [],
        [invocation],
        [],
    )?;

    let mut staged_contributions = contributions.clone();
    staged_contributions.replace(contribution_a);
    staged_contributions.replace(contribution_b);

    *targets = staged_targets;
    *frame_requests = staged_frame_requests;
    *contributions = staged_contributions;
    Ok(())
}

pub(super) fn validate_comparison_extents(
    output_size_px: (u32, u32),
    candidate_size_px: (u32, u32),
) -> Result<()> {
    let (output_width, output_height) = output_size_px;
    let (candidate_width, candidate_height) = candidate_size_px;
    if output_width == 0 || output_height == 0 || candidate_width == 0 || candidate_height == 0 {
        bail!("Render Lab comparison extents must be non-zero");
    }
    if candidate_width >= output_width || candidate_height >= output_height {
        bail!(
            "Render Lab comparison candidate {}x{} must be strictly sub-native to {}x{}",
            candidate_width,
            candidate_height,
            output_width,
            output_height
        );
    }
    if u64::from(candidate_width) * u64::from(output_height)
        != u64::from(candidate_height) * u64::from(output_width)
    {
        bail!(
            "Render Lab comparison candidate {}x{} must preserve output aspect {}x{}",
            candidate_width,
            candidate_height,
            output_width,
            output_height
        );
    }
    if candidate_width.saturating_mul(2) < output_width
        || candidate_height.saturating_mul(2) < output_height
    {
        bail!(
            "Render Lab comparison candidate {}x{} is below the accepted temporal half-resolution bound for {}x{}",
            candidate_width,
            candidate_height,
            output_width,
            output_height
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_evidence_targets_distinct_visualization_outputs() {
        let flow = render_lab_comparison_flow().expect("comparison flow should author");
        let config = RenderLabComparisonEvidenceConfig::bounded("evidence/compare", 6, &flow)
            .expect("comparison evidence config should build");
        assert!(config.is_active());
        assert_eq!(config.submitted_frame_limit(), Some(6));
        assert_eq!(config.capture_selectors().len(), 2);
        assert_ne!(
            config.capture_selectors()[0].pass_id,
            config.capture_selectors()[1].pass_id
        );
        assert_eq!(
            config.capture_selectors()[0].resource_id,
            RL2_COMPARISON_DISPLAY_A_ALIAS
        );
        assert_eq!(
            config.capture_selectors()[1].resource_id,
            RL2_COMPARISON_DISPLAY_B_ALIAS
        );
        assert!(config.texture_diff().is_some());
        assert!(!config.capture_should_arm(4));
        assert!(config.capture_should_arm(5));
        assert!(config.awaiting_capture_completion(6));
    }

    #[test]
    fn comparison_extent_accepts_supported_sub_native_sizes() {
        validate_comparison_extents((1920, 1080), (1440, 810)).unwrap();
        validate_comparison_extents((1920, 1080), (1280, 720)).unwrap();
        validate_comparison_extents((1920, 1080), (960, 540)).unwrap();
        assert!(validate_comparison_extents((1920, 1080), (1920, 1080)).is_err());
        assert!(validate_comparison_extents((1920, 1080), (640, 360)).is_err());
        assert!(validate_comparison_extents((1920, 1080), (1280, 800)).is_err());
    }

    #[test]
    fn comparison_flow_is_explicit_and_has_two_visualizers_then_compare_and_present() {
        let flow = render_lab_comparison_flow().expect("comparison flow should author");
        assert_eq!(
            flow.invocation_policy(),
            engine::plugins::render::RenderFlowInvocationPolicy::ExplicitOnly
        );
        let lexical = flow
            .lexical_pass_order()
            .expect("comparison flow should compile");
        assert_eq!(lexical.len(), 4);
        let compiled = engine::plugins::render::compile_flow_plan(&flow)
            .expect("comparison flow should compile");
        assert_eq!(compiled.render_passes.len(), 4);
        assert_eq!(
            compiled.render_passes[0].pass_id(),
            flow.pass_id(RL2_COMPARISON_VISUALIZE_A_PASS_ID)
                .expect("visualize A pass")
        );
        assert_eq!(
            compiled.render_passes[1].pass_id(),
            flow.pass_id(RL2_COMPARISON_VISUALIZE_B_PASS_ID)
                .expect("visualize B pass")
        );
        assert_eq!(
            compiled.render_passes[2].pass_id(),
            flow.pass_id(RL2_COMPARISON_PASS_ID).expect("compare pass")
        );

        let visualize_a = compiled.render_passes[0].node();
        let visualize_b = compiled.render_passes[1].node();
        assert_eq!(visualize_a.shader, visualize_b.shader);
        assert_eq!(visualize_a.sampled_textures.len(), 1);
        assert_eq!(visualize_b.sampled_textures.len(), 1);
        assert_eq!(
            compiled
                .resource_label(visualize_a.sampled_textures[0])
                .expect("visualize A sampled resource"),
            RL2_COMPARISON_RAW_A_ALIAS
        );
        assert_eq!(
            compiled
                .resource_label(visualize_b.sampled_textures[0])
                .expect("visualize B sampled resource"),
            RL2_COMPARISON_RAW_B_ALIAS
        );

        let compare = compiled.render_passes[2].node();
        assert_eq!(compare.sampled_textures.len(), 2);
        assert_eq!(
            compare
                .sampled_textures
                .iter()
                .map(|resource| {
                    compiled
                        .resource_label(*resource)
                        .expect("sampled resource")
                })
                .collect::<Vec<_>>(),
            vec![
                RL2_COMPARISON_DISPLAY_A_ALIAS.to_string(),
                RL2_COMPARISON_DISPLAY_B_ALIAS.to_string(),
            ]
        );
        assert_eq!(compare.shader_bindings.len(), 3);
        assert!(matches!(
            compare.shader_bindings[0].resource(),
            engine::plugins::render::RenderShaderBindingResource::UniformBuffer(_)
        ));
        for binding in &compare.shader_bindings[1..] {
            assert!(matches!(
                binding.resource(),
                engine::plugins::render::RenderShaderBindingResource::SampledTexture {
                    sample_class: runen_gpu::GpuTextureSampleClass::FloatUnfilterable,
                    ..
                }
            ));
        }
    }

    #[test]
    fn comparison_publication_clones_one_semantic_truth_into_distinct_producers() {
        let flow = render_lab_comparison_flow().expect("comparison flow");
        let publication = build_render_lab_comparison_publication(
            &RenderLabCamera::default(),
            flow.id(),
            (1920, 1080),
            (960, 540),
        )
        .expect("comparison publication");

        assert_eq!(
            publication.contribution_a.scene,
            publication.contribution_b.scene
        );
        assert_eq!(
            publication.contribution_a.request,
            publication.contribution_b.request
        );
        assert_eq!(
            publication.contribution_a.semantic_inputs,
            publication.contribution_b.semantic_inputs
        );
        assert_eq!(
            publication.contribution_a.availability,
            publication.contribution_b.availability
        );
        assert_eq!(
            publication.contribution_a.output_index,
            publication.contribution_b.output_index
        );
        assert_ne!(
            publication.contribution_a.producer_id,
            publication.contribution_b.producer_id
        );
        assert_ne!(
            publication.contribution_a.target_key,
            publication.contribution_b.target_key
        );
        assert_eq!(
            publication.contribution_a.render_surface_id,
            publication.contribution_b.render_surface_id
        );
        assert_eq!(
            publication.contribution_a.render_surface_id,
            RenderSurfaceId::primary()
        );
        assert_eq!(
            publication
                .contribution_a
                .finite_evaluation_extent
                .expect("P100 evaluation")
                .dimensions(),
            (1920, 1080)
        );
        assert_eq!(
            publication
                .contribution_b
                .finite_evaluation_extent
                .expect("candidate evaluation")
                .dimensions(),
            (960, 540)
        );
        assert_ne!(
            publication.display_a_target.key,
            publication.display_b_target.key
        );
        for display in [&publication.display_a_target, &publication.display_b_target] {
            assert_eq!((display.width, display.height), (1920, 1080));
            assert_eq!(display.format, RenderTextureTargetFormat::Rgba8UnormSrgb);
            assert_eq!(display.usage, RenderTextureTargetUsage::color_sampled());
            assert_eq!(
                display.sample_mode,
                RenderTextureSampleMode::NonFilterableFloat
            );
        }
        assert_eq!(
            publication.raw_a_target.format,
            RenderTextureTargetFormat::R32Float
        );
        assert_eq!(
            publication.raw_b_target.format,
            RenderTextureTargetFormat::R32Float
        );
    }

    #[test]
    fn divider_capture_only_starts_inside_hit_band_and_clamps_motion() {
        let mut state = RenderLabComparisonState::active((960, 540));
        let mut input = InputState::new();
        input.handle_cursor_moved(960.0, 200.0);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        input.handle_cursor_moved(1200.0, 200.0);
        apply_divider_pointer_input(&mut state, &input, 1920);
        assert!(state.divider_drag_active);
        assert!((state.divider_normalized - 0.625).abs() < 1.0e-6);

        input.clear_frame();
        input.handle_cursor_moved(4000.0, 200.0);
        apply_divider_pointer_input(&mut state, &input, 1920);
        assert_eq!(state.divider_normalized, 1.0);

        input.clear_frame();
        input.handle_mouse_input(
            winit::event::ElementState::Released,
            winit::event::MouseButton::Left,
        );
        apply_divider_pointer_input(&mut state, &input, 1920);
        assert!(!state.divider_drag_active);

        let mut missed = RenderLabComparisonState::active((960, 540));
        let mut miss_input = InputState::new();
        miss_input.handle_cursor_moved(100.0, 200.0);
        miss_input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        miss_input.handle_cursor_moved(400.0, 200.0);
        apply_divider_pointer_input(&mut missed, &miss_input, 1920);
        assert!(!missed.divider_drag_active);
        assert_eq!(missed.divider_normalized, 0.5);
    }

    #[test]
    fn same_frame_divider_release_still_suppresses_camera_orbit() {
        let mut state = RenderLabComparisonState::active((960, 540));
        let mut input = InputState::new();
        input.handle_cursor_moved(960.0, 200.0);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        input.handle_cursor_moved(1200.0, 200.0);
        input.handle_mouse_input(
            winit::event::ElementState::Released,
            winit::event::MouseButton::Left,
        );
        apply_divider_pointer_input(&mut state, &input, 1920);
        assert!(!state.divider_drag_active);
        assert!(state.suppress_left_camera_drag());
        let mut camera = RenderLabCamera::default();
        crate::camera::update_render_lab_camera(
            &input,
            &mut camera,
            state.suppress_left_camera_drag(),
        );
        assert_eq!(camera, RenderLabCamera::default());
    }

    #[test]
    fn hidden_variants_keep_distinct_publications_across_modes_and_flip() {
        let flow = render_lab_comparison_flow().expect("comparison flow");
        let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
        let mut requests = PreparedRenderFrameRequestResource::default();
        let mut contributions = RenderDeterministicFrameContributionResource::default();
        let mut state = RenderLabComparisonState::active((960, 540));
        let mut stable_identity = None;
        for mode in [
            RenderLabComparisonMode::Split,
            RenderLabComparisonMode::FullA,
            RenderLabComparisonMode::FullB,
            RenderLabComparisonMode::FullA.flipped(),
        ] {
            state.mode = mode;
            let publication = build_render_lab_comparison_publication(
                &RenderLabCamera::default(),
                flow.id(),
                (1920, 1080),
                state.candidate_size_px(),
            )
            .expect("comparison publication");
            let a = publication.producer_a;
            let b = publication.producer_b;
            let a_key = publication.contribution_a.target_key.clone();
            let b_key = publication.contribution_b.target_key.clone();
            let identity = (a, b, a_key.clone(), b_key.clone());
            if let Some(expected) = stable_identity.as_ref() {
                assert_eq!(&identity, expected);
            } else {
                stable_identity = Some(identity);
            }
            stage_render_lab_comparison_publication(
                &mut targets,
                &mut requests,
                &mut contributions,
                publication,
            )
            .expect("both hidden and visible variants stage");
            assert_ne!(a, b);
            assert_ne!(a_key, b_key);
            let mut staged = contributions.clone();
            assert_eq!(staged.remove(a).expect("A contribution").target_key, a_key);
            assert_eq!(staged.remove(b).expect("B contribution").target_key, b_key);
            assert_eq!(targets.snapshot().len(), 4);
            assert_eq!(requests.requested_flow_invocations().len(), 1);
        }
    }

    #[test]
    fn colliding_comparison_targets_and_invocations_preserve_complete_state() {
        let flow = render_lab_comparison_flow().expect("comparison flow");
        let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
        let mut requests = PreparedRenderFrameRequestResource::default();
        let mut contributions = RenderDeterministicFrameContributionResource::default();
        let build = || {
            build_render_lab_comparison_publication(
                &RenderLabCamera::default(),
                flow.id(),
                (1920, 1080),
                (960, 540),
            )
            .expect("comparison publication")
        };
        let initial = build();
        let producer_a = initial.producer_a;
        let producer_b = initial.producer_b;
        stage_render_lab_comparison_publication(
            &mut targets,
            &mut requests,
            &mut contributions,
            initial,
        )
        .expect("initial complete state");
        let original_targets = targets.snapshot();
        let original_contributions = contributions.clone().take_all();
        assert_eq!(original_contributions.len(), 2);
        assert_eq!(original_contributions[0].producer_id, producer_a);
        assert_eq!(original_contributions[1].producer_id, producer_b);
        let original_invocations = requests
            .requested_flow_invocations()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();

        let mut collision = build();
        collision.raw_b_target.key = collision.raw_a_target.key.clone();
        assert!(
            stage_render_lab_comparison_publication(
                &mut targets,
                &mut requests,
                &mut contributions,
                collision
            )
            .is_err()
        );
        assert_eq!(targets.snapshot(), original_targets);
        assert_eq!(contributions.clone().take_all(), original_contributions);
        assert_eq!(
            requests
                .requested_flow_invocations()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
            original_invocations
        );

        let mut collision = build();
        collision.invocation = PreparedFlowInvocationRequest::new("collision", flow.id(), "main");
        let foreign = engine::plugins::render::RenderFrameProducerId::try_from_raw(8000).unwrap();
        requests
            .replace_contribution(
                foreign,
                [],
                [PreparedFlowInvocationRequest::new(
                    "collision",
                    flow.id(),
                    "main",
                )],
            )
            .expect("foreign invocation");
        let before = requests
            .requested_flow_invocations()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            stage_render_lab_comparison_publication(
                &mut targets,
                &mut requests,
                &mut contributions,
                collision
            )
            .is_err()
        );
        assert_eq!(targets.snapshot(), original_targets);
        assert_eq!(contributions.clone().take_all(), original_contributions);
        assert_eq!(
            requests
                .requested_flow_invocations()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
            before
        );
    }

    #[test]
    fn comparison_params_preserve_split_and_full_modes() {
        let mut state = RenderLabComparisonState::active((960, 540));
        assert_eq!(state.params().control, [0.0, 0.5, 0.0, 0.0]);
        state.mode = RenderLabComparisonMode::FullA;
        assert_eq!(state.params().control[0], 1.0);
        state.mode = RenderLabComparisonMode::FullB;
        assert_eq!(state.params().control[0], 2.0);
    }

    #[test]
    fn comparison_flip_alternates_full_views_without_touching_divider() {
        let mut state = RenderLabComparisonState::active((960, 540));
        state.mode = RenderLabComparisonMode::FullA;
        let divider = state.divider_normalized;
        state.mode = state.mode.flipped();
        assert_eq!(state.mode, RenderLabComparisonMode::FullB);
        assert_eq!(state.divider_normalized, divider);
        state.mode = state.mode.flipped();
        assert_eq!(state.mode, RenderLabComparisonMode::FullA);
        assert_eq!(state.divider_normalized, divider);
    }
}
