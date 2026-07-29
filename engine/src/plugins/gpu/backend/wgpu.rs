//! Private WGPU containment for G4A context admission and the one current-render loan.

use crate::plugins::gpu::{
    GpuAdapterClass, GpuAdapterFacts, GpuAlignmentFacts, GpuBackendFamily,
    GpuCandidateSelectionKind, GpuCapabilities, GpuCapabilityFeature, GpuContext,
    GpuContextAdmissionReport, GpuContextDescriptor, GpuContextRequestError,
    GpuContextRequestErrorCategory, GpuDeviceGeneration, GpuFallbackStatus, GpuLimits,
    GpuSoftwareStatus, GpuTextureFormat, GpuTextureFormatCapabilities, admitted_device_facts,
    allocate_context_id, select_candidate_with_host_evidence, validate_descriptor,
};
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use wgpu::Backends;
use wgpu::{
    Adapter, Backend, Device, DeviceDescriptor, DeviceType, ExperimentalFeatures, Features,
    Instance, InstanceDescriptor, Limits, MemoryHints, Queue, RequestAdapterError,
    RequestAdapterOptions, Surface, SurfaceCapabilities, SurfaceConfiguration, SurfaceTarget,
    TextureFormat, TextureFormatFeatureFlags, TextureUsages, Trace,
};

#[derive(Debug)]
pub(crate) struct WgpuContextState {
    instance: Instance,
    adapter: Adapter,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

#[derive(Debug)]
pub(crate) struct CurrentRenderDeviceQueue<'a> {
    pub(crate) device: &'a Device,
    pub(crate) queue: &'a Queue,
}

pub(crate) async fn request_headless(
    descriptor: GpuContextDescriptor,
) -> Result<GpuContext, GpuContextRequestError> {
    validate_descriptor(&descriptor)?;
    if descriptor
        .requirements()
        .iter()
        .any(|requirement| requirement.feature() == GpuCapabilityFeature::Presentation)
    {
        return Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::TemporaryHostCompatibilityFailure,
            "presentation admission requires the current host compatibility terminal",
        ));
    }
    request_with_instance(
        Instance::new(&InstanceDescriptor::default().with_env()),
        descriptor,
        None,
    )
    .await
}

async fn request_with_instance(
    instance: Instance,
    descriptor: GpuContextDescriptor,
    compatible_surface: Option<&Surface<'_>>,
) -> Result<GpuContext, GpuContextRequestError> {
    validate_descriptor(&descriptor)?;
    let (adapter, selection, selection_kind) =
        select_backend_adapter(&instance, &descriptor, compatible_surface).await?;
    let candidate = selection.candidate;
    let adapter_facts = candidate.adapter().clone();
    let requested_features = requested_features(&candidate);
    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            label: descriptor.label(),
            required_features: requested_features,
            required_limits: requested_limits(&candidate),
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
    let id = allocate_context_id()?;
    let device_facts = admitted_device_facts(&candidate);
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
) -> Result<
    (
        Adapter,
        crate::plugins::gpu::GpuCandidateSelection,
        GpuCandidateSelectionKind,
    ),
    GpuContextRequestError,
