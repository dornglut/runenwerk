//! Retained G5-C02 exact-conformance proof for canonical RunenGPU execution.
//!
//! The fixed fixture is the recovered historical `game_of_life_sdf` workload:
//! 160x90 cells, seed 0xC0FF_EE11, toroidal Conway B3/S23 evolution, and exactly
//! 16 steps. The proof uses only public RunenGPU resource, program, work,
//! prepare/submit, completion, and readback semantics.

use engine::plugins::gpu::*;
use std::time::{Duration, Instant};

const WIDTH: u32 = 160;
const HEIGHT: u32 = 90;
const CELL_COUNT: usize = (WIDTH as usize) * (HEIGHT as usize);
const STEP_COUNT: u32 = 16;
const SEED: u32 = 0xC0FF_EE11;
const EXPECTED_LIVE_CELLS: u32 = 2_063;
const EXPECTED_FNV1A64: u64 = 0xBD71_0B88_594C_D584;
const WORKGROUP_SIZE: u32 = 8;
const SOURCE_REVISION: u64 = 1;
const SEED_WGSL: &str = include_str!("gpu_game_of_life_native/seed.wgsl");
const STEP_WGSL: &str = include_str!("gpu_game_of_life_native/step.wgsl");

#[derive(Clone)]
struct ProgramSources {
    seed: GpuAdmittedProgramSource,
    step: GpuAdmittedProgramSource,
}

fn label(value: impl AsRef<str>) -> GpuResourceLabel {
    GpuResourceLabel::new(value.as_ref()).unwrap()
}

fn admitted_sources() -> ProgramSources {
    let [seed, step] = admit_static_wgsl_sources([
        ("proof.game-of-life.seed", SOURCE_REVISION, SEED_WGSL),
        ("proof.game-of-life.step", SOURCE_REVISION, STEP_WGSL),
    ])
    .unwrap();
    ProgramSources { seed, step }
}

fn compute_pipeline(source: &GpuAdmittedProgramSource) -> GpuComputePipelineDescriptor {
    GpuComputePipelineDescriptor::ordinary(source.clone(), "cs_main").unwrap()
}

fn state_buffer(resources: &mut GpuResourceScope, name: &str) -> GpuBufferHandle {
    resources
        .buffer(
            GpuBufferDescriptor::ordinary_owned(
                name,
                GpuResourceLifetime::Transient,
                GpuReconstruction::SourceBacked,
                u64::try_from(CELL_COUNT * std::mem::size_of::<u32>()).unwrap(),
                [GpuBufferUsage::Storage, GpuBufferUsage::CopySource],
                GpuBufferInitialization::Zeroed,
            )
            .unwrap(),
        )
        .unwrap()
}

fn dispatch_size() -> GpuDispatchSize {
    GpuDispatchSize::new(
        WIDTH.div_ceil(WORKGROUP_SIZE),
        HEIGHT.div_ceil(WORKGROUP_SIZE),
        1,
    )
}

fn seed_operation(
    pipeline: &GpuComputePipelineDescriptor,
    state_a: &GpuBufferHandle,
    state_b: &GpuBufferHandle,
) -> GpuComputeOperation {
    let bindings = pipeline
        .runtime_bindings([
            GpuRuntimeBindingValue::whole_buffer(0, 0, state_a),
            GpuRuntimeBindingValue::whole_buffer(0, 1, state_b),
        ])
        .unwrap();
    GpuComputeOperation::new(
        pipeline.clone(),
        bindings,
        GpuDispatchIntent::direct(dispatch_size()),
    )
    .unwrap()
}

fn step_operation(
    pipeline: &GpuComputePipelineDescriptor,
    input: &GpuBufferHandle,
    output: &GpuBufferHandle,
) -> GpuComputeOperation {
    let bindings = pipeline
        .runtime_bindings([
            GpuRuntimeBindingValue::whole_buffer(0, 0, input),
            GpuRuntimeBindingValue::whole_buffer(0, 1, output),
        ])
        .unwrap();
    GpuComputeOperation::new(
        pipeline.clone(),
        bindings,
        GpuDispatchIntent::direct(dispatch_size()),
    )
    .unwrap()
}

