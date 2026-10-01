//! Private GPU behavior proof. Runs the maintained evaluator and camera shader together,
//! then reads their exact output; it does not substitute a CPU copy of shader decisions.

use super::*;
use runen_gpu::{GpuCapabilityProfile, GpuContextDescriptor, GpuContextRequestErrorCategory};
use std::time::{Duration, Instant};

fn context() -> Option<GpuContext> {
    let descriptor = super::super::apply_runenwerk_gpu_context_policy(
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .with_label("RunenRender camera correspondence behavior proof"),
    );
    match pollster::block_on(GpuContext::request(descriptor)) {
        Ok(context) => Some(context),
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable => {
            assert_ne!(
                std::env::var("RUNENRENDER_R7_REQUIRE_GPU").ok().as_deref(),
                Some("1")
            );
            None
        }
        Err(error) => panic!("camera proof context: {error}"),
    }
}

fn scene_input(sphere: bool) -> Vec<u32> {
    let count = if sphere { 2 } else { 1 };
    let emitter = HEADER_WORDS + count * GEOMETRY_WORDS;
    let mut input = vec![0; emitter + EMITTER_WORDS];
    input[..8].copy_from_slice(&[
        1,
        1,
        1,
        1,
        count as u32,
        1,
        OUTPUT_RADIANCE,
        OBSERVATION_PERSPECTIVE_FOOTPRINT,
    ]);
    pack_matrix3(
        &mut input,
        11,
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    )
    .unwrap();
    input[20] = 1.0_f32.to_bits();
    input[21] = 1.0_f32.to_bits();
    input[22] = 1;
    input[23] = 1;
    input[29] = emitter as u32;
    for index in 0..count {
        let base = HEADER_WORDS + index * GEOMETRY_WORDS;
        input[base] = if index == 1 {
            SHAPE_SPHERE
        } else {
            SHAPE_PLANE
        };
        input[base + 1] = index as u32 + 1;
        input[base + 2] = 1.0_f32.to_bits();
        pack_matrix3(
            &mut input,
            base + 4,
            [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        )
        .unwrap();
        pack_matrix3(
            &mut input,
            base + 16,
            [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        )
        .unwrap();
        if index == 0 {
            pack_vec3(&mut input, base + 25, [0.0, 0.0, -4.0]).unwrap();
            pack_vec3(&mut input, base + 28, [0.0, 0.0, 1.0]).unwrap();
        } else {
            pack_vec3(&mut input, base + 25, [0.0, 0.0, -3.0]).unwrap();
            input[base + 28] = 2.0_f32.to_bits();
        }
    }
    let light = if sphere {
        [1.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    pack_vec3(&mut input, emitter, light).unwrap();
    input[emitter + 3] = std::f32::consts::PI.to_bits();
    input
}

fn camera_words(previous_available: bool, changed: bool, completed: u32) -> Vec<u32> {
    let mut words = vec![0; 33];
    words[0] = u32::from(previous_available);
    words[1] = u32::from(changed);
    words[23] = completed;
    pack_matrix3(&mut words, 9, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]).unwrap();
    pack_vec3(&mut words, 18, [0.0, 0.0, -1.0]).unwrap();
    words[21] = 1.0_f32.to_bits();
    words[22] = 1.0_f32.to_bits();
    pack_matrix3(
        &mut words,
        24,
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    )
    .unwrap();
    words
}

fn execute(
    context: &GpuContext,
    input: Vec<u32>,
    previous: [u32; 8],
    camera: Vec<u32>,
) -> [u32; 8] {
    let payloads = [
        input,
        vec![0],
        vec![0],
        vec![0],
        vec![0; 4],
        previous.to_vec(),
        vec![0; 8],
        camera,
        vec![0],
    ];
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let handles = payloads
        .iter()
        .enumerate()
        .map(|(index, words)| {
            allocator
                .allocate_buffer_handle(
                    GpuBufferDescriptor::ordinary_owned(
                        format!("camera proof buffer {index}"),
                        GpuResourceLifetime::Transient,
                        GpuReconstruction::SourceBacked,
                        words.len() as u64 * WORD_BYTES,
                        [
                            GpuBufferUsage::Storage,
                            GpuBufferUsage::CopySource,
                            GpuBufferUsage::CopyDestination,
                        ],
                        GpuBufferInitialization::Uninitialized,
                    )
                    .unwrap(),
                )
                .unwrap()
        })
        .collect::<Vec<_>>();
    let [evaluation_source, camera_source] = admit_static_wgsl_sources([
        (
            "camera-proof.evaluation",
            MAINTAINED_EVALUATOR_REVISION,
            MAINTAINED_WGSL.as_str(),
        ),
        (
            "camera-proof.history",
            u64::from(CAMERA_REPROJECTION_REVISION),
            CAMERA_REPROJECTION_WGSL.as_str(),
        ),
    ])
    .unwrap();
    let evaluation = GpuComputePipelineDescriptor::ordinary(evaluation_source, "main").unwrap();
    let bindings = evaluation
        .runtime_bindings([
            GpuRuntimeBindingValue::whole_buffer(0, 0, &handles[0]),
            GpuRuntimeBindingValue::whole_buffer(0, 1, &handles[1]),
            GpuRuntimeBindingValue::whole_buffer(0, 2, &handles[2]),
            GpuRuntimeBindingValue::whole_buffer(0, 3, &handles[8]),
            GpuRuntimeBindingValue::whole_buffer(0, 4, &handles[3]),
            GpuRuntimeBindingValue::whole_buffer(0, 5, &handles[4]),
        ])
        .unwrap();
    let evaluation = GpuComputeOperation::new(
        evaluation,
        bindings,
        GpuDispatchIntent::direct(GpuDispatchSize::new(1, 1, 1)),
    )
    .unwrap();
    let reconstruction = GpuComputePipelineDescriptor::ordinary(camera_source, "main").unwrap();
    let bindings =
        reconstruction
            .runtime_bindings(handles[..8].iter().enumerate().map(|(index, handle)| {
                GpuRuntimeBindingValue::whole_buffer(0, index as u32, handle)
            }))
            .unwrap();
    let reconstruction = GpuComputeOperation::new(
        reconstruction,
        bindings,
        GpuDispatchIntent::direct(GpuDispatchSize::new(1, 1, 1)),
    )
    .unwrap();
    let readback =
        GpuReadbackOperation::ordinary(GpuBufferRegion::whole(&handles[6]).unwrap().into())
            .unwrap();
    let readback_id = readback.id();
    let work = GpuWorkFragment::build("camera history behavior", |work| {
        for (index, (handle, words)) in handles.iter().zip(&payloads).enumerate() {
            let payload =
                PreparedGpuData::<TransferData>::ordinary_pod_transfer("camera proof input", words)
                    .unwrap();
            work.operation(
                format!("upload {index}"),
                GpuUploadOperation::whole_buffer(handle, payload).unwrap(),
            )?;
        }
        work.operation("current-only evaluation", evaluation)?;
        work.operation("history validation", reconstruction)?;
        work.operation("read exact history", readback)?;
        Ok(())
    })
    .unwrap();
    let submission =
        pollster::block_on(context.submit_work("camera history behavior", [work])).unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        context.progress();
        match submission.readback(readback_id).unwrap().status() {
            GpuReadbackStatus::Ready(bytes) => {
                assert!(matches!(
                    submission.status(),
                    GpuSubmissionStatus::Completed
                ));
                return bytes
                    .as_bytes()
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|word| u32::from_ne_bytes(*word))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
            }
            GpuReadbackStatus::Failed(failure) => panic!("camera proof readback: {failure:?}"),
            GpuReadbackStatus::Pending => {}
        }
        assert!(Instant::now() < deadline, "camera proof readback timed out");
        std::thread::yield_now();
    }
}

#[test]
fn stationary_aggregate_is_distinct_from_latest_phase_and_first_motion_uses_coherent_sample() {
    let Some(context) = context() else {
        return;
    };
    let mut previous = [0; 8];
    let mut samples = Vec::new();
    for index in 0..8 {
        let mut input = scene_input(true);
        input[24] = index % 4;
        let current = execute(
            &context,
            input,
            previous,
            camera_words(index != 0, false, index.min(4)),
        );
        samples.push(f32::from_bits(current[4]));
        if index >= 3 {
            let expected = samples[..4].iter().sum::<f32>() / 4.0;
            assert!((f32::from_bits(current[0]) - expected).abs() < 0.000001);
            assert_eq!(current[2], 4);
        }
        previous = current;
    }
    assert_ne!(
        previous[0], previous[4],
        "smooth-sphere phases must distinguish the settled estimate and latest sample"
    );
    let mut input = scene_input(true);
    input[8] = 0.08_f32.to_bits();
    let current_only = execute(
        &context,
        input.clone(),
        previous,
        camera_words(false, false, 0),
    );
    let moving = execute(&context, input, previous, camera_words(true, true, 4));
    assert_eq!(
        moving[4], current_only[4],
        "the retained coherent lane remains raw current radiance"
    );
    let expected = if moving[2] == 2 {
        (f32::from_bits(previous[4]) + f32::from_bits(current_only[4])) * 0.5
    } else {
        assert_eq!(moving[2], 1);
        f32::from_bits(current_only[4])
    };
    assert_eq!(
        moving[0],
        expected.to_bits(),
        "the aggregate must never feed moving reconstruction"
    );
}

#[test]
fn matched_plane_reuses_one_sample_and_rejects_wrong_radiance_or_outside_current_footprint() {
    let Some(context) = context() else {
        return;
    };
    let mut input = scene_input(false);
    input[8] = 0.08_f32.to_bits();
    let previous = [
        0.25_f32.to_bits(),
        4.0_f32.to_bits(),
        4,
        1,
        1.0_f32.to_bits(),
        2.0_f32.to_bits(),
        (-2.0_f32).to_bits(),
        (-4.0_f32).to_bits(),
    ];
    let accepted = execute(
        &context,
        input.clone(),
        previous,
        camera_words(true, true, 4),
    );
    assert_eq!(
        accepted[2], 2,
        "validation must allow useful moving-camera reuse"
    );
    assert_eq!(f32::from_bits(accepted[0]), 1.0);
    let mut wrong_radiance = previous;
    wrong_radiance[4] = 0;
    let rejected = execute(
        &context,
        input.clone(),
        wrong_radiance,
        camera_words(true, true, 4),
    );
    assert_eq!(
        rejected[2], 1,
        "point/depth agreement must not validate mismatched shading"
    );
    assert_eq!(f32::from_bits(rejected[0]), 1.0);
    let mut outside = previous;
    outside[5] = (-4.04_f32).to_bits();
    let rejected = execute(&context, input, outside, camera_words(true, true, 4));
    assert_eq!(
        rejected[2], 1,
        "previous depth alone must not admit a sample outside current support"
    );
    assert_eq!(f32::from_bits(rejected[0]), 1.0);
}
