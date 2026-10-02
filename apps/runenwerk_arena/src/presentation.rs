use anyhow::{Context, Result, bail};
use engine::plugins::WorldRuntimeSet;
use engine::plugins::render::admission::{
    RenderRepresentationAvailabilityFact, RenderRepresentationAvailabilityState,
};
use engine::plugins::render::appearance::{RenderDiffuseMaterial, RenderDirectionalEmitter};
use engine::plugins::render::backend::RenderSurfaceId;
use engine::plugins::render::field_input::{
    RenderFieldSemanticInputBinding, RenderFieldSemanticInputGeneration,
    RenderFieldSemanticInputRequirement,
};
use engine::plugins::render::frame::{
    PreparedFlowInvocationRequest, PreparedRenderFrameRequestResource,
    PreparedRenderProductSelectionResource, RenderDeterministicFrameContribution,
    RenderDeterministicFrameContributionResource,
};
use engine::plugins::render::participation::{RenderMaterialAssignment, RenderObjectParticipation};
use engine::plugins::render::representation::{
    RENDER_FIELD_DISTANCE_PROTOCOL_REVISION, RENDER_ORIENTED_SURFACE_QUERY_PROTOCOL_REVISION,
    RENDER_SURFACE_QUERY_PROTOCOL_REVISION, RenderFieldDistanceGuarantee,
    RenderFieldDistanceProtocolEvidence, RenderOrientedSurfaceProtocolEvidence,
    RenderRefinementEvidence, RenderRepresentationId, RenderRepresentationRecord,
    RenderSurfaceProtocolEvidence,
};
use engine::plugins::render::request::{
    RenderObservationSpec, RenderOutputSpec, RenderOutputValue, RenderPerspectiveObservation,
    RenderRadiometricRepresentation, RenderRequest, RenderRequestedOutput, RenderResultTopology,
    RenderSamplingSupport, RenderSemanticTolerance,
};
use engine::plugins::render::runtime::{
    RenderDynamicTextureTargetRequestRegistryResource, RenderRuntimeSet,
};
use engine::plugins::render::scene::{
    RenderObjectId, RenderObjectState, RenderSceneSnapshot, RenderSceneStore, RenderSceneUpdate,
};
use engine::plugins::render::space_time::{
    RenderAffineTransform3, RenderHandedness, RenderObjectSpatialState, RenderObjectTemporalState,
    RenderSpaceSpec, RenderSpatialCoverage, RenderTemporalSupport, RenderTimeInterval,
    RenderTimePoint,
};
use engine::plugins::render::surface_input::{
    RenderSurfaceSemanticInput, RenderSurfaceSemanticInputBinding,
    RenderSurfaceSemanticInputGeneration, RenderSurfaceSemanticInputRequirement,
};
use engine::plugins::render::{
    AppRenderExt, RenderDynamicTextureRetention, RenderDynamicTextureTargetDescriptor,
    RenderDynamicTextureTargetKey, RenderFlow, RenderFlowId, RenderFrameProducerId,
    RenderTargetAliasKind, RenderTextureSampleMode, RenderTextureTargetFormat,
    RenderTextureTargetUsage,
};
use engine::plugins::world::build::WorldRuntimeSdfProductCatalogResource;
use engine::plugins::world::prepare::{
    PreparedWorldSdfFieldSource, PreparedWorldSdfFieldSourceResource,
};
use engine::prelude::{
    App, FixedTimeConfig, FixedTimeState, Plugin, PrimaryPresentationMetricsResource, Query,
    RenderPrepare, Res, ResMut, SimulationTick, SystemConfigExt, SystemMobilityExt, WorldMut,
};
use product::{
    ProductIdentity, ProductResidency, RenderProductSelection, RenderResidencyRequest,
    RenderSelectedProduct,
};
use runen_gpu::GpuBindingKey;
use world_sdf::FieldProductDescriptor;

use crate::arena::ARENA_FIELD_PRODUCT_ID;
use crate::player::{
    ArenaMovementConfig, ArenaPlayer, LOCAL_PARTICIPANT_ID, PlayerPhysicalHistory,
};

const ARENA_PRESENTATION_PRODUCER_RAW: u64 = 11_270;
const ARENA_PRESENTATION_VIEW_ID: &str = "runenwerk.arena.main";
const ARENA_PRESENTATION_FLOW_ID: &str = "runenwerk.arena.radiance";
const ARENA_PRESENTATION_PASS_ID: &str = "runenwerk.arena.radiance.display";
const ARENA_PRESENTATION_PRESENT_ID: &str = "runenwerk.arena.radiance.present";
const ARENA_RADIANCE_ALIAS: &str = "arena.radiance";
const ARENA_RADIANCE_TARGET_NAMESPACE: &str = "runenwerk.arena";
const ARENA_RADIANCE_TARGET_ID: &str = "radiance";
const ARENA_RADIANCE_SHADER: &str = "assets/shaders/runenwerk_arena_radiance.wgsl";
const ARENA_WAVELENGTH_METERS: f64 = 550.0e-9;
const ARENA_RADIANCE_TOLERANCE: f64 = 2.0e-4;
const ARENA_REFLECTANCE: f64 = 0.55;
const PLAYER_REFLECTANCE: f64 = 0.80;
const LIGHT_DIRECTION_TO_SOURCE: [f64; 3] = [0.45, 0.80, 0.35];
const LIGHT_SPECTRAL_IRRADIANCE_W_M3: f64 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArenaPresentationSnapshot {
    pub position_scene_meters: [f64; 3],
    pub interpolation_alpha: f32,
    pub source_tick: SimulationTick,
    pub presentation_time_seconds: f64,
}

