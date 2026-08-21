use super::adapter_mapping::known_formats;
use super::device_request::{enforce_runengpu_instance_flags, request_with_instance};
use super::{WgpuDeviceHealth, WgpuErrorAttributionGate};
use crate::plugins::gpu::{
    GpuCapabilityFeature, GpuCapabilityRequirement, GpuCapabilityRequirements, GpuContext,
    GpuContextAffinity, GpuContextDescriptor, GpuContextRequestError,
    GpuContextRequestErrorCategory, GpuExecutionPolicy, GpuRealizationPolicies,
    GpuSurfaceAlphaMode, GpuSurfaceCapabilities, GpuSurfaceConfiguration, GpuSurfaceError,
    GpuSurfaceErrorCategory, GpuSurfaceGeneration, GpuSurfaceHandle, GpuSurfaceId,
    GpuSurfacePresentMode, GpuSurfaceTarget, GpuTextureFormat, GpuTextureUsage,
    allocate_surface_id,
};
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};
use wgpu::{
    Adapter, CompositeAlphaMode, Device, Instance, InstanceDescriptor, PresentMode, Surface,
    SurfaceColorSpace, SurfaceConfiguration, TextureFormat, TextureUsages,
};

struct WgpuSurfaceRecord {
    generation: GpuSurfaceGeneration,
    capabilities: GpuSurfaceCapabilities,
    configuration: Option<GpuSurfaceConfiguration>,
    surface: Surface<'static>,
}

pub(crate) struct WgpuSurfaceState {
    affinity: GpuContextAffinity,
    records: Mutex<BTreeMap<GpuSurfaceId, WgpuSurfaceRecord>>,
}

impl WgpuSurfaceState {
    pub(crate) fn new(affinity: GpuContextAffinity) -> Self {
        Self {
            affinity,
            records: Mutex::new(BTreeMap::new()),
        }
    }

    fn records(
        &self,
        surface: Option<GpuSurfaceId>,
    ) -> Result<MutexGuard<'_, BTreeMap<GpuSurfaceId, WgpuSurfaceRecord>>, GpuSurfaceError> {
        self.records.lock().map_err(|_| {
            GpuSurfaceError::new(
                GpuSurfaceErrorCategory::ContextOrDeviceUnavailableOrLost,
                surface,
                "surface registry was poisoned by an aborted backend operation",
            )
        })
    }

    pub(crate) fn register_surface(
        &self,
        adapter: &Adapter,
        surface: Surface<'static>,
    ) -> Result<GpuSurfaceHandle, GpuSurfaceError> {
        if !adapter.is_surface_supported(&surface) {
            return Err(GpuSurfaceError::new(
                GpuSurfaceErrorCategory::AdapterIncompatible,
                None,
                "the admitted adapter is not compatible with this surface",
            ));
        }
        let capabilities = normalize_surface_capabilities(&surface.get_capabilities(adapter));
        if capabilities.formats().is_empty()
            || capabilities.present_modes().is_empty()
            || capabilities.alpha_modes().is_empty()
            || !capabilities.supports_usage(GpuTextureUsage::ColorAttachment)
        {
            return Err(GpuSurfaceError::new(
                GpuSurfaceErrorCategory::AdapterIncompatible,
                None,
                "the compatible backend surface has no complete normalized G7A presentation baseline",
            ));
        }

        let id = allocate_surface_id()?;
        let generation = GpuSurfaceGeneration::first();
        self.records(Some(id))?.insert(
            id,
            WgpuSurfaceRecord {
                generation,
                capabilities,
                configuration: None,
                surface,
            },
        );
        Ok(GpuSurfaceHandle::new(id, self.affinity, generation))
    }

    pub(crate) fn capabilities(
        &self,
        handle: GpuSurfaceHandle,
    ) -> Result<GpuSurfaceCapabilities, GpuSurfaceError> {
        let records = self.records(Some(handle.id()))?;
        let record = validate_handle(self.affinity, &records, handle)?;
        Ok(record.capabilities.clone())
    }

    pub(crate) fn configuration(
        &self,
        handle: GpuSurfaceHandle,
    ) -> Result<Option<GpuSurfaceConfiguration>, GpuSurfaceError> {
        let records = self.records(Some(handle.id()))?;
        let record = validate_handle(self.affinity, &records, handle)?;
        Ok(record.configuration.clone())
    }

    pub(crate) fn configure(
        &self,
        handle: GpuSurfaceHandle,
        configuration: GpuSurfaceConfiguration,
        device: &Device,
        health: &WgpuDeviceHealth,
        error_attribution_gate: &WgpuErrorAttributionGate,
    ) -> Result<GpuSurfaceHandle, GpuSurfaceError> {
        ensure_surface_health(health, Some(handle.id()))?;
        let _attribution_gate = error_attribution_gate.acquire();
        let mut records = self.records(Some(handle.id()))?;
        let record = validate_handle_mut(self.affinity, &mut records, handle)?;
        validate_configuration(handle.id(), &record.capabilities, &configuration)?;

        let next_generation = if record.configuration.is_some() {
            record.generation.next(handle.id())?
        } else {
            record.generation
        };
        let native = lower_configuration(&configuration);
        record.surface.configure(device, &native);
        ensure_surface_health(health, Some(handle.id()))?;
        record.configuration = Some(configuration);
        record.generation = next_generation;
        Ok(GpuSurfaceHandle::new(
            handle.id(),
            self.affinity,
            next_generation,
        ))
    }
}