> {
    if native_selection_route(descriptor.fallback_policy())
        == NativeAdapterSelectionRoute::ForcedFallback
    {
        return select_backend_selected_adapter(instance, descriptor, compatible_surface, true)
            .await;
    }
    let candidates = instance
        .enumerate_adapters(Backends::all())
        .into_iter()
        .map(|adapter| {
            let surface_supported =
                compatible_surface.is_none_or(|surface| adapter.is_surface_supported(surface));
            let facts = adapter_facts(
                &adapter,
                surface_supported,
                GpuFallbackStatus::ConfirmedNotFallback,
            );
            (facts, adapter, surface_supported)
        })
        .collect::<Vec<_>>();
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
const fn native_selection_route(
    policy: crate::plugins::gpu::GpuSoftwareFallbackPolicy,
) -> NativeAdapterSelectionRoute {
    match policy {
        crate::plugins::gpu::GpuSoftwareFallbackPolicy::Require => {
            NativeAdapterSelectionRoute::ForcedFallback
        }
        crate::plugins::gpu::GpuSoftwareFallbackPolicy::Allow
        | crate::plugins::gpu::GpuSoftwareFallbackPolicy::Forbid => {
            NativeAdapterSelectionRoute::Enumerated
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn select_enumerated_adapter<T>(
    descriptor: &GpuContextDescriptor,
    candidates: Vec<(GpuAdapterFacts, T, bool)>,
) -> Result<(T, crate::plugins::gpu::GpuCandidateSelection), GpuContextRequestError> {
    let facts = candidates
        .iter()
        .map(|(facts, _, surface_supported)| (facts.clone(), *surface_supported))
        .collect::<Vec<_>>();
    let selection = select_candidate_with_host_evidence(descriptor, facts)?;
    let selected_facts = selection.candidate.adapter();
    let adapter = candidates
        .into_iter()
        .find_map(|(facts, adapter, _)| (facts == *selected_facts).then_some(adapter))
        .ok_or_else(|| {
            GpuContextRequestError::new(
                GpuContextRequestErrorCategory::BackendAdapterRequestFailure,
                "selected normalized adapter is absent from native enumeration",
            )
        })?;
    Ok((adapter, selection))
}

#[cfg(target_arch = "wasm32")]
async fn select_backend_adapter(
    instance: &Instance,
    descriptor: &GpuContextDescriptor,
    compatible_surface: Option<&Surface<'_>>,
) -> Result<
    (
        Adapter,
        crate::plugins::gpu::GpuCandidateSelection,
        GpuCandidateSelectionKind,
    ),
    GpuContextRequestError,
> {
    let fallback_required = matches!(
        descriptor.fallback_policy(),
        crate::plugins::gpu::GpuSoftwareFallbackPolicy::Require
    );
    select_backend_selected_adapter(instance, descriptor, compatible_surface, fallback_required)
        .await
}

async fn select_backend_selected_adapter(
    instance: &Instance,
    descriptor: &GpuContextDescriptor,
    compatible_surface: Option<&Surface<'_>>,
    force_fallback_adapter: bool,
) -> Result<
    (
        Adapter,
        crate::plugins::gpu::GpuCandidateSelection,
        GpuCandidateSelectionKind,
    ),
    GpuContextRequestError,
> {
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: map_power_preference(descriptor.power_preference()),
            force_fallback_adapter,
            compatible_surface,
        })
        .await
        .map_err(map_request_adapter_error)?;
    let facts = adapter_facts(
        &adapter,
        compatible_surface.is_some(),
        if force_fallback_adapter {
            GpuFallbackStatus::ConfirmedFallback
        } else {
            GpuFallbackStatus::Unknown
        },
    );
    let selection =
        select_candidate_with_host_evidence(descriptor, [(facts, compatible_surface.is_some())])?;
    Ok((
        adapter,
        selection,
        GpuCandidateSelectionKind::BackendSelectedCandidate,
    ))
}

fn map_request_adapter_error(error: RequestAdapterError) -> GpuContextRequestError {
    let category = match error {
        RequestAdapterError::NotFound { .. } => GpuContextRequestErrorCategory::NoCandidate,
        RequestAdapterError::EnvNotSet => {
            GpuContextRequestErrorCategory::BackendAdapterRequestFailure
        }
        _ => GpuContextRequestErrorCategory::BackendAdapterRequestFailure,
    };
    GpuContextRequestError::new(category, error.to_string())
}

impl GpuContext {
    /// The sole G4A current-host adapter selection terminal. G7 deletes it with surface ownership.
    pub(crate) async fn request_for_current_host<'window>(
        descriptor: GpuContextDescriptor,
        target: impl Into<SurfaceTarget<'window>>,
    ) -> Result<(Self, Surface<'window>), GpuContextRequestError> {
        validate_descriptor(&descriptor)?;
        let instance = Instance::new(&InstanceDescriptor::default().with_env());
        let surface = instance.create_surface(target).map_err(|error| {
            GpuContextRequestError::new(
                GpuContextRequestErrorCategory::TemporaryHostCompatibilityFailure,
                error.to_string(),
            )
        })?;
        let context = request_with_instance(instance, descriptor, Some(&surface)).await?;
        Ok((context, surface))
    }

    pub(crate) fn create_current_host_surface<'window>(
        &self,
        target: impl Into<SurfaceTarget<'window>>,
    ) -> Result<Surface<'window>, wgpu::CreateSurfaceError> {
        self.backend.instance.create_surface(target)
    }

    pub(crate) fn current_host_surface_capabilities(
        &self,
        surface: &Surface<'_>,
    ) -> SurfaceCapabilities {
        surface.get_capabilities(&self.backend.adapter)
    }

    pub(crate) fn configure_current_host_surface(
        &self,
        surface: &Surface<'_>,
        config: &SurfaceConfiguration,
    ) {
        surface.configure(&self.backend.device, config);
    }

    /// The sole crate-private G4A loan to the current renderer. G4C replaces this terminal.
    pub(crate) fn current_render_device_queue(&self) -> CurrentRenderDeviceQueue<'_> {
        CurrentRenderDeviceQueue {
            device: &self.backend.device,
            queue: &self.backend.queue,
        }
    }
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

