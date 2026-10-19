//! Public RunenGPU cross-frame conformance consumed by ordinary Runenwerk GPU integration.
//!
//! This does not duplicate G5 admission or assume that GPU queue ordering grants
//! retained-content continuity. An in-flight read-only participant is conservatively
//! reserved; an unrelated resource may still make progress; a fresh graph is
//! prepared only after its predecessor terminalizes. No rendering window required.
use runen_gpu::{
    GpuBufferDescriptor, GpuBufferHandle, GpuBufferInitialization, GpuBufferRegion, GpuBufferUsage,
    GpuCapabilityProfile, GpuCapabilityRequirements, GpuContext, GpuContextDescriptor,
    GpuContextRequestErrorCategory, GpuExecutionPreference, GpuReadbackId, GpuReadbackOperation,
    GpuReadbackStatus, GpuReconstruction, GpuResourceLabel, GpuResourceLifetime,
    GpuResourceProvenance, GpuSubmission, GpuSubmissionRejectionKind, GpuSubmissionStatus,
    GpuUploadOperation, GpuWorkFragment, GpuWorkFragmentBuilder, GpuWorkOperation,
    GpuWorkResourceIdAllocator, GpuWorkSubmissionError, PreparedGpuData, TransferData,
};
use std::time::{Duration, Instant};

fn label(value: &str) -> GpuResourceLabel {
    GpuResourceLabel::new(value).expect("valid conformance resource label")
}

fn provenance(value: &str) -> GpuResourceProvenance {
    GpuResourceProvenance::new(label(value), None, None)
}

fn retained_buffer(allocator: &mut GpuWorkResourceIdAllocator, name: &str) -> GpuBufferHandle {
    allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::ordinary_owned(
                name,
                GpuResourceLifetime::Retained,
                GpuReconstruction::SourceBacked,
                64,
                [GpuBufferUsage::CopySource],
                GpuBufferInitialization::Zeroed,
            )
            .expect("valid source-backed zeroed retained buffer"),
        )
        .expect("unique retained resource identity")
}

// The same actual retained buffer is independently authored in distinct G5
// fragments. Writers must not be treated as harmless merely because all work
// uses the transfer queue; admission and resource continuity belong to G5.
fn retained_transfer_buffer(
    allocator: &mut GpuWorkResourceIdAllocator,
    name: &str,
) -> GpuBufferHandle {
    allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::ordinary_owned(
                name,
                GpuResourceLifetime::Retained,
                GpuReconstruction::SourceBacked,
                64,
                [GpuBufferUsage::CopySource, GpuBufferUsage::CopyDestination],
                GpuBufferInitialization::Zeroed,
            )
            .expect("valid retained copy-source/destination buffer"),
        )
        .expect("unique retained resource identity")
}

fn write_fragment(buffer: &GpuBufferHandle, name: &str, values: &[u32; 16]) -> GpuWorkFragment {
    let region = GpuBufferRegion::whole(buffer).expect("whole retained buffer");
    let payload = PreparedGpuData::<TransferData>::from_pod_transfer(
        format!("{name}.payload"),
        values,
        provenance(name),
    )
    .expect("valid real GPU write payload");
    let upload = GpuUploadOperation::new(region.into(), payload).expect("valid GPU upload");
    let mut builder = GpuWorkFragmentBuilder::new(label(name), provenance(name));
    builder
        .declare_resource(buffer.clone().into())
        .expect("declare retained write resource");
    builder
        .add_node(
            label(&format!("{name}.upload")),
            GpuWorkOperation::Upload(upload),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance(name),
        )
        .expect("valid G5 write node");
    builder.finish().expect("valid independent writer fragment")
}

fn read_only_fragment(buffer: &GpuBufferHandle, name: &str) -> (GpuWorkFragment, GpuReadbackId) {
    let readback_id = GpuReadbackId::allocate().expect("fresh readback identity");
    let operation = GpuReadbackOperation::new(
        GpuBufferRegion::whole(buffer)
            .expect("whole retained buffer")
            .into(),
        readback_id,
    )
    .expect("valid readback operation");
    let mut builder = GpuWorkFragmentBuilder::new(label(name), provenance(name));
    builder
        .declare_resource(buffer.clone().into())
        .expect("declare retained buffer");
    builder
        .add_node(
            label(&format!("{name}.readback")),
            GpuWorkOperation::Readback(operation),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance(name),
        )
        .expect("read-only G5 work node");
    (
        builder.finish().expect("valid read-only fragment"),
        readback_id,
    )
}