#[derive(Debug, Clone, Default, PartialEq, runen_ecs::Resource)]
pub struct ArenaPresentationState {
    snapshot: Option<ArenaPresentationSnapshot>,
}

impl ArenaPresentationState {
    pub const fn snapshot(&self) -> Option<ArenaPresentationSnapshot> {
        self.snapshot
    }
}

#[derive(Debug, Clone, Copy, runen_ecs::Resource)]
struct ArenaPresentationFlowId(RenderFlowId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, runen_ecs::SystemSet)]
enum ArenaPresentationSet {
    Derive,
    ProductSelection,
    FramePublication,
}

#[derive(Debug, runen_ecs::Resource)]
pub struct ArenaPresentationSceneResource {
    store: RenderSceneStore,
    arena_object_id: RenderObjectId,
    arena_representation_id: RenderRepresentationId,
    player_object_id: RenderObjectId,
    player_representation_id: RenderRepresentationId,
    light_object_id: RenderObjectId,
}

impl ArenaPresentationSceneResource {
    fn new() -> Self {
        let mut store = RenderSceneStore::new();
        let arena_object_id = store
            .allocate_object_id()
            .expect("arena presentation object id allocation must succeed");
        let player_object_id = store
            .allocate_object_id()
            .expect("player presentation object id allocation must succeed");
        let light_object_id = store
            .allocate_object_id()
            .expect("light presentation object id allocation must succeed");
        let arena_representation_id = store
            .allocate_representation_id(arena_object_id)
            .expect("arena presentation representation id allocation must succeed");
        let player_representation_id = store
            .allocate_representation_id(player_object_id)
            .expect("player presentation representation id allocation must succeed");

        let mut insert = RenderSceneUpdate::new();
        insert
            .insert_with_state(
                arena_object_id,
                object_state(
                    RenderAffineTransform3::identity(),
                    RenderSpatialCoverage::unbounded(),
                ),
            )
            .insert_with_state(
                player_object_id,
                object_state(
                    RenderAffineTransform3::identity(),
                    RenderSpatialCoverage::unbounded(),
                ),
            )
            .insert_with_state(
                light_object_id,
                object_state(
                    RenderAffineTransform3::identity(),
                    RenderSpatialCoverage::unbounded(),
                ),
            );
        store
            .commit(insert)
            .expect("arena presentation scene bootstrap must be valid");

        Self {
            store,
            arena_object_id,
            arena_representation_id,
            player_object_id,
            player_representation_id,
            light_object_id,
        }
    }

    pub const fn arena_object_id(&self) -> RenderObjectId {
        self.arena_object_id
    }

    pub const fn arena_representation_id(&self) -> RenderRepresentationId {
        self.arena_representation_id
    }

    pub const fn player_object_id(&self) -> RenderObjectId {
        self.player_object_id
    }

    pub const fn player_representation_id(&self) -> RenderRepresentationId {
        self.player_representation_id
    }

    pub const fn light_object_id(&self) -> RenderObjectId {
        self.light_object_id
    }

    pub fn snapshot(&self) -> RenderSceneSnapshot {
        self.store.snapshot()
    }
}

pub struct ArenaPresentationPlugin;

impl Plugin for ArenaPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArenaPresentationState>();
        app.insert_resource(ArenaPresentationSceneResource::new());

        let flow = arena_radiance_flow();
        let flow_id = flow.id();
        app.add_render_flow(flow);
        app.insert_resource(ArenaPresentationFlowId(flow_id));

        app.add_systems(
            RenderPrepare,
            derive_arena_presentation_state_system
                .in_set(ArenaPresentationSet::Derive)
                .before(RenderRuntimeSet::FramePrepare),
        );
        app.add_systems(
            RenderPrepare,
            publish_arena_product_selection_system
                .on_invoker_thread()
                .in_set(ArenaPresentationSet::ProductSelection)
                .before(WorldRuntimeSet::RenderSdfBridge),
        );
        app.add_systems(
            RenderPrepare,
            publish_arena_frame_system
                .on_invoker_thread()
                .in_set(ArenaPresentationSet::FramePublication)
                .after(ArenaPresentationSet::Derive)
                .after(WorldRuntimeSet::RenderSdfFieldProjection)
                .before(RenderRuntimeSet::FramePrepare),
        );
    }
}

fn arena_radiance_flow() -> RenderFlow {
    RenderFlow::new(ARENA_PRESENTATION_FLOW_ID)
        .explicit_invocations_only()
        .with_target_alias(ARENA_RADIANCE_ALIAS, RenderTargetAliasKind::Texture)
        .expect("arena radiance alias must be valid")
        .with_surface_color()
        .expect("arena presentation surface color must be valid")
        .fullscreen_pass(ARENA_PRESENTATION_PASS_ID)
        .main_surface_only()
        .shader_asset(ARENA_RADIANCE_SHADER)
        .sample_texture_load(
            GpuBindingKey::try_new(0, 0).expect("arena radiance binding key must be valid"),
            ARENA_RADIANCE_ALIAS,
        )
        .clear_color([0.0, 0.0, 0.0, 1.0])
        .write_surface_color()
        .expect("arena radiance pass must target primary surface color")
        .finish()
        .present_pass(ARENA_PRESENTATION_PRESENT_ID)
        .expect("arena present pass must be valid")
        .main_surface_only()
        .surface_color()
        .expect("arena present pass must read surface color")
        .finish()
        .validate()
        .expect("arena presentation flow must validate")
}

