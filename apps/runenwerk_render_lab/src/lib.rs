//! The Runenwerk Render Lab's first headless product scenario.
//!
//! `founding-direct` is deliberately a product-sized one-shot command. It owns the fixture,
//! execution policy, independent reference oracle, spectral visualization, and evidence bundle;
//! RunenRender and RunenGPU retain their respective semantic and physical authorities.

use anyhow::{Context, Result, bail};
use engine::plugins::render::RenderResult;
use engine::plugins::render::admission::{
    RenderOutputBinding, RenderOutputDestination, RenderRepresentationAvailabilityFact,
    RenderRepresentationAvailabilityState,
};
use engine::plugins::render::appearance::{RenderDiffuseMaterial, RenderDirectionalEmitter};
use engine::plugins::render::deterministic_admission::admit_deterministic_render;
use engine::plugins::render::deterministic_execution::{
    RenderCapturedDeterministicRadiance, SubmittedDeterministicRender,
    submit_deterministic_render_for_verified_result,
};
use engine::plugins::render::participation::{RenderMaterialAssignment, RenderObjectParticipation};
use engine::plugins::render::representation::{
    RENDER_FIELD_DISTANCE_PROTOCOL_REVISION, RENDER_ORIENTED_SURFACE_QUERY_PROTOCOL_REVISION,
    RENDER_SURFACE_QUERY_PROTOCOL_REVISION, RenderFieldDistanceGuarantee,
    RenderFieldDistanceProtocolEvidence, RenderOrientedSurfaceProtocolEvidence,
    RenderRefinementEvidence, RenderRepresentationRecord, RenderSurfaceProtocolEvidence,
};
use engine::plugins::render::request::{
    RenderObservationSpec, RenderOutputSpec, RenderOutputValue, RenderPerspectiveObservation,
    RenderRadiometricRepresentation, RenderRequest, RenderRequestedOutput, RenderResultTopology,
    RenderSamplingSupport, RenderSemanticTolerance,
};
use engine::plugins::render::scene::{
    RenderObjectState, RenderSceneSnapshot, RenderSceneStore, RenderSceneUpdate,
};
use engine::plugins::render::space_time::{
    RenderAffineTransform3, RenderHandedness, RenderObjectSpatialState, RenderObjectTemporalState,
    RenderSpaceSpec, RenderSpatialCoverage, RenderTemporalSupport, RenderTimeInterval,
    RenderTimePoint,
};
use engine::plugins::render::surface_input::{
    RenderSurfaceSemanticInput, RenderSurfaceSemanticInputBinding,
    RenderSurfaceSemanticInputRequirement,
};
use image::{GrayImage, ImageFormat};
use runen_gpu::{
    GpuCapabilityProfile, GpuContext, GpuContextDescriptor, GpuContextRequestErrorCategory,
    GpuFormatRole, GpuReadbackOperation, GpuReconstruction, GpuResourceLifetime, GpuSubmission,
    GpuSubmissionStatus, GpuTextureDescriptor, GpuTextureFormat, GpuTextureInitialization,
    GpuTextureUsage, GpuWorkFragment, GpuWorkResourceIdAllocator,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const SCENARIO_ID: &str = "founding-direct";
pub const SCENARIO_REVISION: u32 = 1;
pub const WIDTH: u32 = 128;
pub const HEIGHT: u32 = 128;
pub const WAVELENGTH_METERS: f64 = 550.0e-9;
pub const FIXED_EXPOSURE: f32 = 0.25;
pub const ORACLE_TOLERANCE: f64 = 2.0e-4;
const SPHERE_REFLECTANCE: f64 = 0.72;
const PLANE_REFLECTANCE: f64 = 0.48;
const LIGHT_DIRECTION: [f64; 3] = [0.45, 0.80, 0.35];
const LIGHT_IRRADIANCE: f64 = 12.0;
const SPHERE_CENTER: [f64; 3] = [0.0, 0.0, -3.0];
const SPHERE_RADIUS: f64 = 1.0;
const PLANE_POINT: [f64; 3] = [0.0, -1.0, 0.0];
const PLANE_NORMAL: [f64; 3] = [0.0, 1.0, 0.0];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Classification {
    Pass,
    Fail,
    Inconclusive,
}

impl Classification {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Inconclusive => "inconclusive",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OracleSummary {
    pub sample_count: usize,
    pub mismatch_count: usize,
    pub maximum_absolute_error: f64,
    pub tolerance: f64,
    pub result: Classification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPaths {
    pub directory: PathBuf,
    pub radiance_png: PathBuf,
    pub evidence_json: PathBuf,
}

#[derive(Debug, Serialize)]
struct Evidence {
    evidence_schema_revision: u32,
    scenario_id: &'static str,
    scenario_revision: u32,
    source_git_revision: Option<String>,
    width: u32,
    height: u32,
    spectral_wavelength_meters: f64,
    fixed_exposure: f32,
    visualization_mapping: &'static str,
    fixture: FixtureEvidence,
    render_result: RenderResultEvidence,
    execution_path: &'static str,
    renderer_completion: &'static str,
    retained_continuity: &'static str,
    product_readback: &'static str,
    radiance_capture_interpretation: &'static str,
    field_backed_participation: FieldParticipationEvidence,
    oracle: OracleEvidence,
    png_path: &'static str,
    png_blake3: String,
    classification: &'static str,
    classification_reason: &'static str,
}

#[derive(Debug, Serialize)]
struct FixtureEvidence {
    shutter: &'static str,
    observation: &'static str,
    observation_basis: &'static str,
    geometry: &'static str,
    illumination: &'static str,
    output: &'static str,
}

#[derive(Debug, Serialize)]
struct RenderResultEvidence {
    scene_revision: String,
    output_count: usize,
    output_index: usize,
    output_topology: &'static str,
    semantic_result_formed: bool,
}

#[derive(Debug, Serialize)]
struct FieldParticipationEvidence {
    participated: bool,
    representation_contract: &'static str,
    traversal_claim: &'static str,
}

#[derive(Debug, Serialize)]
struct OracleEvidence {
    sample_count: usize,
    mismatch_count: usize,
    maximum_absolute_error: f64,
    tolerance: f64,
    result: &'static str,
}

struct FoundingFixture {
    scene: RenderSceneSnapshot,
    request: RenderRequest,
    semantic_inputs: Vec<RenderSurfaceSemanticInputBinding>,
    availability: Vec<RenderRepresentationAvailabilityFact>,
}

#[derive(Clone, Copy)]
struct ReferenceHit {
    t: f64,
    normal: [f64; 3],
    reflectance: f64,
}

pub fn run_founding_direct(output_root: impl AsRef<Path>) -> Result<ArtifactPaths> {
    let output_root = output_root.as_ref();
    let artifact_directory = output_root.join(SCENARIO_ID);
    fs::create_dir_all(&artifact_directory).with_context(|| {
        format!(
            "create Render Lab artifact directory {}",
            artifact_directory.display()
        )
    })?;

    let context = request_context()?;
    let fixture = founding_fixture()?;
    let destination = output_destination()?;
    let output_bindings = [RenderOutputBinding::new(
        0,
        RenderOutputDestination::SampleLatticeTexture(destination),
    )];
    let admitted = admit_deterministic_render(
        &fixture.scene,
        &fixture.request,
        &fixture.semantic_inputs,
        &fixture.availability,
        &output_bindings,
        &context,
    )
    .context("admit founding-direct through the maintained deterministic renderer")?;
    let mut submitted = pollster::block_on(submit_deterministic_render_for_verified_result(
        admitted, &context,
    ))
    .context("submit founding-direct through public RunenRender and RunenGPU")?;
    let result = form_result(&context, &mut submitted)?;

    let capture_request = submitted
        .request_deterministic_radiance_capture(0)
        .context("mint the formed-result founding-direct radiance capture request")?;
    let readback = GpuReadbackOperation::new(
        capture_request.source().clone(),
        capture_request.readback_id(),
    )
    .context("author the separate product-owned public radiance readback")?;
    let fragment = GpuWorkFragment::build("Render Lab founding-direct readback", |work| {
        work.operation("read back retained radiance lattice", readback)?;
        Ok(())
    })
    .context("build the product-owned radiance readback work")?;
    let product_submission =
        pollster::block_on(context.submit_work("Render Lab founding-direct readback", [fragment]))
            .context("submit the product-owned radiance readback")?;
    wait_for_readback(&context, &product_submission, capture_request.readback_id())?;
    let captured = submitted
        .capture_deterministic_radiance(capture_request, &context, &product_submission)
        .context("interpret the exact product readback through RunenRender #627")?;
    let field_backed_participation = field_participation(&result);
    if !field_backed_participation.participated {
        bail!("founding-direct result did not retain field-backed representation participation");
    }

    let oracle = compare_with_oracle(&captured);
    if oracle.result != Classification::Pass {
        for (index, actual) in captured
            .samples()
            .iter()
            .copied()
            .enumerate()
            .filter(|(index, actual)| {
                (f64::from(*actual) - reference_radiance(*index)).abs() > ORACLE_TOLERANCE
            })
            .take(8)
        {
            eprintln!(
                "founding-direct oracle mismatch at ({}, {}): actual={} expected={}",
                index % usize::try_from(WIDTH).expect("width fits usize"),
                index / usize::try_from(WIDTH).expect("width fits usize"),
                actual,
                reference_radiance(index),
            );
        }
        bail!(
            "founding-direct independent oracle rejected the captured radiance lattice: {oracle:?}"
        );
    }
    let pixels = map_to_grayscale(captured.samples())?;
    validate_pixels(&pixels)?;
    let radiance_png = artifact_directory.join("radiance.png");
    write_png(&radiance_png, &pixels)?;
    let encoded = image::open(&radiance_png).context("decode encoded radiance PNG")?;
    if encoded.width() != WIDTH || encoded.height() != HEIGHT {
        bail!("encoded radiance PNG dimensions do not match the founding lattice");
    }
    validate_pixels(&encoded.to_luma8().into_raw())?;
    let png_bytes = fs::read(&radiance_png).context("read encoded radiance PNG")?;
    let png_checksum = blake3::hash(&png_bytes).to_hex().to_string();
    let evidence = Evidence {
        evidence_schema_revision: 1,
        scenario_id: SCENARIO_ID,
        scenario_revision: SCENARIO_REVISION,
        source_git_revision: std::env::var("RUNENWERK_SOURCE_REVISION").ok(),
        width: WIDTH,
        height: HEIGHT,
        spectral_wavelength_meters: WAVELENGTH_METERS,
        fixed_exposure: FIXED_EXPOSURE,
        visualization_mapping: "display = clamp(radiance * fixed_exposure, 0, 1), encoded as 8-bit grayscale",
        fixture: FixtureEvidence {
            shutter: "instant at renderer time 0 seconds",
            observation: "ideal-ray perspective, vertical FOV pi/3, aspect 1",
            observation_basis: "identity observation-to-scene basis, right-handed metric spaces",
            geometry: "field-backed analytic sphere through oriented-surface input plus analytic plane",
            illumination: "one deterministic directional emitter at the declared wavelength",
            output: "spectral radiance sample lattice, not RGB",
        },
        render_result: RenderResultEvidence {
            scene_revision: format!("{:?}", result.scene_revision()),
            output_count: result.outputs().len(),
            output_index: 0,
            output_topology: "128x128 row-major sample lattice",
            semantic_result_formed: true,
        },
        execution_path: "maintained-deterministic",
        renderer_completion: "completed",
        retained_continuity: "established renderer write remains current",
        product_readback: "completed separate public RunenGPU submission",
        radiance_capture_interpretation: "RunenRender #627 interpreted finite maintained radiance samples",
        field_backed_participation,
        oracle: OracleEvidence {
            sample_count: oracle.sample_count,
            mismatch_count: oracle.mismatch_count,
            maximum_absolute_error: oracle.maximum_absolute_error,
            tolerance: oracle.tolerance,
            result: oracle.result.as_str(),
        },
        png_path: "render-lab/founding-direct/radiance.png",
        png_blake3: png_checksum,
        classification: "pass",
        classification_reason: "renderer, retained continuity, separate readback, RunenRender interpretation, oracle, and nonuniform PNG evidence all passed",
    };
    let evidence_bytes = serde_json::to_vec_pretty(&evidence).context("serialize evidence JSON")?;
    fs::write(artifact_directory.join("evidence.json"), evidence_bytes)
        .context("write founding-direct evidence JSON")?;

    Ok(ArtifactPaths {
        directory: artifact_directory,
        radiance_png,
        evidence_json: output_root.join(SCENARIO_ID).join("evidence.json"),
    })
}

fn request_context() -> Result<GpuContext> {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .require_format_role(GpuTextureFormat::R32Uint, GpuFormatRole::CopyDestination)
            .require_format_role(GpuTextureFormat::R32Uint, GpuFormatRole::CopySource)
            .with_label("Runenwerk Render Lab founding-direct");
    match pollster::block_on(GpuContext::request(descriptor)) {
        Ok(context) => Ok(context),
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable => {
            bail!("no compatible RunenGPU adapter is available for founding-direct")
        }
        Err(error) => Err(error).context("request RunenGPU context"),
    }
}

fn output_destination() -> Result<runen_gpu::GpuTextureHandle> {
    let mut allocator = GpuWorkResourceIdAllocator::new();
    allocator
        .allocate_texture_handle(GpuTextureDescriptor::ordinary_owned_2d(
            "Render Lab founding-direct retained radiance",
            GpuResourceLifetime::Retained,
            GpuReconstruction::SourceBacked,
            WIDTH,
            HEIGHT,
            GpuTextureFormat::R32Uint,
            [
                GpuTextureUsage::CopyDestination,
                GpuTextureUsage::CopySource,
            ],
            GpuTextureInitialization::Uninitialized,
        )?)
        .context("allocate retained founding-direct radiance destination")
}

fn founding_fixture() -> Result<FoundingFixture> {
    let mut store = RenderSceneStore::new();
    let sphere_id = store.allocate_object_id()?;
    let plane_id = store.allocate_object_id()?;
    let state = |translation: [f64; 3]| {
        RenderObjectState::new(
            RenderObjectSpatialState::new(
                RenderSpaceSpec::new(1.0, RenderHandedness::Right).expect("valid metric space"),
                RenderAffineTransform3::from_row_major_3x4([
                    1.0,
                    0.0,
                    0.0,
                    translation[0],
                    0.0,
                    1.0,
                    0.0,
                    translation[1],
                    0.0,
                    0.0,
                    1.0,
                    translation[2],
                ])
                .expect("finite fixture transform"),
                RenderSpatialCoverage::unbounded(),
            ),
            RenderObjectTemporalState::new(RenderTemporalSupport::unbounded()),
        )
    };
    let mut insert = RenderSceneUpdate::new();
    insert
        .insert_with_state(sphere_id, state([0.0, 0.0, 0.0]))
        .insert_with_state(plane_id, state([0.0, 0.0, 0.0]));
    store
        .commit(insert)
        .context("commit founding-direct object state")?;

    let sphere_representation_id = store.allocate_representation_id(sphere_id)?;
    let plane_representation_id = store.allocate_representation_id(plane_id)?;
    let oriented = RenderSurfaceProtocolEvidence::exact(RENDER_SURFACE_QUERY_PROTOCOL_REVISION)?
        .with_oriented_surface(RenderOrientedSurfaceProtocolEvidence::exact(
            RENDER_ORIENTED_SURFACE_QUERY_PROTOCOL_REVISION,
        )?)
        .with_semantic_input_requirement(RenderSurfaceSemanticInputRequirement::current());
    let field_backed_representation = RenderRepresentationRecord::new(
        sphere_representation_id,
        RenderSpatialCoverage::unbounded(),
        RenderTemporalSupport::unbounded(),
        RenderRefinementEvidence::none(),
        Some(oriented),
        Some(RenderFieldDistanceProtocolEvidence::new(
            RENDER_FIELD_DISTANCE_PROTOCOL_REVISION,
            RenderFieldDistanceGuarantee::exact(),
        )?),
    )?;
    let plane_representation = RenderRepresentationRecord::new(
        plane_representation_id,
        RenderSpatialCoverage::unbounded(),
        RenderTemporalSupport::unbounded(),
        RenderRefinementEvidence::none(),
        Some(oriented),
        None,
    )?;
    let material = |reflectance| {
        RenderMaterialAssignment::new(
            RenderDiffuseMaterial::new(reflectance).expect("fixture material"),
        )
    };
    let mut attach = RenderSceneUpdate::new();
    attach
        .replace_participation(
            sphere_id,
            RenderObjectParticipation::new(
                vec![field_backed_representation],
                Some(material(SPHERE_REFLECTANCE)),
                None,
            )?,
        )
        .replace_participation(
            plane_id,
            RenderObjectParticipation::new(
                vec![plane_representation],
                Some(material(PLANE_REFLECTANCE)),
                Some(RenderDirectionalEmitter::new(
                    LIGHT_DIRECTION,
                    WAVELENGTH_METERS,
                    LIGHT_IRRADIANCE,
                )?),
            )?,
        );
    store
        .commit(attach)
        .context("commit founding-direct representations and illumination")?;

    let shutter = RenderTimeInterval::instant(RenderTimePoint::from_seconds(0.0)?);
    let observation = RenderObservationSpec::Perspective(RenderPerspectiveObservation::new(
        RenderAffineTransform3::identity(),
        std::f64::consts::FRAC_PI_3,
        1.0,
        shutter,
        RenderSamplingSupport::ideal_ray(),
    )?);
    let representation =
        RenderRadiometricRepresentation::spectral_at_wavelength_meters(WAVELENGTH_METERS)?;
    let output = RenderOutputSpec::new(
        RenderOutputValue::Radiance { representation },
        RenderResultTopology::sample_lattice_2d(WIDTH, HEIGHT)?,
        RenderSemanticTolerance::absolute(ORACLE_TOLERANCE)?,
    )?;
    let request = RenderRequest::new(
        shutter,
        vec![observation],
        vec![RenderRequestedOutput::new(0, output)],
    )?;
    Ok(FoundingFixture {
        scene: store.snapshot(),
        request,
        semantic_inputs: vec![
            RenderSurfaceSemanticInputBinding::new(
                sphere_representation_id,
                RenderSurfaceSemanticInput::sphere(
                    SPHERE_CENTER,
                    SPHERE_RADIUS,
                    RenderTemporalSupport::unbounded(),
                )?,
            ),
            RenderSurfaceSemanticInputBinding::new(
                plane_representation_id,
                RenderSurfaceSemanticInput::plane(
                    PLANE_POINT,
                    PLANE_NORMAL,
                    RenderTemporalSupport::unbounded(),
                )?,
            ),
        ],
        availability: vec![
            RenderRepresentationAvailabilityFact::new(
                sphere_representation_id,
                RenderRepresentationAvailabilityState::Available,
            ),
            RenderRepresentationAvailabilityFact::new(
                plane_representation_id,
                RenderRepresentationAvailabilityState::Available,
            ),
        ],
    })
}

fn form_result(
    context: &GpuContext,
    submitted: &mut SubmittedDeterministicRender,
) -> Result<RenderResult> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        context.progress();
        match submitted
            .try_form_verified_result()
            .context("poll verified founding-direct result")?
        {
            Some(result) => return Ok(result),
            None if Instant::now() < deadline => std::thread::yield_now(),
            None => bail!("founding-direct verified result did not form before timeout"),
        }
    }
}

fn wait_for_readback(
    context: &GpuContext,
    submission: &GpuSubmission,
    readback_id: runen_gpu::GpuReadbackId,
) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        context.progress();
        match submission.status() {
            GpuSubmissionStatus::Failed(failure) => {
                bail!("RunenGPU submission failed: {failure:?}")
            }
            GpuSubmissionStatus::Accepted | GpuSubmissionStatus::Completed => {}
        }
        let readback = submission
            .readback(readback_id)
            .context("product readback correlation disappeared from its submission")?;
        match readback.status() {
            runen_gpu::GpuReadbackStatus::Ready(_) => {
                if matches!(submission.status(), GpuSubmissionStatus::Completed) {
                    return Ok(());
                }
            }
            runen_gpu::GpuReadbackStatus::Failed(failure) => {
                bail!("RunenGPU readback failed: {failure:?}")
            }
            runen_gpu::GpuReadbackStatus::Pending => {}
        }
        if Instant::now() >= deadline {
            bail!("RunenGPU readback did not complete before timeout");
        }
        std::thread::yield_now();
    }
}