fn author_work(sources: &ProgramSources) -> (GpuWorkFragment, GpuReadbackId) {
    let mut resources = GpuResourceScope::new();
    let state_a = state_buffer(&mut resources, "game of life state a");
    let state_b = state_buffer(&mut resources, "game of life state b");
    let seed = compute_pipeline(&sources.seed);
    let step = compute_pipeline(&sources.step);

    let mut readback_id = None;
    let fragment = GpuWorkFragment::build("G5-C02 Game of Life exact conformance", |work| {
        work.operation(
            "game of life deterministic seed",
            seed_operation(&seed, &state_a, &state_b),
        )?;

        for step_index in 0..STEP_COUNT {
            let (input, output) = if step_index % 2 == 0 {
                (&state_a, &state_b)
            } else {
                (&state_b, &state_a)
            };
            work.operation(
                format!("game of life step {:02}", step_index + 1),
                step_operation(&step, input, output),
            )?;
        }

        let final_state = if STEP_COUNT % 2 == 0 {
            &state_a
        } else {
            &state_b
        };
        let readback =
            GpuReadbackOperation::ordinary(GpuBufferRegion::whole(final_state).unwrap().into())
                .unwrap();
        readback_id = Some(readback.id());
        work.operation("game of life final full-grid readback", readback)?;
        Ok(())
    })
    .unwrap();

    assert_eq!(fragment.resources().len(), 2);
    assert_eq!(
        fragment.nodes().len(),
        usize::try_from(STEP_COUNT).unwrap() + 2
    );
    (fragment, readback_id.unwrap())
}

fn native_compute_context() -> GpuContext {
    let mut requirements = GpuCapabilityRequirements::new();
    for feature in [GpuCapabilityFeature::Compute, GpuCapabilityFeature::Copy] {
        requirements
            .insert(GpuCapabilityRequirement::Required(feature))
            .unwrap();
    }
    let descriptor = GpuContextDescriptor::new(requirements)
        .with_fallback_policy(GpuSoftwareFallbackPolicy::Require)
        .with_allowed_backends([GpuBackendFamily::Vulkan])
        .with_label("G5-C02 Game of Life exact-conformance proof");
    let context = pollster::block_on(GpuContext::request(descriptor))
        .expect("native conformance environment must provide a Vulkan fallback adapter");
    assert_eq!(context.adapter_facts().backend(), GpuBackendFamily::Vulkan);
    assert_eq!(
        context.adapter_facts().fallback(),
        GpuFallbackStatus::ConfirmedFallback
    );
    context
}

fn progress_to_readback(
    context: &GpuContext,
    submission: &GpuSubmission,
    id: GpuReadbackId,
) -> GpuReadbackBytes {
    let handle = submission
        .readback(id)
        .expect("Game-of-Life readback must remain observable")
        .clone();
    let deadline = Instant::now() + Duration::from_secs(30);

    loop {
        context.progress();
        match handle.status() {
            GpuReadbackStatus::Ready(bytes) => {
                if matches!(submission.status(), GpuSubmissionStatus::Completed) {
                    return bytes;
                }
            }
            GpuReadbackStatus::Failed(failure) => {
                panic!("Game-of-Life readback failed: {failure:?}")
            }
            GpuReadbackStatus::Pending => {}
        }
        if let GpuSubmissionStatus::Failed(failure) = submission.status() {
            panic!("Game-of-Life submission failed: {failure:?}");
        }
        assert!(Instant::now() < deadline, "Game-of-Life proof timed out");
        std::thread::yield_now();
    }
}

fn decode_u32(bytes: &GpuReadbackBytes) -> Vec<u32> {
    let (words, remainder) = bytes.as_bytes().as_chunks::<4>();
    assert!(remainder.is_empty());
    words.iter().copied().map(u32::from_le_bytes).collect()
}

fn hash_cell(x: u32, y: u32) -> u32 {
    let mut h = x
        .wrapping_mul(1_664_525)
        .wrapping_add(y.wrapping_mul(1_013_904_223))
        .wrapping_add(SEED.wrapping_mul(747_796_405))
        .wrapping_add(2_891_336_453);
    h = (h ^ (h >> 16)).wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(3_266_489_917);
    h ^ (h >> 16)
}

fn seeded_alive(x: u32, y: u32) -> u32 {
    let noise = u32::from((hash_cell(x, y) & 0x00FF_FFFF) < 0x002A_AAAA);
    let local_x = x % 24;
    let local_y = y % 24;
    if local_y == 12 && (10..=12).contains(&local_x) {
        1
    } else {
        noise
    }
}