fn presentation_producer_id() -> RenderFrameProducerId {
    RenderFrameProducerId::try_from_raw(ARENA_PRESENTATION_PRODUCER_RAW)
        .expect("arena presentation producer id must be non-zero")
}

fn derive_arena_presentation_state_system(
    fixed_config: Res<FixedTimeConfig>,
    fixed_state: Res<FixedTimeState>,
    tick: Res<SimulationTick>,
    mut query: Query<(&ArenaPlayer, &PlayerPhysicalHistory)>,
    mut presentation: ResMut<ArenaPresentationState>,
) {
    let Some(alpha) =
        interpolation_alpha(fixed_state.accumulator_seconds, fixed_config.step_seconds)
    else {
        presentation.snapshot = None;
        return;
    };
    let Some(history) = query.iter().find_map(|(player, history)| {
        (player.participant == LOCAL_PARTICIPANT_ID).then_some(*history)
    }) else {
        presentation.snapshot = None;
        return;
    };

    let position = interpolate_position(history, alpha);
    let completed_steps = fixed_state.total_completed_steps as f64;
    let presentation_time_seconds = ((completed_steps - 1.0 + f64::from(alpha))
        * f64::from(fixed_config.step_seconds))
    .max(0.0);

    presentation.snapshot = Some(ArenaPresentationSnapshot {
        position_scene_meters: position.map(f64::from),
        interpolation_alpha: alpha,
        source_tick: *tick,
        presentation_time_seconds,
    });
}

fn interpolation_alpha(accumulator_seconds: f32, step_seconds: f32) -> Option<f32> {
    if !accumulator_seconds.is_finite() || !step_seconds.is_finite() || step_seconds <= 0.0 {
        return None;
    }
    Some((accumulator_seconds / step_seconds).clamp(0.0, 1.0))
}

fn interpolate_position(history: PlayerPhysicalHistory, alpha: f32) -> [f32; 3] {
    std::array::from_fn(|axis| {
        history.previous.position[axis] * (1.0 - alpha) + history.current.position[axis] * alpha
    })
}

fn arena_field_descriptor(
    products: &WorldRuntimeSdfProductCatalogResource,
) -> Option<&FieldProductDescriptor> {
    products
        .products()
        .values()
        .find(|descriptor| descriptor.product_id == ARENA_FIELD_PRODUCT_ID)
}

fn arena_product_selection(descriptor: &FieldProductDescriptor) -> RenderProductSelection {
    let core = descriptor.product_core();
    RenderProductSelection::new(ARENA_PRESENTATION_VIEW_ID)
        .with_selected_product(RenderSelectedProduct {
            product_id: core.identity,
            scale_band: core.scale_band,
            generation: core.lineage.generation,
            freshness: core.freshness,
            residency: core.residency,
            authority_class: core.authority_class,
            query_policy: core.query_policy,
        })
        .with_residency_request(RenderResidencyRequest::new(
            core.identity,
            ProductResidency::Resident,
            100,
            true,
        ))
}

fn publish_arena_product_selection_system(mut world: WorldMut) -> Result<()> {
    let selection = {
        let products = world
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .context("ArenaPresentation requires the World SDF product catalog")?;
        arena_field_descriptor(products).map(arena_product_selection)
    };

    let selections = world
        .resource_mut::<PreparedRenderProductSelectionResource>()
        .context("ArenaPresentation requires Render product selection state")?;
    match selection {
        Some(selection) => {
            selections
                .replace_contribution(presentation_producer_id(), [selection])
                .context("publish maintained arena Render product selection")?;
        }
        None => {
            selections.remove_contribution(presentation_producer_id());
        }
    }
    Ok(())
}

fn publish_arena_frame_system(mut world: WorldMut) -> Result<()> {
    let presentation = world
        .resource::<ArenaPresentationState>()
        .context("ArenaPresentation state must exist")?
        .snapshot();
    let descriptor_core = {
        let products = world
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .context("ArenaPresentation requires the World SDF product catalog")?;
        arena_field_descriptor(products).map(FieldProductDescriptor::product_core)
    };
    let prepared_arena = descriptor_core.as_ref().and_then(|core| {
        world
            .resource::<PreparedWorldSdfFieldSourceResource>()
            .ok()?
            .source(core.identity)
            .cloned()
    });

    let (Some(presentation), Some(descriptor_core), Some(prepared_arena)) =
        (presentation, descriptor_core, prepared_arena)
    else {
        withdraw_arena_frame_publication(&mut world)?;
        return Ok(());
    };
    if prepared_arena.product_generation() != descriptor_core.lineage.generation {
        withdraw_arena_frame_publication(&mut world)?;
        return Ok(());
    }

    let movement = *world
        .resource::<ArenaMovementConfig>()
        .context("ArenaPresentation requires ArenaMovementConfig")?;
    let extent = world
        .resource::<PrimaryPresentationMetricsResource>()
        .context("ArenaPresentation requires primary presentation metrics")?
        .size_px();
    if extent.0 == 0 || extent.1 == 0 {
        withdraw_arena_frame_publication(&mut world)?;
        return Ok(());
    }
    let flow_id = world
        .resource::<ArenaPresentationFlowId>()
        .context("ArenaPresentation flow id must exist")?
        .0;

    let publication = {
        let scene = world
            .resource_mut::<ArenaPresentationSceneResource>()
            .context("ArenaPresentation scene authority must exist")?;
        build_arena_frame_publication(
            scene,
            presentation,
            &prepared_arena,
            descriptor_core.identity,
            movement,
            extent,
            flow_id,
        )?
    };

    stage_arena_frame_publication(&mut world, publication)
}