fn field_participation(result: &RenderResult) -> FieldParticipationEvidence {
    let participated = result.outputs().iter().any(|output| {
        output.object_representations().iter().any(|object| {
            result
                .scene()
                .object_participation(object.object_id())
                .and_then(|participation| {
                    participation.representation(object.representation().representation_id())
                })
                .is_some_and(|representation| representation.supports_field_distance())
        })
    });
    FieldParticipationEvidence {
        participated,
        representation_contract: "field-distance-capable representation admitted through oriented-surface protocol",
        traversal_claim: "no maintained field-distance traversal claimed; surface input is the execution contract",
    }
}

pub fn map_radiance_to_display(radiance: f32) -> u8 {
    let display = (radiance * FIXED_EXPOSURE).clamp(0.0, 1.0);
    (display * 255.0).round() as u8
}

fn map_to_grayscale(samples: &[f32]) -> Result<Vec<u8>> {
    if samples.len() != usize::try_from(WIDTH * HEIGHT)? {
        bail!("captured sample count does not match the founding-direct lattice");
    }
    if samples.iter().any(|sample| !sample.is_finite()) {
        bail!("captured radiance contains a non-finite sample");
    }
    Ok(samples
        .iter()
        .copied()
        .map(map_radiance_to_display)
        .collect())
}