impl core::fmt::Debug for WgpuSurfaceState {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let count = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        formatter
            .debug_struct("WgpuSurfaceState")
            .field("affinity", &self.affinity)
            .field("surface_count", &count)
            .finish()
    }
}

fn validate_handle<'a>(
    expected: GpuContextAffinity,
    records: &'a BTreeMap<GpuSurfaceId, WgpuSurfaceRecord>,
    handle: GpuSurfaceHandle,
) -> Result<&'a WgpuSurfaceRecord, GpuSurfaceError> {
    validate_surface_affinity(expected, handle)?;
    let record = records.get(&handle.id()).ok_or_else(|| {
        GpuSurfaceError::new(
            GpuSurfaceErrorCategory::UnknownSurface,
            Some(handle.id()),
            "surface identity is absent from this context-local surface owner",
        )
    })?;
    if record.generation != handle.generation() {
        return Err(GpuSurfaceError::new(
            GpuSurfaceErrorCategory::StaleGeneration,
            Some(handle.id()),
            "surface reference names a stale configuration generation",
        ));
    }
    Ok(record)
}

fn validate_handle_mut<'a>(
    expected: GpuContextAffinity,
    records: &'a mut BTreeMap<GpuSurfaceId, WgpuSurfaceRecord>,
    handle: GpuSurfaceHandle,
) -> Result<&'a mut WgpuSurfaceRecord, GpuSurfaceError> {
    validate_surface_affinity(expected, handle)?;
    let record = records.get_mut(&handle.id()).ok_or_else(|| {
        GpuSurfaceError::new(
            GpuSurfaceErrorCategory::UnknownSurface,
            Some(handle.id()),
            "surface identity is absent from this context-local surface owner",
        )
    })?;
    if record.generation != handle.generation() {
        return Err(GpuSurfaceError::new(
            GpuSurfaceErrorCategory::StaleGeneration,
            Some(handle.id()),
            "surface reference names a stale configuration generation",
        ));
    }
    Ok(record)
}

