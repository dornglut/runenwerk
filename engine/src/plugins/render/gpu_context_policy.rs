use runen_gpu::{GpuBackendFamily, GpuContextDescriptor};

const RUNENWERK_GPU_BACKEND_PREFERENCE: [GpuBackendFamily; 5] = [
    GpuBackendFamily::Vulkan,
    GpuBackendFamily::Metal,
    GpuBackendFamily::Direct3D12,
    GpuBackendFamily::BrowserWebGpu,
    GpuBackendFamily::OpenGl,
];

/// Returns Runenwerk's maintained backend preference from most to least preferred.
///
/// This is integration policy, not an allowlist. RunenGPU still owns candidate discovery,
/// capability admission, ranking semantics, ambiguity rejection, and exact-candidate retry.
pub const fn runenwerk_gpu_backend_preference() -> &'static [GpuBackendFamily] {
    &RUNENWERK_GPU_BACKEND_PREFERENCE
}

/// Applies Runenwerk's maintained physical GPU-selection policy to one RunenGPU request.
///
/// Vulkan is first because it is the repository's maintained hosted native execution lane.
/// The remaining first-tier platform backends stay ahead of OpenGL. Environment/backend
/// constraints may reduce the candidate set before this preference participates in ranking.
pub fn apply_runenwerk_gpu_context_policy(
    descriptor: GpuContextDescriptor,
) -> GpuContextDescriptor {
    descriptor.with_backend_preference(RUNENWERK_GPU_BACKEND_PREFERENCE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use runen_gpu::GpuCapabilityRequirements;

    #[test]
    fn maintained_policy_orders_backends_without_becoming_an_allowlist() {
        let descriptor = apply_runenwerk_gpu_context_policy(GpuContextDescriptor::new(
            GpuCapabilityRequirements::new(),
        ));

        assert_eq!(
            descriptor.backend_preference_order().collect::<Vec<_>>(),
            runenwerk_gpu_backend_preference().to_vec()
        );
        assert_eq!(descriptor.backend_allowlist().count(), 0);
    }
}