#[derive(Clone)]
struct ArenaFramePublication {
    target: RenderDynamicTextureTargetDescriptor,
    invocation: PreparedFlowInvocationRequest,
    contribution: RenderDeterministicFrameContribution,
}

fn build_arena_frame_publication(
    scene: &mut ArenaPresentationSceneResource,
    presentation: ArenaPresentationSnapshot,
    prepared_arena: &PreparedWorldSdfFieldSource,
    expected_product_identity: ProductIdentity,
    movement: ArenaMovementConfig,
    extent: (u32, u32),
    flow_id: RenderFlowId,
) -> Result<ArenaFramePublication> {
    if prepared_arena.product_id() != expected_product_identity {
        bail!("prepared arena field source has the wrong product identity");
    }

    let arena_input = prepared_arena.input();
    let arena_coverage = RenderSpatialCoverage::axis_aligned_bounds(
        arena_input.origin_local_meters(),
        arena_input.max_local_meters(),
    )
    .context("arena field bounds must be valid renderer coverage")?;
    let radius = f64::from(movement.character.radius);
    let player_coverage = RenderSpatialCoverage::axis_aligned_bounds(
        [-radius, -radius, -radius],
        [radius, radius, radius],
    )
    .context("player sphere bounds must be valid renderer coverage")?;

    let mut state_update = RenderSceneUpdate::new();
    state_update
        .replace_state(
            scene.arena_object_id,
            object_state(RenderAffineTransform3::identity(), arena_coverage.clone()),
        )
        .replace_state(
            scene.player_object_id,
            object_state(
                translation(presentation.position_scene_meters)?,
                translated_coverage(&player_coverage, presentation.position_scene_meters)?,
            ),
        );
    scene
        .store
        .commit(state_update)
        .context("commit arena presentation spatial state")?;

    let arena_field_evidence = RenderFieldDistanceProtocolEvidence::new(
        RENDER_FIELD_DISTANCE_PROTOCOL_REVISION,
        RenderFieldDistanceGuarantee::conservative(
            arena_input.max_absolute_query_error_local_meters(),
        )?,
    )?
    .with_semantic_input_requirement(RenderFieldSemanticInputRequirement::current());
    let arena_representation = RenderRepresentationRecord::new(
        scene.arena_representation_id,
        arena_coverage,
        RenderTemporalSupport::unbounded(),
        RenderRefinementEvidence::none(),
        None,
        Some(arena_field_evidence),
    )?;

    let player_surface_evidence =
        RenderSurfaceProtocolEvidence::exact(RENDER_SURFACE_QUERY_PROTOCOL_REVISION)?
            .with_oriented_surface(RenderOrientedSurfaceProtocolEvidence::exact(
                RENDER_ORIENTED_SURFACE_QUERY_PROTOCOL_REVISION,
            )?)
            .with_semantic_input_requirement(RenderSurfaceSemanticInputRequirement::current());
    let player_representation = RenderRepresentationRecord::new(
        scene.player_representation_id,
        player_coverage,
        RenderTemporalSupport::unbounded(),
        RenderRefinementEvidence::none(),
        Some(player_surface_evidence),
        None,
    )?;

    let mut participation_update = RenderSceneUpdate::new();
    participation_update
        .replace_participation(
            scene.arena_object_id,
            RenderObjectParticipation::new(
                vec![arena_representation],
                Some(RenderMaterialAssignment::new(RenderDiffuseMaterial::new(
                    ARENA_REFLECTANCE,
                )?)),
                None,
            )?,
        )
        .replace_participation(
            scene.player_object_id,
            RenderObjectParticipation::new(
                vec![player_representation],
                Some(RenderMaterialAssignment::new(RenderDiffuseMaterial::new(
                    PLAYER_REFLECTANCE,
                )?)),
                None,
            )?,
        )
        .replace_participation(
            scene.light_object_id,
            RenderObjectParticipation::new(
                Vec::new(),
                None,
                Some(RenderDirectionalEmitter::new(
                    LIGHT_DIRECTION_TO_SOURCE,
                    ARENA_WAVELENGTH_METERS,
                    LIGHT_SPECTRAL_IRRADIANCE_W_M3,
                )?),
            )?,
        );
    scene
        .store
        .commit(participation_update)
        .context("commit arena presentation representations and light")?;

    let shutter = RenderTimeInterval::instant(RenderTimePoint::from_seconds(
        presentation.presentation_time_seconds,
    )?);
    let observation = RenderObservationSpec::Perspective(RenderPerspectiveObservation::new(
        top_down_camera_transform(presentation.position_scene_meters)?,
        std::f64::consts::FRAC_PI_3,
        f64::from(extent.0) / f64::from(extent.1),
        shutter,
        RenderSamplingSupport::ideal_ray(),
    )?);
    let radiance =
        RenderRadiometricRepresentation::spectral_at_wavelength_meters(ARENA_WAVELENGTH_METERS)?;
    let output = RenderOutputSpec::new(
        RenderOutputValue::Radiance {
            representation: radiance,
        },
        RenderResultTopology::sample_lattice_2d(extent.0, extent.1)?,
        RenderSemanticTolerance::absolute(ARENA_RADIANCE_TOLERANCE)?,
    )?;
    let request = RenderRequest::new(
        shutter,
        vec![observation],
        vec![RenderRequestedOutput::new(0, output)],
    )?;

    let surface_binding = RenderSurfaceSemanticInputBinding::new(
        scene.player_representation_id,
        RenderSurfaceSemanticInput::sphere(
            [0.0, 0.0, 0.0],
            radius,
            RenderTemporalSupport::unbounded(),
        )?,
    )
    .with_generation(RenderSurfaceSemanticInputGeneration::new(
        presentation.source_tick.0,
    ));
    let field_binding =
        RenderFieldSemanticInputBinding::new(scene.arena_representation_id, arena_input.clone())
            .with_generation(RenderFieldSemanticInputGeneration::new(
                prepared_arena.product_generation(),
            ));
    let availability = vec![
        RenderRepresentationAvailabilityFact::new(
            scene.arena_representation_id,
            RenderRepresentationAvailabilityState::Available,
        ),
        RenderRepresentationAvailabilityFact::new(
            scene.player_representation_id,
            RenderRepresentationAvailabilityState::Available,
        ),
    ];

    let target_key = RenderDynamicTextureTargetKey::new(
        ARENA_RADIANCE_TARGET_NAMESPACE,
        ARENA_RADIANCE_TARGET_ID,
    );
    let target = RenderDynamicTextureTargetDescriptor::new(
        target_key.clone(),
        extent.0,
        extent.1,
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
    );
    let invocation = PreparedFlowInvocationRequest::new(
        format!("{ARENA_PRESENTATION_FLOW_ID}.main"),
        flow_id,
        "main",
    )
    .bind_dynamic_texture_alias(ARENA_RADIANCE_ALIAS, target_key.clone())?;

    Ok(ArenaFramePublication {
        target,
        invocation,
        contribution: RenderDeterministicFrameContribution {
            producer_id: presentation_producer_id(),
            render_surface_id: RenderSurfaceId::primary(),
            scene: scene.store.snapshot(),
            request,
            semantic_inputs: vec![surface_binding],
            field_semantic_inputs: vec![field_binding],
            availability,
            output_index: 0,
            target_key,
            finite_evaluation_extent: None,
        },
    })
}