fn validate_surface_affinity(
    expected: GpuContextAffinity,
    handle: GpuSurfaceHandle,
) -> Result<(), GpuSurfaceError> {
    if handle.affinity().context() != expected.context() {
        return Err(GpuSurfaceError::new(
            GpuSurfaceErrorCategory::ForeignContext,
            Some(handle.id()),
            "surface reference belongs to a different GPU context",
        ));
    }
    if handle.affinity().generation() != expected.generation() {
        return Err(GpuSurfaceError::new(
            GpuSurfaceErrorCategory::StaleGeneration,
            Some(handle.id()),
            "surface reference belongs to a stale device generation",
        ));
    }
    Ok(())
}

fn ensure_surface_health(
    health: &WgpuDeviceHealth,
    surface: Option<GpuSurfaceId>,
) -> Result<(), GpuSurfaceError> {
    if let Some(fault) = health.terminal_fault() {
        Err(GpuSurfaceError::new(
            GpuSurfaceErrorCategory::ContextOrDeviceUnavailableOrLost,
            surface,
            fault.detail,
        ))
    } else {
        Ok(())
    }
}

fn normalize_surface_capabilities(native: &wgpu::SurfaceCapabilities) -> GpuSurfaceCapabilities {
    let mut formats = Vec::new();
    for native_format in &native.formats {
        if let Some(format) = normalize_texture_format(*native_format)
            && !formats.contains(&format)
        {
            formats.push(format);
        }
    }

    let mut usages = Vec::new();
    for (native_usage, normalized) in [
        (
            TextureUsages::RENDER_ATTACHMENT,
            GpuTextureUsage::ColorAttachment,
        ),
        (TextureUsages::COPY_SRC, GpuTextureUsage::CopySource),
        (TextureUsages::COPY_DST, GpuTextureUsage::CopyDestination),
    ] {
        if native.usages.contains(native_usage) {
            usages.push(normalized);
        }
    }

    let mut present_modes = Vec::new();
    for native_mode in &native.present_modes {
        if let Some(mode) = normalize_present_mode(*native_mode)
            && !present_modes.contains(&mode)
        {
            present_modes.push(mode);
        }
    }

    let mut alpha_modes = Vec::new();
    for native_mode in &native.alpha_modes {
        if let Some(mode) = normalize_alpha_mode(*native_mode)
            && !alpha_modes.contains(&mode)
        {
            alpha_modes.push(mode);
        }
    }

    GpuSurfaceCapabilities::from_normalized_facts(formats, usages, present_modes, alpha_modes)
}

fn validate_configuration(
    surface: GpuSurfaceId,
    capabilities: &GpuSurfaceCapabilities,
    configuration: &GpuSurfaceConfiguration,
) -> Result<(), GpuSurfaceError> {
    if !capabilities.supports_format(configuration.format()) {
        return Err(GpuSurfaceError::new(
            GpuSurfaceErrorCategory::UnsupportedFormat,
            Some(surface),
            "configured surface format is absent from the normalized surface capability set",
        ));
    }
    if let Some(usage) = configuration
        .usages()
        .iter()
        .find(|usage| !capabilities.supports_usage(**usage))
    {
        return Err(GpuSurfaceError::new(
            GpuSurfaceErrorCategory::UnsupportedUsage,
            Some(surface),
            format!("configured surface usage is unsupported: {usage:?}"),
        ));
    }
    if !capabilities.supports_present_mode(configuration.present_mode()) {
        return Err(GpuSurfaceError::new(
            GpuSurfaceErrorCategory::UnsupportedPresentMode,
            Some(surface),
            "configured present mode is absent from the normalized surface capability set",
        ));
    }
    if !capabilities.supports_alpha_mode(configuration.alpha_mode()) {
        return Err(GpuSurfaceError::new(
            GpuSurfaceErrorCategory::UnsupportedAlphaMode,
            Some(surface),
            "configured alpha mode is absent from the normalized surface capability set",
        ));
    }
    if configuration
        .view_formats()
        .iter()
        .any(|format| map_texture_format(*format).is_none())
    {
        return Err(GpuSurfaceError::new(
            GpuSurfaceErrorCategory::UnsupportedViewFormat,
            Some(surface),
            "configured surface view format is absent from the normalized backend format vocabulary",
        ));
    }
    Ok(())
}

