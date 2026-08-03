use super::WgpuContextState;
use super::adapter_mapping::adapter_facts;
use crate::plugins::gpu::{
    GpuAdapterFacts, GpuAlignmentFacts, GpuCandidateEnvironmentEvidence, GpuCandidateId,
    GpuCandidateInput, GpuCandidateSelection, GpuCandidateSelectionKind, GpuCapabilityFeature,
    GpuContext, GpuContextAdmissionReport, GpuContextDescriptor, GpuContextRequestError,
    GpuContextRequestErrorCategory, GpuDeviceGeneration, GpuDeviceLimits, GpuDeviceRequestProfile,
    GpuFallbackStatus, GpuLimits, GpuSoftwareFallbackPolicy, admitted_device_facts,
    allocate_context_id, canonical_candidate_input_key, select_candidate_inputs,
};
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use wgpu::Backends;
use wgpu::{
    Adapter, DeviceDescriptor, ExperimentalFeatures, Features, Instance, InstanceDescriptor,
    Limits, MemoryHints, RequestAdapterError, RequestAdapterOptions, Surface, Trace,
};

#[cfg(not(target_arch = "wasm32"))]
struct NativeAdapterCandidate<T> {
    id: GpuCandidateId,
    facts: GpuAdapterFacts,
    adapter: T,
    environment: GpuCandidateEnvironmentEvidence,
}

pub(crate) async fn request_headless(
    descriptor: GpuContextDescriptor,
) -> Result<GpuContext, GpuContextRequestError> {
    request_with_instance(
        Instance::new(&InstanceDescriptor::default().with_env()),
        descriptor,
        None,
    )
    .await
}

pub(super) async fn request_with_instance(
    instance: Instance,
    descriptor: GpuContextDescriptor,
    compatible_surface: Option<&Surface<'_>>,
) -> Result<GpuContext, GpuContextRequestError> {
    crate::plugins::gpu::validate_descriptor(&descriptor)?;
    let (adapter, selection, selection_kind) =
        select_backend_adapter(&instance, &descriptor, compatible_surface).await?;
    let candidate = selection.candidate;
    let requested_limits = requested_limits(&candidate)?;
    if !requested_limits.check_limits(&adapter.limits()) {
        return Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::DeviceRequestProfileUnsupported,
            "selected adapter cannot satisfy the complete requested device profile",
        ));
    }
    let requested_features = requested_features(&candidate);
    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            label: descriptor.label(),
            required_features: requested_features,
            required_limits: requested_limits.clone(),
            experimental_features: ExperimentalFeatures::disabled(),
            memory_hints: MemoryHints::Performance,
            trace: Trace::Off,
        })
        .await
        .map_err(|error| {
            GpuContextRequestError::new(
                GpuContextRequestErrorCategory::BackendDeviceRequestFailure,
                error.to_string(),
            )
        })?;
    verify_requested_features(requested_features, device.features())?;
    let actual_native_limits = device.limits();
    if !requested_limits.check_limits(&actual_native_limits) {
        return Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::BackendDeviceRequestFailure,
            "created device did not expose the complete admitted request profile",
        ));
    }
    let device_facts = admitted_device_facts(
        &candidate,
        map_device_limits(&actual_native_limits),
        selection.dispositions.clone(),
    )?;
    let id = allocate_context_id()?;
    let adapter_facts = candidate.adapter().clone();
    Ok(GpuContext {
        id,
        generation: GpuDeviceGeneration::first(),
        adapter: adapter_facts,
        device: device_facts,
        report: GpuContextAdmissionReport {
            selected: selection_kind,
            candidate,
            candidate_dispositions: selection.dispositions,
            selection_evidence: selection.evidence,
        },
        backend: WgpuContextState {
            instance,
            adapter,
            device: Arc::new(device),
            queue: Arc::new(queue),
        },
    })
}