fn gpu_context() -> Option<GpuContext> {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .with_label("Runenwerk G5 cross-frame public admission proof");
    match pollster::block_on(GpuContext::request(descriptor)) {
        Ok(context) => Some(context),
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable => {
            assert_ne!(
                std::env::var("RUNENWERK_G5_REQUIRE_GPU").ok().as_deref(),
                Some("1"),
                "Vulkan CI requires a usable public RunenGPU context"
            );
            None
        }
        Err(error) => panic!("GPU cross-frame proof context failed: {error}"),
    }
}

fn submit(
    context: &GpuContext,
    buffer: &GpuBufferHandle,
    name: &str,
) -> Result<(GpuSubmission, GpuReadbackId), GpuWorkSubmissionError> {
    let (fragment, id) = read_only_fragment(buffer, name);
    let submitted = pollster::block_on(context.submit_work(name, [fragment]))?;
    Ok((submitted, id))
}

fn complete(context: &GpuContext, submission: &GpuSubmission) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        context.progress();
        match submission.status() {
            GpuSubmissionStatus::Completed => return,
            GpuSubmissionStatus::Accepted if Instant::now() < deadline => {
                std::thread::yield_now();
            }
            state => panic!("native G5 submission did not terminally complete: {state:?}"),
        }
    }
}

#[test]
fn retained_read_only_reservation_is_not_an_implicit_ordered_continuation() {
    let Some(context) = gpu_context() else {
        return;
    };
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let shared = retained_buffer(&mut allocator, "cross.frame.shared");
    let unrelated = retained_buffer(&mut allocator, "cross.frame.unrelated");

    let (first, first_readback) =
        submit(&context, &shared, "cross.frame.first").expect("first admission");
    assert_eq!(
        first.status(),
        GpuSubmissionStatus::Accepted,
        "accepted does not imply terminally completed"
    );

    let attempted = submit(&context, &shared, "cross.frame.overlap");
    let rejected_reason = match attempted {
        Err(GpuWorkSubmissionError::SubmissionRejected(reason)) => reason,
        Err(other) => panic!("expected typed G5 rejection, got {other:?}"),
        Ok(_) => panic!("read-only retained overlap must not bypass G5 reservation"),
    };
    assert_eq!(
        rejected_reason.kind(),
        GpuSubmissionRejectionKind::RetainedContinuityChanged,
        "RunenGPU G5 owns conservative retained-resource participation, including reads"
    );
    assert_eq!(first.status(), GpuSubmissionStatus::Accepted);

    let (independent, independent_readback) =
        submit(&context, &unrelated, "cross.frame.independent")
            .expect("unrelated retained resource can submit without globally waiting");
    assert_eq!(independent.status(), GpuSubmissionStatus::Accepted);

    complete(&context, &first);
    complete(&context, &independent);
    for (submission, readback) in [
        (&first, first_readback),
        (&independent, independent_readback),
    ] {
        match submission
            .readback(readback)
            .expect("readback exists")
            .status()
        {
            GpuReadbackStatus::Ready(bytes) => {
                assert_eq!(bytes.as_bytes(), &[0_u8; 64]);
            }
            other => panic!("terminally completed retained readback must be ready: {other:?}"),
        }
    }

    // Rebuild from the current RunenGPU context after predecessor completion.
    // A rejected prepared graph must not be cached or replayed across a frame.
    let (fresh, fresh_readback) =
        submit(&context, &shared, "cross.frame.fresh").expect("fresh post-terminal admission");
    complete(&context, &fresh);
    assert!(matches!(
        fresh
            .readback(fresh_readback)
            .expect("fresh readback")
            .status(),
        GpuReadbackStatus::Ready(_)
    ));
}