fn lower_configuration(configuration: &GpuSurfaceConfiguration) -> SurfaceConfiguration {
    let usage = configuration
        .usages()
        .iter()
        .fold(TextureUsages::empty(), |native, usage| {
            native
                | match usage {
                    GpuTextureUsage::ColorAttachment => TextureUsages::RENDER_ATTACHMENT,
                    GpuTextureUsage::CopySource => TextureUsages::COPY_SRC,
                    GpuTextureUsage::CopyDestination => TextureUsages::COPY_DST,
                    _ => TextureUsages::empty(),
                }
        });
    SurfaceConfiguration {
        usage,
        format: map_texture_format(configuration.format())
            .expect("surface configuration construction accepts only normalized backend formats"),
        color_space: SurfaceColorSpace::Auto,
        width: configuration.width(),
        height: configuration.height(),
        present_mode: map_present_mode(configuration.present_mode()),
        desired_maximum_frame_latency: configuration.desired_maximum_frame_latency(),
        alpha_mode: map_alpha_mode(configuration.alpha_mode()),
        view_formats: configuration
            .view_formats()
            .iter()
            .copied()
            .map(|format| {
                map_texture_format(format).expect(
                    "surface view format construction accepts only normalized backend formats",
                )
            })
            .collect(),
    }
}

fn normalize_texture_format(native: TextureFormat) -> Option<GpuTextureFormat> {
    known_formats()
        .into_iter()
        .find_map(|(normalized, candidate)| (candidate == native).then_some(normalized))
}

fn map_texture_format(format: GpuTextureFormat) -> Option<TextureFormat> {
    known_formats()
        .into_iter()
        .find_map(|(candidate, native)| (candidate == format).then_some(native))
}

const fn normalize_present_mode(native: PresentMode) -> Option<GpuSurfacePresentMode> {
    match native {
        PresentMode::Fifo => Some(GpuSurfacePresentMode::Fifo),
        PresentMode::FifoRelaxed => Some(GpuSurfacePresentMode::FifoRelaxed),
        PresentMode::Immediate => Some(GpuSurfacePresentMode::Immediate),
        PresentMode::Mailbox => Some(GpuSurfacePresentMode::Mailbox),
        PresentMode::AutoVsync | PresentMode::AutoNoVsync => None,
    }
}

const fn map_present_mode(mode: GpuSurfacePresentMode) -> PresentMode {
    match mode {
        GpuSurfacePresentMode::Fifo => PresentMode::Fifo,
        GpuSurfacePresentMode::FifoRelaxed => PresentMode::FifoRelaxed,
        GpuSurfacePresentMode::Immediate => PresentMode::Immediate,
        GpuSurfacePresentMode::Mailbox => PresentMode::Mailbox,
    }
}

const fn normalize_alpha_mode(native: CompositeAlphaMode) -> Option<GpuSurfaceAlphaMode> {
    match native {
        CompositeAlphaMode::Opaque => Some(GpuSurfaceAlphaMode::Opaque),
        CompositeAlphaMode::PreMultiplied => Some(GpuSurfaceAlphaMode::PreMultiplied),
        CompositeAlphaMode::PostMultiplied => Some(GpuSurfaceAlphaMode::PostMultiplied),
        CompositeAlphaMode::Inherit => Some(GpuSurfaceAlphaMode::Inherit),
        CompositeAlphaMode::Auto => None,
    }
}

const fn map_alpha_mode(mode: GpuSurfaceAlphaMode) -> CompositeAlphaMode {
    match mode {
        GpuSurfaceAlphaMode::Opaque => CompositeAlphaMode::Opaque,
        GpuSurfaceAlphaMode::PreMultiplied => CompositeAlphaMode::PreMultiplied,
        GpuSurfaceAlphaMode::PostMultiplied => CompositeAlphaMode::PostMultiplied,
        GpuSurfaceAlphaMode::Inherit => CompositeAlphaMode::Inherit,
    }
}