fn stage_arena_frame_publication_resources(
    mut targets: RenderDynamicTextureTargetRequestRegistryResource,
    mut frame_requests: PreparedRenderFrameRequestResource,
    mut contributions: RenderDeterministicFrameContributionResource,
    publication: ArenaFramePublication,
) -> Result<(
    RenderDynamicTextureTargetRequestRegistryResource,
    PreparedRenderFrameRequestResource,
    RenderDeterministicFrameContributionResource,
)> {
    if publication.contribution.producer_id != presentation_producer_id()
        || publication.contribution.render_surface_id != RenderSurfaceId::primary()
    {
        bail!("arena frame publication lost its stable producer or primary-surface identity");
    }

    let producer_id = presentation_producer_id();
    targets
        .replace_surface_contribution(
            producer_id,
            RenderSurfaceId::primary(),
            [publication.target],
        )
        .context("stage arena radiance target")?;
    frame_requests
        .replace_surface_contribution_with_automatic_main_replacements(
            producer_id,
            RenderSurfaceId::primary(),
            [],
            [publication.invocation],
            [],
        )
        .context("stage arena display flow invocation")?;
    contributions.replace(publication.contribution);

    Ok((targets, frame_requests, contributions))
}

fn stage_arena_frame_publication(
    world: &mut WorldMut,
    publication: ArenaFramePublication,
) -> Result<()> {
    let targets = world
        .resource::<RenderDynamicTextureTargetRequestRegistryResource>()
        .context("ArenaPresentation requires Render dynamic target requests")?
        .clone();
    let frame_requests = world
        .resource::<PreparedRenderFrameRequestResource>()
        .context("ArenaPresentation requires prepared frame requests")?
        .clone();
    let contributions = world
        .resource::<RenderDeterministicFrameContributionResource>()
        .context("ArenaPresentation requires deterministic frame contributions")?
        .clone();
    let (targets, frame_requests, contributions) = stage_arena_frame_publication_resources(
        targets,
        frame_requests,
        contributions,
        publication,
    )?;

    *world
        .resource_mut::<RenderDynamicTextureTargetRequestRegistryResource>()
        .context("ArenaPresentation dynamic target authority disappeared")? = targets;
    *world
        .resource_mut::<PreparedRenderFrameRequestResource>()
        .context("ArenaPresentation frame request authority disappeared")? = frame_requests;
    *world
        .resource_mut::<RenderDeterministicFrameContributionResource>()
        .context("ArenaPresentation deterministic contribution authority disappeared")? =
        contributions;

    Ok(())
}

fn withdraw_arena_frame_publication(world: &mut WorldMut) -> Result<()> {
    let producer_id = presentation_producer_id();
    world
        .resource_mut::<RenderDynamicTextureTargetRequestRegistryResource>()
        .context("ArenaPresentation requires Render dynamic target requests")?
        .remove_contribution(producer_id);
    world
        .resource_mut::<PreparedRenderFrameRequestResource>()
        .context("ArenaPresentation requires prepared frame requests")?
        .remove_contribution(producer_id);
    world
        .resource_mut::<RenderDeterministicFrameContributionResource>()
        .context("ArenaPresentation requires deterministic frame contributions")?
        .remove(producer_id);
    Ok(())
}

fn object_state(
    local_to_scene: RenderAffineTransform3,
    scene_coverage: RenderSpatialCoverage,
) -> RenderObjectState {
    RenderObjectState::new(
        RenderObjectSpatialState::new(
            RenderSpaceSpec::new(1.0, RenderHandedness::Right)
                .expect("arena presentation uses metric renderer space"),
            local_to_scene,
            scene_coverage,
        ),
        RenderObjectTemporalState::new(RenderTemporalSupport::unbounded()),
    )
}