fn validate_pixels(pixels: &[u8]) -> Result<()> {
    if pixels.is_empty() {
        bail!("radiance image is empty");
    }
    let min = pixels.iter().copied().min().unwrap_or_default();
    let max = pixels.iter().copied().max().unwrap_or_default();
    let distinct = pixels
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    if min == max || distinct < 8 {
        bail!(
            "radiance image is not meaningfully nonuniform (min={min}, max={max}, distinct={distinct})"
        );
    }
    Ok(())
}

fn write_png(path: &Path, pixels: &[u8]) -> Result<()> {
    let image = GrayImage::from_raw(WIDTH, HEIGHT, pixels.to_vec())
        .context("construct grayscale founding-direct image")?;
    image
        .save_with_format(path, ImageFormat::Png)
        .context("encode founding-direct PNG")
}

pub fn compare_with_oracle(captured: &RenderCapturedDeterministicRadiance) -> OracleSummary {
    let mut mismatches = 0;
    let mut maximum_absolute_error: f64 = 0.0;
    for (index, actual) in captured.samples().iter().copied().enumerate() {
        let expected = reference_radiance(index);
        let error = (f64::from(actual) - expected).abs();
        maximum_absolute_error = maximum_absolute_error.max(error);
        if !actual.is_finite() || error > ORACLE_TOLERANCE {
            mismatches += 1;
        }
    }
    OracleSummary {
        sample_count: captured.samples().len(),
        mismatch_count: mismatches,
        maximum_absolute_error,
        tolerance: ORACLE_TOLERANCE,
        result: if mismatches == 0 {
            Classification::Pass
        } else {
            Classification::Fail
        },
    }
}

