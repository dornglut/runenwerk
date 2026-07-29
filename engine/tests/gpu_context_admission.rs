use engine::plugins::gpu::{
    GpuCapabilityFeature, GpuCapabilityProfile, GpuContext, GpuContextDescriptor,
    GpuContextRequestErrorCategory,
};
use std::collections::BTreeSet;

#[test]
fn headless_context_admission_reports_a_real_context_or_a_strict_environment_outcome() {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .with_label("engine headless context admission test");

    match pollster::block_on(GpuContext::request(descriptor)) {
        Ok(context) => {
            let enabled = context
                .device_facts()
                .enabled_features()
                .collect::<BTreeSet<_>>();
            assert!(
                enabled.contains(&GpuCapabilityFeature::Compute)
                    && enabled.contains(&GpuCapabilityFeature::Copy)
            );
            assert_eq!(enabled.len(), 2, "no unrelated optional feature is enabled");
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
            assert_eq!(
                context.adapter_facts(),
                context.admission_report().candidate().adapter()
            );
            assert_eq!(
                context.device_facts().effective_limits(),
                context.admission_report().candidate().effective_limits()
            );
        }
        Err(error) => assert!(matches!(
            error.category(),
            GpuContextRequestErrorCategory::NoCandidate
                | GpuContextRequestErrorCategory::BackendAdapterRequestFailure
                | GpuContextRequestErrorCategory::BackendDeviceRequestFailure
        )),
    }
}