#[cfg(not(target_arch = "wasm32"))]
async fn select_backend_adapter(
    instance: &Instance,
    descriptor: &GpuContextDescriptor,
    compatible_surface: Option<&Surface<'_>>,
) -> Result<(Adapter, GpuCandidateSelection, GpuCandidateSelectionKind), GpuContextRequestError> {
    if native_selection_route(descriptor.fallback_policy())
        == NativeAdapterSelectionRoute::ForcedFallback
    {
        return select_backend_selected_adapter(instance, descriptor, compatible_surface, true)
            .await;
    }
    let candidates = instance
        .enumerate_adapters(Backends::all())
        .into_iter()
        .map(|adapter| -> Result<_, GpuContextRequestError> {
            // Absence of a surface is not surface compatibility evidence.
            let surface_supported =
                compatible_surface.is_some_and(|surface| adapter.is_surface_supported(surface));
            let environment = if compatible_surface.is_some() {
                GpuCandidateEnvironmentEvidence::current_host(surface_supported)
            } else {
                GpuCandidateEnvironmentEvidence::headless()
            };
            Ok(NativeAdapterCandidate {
                id: GpuCandidateId::allocate()?,
                facts: adapter_facts(
                    &adapter,
                    surface_supported,
                    ordinary_enumeration_fallback_status(),
                ),
                adapter,
                environment,
            })
        })
        .collect::<Result<Vec<_>, GpuContextRequestError>>()?;
    select_enumerated_adapter(descriptor, candidates).map(|(adapter, selection)| {
        (
            adapter,
            selection,
            GpuCandidateSelectionKind::DeterministicallyRanked,
        )
    })
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeAdapterSelectionRoute {
    Enumerated,
    ForcedFallback,
}

#[cfg(not(target_arch = "wasm32"))]
const fn native_selection_route(policy: GpuSoftwareFallbackPolicy) -> NativeAdapterSelectionRoute {
    match policy {
        GpuSoftwareFallbackPolicy::Require => NativeAdapterSelectionRoute::ForcedFallback,
        GpuSoftwareFallbackPolicy::Allow | GpuSoftwareFallbackPolicy::Forbid => {
            NativeAdapterSelectionRoute::Enumerated
        }
    }
}

/// Native enumeration does not prove that an adapter is non-fallback. Only the forced WGPU
/// fallback path has evidence strong enough to publish `ConfirmedFallback`.
const fn ordinary_enumeration_fallback_status() -> GpuFallbackStatus {
    GpuFallbackStatus::Unknown
}

#[cfg(not(target_arch = "wasm32"))]
fn select_enumerated_adapter<T>(
    descriptor: &GpuContextDescriptor,
    mut candidates: Vec<NativeAdapterCandidate<T>>,
) -> Result<(T, GpuCandidateSelection), GpuContextRequestError> {
    candidates.sort_by_key(|candidate| {
        canonical_candidate_input_key(&candidate.facts, candidate.environment)
    });
    let selection = select_candidate_inputs(
        descriptor,
        candidates.iter().map(|candidate| GpuCandidateInput {
            id: candidate.id,
            adapter: candidate.facts.clone(),
            environment: candidate.environment,
        }),
    )?;
    let selected = selection.backend_candidate_id;
    let adapter = candidates
        .into_iter()
        .find(|candidate| candidate.id == selected)
        .map(|candidate| candidate.adapter)
        .ok_or_else(|| {
            GpuContextRequestError::new(
                GpuContextRequestErrorCategory::BackendAdapterRequestFailure,
                "selected candidate ID is absent from the native candidate set",
            )
        })?;
    Ok((adapter, selection))
}

#[cfg(target_arch = "wasm32")]
async fn select_backend_adapter(
    instance: &Instance,
    descriptor: &GpuContextDescriptor,
    compatible_surface: Option<&Surface<'_>>,
) -> Result<(Adapter, GpuCandidateSelection, GpuCandidateSelectionKind), GpuContextRequestError> {
    let fallback_required = matches!(
        descriptor.fallback_policy(),
        GpuSoftwareFallbackPolicy::Require
    );
    select_backend_selected_adapter(instance, descriptor, compatible_surface, fallback_required)
        .await
}

async fn select_backend_selected_adapter(
    instance: &Instance,
    descriptor: &GpuContextDescriptor,
    compatible_surface: Option<&Surface<'_>>,
    force_fallback_adapter: bool,
) -> Result<(Adapter, GpuCandidateSelection, GpuCandidateSelectionKind), GpuContextRequestError> {
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: map_power_preference(descriptor.power_preference()),
            force_fallback_adapter,
            compatible_surface,
        })
        .await
        .map_err(map_request_adapter_error)?;
    let surface_supported =
        compatible_surface.is_some_and(|surface| adapter.is_surface_supported(surface));
    let environment = if compatible_surface.is_some() {
        GpuCandidateEnvironmentEvidence::current_host(surface_supported)
    } else {
        GpuCandidateEnvironmentEvidence::headless()
    };
    let selection = select_candidate_inputs(
        descriptor,
        [GpuCandidateInput {
            id: GpuCandidateId::allocate()?,
            adapter: adapter_facts(
                &adapter,
                surface_supported,
                backend_selected_fallback_status(force_fallback_adapter),
            ),
            environment,
        }],
    )?;
    Ok((
        adapter,
        selection,
        GpuCandidateSelectionKind::BackendSelectedCandidate,
    ))
}

const fn backend_selected_fallback_status(force_fallback_adapter: bool) -> GpuFallbackStatus {
    if force_fallback_adapter {
        GpuFallbackStatus::ConfirmedFallback
    } else {
        GpuFallbackStatus::Unknown
    }
}