fn cpu_seed() -> Vec<u32> {
    let mut cells = Vec::with_capacity(CELL_COUNT);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            cells.push(seeded_alive(x, y));
        }
    }
    cells
}

fn cpu_step(input: &[u32]) -> Vec<u32> {
    assert_eq!(input.len(), CELL_COUNT);
    let mut output = vec![0_u32; CELL_COUNT];
    let width_i = i32::try_from(WIDTH).unwrap();
    let height_i = i32::try_from(HEIGHT).unwrap();

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut neighbors = 0_u32;
            for dy in -1_i32..=1 {
                for dx in -1_i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = (i32::try_from(x).unwrap() + dx).rem_euclid(width_i) as u32;
                    let ny = (i32::try_from(y).unwrap() + dy).rem_euclid(height_i) as u32;
                    let neighbor_index = usize::try_from(ny * WIDTH + nx).unwrap();
                    neighbors += input[neighbor_index];
                }
            }

            let index = usize::try_from(y * WIDTH + x).unwrap();
            let current = input[index];
            output[index] = u32::from(
                (current == 1 && (neighbors == 2 || neighbors == 3))
                    || (current == 0 && neighbors == 3),
            );
        }
    }

    output
}

fn cpu_final_grid() -> Vec<u32> {
    let mut cells = cpu_seed();
    for _ in 0..STEP_COUNT {
        cells = cpu_step(&cells);
    }
    cells
}

fn fnv1a64_le_u32(cells: &[u32]) -> u64 {
    const OFFSET: u64 = 0xCBF2_9CE4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01B3;

    let mut hash = OFFSET;
    for cell in cells {
        for byte in cell.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(PRIME);
        }
    }
    hash
}

fn assert_canonical_oracle(cells: &[u32]) {
    assert_eq!(cells.len(), CELL_COUNT);
    assert!(cells.iter().all(|cell| matches!(cell, 0 | 1)));
    let live_cells = cells.iter().copied().sum::<u32>();
    assert_eq!(live_cells, EXPECTED_LIVE_CELLS);
    assert_eq!(fnv1a64_le_u32(cells), EXPECTED_FNV1A64);
}

#[test]
#[ignore = "requires a real Vulkan fallback adapter; executed by RunenGPU Conformance CI"]
fn native_game_of_life_proves_exact_160x90_seed_16_step_oracle() {
    let expected = cpu_final_grid();
    assert_canonical_oracle(&expected);

    let context = native_compute_context();
    let sources = admitted_sources();
    let (fragment, readback_id) = author_work(&sources);
    let graph =
        GpuPreparedWorkGraph::prepare(label("G5-C02 Game of Life prepared graph"), [fragment])
            .unwrap();

    assert_eq!(
        graph.nodes().len(),
        usize::try_from(STEP_COUNT).unwrap() + 2
    );
    assert_eq!(graph.topological_order().len(), graph.nodes().len());
    assert!(
        graph
            .dependencies()
            .iter()
            .flat_map(|dependency| dependency.reasons())
            .all(|reason| !matches!(reason, GpuDependencyReason::ExplicitNonData { .. })),
        "Game-of-Life ping-pong must be ordered by typed resource hazards, not manual ordering"
    );
    for feature in [GpuCapabilityFeature::Compute, GpuCapabilityFeature::Copy] {
        assert!(graph.requirements().get(feature).is_some());
    }

    let prepared = pollster::block_on(context.prepare_submission(graph)).unwrap();
    let submission = context.submit_prepared(prepared).unwrap();
    let bytes = progress_to_readback(&context, &submission, readback_id);
    let actual = decode_u32(&bytes);

    assert_eq!(
        actual, expected,
        "Game-of-Life GPU grid must equal the CPU oracle"
    );
    assert_canonical_oracle(&actual);

    let stats = context.execution_stats();
    assert_eq!(stats.prepared_submissions(), 0);
    assert_eq!(stats.in_flight_submissions(), 0);
    assert_eq!(stats.upload_bytes_in_flight(), 0);
    assert_eq!(stats.readback_bytes_in_flight(), 0);
    assert_eq!(stats.pending_readbacks(), 0);
}
