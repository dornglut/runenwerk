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
            assert!(context.id().is_nonzero());
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
            assert_eq!(
                context.admission_report().selection_kind(),
                engine::plugins::gpu::GpuCandidateSelectionKind::DeterministicallyRanked
            );
            assert!(
                !context
                    .admission_report()
                    .candidate_dispositions()
                    .is_empty()
            );
            assert!(
                context
                    .admission_report()
                    .candidate_dispositions_are_canonically_ordered()
            );
            let selected = context.admission_report().candidate().adapter();
            let rank = context.admission_report().selection_evidence().rank();
            assert_eq!(rank.vendor(), selected.vendor());
            assert_eq!(rank.device(), selected.device());
            assert_eq!(rank.diagnostic_name(), selected.diagnostic_name());
            assert_eq!(
                rank.fallback_priority(),
                match selected.fallback() {
                    engine::plugins::gpu::GpuFallbackStatus::ConfirmedNotFallback => 0,
                    engine::plugins::gpu::GpuFallbackStatus::Unknown => 1,
                    engine::plugins::gpu::GpuFallbackStatus::ConfirmedFallback => 2,
                }
            );
        }
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoCandidate => {}
        Err(error) => panic!(
            "unexpected native GPU context admission failure: {:?}: {:?}",
            error.category(),
            error.detail()
        ),
    }
}