fn descriptor_with_required_presentation(
    descriptor: GpuContextDescriptor,
) -> Result<GpuContextDescriptor, GpuContextRequestError> {
    let label = descriptor.label().map(str::to_owned);
    let provenance = descriptor.provenance().map(str::to_owned);

    let mut requirements = GpuCapabilityRequirements::new();
    requirements
        .insert(GpuCapabilityRequirement::Required(
            GpuCapabilityFeature::Presentation,
        ))
        .map_err(|error| {
            GpuContextRequestError::new(
                GpuContextRequestErrorCategory::ContradictoryRequest,
                error.to_string(),
            )
        })?;
    let surface_constraint = GpuContextDescriptor::new(requirements);
    let mut merged = descriptor.merge(&surface_constraint)?;

    if let Some(label) = label {
        merged = merged.with_label(label);
    }
    if let Some(provenance) = provenance {
        merged = merged.with_provenance(provenance);
    }
    Ok(merged)
}

pub(crate) async fn request_for_surface<T>(
    descriptor: GpuContextDescriptor,
    realization_policies: GpuRealizationPolicies,
    execution_policy: GpuExecutionPolicy,
    target: T,
) -> Result<(GpuContext, GpuSurfaceHandle), GpuContextRequestError>
where
    T: GpuSurfaceTarget,
{
    let descriptor = descriptor_with_required_presentation(descriptor)?;
    let instance = Instance::new(surface_instance_descriptor(&target));
    let surface = instance.create_surface(target).map_err(|error| {
        GpuContextRequestError::new(
            GpuContextRequestErrorCategory::SurfaceCreationFailure,
            error.to_string(),
        )
    })?;
    let context = request_with_instance(
        instance,
        descriptor,
        Some(&surface),
        realization_policies,
        execution_policy,
    )
    .await?;
    let handle = context
        .backend
        .surfaces
        .register_surface(&context.backend.adapter, surface)
        .map_err(map_surface_registration_error)?;
    Ok((context, handle))
}

fn surface_instance_descriptor<T: GpuSurfaceTarget>(target: &T) -> InstanceDescriptor {
    #[cfg(not(target_arch = "wasm32"))]
    {
        enforce_runengpu_instance_flags(InstanceDescriptor::new_with_display_handle_from_env(
            Box::new(target.clone()),
        ))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = target;
        enforce_runengpu_instance_flags(InstanceDescriptor::new_without_display_handle_from_env())
    }
}

fn map_surface_registration_error(error: GpuSurfaceError) -> GpuContextRequestError {
    let category = match error.category() {
        GpuSurfaceErrorCategory::IdentityExhausted => {
            GpuContextRequestErrorCategory::IdentityExhausted
        }
        GpuSurfaceErrorCategory::ContextOrDeviceUnavailableOrLost => {
            GpuContextRequestErrorCategory::BackendDeviceRequestFailure
        }
        _ => GpuContextRequestErrorCategory::SurfaceCompatibilityFailure,
    };
    GpuContextRequestError::new(category, error.to_string())
}

impl GpuContext {
    pub async fn request_for_surface<T>(
        descriptor: GpuContextDescriptor,
        target: T,
    ) -> Result<(Self, GpuSurfaceHandle), GpuContextRequestError>
    where
        T: GpuSurfaceTarget,
    {
        Self::request_for_surface_with_policies(
            descriptor,
            GpuRealizationPolicies::default(),
            GpuExecutionPolicy::default(),
            target,
        )
        .await
    }

    pub async fn request_for_surface_with_policies<T>(
        descriptor: GpuContextDescriptor,
        realization_policies: GpuRealizationPolicies,
        execution_policy: GpuExecutionPolicy,
        target: T,
    ) -> Result<(Self, GpuSurfaceHandle), GpuContextRequestError>
    where
        T: GpuSurfaceTarget,
    {
        request_for_surface(descriptor, realization_policies, execution_policy, target).await
    }

