//! Private WGPU containment for G4A context admission and the one current-render loan.

use crate::plugins::gpu::{
    GpuAdapterClass, GpuAdapterFacts, GpuAlignmentFacts, GpuBackendFamily,
    GpuCandidateSelectionKind, GpuCapabilities, GpuCapabilityFeature, GpuContext,
    GpuContextAdmissionReport, GpuContextDescriptor, GpuContextRequestError,
    GpuContextRequestErrorCategory, GpuDeviceGeneration, GpuLimits, GpuSoftwareStatus,
    GpuTextureFormat, GpuTextureFormatCapabilities, admitted_device_facts, allocate_context_id,
    select_candidate, validate_descriptor,
};
use std::sync::Arc;
use wgpu::{
    Adapter, Backend, Device, DeviceDescriptor, DeviceType, ExperimentalFeatures, Features,
    Instance, InstanceDescriptor, Limits, MemoryHints, Queue, RequestAdapterOptions, Surface,
    SurfaceCapabilities, SurfaceConfiguration, SurfaceTarget, TextureFormat,
    TextureFormatFeatureFlags, TextureUsages, Trace,
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
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: map_power_preference(descriptor.power_preference()),
            force_fallback_adapter: matches!(
                descriptor.fallback_policy(),
                crate::plugins::gpu::GpuSoftwareFallbackPolicy::Require
            ),
            compatible_surface,
        })
        .await
        .map_err(|error| {
            GpuContextRequestError::new(
                GpuContextRequestErrorCategory::BackendAdapterRequestFailure,
                error.to_string(),
            )
        })?;
    let adapter_facts = adapter_facts(
        &adapter,
        compatible_surface.is_some(),
        matches!(
            descriptor.fallback_policy(),
            crate::plugins::gpu::GpuSoftwareFallbackPolicy::Require
        ),
    );
    let selection = select_candidate(
        &descriptor,
        [adapter_facts.clone()],
        compatible_surface.is_some(),
    )?;
    let candidate = selection.candidate;
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
            selected: GpuCandidateSelectionKind::BackendSelectedCandidate,
            candidate,
            candidate_dispositions: selection.dispositions,
        },
        backend: WgpuContextState {
            instance,
            adapter,
            device: Arc::new(device),
            queue: Arc::new(queue),
        },
    })
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
    fallback_requested: bool,
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
        fallback_requested,
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