fn requested_limits(candidate: &crate::plugins::gpu::GpuCandidateAdmissionReport) -> Limits {
    let facts = candidate.effective_limits();
    Limits {
        max_uniform_buffer_binding_size: facts.max_uniform_buffer_binding_size() as u32,
        max_storage_buffer_binding_size: facts.max_storage_buffer_binding_size() as u32,
        max_color_attachments: facts.max_color_attachments(),
        max_vertex_buffers: facts.max_vertex_buffers(),
        max_bindings_per_bind_group: facts.max_bindings_per_group(),
        ..Limits::default()
    }
}

fn adapter_facts(
    adapter: &Adapter,
    host_compatible: bool,
    fallback: GpuFallbackStatus,
) -> GpuAdapterFacts {
    let info = adapter.get_info();
    let features = adapter.features();
    let limits = adapter.limits();
    let formats = known_formats().into_iter().map(|(normalized, native)| {
        (
            normalized,
            format_capabilities(native, adapter.get_texture_format_features(native)),
        )
    });
    let mut supported = vec![
        GpuCapabilityFeature::Compute,
        GpuCapabilityFeature::RenderPipeline,
        GpuCapabilityFeature::Copy,
        GpuCapabilityFeature::IndirectDraw,
    ];
    if features.contains(Features::TIMESTAMP_QUERY) {
        supported.push(GpuCapabilityFeature::TimestampQuery);
    }
    if host_compatible {
        supported.push(GpuCapabilityFeature::Presentation);
    }
    if known_formats().iter().any(|(_, format)| {
        adapter
            .get_texture_format_features(*format)
            .allowed_usages
            .contains(TextureUsages::STORAGE_BINDING)
    }) {
        supported.push(GpuCapabilityFeature::StorageTexture);
    }
    if adapter
        .get_texture_format_features(TextureFormat::Depth32Float)
        .allowed_usages
        .contains(TextureUsages::RENDER_ATTACHMENT)
    {
        supported.push(GpuCapabilityFeature::DepthAttachment);
    }
    let limits = GpuLimits::from_validated_adapter_facts(
        u64::from(limits.max_uniform_buffer_binding_size),
        u64::from(limits.max_storage_buffer_binding_size),
        limits.max_color_attachments,
        limits.max_vertex_buffers,
        limits.max_bindings_per_bind_group,
    );
    GpuAdapterFacts::new(
        map_backend(info.backend),
        map_class(info.device_type),
        map_software(info.device_type),
        fallback,
        GpuCapabilities::from_normalized_facts(supported, limits, formats),
        GpuAlignmentFacts {
            uniform_dynamic_offset: Some(u64::from(limits_for_alignment(
                adapter.limits().min_uniform_buffer_offset_alignment,
            ))),
            storage_dynamic_offset: Some(u64::from(limits_for_alignment(
                adapter.limits().min_storage_buffer_offset_alignment,
            ))),
            copy_buffer_offset: Some(wgpu::COPY_BUFFER_ALIGNMENT),
            bytes_per_row: Some(u64::from(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)),
            query_resolve_destination: Some(wgpu::QUERY_RESOLVE_BUFFER_ALIGNMENT),
        },
    )
    .with_diagnostics(info.name, info.vendor, info.device)
}