    pub fn attach_surface<T>(&self, target: T) -> Result<GpuSurfaceHandle, GpuSurfaceError>
    where
        T: GpuSurfaceTarget,
    {
        ensure_surface_health(&self.backend.health, None)?;
        let surface = self.backend.instance.create_surface(target).map_err(|error| {
            GpuSurfaceError::new(
                GpuSurfaceErrorCategory::BackendCreationFailure,
                None,
                error.to_string(),
            )
        })?;
        self.backend
            .surfaces
            .register_surface(&self.backend.adapter, surface)
    }

    pub fn surface_capabilities(
        &self,
        surface: GpuSurfaceHandle,
    ) -> Result<GpuSurfaceCapabilities, GpuSurfaceError> {
        ensure_surface_health(&self.backend.health, Some(surface.id()))?;
        self.backend.surfaces.capabilities(surface)
    }

    pub fn surface_configuration(
        &self,
        surface: GpuSurfaceHandle,
    ) -> Result<Option<GpuSurfaceConfiguration>, GpuSurfaceError> {
        ensure_surface_health(&self.backend.health, Some(surface.id()))?;
        self.backend.surfaces.configuration(surface)
    }

    pub fn configure_surface(
        &self,
        surface: GpuSurfaceHandle,
        configuration: GpuSurfaceConfiguration,
    ) -> Result<GpuSurfaceHandle, GpuSurfaceError> {
        self.backend.surfaces.configure(
            surface,
            configuration,
            &self.backend.device,
            &self.backend.health,
            &self.backend.error_attribution_gate,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{
        GpuContextId, GpuDeviceGeneration, GpuPreferredFallback,
    };
    use std::num::NonZeroU64;

    fn capabilities() -> GpuSurfaceCapabilities {
        GpuSurfaceCapabilities::from_normalized_facts(
            vec![
                GpuTextureFormat::Bgra8Unorm,
                GpuTextureFormat::Bgra8UnormSrgb,
            ],
            vec![
                GpuTextureUsage::ColorAttachment,
                GpuTextureUsage::CopySource,
            ],
            vec![GpuSurfacePresentMode::Fifo],
            vec![GpuSurfaceAlphaMode::Opaque],
        )
    }

    #[test]
    fn capability_normalization_keeps_only_explicit_g7a_vocabulary() {
        let native = wgpu::SurfaceCapabilities {
            formats: vec![TextureFormat::Bgra8UnormSrgb, TextureFormat::Rgba16Float],
            format_capabilities: Vec::new(),
            present_modes: vec![PresentMode::AutoVsync, PresentMode::Fifo],
            alpha_modes: vec![CompositeAlphaMode::Auto, CompositeAlphaMode::Opaque],
            usages: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_SRC
                | TextureUsages::TEXTURE_BINDING,
        };
        let normalized = normalize_surface_capabilities(&native);
        assert_eq!(normalized.formats(), &[GpuTextureFormat::Bgra8UnormSrgb]);
        assert_eq!(
            normalized.usages(),
            &[
                GpuTextureUsage::ColorAttachment,
                GpuTextureUsage::CopySource
            ]
        );
        assert_eq!(normalized.present_modes(), &[GpuSurfacePresentMode::Fifo]);
        assert_eq!(normalized.alpha_modes(), &[GpuSurfaceAlphaMode::Opaque]);
    }

    #[test]
    fn configuration_admission_uses_surface_local_capability_evidence() {
        let surface = allocate_surface_id().unwrap();
        let accepted = GpuSurfaceConfiguration::new(
            640,
            480,
            GpuTextureFormat::Bgra8Unorm,
            [
                GpuTextureUsage::ColorAttachment,
                GpuTextureUsage::CopySource,
            ],
            GpuSurfacePresentMode::Fifo,
            GpuSurfaceAlphaMode::Opaque,
            2,
            [GpuTextureFormat::Bgra8UnormSrgb],
        )
        .unwrap();
        assert!(validate_configuration(surface, &capabilities(), &accepted).is_ok());

        let unsupported = GpuSurfaceConfiguration::new(
            640,
            480,
            GpuTextureFormat::Bgra8Unorm,
            [
                GpuTextureUsage::ColorAttachment,
                GpuTextureUsage::CopyDestination,
            ],
            GpuSurfacePresentMode::Fifo,
            GpuSurfaceAlphaMode::Opaque,
            2,
            [],
        )
        .unwrap();
        assert!(matches!(
            validate_configuration(surface, &capabilities(), &unsupported),
            Err(error) if error.category() == GpuSurfaceErrorCategory::UnsupportedUsage
        ));
    }

    #[test]
    fn presentation_first_descriptor_contributes_required_presentation_and_keeps_diagnostics() {
        let descriptor = GpuContextDescriptor::new(GpuCapabilityRequirements::new())
            .with_label("surface-first")
            .with_provenance("g7a1-test");
        let descriptor = descriptor_with_required_presentation(descriptor).unwrap();
        assert_eq!(
            descriptor
                .requirements()
                .get(GpuCapabilityFeature::Presentation),
            Some(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::Presentation
            ))
        );
        assert_eq!(descriptor.label(), Some("surface-first"));
        assert_eq!(descriptor.provenance(), Some("g7a1-test"));

        let mut preferred = GpuCapabilityRequirements::new();
        preferred
            .insert(GpuCapabilityRequirement::Preferred {
                feature: GpuCapabilityFeature::Presentation,
                fallback: GpuPreferredFallback::ContinueWithoutFeature,
            })
            .unwrap();
        let descriptor =
            descriptor_with_required_presentation(GpuContextDescriptor::new(preferred)).unwrap();
        assert_eq!(
            descriptor
                .requirements()
                .get(GpuCapabilityFeature::Presentation),
            Some(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::Presentation
            ))
        );
    }