#[test]
fn independently_authored_read_write_flows_cannot_bypass_retained_continuity() {
    let Some(context) = gpu_context() else {
        return;
    };
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let shared = retained_transfer_buffer(&mut allocator, "cross.flow.read.write.shared");
    let first_values = [0x0102_0304_u32; 16];
    let second_values = [0x80ab_1020_u32; 16];

    // Flow A physically writes the retained storage; an independently
    // authored Flow B cannot read (RAW) or write (WAW) while A owns G5.
    let writer = pollster::block_on(context.submit_work(
        "cross.flow.writer.a",
        [write_fragment(
            &shared,
            "cross.flow.writer.a",
            &first_values,
        )],
    ))
    .expect("first retained writer must be admitted");
    assert_eq!(writer.status(), GpuSubmissionStatus::Accepted);
    let refused_read = submit(&context, &shared, "cross.flow.reader.b");
    let refused_write = pollster::block_on(context.submit_work(
        "cross.flow.writer.b",
        [write_fragment(
            &shared,
            "cross.flow.writer.b",
            &second_values,
        )],
    ));
    for refusal in [
        refused_read.map(|(submission, _)| submission),
        refused_write,
    ] {
        match refusal {
            Err(GpuWorkSubmissionError::SubmissionRejected(reason)) => assert_eq!(
                reason.kind(),
                GpuSubmissionRejectionKind::RetainedContinuityChanged
            ),
            Err(other) => panic!("expected typed G5 retained-resource refusal: {other:?}"),
            Ok(_) => panic!("distinct producer bypassed an accepted retained writer"),
        }
    }
    assert_eq!(writer.status(), GpuSubmissionStatus::Accepted);
    complete(&context, &writer);

    // A fresh, newly authored reader after writer completion observes the
    // actual 64 GPU bytes, not an assumed CPU-side state transition.
    let (reader, id) = submit(&context, &shared, "cross.flow.reader.after.writer")
        .expect("reader must be re-prepared after the writer terminalizes");
    assert_eq!(reader.status(), GpuSubmissionStatus::Accepted);
    let refused_writer = pollster::block_on(context.submit_work(
        "cross.flow.writer.after.reader.pending",
        [write_fragment(
            &shared,
            "cross.flow.writer.after.reader.pending",
            &second_values,
        )],
    ));
    match refused_writer {
        Err(GpuWorkSubmissionError::SubmissionRejected(reason)) => assert_eq!(
            reason.kind(),
            GpuSubmissionRejectionKind::RetainedContinuityChanged,
            "an accepted read participant must block a new independent writer"
        ),
        Err(other) => panic!("expected typed G5 read/write refusal: {other:?}"),
        Ok(_) => panic!("retained writer bypassed an in-flight read"),
    }
    complete(&context, &reader);
    let readback = reader
        .readback(id)
        .expect("exact accepted reader G5 result");
    match readback.status() {
        GpuReadbackStatus::Ready(bytes) => {
            let expected = first_values
                .iter()
                .flat_map(|value| value.to_ne_bytes())
                .collect::<Vec<_>>();
            assert_eq!(bytes.as_bytes(), expected);
        }
        other => panic!("read of retained writer result was not Ready: {other:?}"),
    }

    // Only a genuinely fresh submission after the last reader terminalizes
    // may replace the buffer; this does not synthesize ordered continuation.
    let replacement = pollster::block_on(context.submit_work(
        "cross.flow.writer.after.reader.terminal",
        [write_fragment(
            &shared,
            "cross.flow.writer.after.reader.terminal",
            &second_values,
        )],
    ))
    .expect("fresh post-reader write admission");
    complete(&context, &replacement);
    let (final_read, final_id) =
        submit(&context, &shared, "cross.flow.final.read").expect("final new graph admission");
    complete(&context, &final_read);
    let expected = second_values
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect::<Vec<_>>();
    match final_read
        .readback(final_id)
        .expect("exact final G5 readback")
        .status()
    {
        GpuReadbackStatus::Ready(bytes) => assert_eq!(bytes.as_bytes(), expected),
        other => panic!("final retained-state read not Ready: {other:?}"),
    }
}