const fn limits_for_alignment(value: u32) -> u32 {
    value
}

fn known_formats() -> [(GpuTextureFormat, TextureFormat); 6] {
    [
        (GpuTextureFormat::Rgba8Unorm, TextureFormat::Rgba8Unorm),
        (
            GpuTextureFormat::Rgba8UnormSrgb,
            TextureFormat::Rgba8UnormSrgb,
        ),
        (GpuTextureFormat::Bgra8Unorm, TextureFormat::Bgra8Unorm),
        (
            GpuTextureFormat::Bgra8UnormSrgb,
            TextureFormat::Bgra8UnormSrgb,
        ),
        (GpuTextureFormat::R32Uint, TextureFormat::R32Uint),
        (GpuTextureFormat::Depth32Float, TextureFormat::Depth32Float),
    ]
}

fn format_capabilities(
    format: TextureFormat,
    features: wgpu::TextureFormatFeatures,
) -> GpuTextureFormatCapabilities {
    GpuTextureFormatCapabilities {
        sampled: features
            .allowed_usages
            .contains(TextureUsages::TEXTURE_BINDING),
        filterable: features
            .flags
            .contains(TextureFormatFeatureFlags::FILTERABLE),
        storage_read: features
            .flags
            .contains(TextureFormatFeatureFlags::STORAGE_READ_ONLY)
            || features
                .flags
                .contains(TextureFormatFeatureFlags::STORAGE_READ_WRITE),
        storage_write: features
            .flags
            .contains(TextureFormatFeatureFlags::STORAGE_WRITE_ONLY)
            || features
                .flags
                .contains(TextureFormatFeatureFlags::STORAGE_READ_WRITE),
        color_attachment: !format.is_depth_stencil_format()
            && features
                .allowed_usages
                .contains(TextureUsages::RENDER_ATTACHMENT),
        depth_stencil: format.is_depth_stencil_format()
            && features
                .allowed_usages
                .contains(TextureUsages::RENDER_ATTACHMENT),
        copy_source: features.allowed_usages.contains(TextureUsages::COPY_SRC),
        copy_destination: features.allowed_usages.contains(TextureUsages::COPY_DST),
        block_dimensions: Some(format.block_dimensions()),
        block_copy_size: format.block_copy_size(None),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod native_selection_tests {
    use super::*;
    use crate::plugins::gpu::{
        GpuCapabilityRequirements, GpuContextDescriptor, GpuFallbackStatus, GpuLimits,
    };

    fn facts(
        class: GpuAdapterClass,
        fallback: GpuFallbackStatus,
        presentation: bool,
    ) -> GpuAdapterFacts {
        let mut features = vec![GpuCapabilityFeature::Compute, GpuCapabilityFeature::Copy];
        if presentation {
            features.push(GpuCapabilityFeature::Presentation);
        }
        GpuAdapterFacts::new(
            GpuBackendFamily::Vulkan,
            class,
            GpuSoftwareStatus::Hardware,
            fallback,
            GpuCapabilities::from_normalized_facts(
                features,
                GpuLimits::new(64 * 1024, 128 * 1024 * 1024, 1, 8, 16).unwrap(),
                [],
            ),
            GpuAlignmentFacts {
                uniform_dynamic_offset: Some(256),
                storage_dynamic_offset: Some(256),
                copy_buffer_offset: Some(4),
                bytes_per_row: Some(256),
                query_resolve_destination: Some(256),
            },
        )
    }

    #[test]
    fn native_enumerated_candidates_are_ranked_before_retaining_the_adapter_handle() {
        let descriptor = GpuContextDescriptor::new(GpuCapabilityRequirements::new())
            .with_power_preference(crate::plugins::gpu::GpuPowerPreference::HighPerformance);
        let forward = select_enumerated_adapter(
            &descriptor,
            vec![
                (
                    facts(
                        GpuAdapterClass::Integrated,
                        GpuFallbackStatus::ConfirmedNotFallback,
                        false,
                    ),
                    "integrated",
                    true,
                ),
                (
                    facts(
                        GpuAdapterClass::Discrete,
                        GpuFallbackStatus::ConfirmedNotFallback,
                        false,
                    ),
                    "discrete",
                    true,
                ),
            ],
        )
        .unwrap();
        let reverse = select_enumerated_adapter(
            &descriptor,
            vec![
                (
                    facts(
                        GpuAdapterClass::Discrete,
                        GpuFallbackStatus::ConfirmedNotFallback,
                        false,
                    ),
                    "discrete",
                    true,
                ),
                (
                    facts(
                        GpuAdapterClass::Integrated,
                        GpuFallbackStatus::ConfirmedNotFallback,
                        false,
                    ),
                    "integrated",
                    true,
                ),
            ],
        )
        .unwrap();
        assert_eq!(forward.0, "discrete");
        assert_eq!(reverse.0, "discrete");
        assert_eq!(forward.1.evidence, reverse.1.evidence);
    }

    #[test]
    fn native_fallback_policies_route_to_proven_selection_paths() {
        use crate::plugins::gpu::{
            GpuCapabilityRequirement, GpuPowerPreference, GpuSoftwareFallbackPolicy,
        };

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

        let mut requirements = GpuCapabilityRequirements::new();
        requirements
            .insert(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::Presentation,
            ))
            .unwrap();
        let host_descriptor = GpuContextDescriptor::new(requirements)
            .with_power_preference(GpuPowerPreference::HighPerformance);
        let enumerated = select_enumerated_adapter(
            &host_descriptor,
            vec![(
                facts(
                    GpuAdapterClass::Discrete,
                    GpuFallbackStatus::ConfirmedNotFallback,
                    true,
                ),
                "enumerated",
                true,
            )],
        )
        .unwrap();
        assert_eq!(enumerated.0, "enumerated");
        assert_eq!(
            enumerated.1.candidate.adapter().fallback(),
            GpuFallbackStatus::ConfirmedNotFallback
        );
        let forbidding = host_descriptor
            .clone()
            .with_fallback_policy(GpuSoftwareFallbackPolicy::Forbid);
        assert!(
            select_enumerated_adapter(
                &forbidding,
                vec![(
                    facts(
                        GpuAdapterClass::Discrete,
                        GpuFallbackStatus::ConfirmedNotFallback,
                        true,
                    ),
                    "forbidding",
                    true,
                )],
            )
            .is_ok()
        );
        assert!(matches!(
            select_enumerated_adapter(
                &host_descriptor,
                vec![(
                    facts(
                        GpuAdapterClass::Discrete,
                        GpuFallbackStatus::ConfirmedNotFallback,
                        true,
                    ),
                    "incompatible",
                    false,
                )],
            ),
            Err(error) if error.category() == GpuContextRequestErrorCategory::NoCandidate
        ));

        let requiring = host_descriptor
            .clone()
            .with_fallback_policy(GpuSoftwareFallbackPolicy::Require);
        let forced = select_candidate_with_host_evidence(
            &requiring,
            [(
                facts(
                    GpuAdapterClass::Discrete,
                    GpuFallbackStatus::ConfirmedFallback,
                    true,
                ),
                true,
            )],
        )
        .unwrap();
        assert_eq!(
            forced.candidate.adapter().fallback(),
            GpuFallbackStatus::ConfirmedFallback
        );
        assert_eq!(forced.evidence.rank().fallback_priority(), 2);
        assert!(matches!(
            select_candidate_with_host_evidence(
                &requiring,
                [(
                    facts(
                        GpuAdapterClass::Discrete,
                        GpuFallbackStatus::ConfirmedFallback,
                        true,
                    ),
                    false,
                )],
            ),
            Err(error) if error.category() == GpuContextRequestErrorCategory::NoCandidate
        ));
    }
}

const fn map_backend(backend: Backend) -> GpuBackendFamily {
    match backend {
        Backend::Vulkan => GpuBackendFamily::Vulkan,
        Backend::Metal => GpuBackendFamily::Metal,
        Backend::Dx12 => GpuBackendFamily::Direct3D12,
        Backend::Gl => GpuBackendFamily::OpenGl,
        Backend::BrowserWebGpu => GpuBackendFamily::BrowserWebGpu,
        Backend::Noop => GpuBackendFamily::UnknownBackend,
    }
}

const fn map_class(class: DeviceType) -> GpuAdapterClass {
    match class {
        DeviceType::DiscreteGpu => GpuAdapterClass::Discrete,
        DeviceType::IntegratedGpu => GpuAdapterClass::Integrated,
        DeviceType::VirtualGpu => GpuAdapterClass::Virtual,
        DeviceType::Cpu => GpuAdapterClass::Cpu,
        DeviceType::Other => GpuAdapterClass::Other,
    }
}

const fn map_software(class: DeviceType) -> GpuSoftwareStatus {
    match class {
        DeviceType::Cpu => GpuSoftwareStatus::Software,
        _ => GpuSoftwareStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_wgpu_backend_and_adapter_mappings_are_exhaustive() {
        assert_eq!(map_backend(Backend::Vulkan), GpuBackendFamily::Vulkan);
        assert_eq!(map_backend(Backend::Metal), GpuBackendFamily::Metal);
        assert_eq!(map_backend(Backend::Dx12), GpuBackendFamily::Direct3D12);
        assert_eq!(map_backend(Backend::Gl), GpuBackendFamily::OpenGl);
        assert_eq!(
            map_backend(Backend::BrowserWebGpu),
            GpuBackendFamily::BrowserWebGpu
        );
        assert_eq!(map_backend(Backend::Noop), GpuBackendFamily::UnknownBackend);

        assert_eq!(
            map_class(DeviceType::DiscreteGpu),
            GpuAdapterClass::Discrete
        );
        assert_eq!(
            map_class(DeviceType::IntegratedGpu),
            GpuAdapterClass::Integrated
        );
        assert_eq!(map_class(DeviceType::VirtualGpu), GpuAdapterClass::Virtual);
        assert_eq!(map_class(DeviceType::Cpu), GpuAdapterClass::Cpu);
        assert_eq!(map_class(DeviceType::Other), GpuAdapterClass::Other);
        assert_eq!(map_software(DeviceType::Cpu), GpuSoftwareStatus::Software);
        assert_eq!(
            map_software(DeviceType::DiscreteGpu),
            GpuSoftwareStatus::Unknown
        );
    }

    #[test]
    fn pinned_wgpu_texture_mapping_preserves_roles_and_block_facts() {
        let features = wgpu::TextureFormatFeatures {
            allowed_usages: TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST,
            flags: TextureFormatFeatureFlags::FILTERABLE,
        };
        let color = format_capabilities(TextureFormat::Rgba8Unorm, features);
        assert!(color.color_attachment);
        assert!(!color.depth_stencil);
        assert_eq!(color.block_dimensions, Some((1, 1)));
        assert_eq!(color.block_copy_size, Some(4));

        let depth = format_capabilities(TextureFormat::Depth32Float, features);
        assert!(!depth.color_attachment);
        assert!(depth.depth_stencil);
        assert_eq!(depth.block_dimensions, Some((1, 1)));
        assert_eq!(depth.block_copy_size, Some(4));
    }
}