fn map_request_adapter_error(error: RequestAdapterError) -> GpuContextRequestError {
    let category = match error {
        RequestAdapterError::NotFound { .. } => GpuContextRequestErrorCategory::NoAdapterAvailable,
        RequestAdapterError::EnvNotSet => {
            GpuContextRequestErrorCategory::BackendAdapterRequestFailure
        }
        _ => GpuContextRequestErrorCategory::BackendAdapterRequestFailure,
    };
    GpuContextRequestError::new(category, error.to_string())
}

fn map_power_preference(
    preference: crate::plugins::gpu::GpuPowerPreference,
) -> wgpu::PowerPreference {
    match preference {
        crate::plugins::gpu::GpuPowerPreference::HighPerformance => {
            wgpu::PowerPreference::HighPerformance
        }
        crate::plugins::gpu::GpuPowerPreference::LowPower => wgpu::PowerPreference::LowPower,
        crate::plugins::gpu::GpuPowerPreference::NoPreference => wgpu::PowerPreference::None,
    }
}

fn requested_features(candidate: &crate::plugins::gpu::GpuCandidateAdmissionReport) -> Features {
    candidate
        .enabled_features()
        .fold(Features::empty(), |features, feature| match feature {
            GpuCapabilityFeature::TimestampQuery => features | Features::TIMESTAMP_QUERY,
            _ => features,
        })
}

fn verify_requested_features(
    requested: Features,
    actual: Features,
) -> Result<(), GpuContextRequestError> {
    if actual.contains(requested) {
        Ok(())
    } else {
        Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::BackendDeviceRequestFailure,
            "created device did not expose every admitted WGPU feature",
        ))
    }
}

/// Returns a fully initialized pinned-WGPU profile; no struct-update defaults hide a limit.
pub(super) const fn profile_limits(profile: GpuDeviceRequestProfile) -> Limits {
    match profile {
        GpuDeviceRequestProfile::ModernPortable | GpuDeviceRequestProfile::BrowserWebGpu => {
            Limits::defaults()
        }
        GpuDeviceRequestProfile::Downlevel => Limits::downlevel_defaults(),
        GpuDeviceRequestProfile::DownlevelWebGl2 => Limits::downlevel_webgl2_defaults(),
    }
}

fn requested_limits(
    candidate: &crate::plugins::gpu::GpuCandidateAdmissionReport,
) -> Result<Limits, GpuContextRequestError> {
    let contract = candidate.contract();
    let budget = contract.workload_budget().limits();
    let mut limits = profile_limits(contract.device_request_profile());
    limits.max_uniform_buffer_binding_size =
        u32::try_from(budget.max_uniform_buffer_binding_size()).map_err(limit_cast_error)?;
    limits.max_storage_buffer_binding_size =
        u32::try_from(budget.max_storage_buffer_binding_size()).map_err(limit_cast_error)?;
    limits.max_color_attachments = budget.max_color_attachments();
    limits.max_vertex_buffers = budget.max_vertex_buffers();
    limits.max_bindings_per_bind_group = budget.max_bindings_per_group();
    let alignments = contract.selected_alignments();
    limits.min_uniform_buffer_offset_alignment =
        requested_alignment(alignments.uniform_dynamic_offset, "uniform dynamic offset")?;
    limits.min_storage_buffer_offset_alignment =
        requested_alignment(alignments.storage_dynamic_offset, "storage dynamic offset")?;
    Ok(limits)
}

fn limit_cast_error(_: std::num::TryFromIntError) -> GpuContextRequestError {
    GpuContextRequestError::new(
        GpuContextRequestErrorCategory::DeviceRequestProfileUnsupported,
        "admitted normalized limit cannot be represented by pinned WGPU",
    )
}

fn requested_alignment(
    value: Option<u64>,
    label: &'static str,
) -> Result<u32, GpuContextRequestError> {
    let value = value.ok_or_else(|| {
        GpuContextRequestError::new(
            GpuContextRequestErrorCategory::AlignmentIncompatibility,
            format!("{label} has no requestable admitted value"),
        )
    })?;
    u32::try_from(value).map_err(|_| {
        GpuContextRequestError::new(
            GpuContextRequestErrorCategory::AlignmentIncompatibility,
            format!("{label} exceeds the pinned WGPU alignment domain"),
        )
    })
}