fn translation(position: [f64; 3]) -> Result<RenderAffineTransform3> {
    Ok(RenderAffineTransform3::from_row_major_3x4([
        1.0,
        0.0,
        0.0,
        position[0],
        0.0,
        1.0,
        0.0,
        position[1],
        0.0,
        0.0,
        1.0,
        position[2],
    ])?)
}

fn translated_coverage(
    local_coverage: &RenderSpatialCoverage,
    translation: [f64; 3],
) -> Result<RenderSpatialCoverage> {
    let Some((min, max)) = local_coverage.axis_aligned_bounds_value() else {
        return Ok(RenderSpatialCoverage::unbounded());
    };
    RenderSpatialCoverage::axis_aligned_bounds(
        std::array::from_fn(|axis| min[axis] + translation[axis]),
        std::array::from_fn(|axis| max[axis] + translation[axis]),
    )
    .context("translated player coverage must remain finite")
}

fn top_down_camera_transform(position: [f64; 3]) -> Result<RenderAffineTransform3> {
    let origin = [position[0], position[1] + 4.0, position[2]];
    Ok(RenderAffineTransform3::from_row_major_3x4([
        1.0, 0.0, 0.0, origin[0], 0.0, 0.0, 1.0, origin[1], 0.0, -1.0, 0.0, origin[2],
    ])?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::plugins::render::field_input::RenderFieldSemanticInput;

    fn history(previous: [f32; 3], current: [f32; 3]) -> PlayerPhysicalHistory {
        PlayerPhysicalHistory {
            previous: physics::CharacterPhysicalState {
                position: previous,
                ..Default::default()
            },
            current: physics::CharacterPhysicalState {
                position: current,
                ..Default::default()
            },
        }
    }

    fn prepared_field() -> PreparedWorldSdfFieldSource {
        let input = RenderFieldSemanticInput::dense(
            [0.0; 3],
            [0.5; 3],
            [9, 9, 9],
            vec![1.0; 9 * 9 * 9],
            0.125,
            RenderTemporalSupport::unbounded(),
        )
        .expect("test field");
        let payload = crate::arena::build_arena_chunk_payload();
        let descriptor = crate::arena::arena_field_product_descriptor(&payload);
        PreparedWorldSdfFieldSource::new(
            descriptor.product_core().identity,
            7,
            crate::arena::arena_metric_payload_ref(&payload).payload_ref,
            input,
        )
    }

    #[test]
    fn product_selection_copies_exact_arena_descriptor_facts_and_requests_residency() {
        let payload = crate::arena::build_arena_chunk_payload();
        let descriptor = crate::arena::arena_field_product_descriptor(&payload);
        let core = descriptor.product_core();

        let selection = arena_product_selection(&descriptor);

        assert_eq!(selection.view_id, ARENA_PRESENTATION_VIEW_ID);
        assert_eq!(selection.selected_products.len(), 1);
        let selected = &selection.selected_products[0];
        assert_eq!(descriptor.product_id, ARENA_FIELD_PRODUCT_ID);
        assert_eq!(selected.product_id, core.identity);
        assert_eq!(selected.scale_band, core.scale_band);
        assert_eq!(selected.generation, core.lineage.generation);
        assert_eq!(selected.freshness, core.freshness);
        assert_eq!(selected.residency, core.residency);
        assert_eq!(selected.authority_class, core.authority_class);
        assert_eq!(selected.query_policy, core.query_policy);
        assert_eq!(selection.residency_requests.len(), 1);
        let residency = &selection.residency_requests[0];
        assert_eq!(residency.product_id, core.identity);
        assert_eq!(residency.residency, ProductResidency::Resident);
        assert!(residency.hard_pin);
    }

    #[test]
    fn real_gp1b3_projection_is_the_arena_field_binding() {
        use engine::plugins::render::RenderPlugin;
        use engine::prelude::AppFixedStepExt;

        let mut app = crate::build_headless_game_app();
        app.add_plugin(RenderPlugin);
        app = app
            .run_for_fixed_steps(1)
            .expect("arena truth should integrate before projection");

        let descriptor = app
            .world()
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .expect("arena product catalog")
            .products()
            .values()
            .find(|descriptor| descriptor.product_id == ARENA_FIELD_PRODUCT_ID)
            .expect("arena descriptor")
            .clone();
        app.world_mut()
            .resource_mut::<PreparedRenderProductSelectionResource>()
            .expect("Render product selection")
            .replace_contribution(
                presentation_producer_id(),
                [arena_product_selection(&descriptor)],
            )
            .expect("arena selection");

        app = app
            .run_for_frames(1)
            .expect("real GP1B3 projection should run");

        let prepared = app
            .world()
            .resource::<PreparedWorldSdfFieldSourceResource>()
            .expect("prepared field source resource")
            .source(descriptor.product_core().identity)
            .expect("real arena GP1B3 projection")
            .clone();
        let history = crate::player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID)
            .expect("local player history");
        let mut scene = ArenaPresentationSceneResource::new();
        let publication = build_arena_frame_publication(
            &mut scene,
            ArenaPresentationSnapshot {
                position_scene_meters: history.current.position.map(f64::from),
                interpolation_alpha: 1.0,
                source_tick: *app
                    .world()
                    .resource::<SimulationTick>()
                    .expect("simulation tick"),
                presentation_time_seconds: 0.0,
            },
            &prepared,
            descriptor.product_core().identity,
            *app.world()
                .resource::<ArenaMovementConfig>()
                .expect("movement config"),
            (1280, 720),
            arena_radiance_flow().id(),
        )
        .expect("game presentation should consume real GP1B3 input");

        assert_eq!(publication.contribution.field_semantic_inputs.len(), 1);
        assert_eq!(
            publication.contribution.field_semantic_inputs[0].input(),
            prepared.input()
        );
        assert_eq!(
            publication.contribution.field_semantic_inputs[0].generation(),
            Some(RenderFieldSemanticInputGeneration::new(
                prepared.product_generation()
            ))
        );
    }

    #[test]
    fn interpolation_clamps_remainder_and_preserves_source_history() {
        let history = history([1.0, 2.0, 3.0], [5.0, 6.0, 7.0]);
        let before = history;

        assert_eq!(interpolation_alpha(-1.0, 1.0), Some(0.0));
        assert_eq!(interpolation_alpha(0.25, 1.0), Some(0.25));
        assert_eq!(interpolation_alpha(2.0, 1.0), Some(1.0));
        assert_eq!(interpolation_alpha(1.0, 0.0), None);
        assert_eq!(interpolate_position(history, 0.25), [2.0, 3.0, 4.0]);
        assert_eq!(history, before);
    }

    #[test]
    fn top_down_camera_uses_exact_follow_basis() {
        let matrix = top_down_camera_transform([2.0, 0.75, 3.0])
            .expect("camera transform")
            .row_major_3x4();
        assert_eq!(
            matrix,
            [1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 1.0, 4.75, 0.0, -1.0, 0.0, 3.0]
        );
    }

    #[test]
    fn frame_publication_uses_stable_ids_real_field_and_both_semantic_input_families() {
        let mut scene = ArenaPresentationSceneResource::new();
        let ids = (
            scene.arena_object_id(),
            scene.arena_representation_id(),
            scene.player_object_id(),
            scene.player_representation_id(),
            scene.light_object_id(),
        );
        let prepared = prepared_field();
        let movement = ArenaMovementConfig::default();
        let snapshot = ArenaPresentationSnapshot {
            position_scene_meters: [2.0, 0.75, 2.0],
            interpolation_alpha: 0.5,
            source_tick: SimulationTick(4),
            presentation_time_seconds: 0.05,
        };
        let flow_id = arena_radiance_flow().id();

        let publication =
            build_arena_frame_publication(
                &mut scene,
                snapshot,
                &prepared,
                crate::arena::arena_field_product_descriptor(
                    &crate::arena::build_arena_chunk_payload(),
                )
                .product_core()
                .identity,
                movement,
                (1280, 720),
                flow_id,
            )
            .expect("arena frame publication");

        assert_eq!(publication.invocation.view_id, "main");
        assert_eq!(publication.contribution.semantic_inputs.len(), 1);
        assert_eq!(publication.contribution.field_semantic_inputs.len(), 1);
        assert_eq!(
            publication.contribution.semantic_inputs[0].representation_id(),
            ids.3
        );
        assert_eq!(
            publication.contribution.field_semantic_inputs[0].representation_id(),
            ids.1
        );
        assert_eq!(
            publication.contribution.field_semantic_inputs[0].input(),
            prepared.input()
        );
        assert_eq!(
            (
                scene.arena_object_id(),
                scene.arena_representation_id(),
                scene.player_object_id(),
                scene.player_representation_id(),
                scene.light_object_id(),
            ),
            ids
        );

        let arena = publication
            .contribution
            .scene
            .object_participation(ids.0)
            .expect("arena participation");
        let arena_representation = arena.representation(ids.1).expect("arena representation");
        assert_eq!(
            arena_representation
                .spatial_coverage()
                .axis_aligned_bounds_value(),
            Some((
                prepared.input().origin_local_meters(),
                prepared.input().max_local_meters()
            ))
        );
        assert_eq!(
            arena_representation
                .refinement()
                .finest_absolute_error_meters(),
            None
        );
        assert_eq!(
            arena_representation
                .field_distance_protocol(RENDER_FIELD_DISTANCE_PROTOCOL_REVISION)
                .expect("field protocol")
                .guarantee()
                .max_absolute_error_meters(),
            prepared.input().max_absolute_query_error_local_meters()
        );

        let player = publication
            .contribution
            .scene
            .object_participation(ids.2)
            .expect("player participation");
        assert_eq!(
            player
                .material_assignment()
                .expect("player material")
                .material()
                .reflectance(),
            PLAYER_REFLECTANCE
        );
        let light = publication
            .contribution
            .scene
            .object_participation(ids.4)
            .expect("light participation")
            .emitter()
            .expect("directional emitter");
        assert_eq!(light.wavelength_meters(), ARENA_WAVELENGTH_METERS);
        assert_eq!(
            light.spectral_irradiance_w_m3(),
            LIGHT_SPECTRAL_IRRADIANCE_W_M3
        );
        let direction_length = LIGHT_DIRECTION_TO_SOURCE
            .iter()
            .map(|component| component * component)
            .sum::<f64>()
            .sqrt();
        assert_eq!(
            light.direction_to_source_scene(),
            LIGHT_DIRECTION_TO_SOURCE.map(|component| component / direction_length)
        );

        let arena_material = arena
            .material_assignment()
            .expect("arena material")
            .material();
        assert_eq!(arena_material.reflectance(), ARENA_REFLECTANCE);

        let player_representation = player
            .representation(ids.3)
            .expect("player representation");
        let radius = f64::from(movement.character.radius);
        assert_eq!(
            player_representation
                .spatial_coverage()
                .axis_aligned_bounds_value(),
            Some(([-radius; 3], [radius; 3]))
        );
        assert_eq!(
            publication.contribution.semantic_inputs[0].input(),
            &RenderSurfaceSemanticInput::sphere(
                [0.0; 3],
                radius,
                RenderTemporalSupport::unbounded(),
            )
            .expect("expected player sphere")
        );
        assert_eq!(
            publication
                .contribution
                .scene
                .object_state(ids.2)
                .expect("player object state")
                .spatial()
                .local_to_scene()
                .row_major_3x4(),
            translation(snapshot.position_scene_meters)
                .expect("player translation")
                .row_major_3x4()
        );

        let RenderObservationSpec::Perspective(observation) =
            publication.contribution.request.observations()[0]
        else {
            panic!("arena presentation must use one perspective observation");
        };
        assert_eq!(observation.aspect_ratio(), 1280.0 / 720.0);
        assert!(observation.sampling_support().is_ideal_ray());
        assert_eq!(
            observation.observation_to_scene().row_major_3x4(),
            top_down_camera_transform(snapshot.position_scene_meters)
                .expect("camera transform")
                .row_major_3x4()
        );
        let output = publication.contribution.request.outputs()[0].spec();
        assert_eq!(output.topology().sample_lattice_dimensions(), Some((1280, 720)));
        assert_eq!(
            output.tolerance().absolute_max_error(),
            Some(ARENA_RADIANCE_TOLERANCE)
        );
        match output.value() {
            RenderOutputValue::Radiance { representation } => {
                assert_eq!(representation.wavelength_meters(), ARENA_WAVELENGTH_METERS);
            }
            other => panic!("arena output must be radiance, got {other:?}"),
        }

        assert_eq!(publication.target.width, 1280);
        assert_eq!(publication.target.height, 720);
        assert_eq!(publication.target.format, RenderTextureTargetFormat::R32Float);
        assert!(publication.target.usage.sampled);
        assert!(publication.target.usage.copy_dst);
        assert_eq!(publication.invocation.view_id, "main");
        assert_eq!(publication.invocation.flow_id, flow_id);
    }

    #[test]
    fn frame_publication_replacement_is_atomic_on_cross_producer_request_collision() {
        let mut scene = ArenaPresentationSceneResource::new();
        let prepared = prepared_field();
        let descriptor = crate::arena::arena_field_product_descriptor(
            &crate::arena::build_arena_chunk_payload(),
        );
        let movement = ArenaMovementConfig::default();
        let flow_id = arena_radiance_flow().id();
        let snapshot = ArenaPresentationSnapshot {
            position_scene_meters: [1.5, 0.75, 1.5],
            interpolation_alpha: 0.25,
            source_tick: SimulationTick(7),
            presentation_time_seconds: 0.1,
        };
        let old_publication = build_arena_frame_publication(
            &mut scene,
            snapshot,
            &prepared,
            descriptor.product_core().identity,
            movement,
            (1280, 720),
            flow_id,
        )
        .expect("old arena publication");

        let (targets, mut frame_requests, contributions) =
            stage_arena_frame_publication_resources(
                RenderDynamicTextureTargetRequestRegistryResource::default(),
                PreparedRenderFrameRequestResource::default(),
                RenderDeterministicFrameContributionResource::default(),
                old_publication,
            )
            .expect("initial publication should stage");

        let foreign_producer =
            RenderFrameProducerId::try_from_raw(99_1127).expect("foreign producer id");
        frame_requests
            .replace_surface_contribution_with_automatic_main_replacements(
                foreign_producer,
                RenderSurfaceId::primary(),
                [],
                [PreparedFlowInvocationRequest::new(
                    "foreign.arena.collision",
                    flow_id,
                    "main",
                )],
                [],
            )
            .expect("foreign request should stage before collision");

        let before_targets = targets.snapshot_for_surface(RenderSurfaceId::primary());
        let before_invocations = frame_requests
            .requested_flow_invocations_for_surface(RenderSurfaceId::primary())
            .into_iter()
            .map(|request| request.invocation_id.clone())
            .collect::<Vec<_>>();
        let before_contributions = contributions.clone().take_all();

        let mut replacement = build_arena_frame_publication(
            &mut scene,
            ArenaPresentationSnapshot {
                position_scene_meters: [2.0, 0.75, 2.0],
                ..snapshot
            },
            &prepared,
            descriptor.product_core().identity,
            movement,
            (1280, 720),
            flow_id,
        )
        .expect("replacement arena publication");
        replacement.invocation = PreparedFlowInvocationRequest::new(
            "foreign.arena.collision",
            flow_id,
            "main",
        )
        .bind_dynamic_texture_alias(ARENA_RADIANCE_ALIAS, replacement.target.key.clone())
        .expect("replacement alias binding");

        let error = stage_arena_frame_publication_resources(
            targets.clone(),
            frame_requests.clone(),
            contributions.clone(),
            replacement,
        )
        .expect_err("cross-producer invocation collision must reject the whole replacement");
        assert!(
            error.to_string().contains("invocation"),
            "collision should report the request boundary: {error:#}"
        );

        assert_eq!(
            targets.snapshot_for_surface(RenderSurfaceId::primary()),
            before_targets
        );
        assert_eq!(
            frame_requests
                .requested_flow_invocations_for_surface(RenderSurfaceId::primary())
                .into_iter()
                .map(|request| request.invocation_id.clone())
                .collect::<Vec<_>>(),
            before_invocations
        );
        assert_eq!(contributions.clone().take_all(), before_contributions);
        assert!(
            frame_requests
                .requested_flow_invocations_for_surface(RenderSurfaceId::primary())
                .iter()
                .any(|request| request.invocation_id.0 == "foreign.arena.collision"),
            "unrelated producer publication must survive the rejected game replacement"
        );
    }
}
