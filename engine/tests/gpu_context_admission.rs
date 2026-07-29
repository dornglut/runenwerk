use engine::plugins::gpu::{
    GpuCapabilityFeature, GpuCapabilityProfile, GpuContext, GpuContextDescriptor,
    GpuContextRequestErrorCategory,
};

#[test]
fn headless_context_admission_reports_a_real_context_or_a_typed_backend_outcome() {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .with_label("engine headless context admission test");

    match pollster::block_on(GpuContext::request(descriptor)) {
        Ok(context) => {
            assert!(
                context
                    .device_facts()
                    .is_enabled(GpuCapabilityFeature::Compute)
            );
            assert_eq!(
                context.generation(),
                engine::plugins::gpu::GpuDeviceGeneration::first()
            );
            assert!(
                context
                    .adapter_facts()
                    .supported()
                    .supports(GpuCapabilityFeature::Copy)
            );
        }
        Err(error) => assert!(matches!(
            error.category(),
            GpuContextRequestErrorCategory::BackendAdapterRequestFailure
                | GpuContextRequestErrorCategory::BackendDeviceRequestFailure
                | GpuContextRequestErrorCategory::NoCandidate
        )),
    }
}