fn map_device_limits(native: &Limits) -> GpuDeviceLimits {
    GpuDeviceLimits::new(
        GpuLimits::from_validated_adapter_facts(
            u64::from(native.max_uniform_buffer_binding_size),
            u64::from(native.max_storage_buffer_binding_size),
            native.max_color_attachments,
            native.max_vertex_buffers,
            native.max_bindings_per_bind_group,
        ),
        GpuAlignmentFacts {
            uniform_dynamic_offset: Some(u64::from(native.min_uniform_buffer_offset_alignment)),
            storage_dynamic_offset: Some(u64::from(native.min_storage_buffer_offset_alignment)),
            copy_buffer_offset: Some(wgpu::COPY_BUFFER_ALIGNMENT),
            bytes_per_row: Some(u64::from(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)),
            query_resolve_destination: Some(wgpu::QUERY_RESOLVE_BUFFER_ALIGNMENT),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{
        GpuAdapterClass, GpuAdapterLimits, GpuBackendFamily, GpuCapabilities,
        GpuCapabilityRequirements, GpuContextDescriptor, GpuFallbackStatus, GpuSoftwareStatus,
        select_candidate_with_host_evidence,
    };

    fn candidate() -> crate::plugins::gpu::GpuCandidateAdmissionReport {
        let facts = GpuAdapterFacts::new(
            GpuBackendFamily::Vulkan,
            GpuAdapterClass::Discrete,
            GpuSoftwareStatus::Hardware,
            GpuFallbackStatus::ConfirmedNotFallback,
            GpuCapabilities::from_normalized_facts(
                [],
                GpuLimits::new(256 * 1024, 512 * 1024 * 1024, 8, 16, 128).unwrap(),
                [],
            ),
            GpuAdapterLimits::new(
                GpuLimits::new(256 * 1024, 512 * 1024 * 1024, 8, 16, 128).unwrap(),
            ),
            GpuAlignmentFacts {
                uniform_dynamic_offset: Some(256),
                storage_dynamic_offset: Some(256),
                copy_buffer_offset: Some(4),
                bytes_per_row: Some(256),
                query_resolve_destination: Some(256),
            },
        );
        select_candidate_with_host_evidence(
            &GpuContextDescriptor::new(GpuCapabilityRequirements::new()),
            [(facts, GpuCandidateEnvironmentEvidence::headless())],
        )
        .unwrap()
        .candidate
    }

    #[test]
    fn every_profile_is_complete_and_minimal_g4a_budget_does_not_request_adapter_maxima() {
        assert_eq!(
            profile_limits(GpuDeviceRequestProfile::ModernPortable),
            Limits::defaults()
        );
        assert_eq!(
            profile_limits(GpuDeviceRequestProfile::BrowserWebGpu),
            Limits::defaults()
        );
        assert_eq!(
            profile_limits(GpuDeviceRequestProfile::Downlevel),
            Limits::downlevel_defaults()
        );
        assert_eq!(
            profile_limits(GpuDeviceRequestProfile::DownlevelWebGl2),
            Limits::downlevel_webgl2_defaults()
        );
        let requested = requested_limits(&candidate()).unwrap();
        assert_eq!(requested.max_uniform_buffer_binding_size, 64 * 1024);
        assert_eq!(requested.max_storage_buffer_binding_size, 128 * 1024 * 1024);
        assert_eq!(requested.max_color_attachments, 1);
        assert_eq!(requested.max_vertex_buffers, 8);
        assert_eq!(requested.max_bindings_per_bind_group, 16);
        assert_eq!(requested.min_uniform_buffer_offset_alignment, 256);
        assert_eq!(requested.min_storage_buffer_offset_alignment, 256);
    }

    #[test]
    fn actual_device_mapping_records_only_actual_native_facts() {
        let mut native = Limits::defaults();
        native.max_vertex_buffers = 12;
        native.min_uniform_buffer_offset_alignment = 512;
        let facts = map_device_limits(&native);
        assert_eq!(facts.values().max_vertex_buffers(), 12);
        assert_eq!(facts.alignments().uniform_dynamic_offset, Some(512));
    }

    #[test]
    fn created_device_must_verify_every_requested_wgpu_feature() {
        assert!(verify_requested_features(Features::TIMESTAMP_QUERY, Features::empty()).is_err());
        assert!(
            verify_requested_features(Features::TIMESTAMP_QUERY, Features::TIMESTAMP_QUERY).is_ok()
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_fallback_policies_have_only_the_explicit_selection_routes() {
        assert_eq!(
            native_selection_route(GpuSoftwareFallbackPolicy::Allow),
            NativeAdapterSelectionRoute::Enumerated
        );
        assert_eq!(
            native_selection_route(GpuSoftwareFallbackPolicy::Forbid),
            NativeAdapterSelectionRoute::Enumerated
        );
        assert_eq!(
            native_selection_route(GpuSoftwareFallbackPolicy::Require),
            NativeAdapterSelectionRoute::ForcedFallback
        );
        assert_eq!(
            ordinary_enumeration_fallback_status(),
            GpuFallbackStatus::Unknown,
            "native enumeration must not claim non-fallback evidence without backend proof"
        );
        assert_eq!(
            backend_selected_fallback_status(true),
            GpuFallbackStatus::ConfirmedFallback
        );
        assert_eq!(
            backend_selected_fallback_status(false),
            GpuFallbackStatus::Unknown
        );
    }
}