fn reference_radiance(index: usize) -> f64 {
    let x = index % usize::try_from(WIDTH).expect("width fits usize");
    let y = index / usize::try_from(WIDTH).expect("width fits usize");
    let u = (x as f64 + 0.5) / f64::from(WIDTH);
    let v = (y as f64 + 0.5) / f64::from(HEIGHT);
    let direction = normalize([
        (2.0 * u - 1.0) * (std::f64::consts::FRAC_PI_3 * 0.5).tan(),
        (1.0 - 2.0 * v) * (std::f64::consts::FRAC_PI_3 * 0.5).tan(),
        -1.0,
    ]);
    let origin = [0.0, 0.0, 0.0];
    let Some(hit) = nearest_reference_hit(origin, direction) else {
        return 0.0;
    };
    let light_direction = normalize(LIGHT_DIRECTION);
    let cosine = dot(hit.normal, light_direction).max(0.0);
    if cosine == 0.0 {
        return 0.0;
    }
    // The controlled light points toward the camera, so neither the plane nor the sphere blocks it.
    hit.reflectance * LIGHT_IRRADIANCE * cosine / std::f64::consts::PI
}

fn nearest_reference_hit(origin: [f64; 3], direction: [f64; 3]) -> Option<ReferenceHit> {
    let sphere = sphere_hit(origin, direction);
    let plane = plane_hit(origin, direction);
    [sphere, plane]
        .into_iter()
        .flatten()
        .min_by(|left, right| left.t.total_cmp(&right.t))
}