    #[test]
    fn presentation_first_descriptor_rejects_disabled_presentation_before_backend_action() {
        let mut requirements = GpuCapabilityRequirements::new();
        requirements
            .insert(GpuCapabilityRequirement::Disabled(
                GpuCapabilityFeature::Presentation,
            ))
            .unwrap();
        let error = descriptor_with_required_presentation(GpuContextDescriptor::new(requirements))
            .unwrap_err();
        assert_eq!(
            error.category(),
            GpuContextRequestErrorCategory::ContradictoryRequest
        );
    }

    #[test]
    fn surface_handle_affinity_rejects_foreign_context_and_stale_device_generation() {
        let surface = allocate_surface_id().unwrap();
        let context_one = GpuContextId::test_value(NonZeroU64::new(1).unwrap());
        let context_two = GpuContextId::test_value(NonZeroU64::new(2).unwrap());
        let generation_one = GpuDeviceGeneration::first();
        let generation_two = GpuDeviceGeneration::test_value(NonZeroU64::new(2).unwrap());
        let expected = GpuContextAffinity::test_value(context_one, generation_one);

        let foreign = GpuSurfaceHandle::new(
            surface,
            GpuContextAffinity::test_value(context_two, generation_one),
            GpuSurfaceGeneration::first(),
        );
        assert!(matches!(
            validate_surface_affinity(expected, foreign),
            Err(error) if error.category() == GpuSurfaceErrorCategory::ForeignContext
        ));

        let stale = GpuSurfaceHandle::new(
            surface,
            GpuContextAffinity::test_value(context_one, generation_two),
            GpuSurfaceGeneration::first(),
        );
        assert!(matches!(
            validate_surface_affinity(expected, stale),
            Err(error) if error.category() == GpuSurfaceErrorCategory::StaleGeneration
        ));
    }
}