fn sphere_hit(origin: [f64; 3], direction: [f64; 3]) -> Option<ReferenceHit> {
    let relative = sub(origin, SPHERE_CENTER);
    let a = dot(direction, direction);
    let b = 2.0 * dot(relative, direction);
    let c = dot(relative, relative) - SPHERE_RADIUS * SPHERE_RADIUS;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }
    let root = discriminant.sqrt();
    let first = (-b - root) / (2.0 * a);
    let second = (-b + root) / (2.0 * a);
    let t = [first, second]
        .into_iter()
        .filter(|candidate| *candidate >= 0.0)
        .min_by(f64::total_cmp)?;
    let position = add(origin, scale(direction, t));
    Some(ReferenceHit {
        t,
        normal: normalize(sub(position, SPHERE_CENTER)),
        reflectance: SPHERE_REFLECTANCE,
    })
}

fn plane_hit(origin: [f64; 3], direction: [f64; 3]) -> Option<ReferenceHit> {
    let denominator = dot(PLANE_NORMAL, direction);
    if denominator == 0.0 {
        return None;
    }
    let t = dot(sub(PLANE_POINT, origin), PLANE_NORMAL) / denominator;
    if t < 0.0 {
        return None;
    }
    Some(ReferenceHit {
        t,
        normal: normalize(PLANE_NORMAL),
        reflectance: PLANE_REFLECTANCE,
    })
}

fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn normalize(value: [f64; 3]) -> [f64; 3] {
    let length = dot(value, value).sqrt();
    value.map(|component| component / length)
}

fn add(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn sub(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn scale(value: [f64; 3], factor: f64) -> [f64; 3] {
    [value[0] * factor, value[1] * factor, value[2] * factor]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_exposure_mapping_is_stable_and_clamped() {
        assert_eq!(map_radiance_to_display(0.0), 0);
        assert_eq!(map_radiance_to_display(1.0), 64);
        assert_eq!(map_radiance_to_display(100.0), 255);
        assert_eq!(map_radiance_to_display(-1.0), 0);
    }

    #[test]
    fn founding_reference_has_geometry_driven_variation() {
        let center = reference_radiance(
            (usize::try_from(HEIGHT / 2).unwrap() * usize::try_from(WIDTH).unwrap())
                + usize::try_from(WIDTH / 2).unwrap(),
        );
        let corner = reference_radiance(0);
        let lower = reference_radiance(
            usize::try_from(HEIGHT - 1).unwrap() * usize::try_from(WIDTH).unwrap()
                + usize::try_from(WIDTH / 2).unwrap(),
        );
        assert!(center > corner);
        assert!(lower > corner);
        assert_ne!(center, lower);
    }

    #[test]
    fn pixel_validation_rejects_uniform_artifacts() {
        assert!(validate_pixels(&vec![0; usize::try_from(WIDTH * HEIGHT).unwrap()]).is_err());
        assert!(
            validate_pixels(
                &(0..=127)
                    .cycle()
                    .take(usize::try_from(WIDTH * HEIGHT).unwrap())
                    .collect::<Vec<_>>()
            )
            .is_ok()
        );
    }
}
