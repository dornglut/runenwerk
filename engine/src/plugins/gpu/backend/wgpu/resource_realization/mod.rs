//! Context/device-generation-bound private WGPU resource realization.

mod current_render_resource_bridge;
mod lowering;
mod records;
mod registry;

pub(crate) use current_render_resource_bridge::*;
pub(crate) use records::{
    BufferRealizationRecord, QuerySetRealizationRecord, SamplerRealizationRecord,
    TextureRealizationRecord, TextureViewRealizationRecord,
};

use super::WgpuContextState;
use crate::plugins::gpu::{
    GpuBufferHandle, GpuContext, GpuContextAffinity, GpuContextAffinityError, GpuQuerySetHandle,
    GpuRealizedBuffer, GpuRealizedQuerySet, GpuRealizedSampler, GpuRealizedTexture,
    GpuRealizedTextureView, GpuResourceRealizationError, GpuResourceRealizationErrorCategory,
    GpuResourceRealizationPolicy, GpuResourceRealizationStats, GpuSamplerHandle, GpuTextureHandle,
    GpuTextureViewHandle, GpuWorkResourceId,
};
use registry::{ResourceKind, ResourceRegistries};
use std::sync::{Arc, Mutex, MutexGuard};
use wgpu::{
    BufferDescriptor, Device, QuerySetDescriptor, SamplerDescriptor, TextureDescriptor,
    TextureViewDescriptor,
};

#[derive(Debug, Clone)]
struct DeviceFault {
    category: GpuResourceRealizationErrorCategory,
    detail: String,
}

/// Device-wide WGPU fault observation shared by all current G4C1 realization operations.
///
/// WGPU may report validation and allocation failures after the constructor call returns. Those
/// faults therefore cannot be truthfully attributed to a thread-local synchronous constructor
/// slot. The first observed device fault instead makes this context unavailable; subsequent
/// realization and bridge access returns the retained structured category and bounded evidence.
#[derive(Debug)]
struct DeviceHealth {
    fault: Mutex<Option<DeviceFault>>,
}

impl DeviceHealth {
    fn new() -> Self {
        Self {
            fault: Mutex::new(None),
        }
    }

    fn mark_fault(&self, category: GpuResourceRealizationErrorCategory, detail: impl Into<String>) {
        let detail = detail.into().chars().take(256).collect::<String>();
        let mut retained = self
            .fault
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if retained.is_none() {
            *retained = Some(DeviceFault { category, detail });
        }
    }

    fn mark_uncaptured(&self, error: wgpu::Error) {
        let category = match error {
            wgpu::Error::OutOfMemory { .. } => {
                GpuResourceRealizationErrorCategory::BackendResourceExhaustion
            }
            wgpu::Error::Validation { .. } => {
                GpuResourceRealizationErrorCategory::UnexpectedBackendValidationRejection
            }
            wgpu::Error::Internal { .. } => {
                GpuResourceRealizationErrorCategory::ContextOrDeviceUnavailableOrLost
            }
        };
        self.mark_fault(category, format!("uncaptured WGPU backend error: {error}"));
    }

    fn mark_lost(&self, reason: wgpu::DeviceLostReason, detail: String) {
        let bounded = detail.chars().take(256).collect::<String>();
        let diagnostic = if bounded.trim().is_empty() {
            format!("device became unavailable ({reason:?})")
        } else {
            format!("device became unavailable ({reason:?}): {bounded}")
        };
        self.mark_fault(
            GpuResourceRealizationErrorCategory::ContextOrDeviceUnavailableOrLost,
            diagnostic,
        );
    }

    fn ensure_available(
        &self,
        resource: GpuWorkResourceId,
    ) -> Result<(), GpuResourceRealizationError> {
        let retained = self
            .fault
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(fault) = retained.as_ref() else {
            return Ok(());
        };
        Err(GpuResourceRealizationError::new(
            fault.category,
            Some(resource),
            fault.detail.clone(),
        ))
    }
}

/// The sole authoritative owner of G4C1 lookup state for one context/device generation.
pub(crate) struct ResourceRealizationState {
    affinity: GpuContextAffinity,
    policy: GpuResourceRealizationPolicy,
    registries: Mutex<ResourceRegistries>,
    health: Arc<DeviceHealth>,
}

impl ResourceRealizationState {
    pub(crate) fn new(affinity: GpuContextAffinity, policy: GpuResourceRealizationPolicy) -> Self {
        Self {
            affinity,
            policy,
            registries: Mutex::new(ResourceRegistries::default()),
            health: Arc::new(DeviceHealth::new()),
        }
    }

    pub(crate) fn install_device_observers(&self, device: &Device) {
        let lost_health = Arc::clone(&self.health);
        device.set_device_lost_callback(move |reason, detail| lost_health.mark_lost(reason, detail));
        let uncaptured_health = Arc::clone(&self.health);
        device.on_uncaptured_error(Arc::new(move |error| {
            uncaptured_health.mark_uncaptured(error)
        }));
    }

    pub(crate) const fn policy(&self) -> GpuResourceRealizationPolicy {
        self.policy
    }

    pub(crate) fn stats(&self) -> GpuResourceRealizationStats {
        self.registries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stats(self.policy)
    }

    fn ensure_available(
        &self,
        resource: GpuWorkResourceId,
    ) -> Result<(), GpuResourceRealizationError> {
        self.health.ensure_available(resource)
    }

    fn registries(
        &self,
        resource: GpuWorkResourceId,
    ) -> Result<MutexGuard<'_, ResourceRegistries>, GpuResourceRealizationError> {
        self.registries.lock().map_err(|_| {
            GpuResourceRealizationError::new(
                GpuResourceRealizationErrorCategory::ContextOrDeviceUnavailableOrLost,
                Some(resource),
                "resource-realization authority was poisoned by an aborted backend operation",
            )
        })
    }

    fn create_backend_object<Object>(
        &self,
        resource: GpuWorkResourceId,
        create: impl FnOnce() -> Object,
    ) -> Result<Object, GpuResourceRealizationError> {
        self.ensure_available(resource)?;
        let created = create();
        // A synchronously delivered WGPU callback is observed before publication. WGPU may also
        // deliver a backend fault later; in that case the context becomes unavailable and every
        // later realization/bridge access rejects it rather than attributing the fault to another
        // constructor call.
        self.ensure_available(resource)?;
        Ok(created)
    }
}

impl core::fmt::Debug for ResourceRealizationState {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ResourceRealizationState")
            .field("affinity", &self.affinity)
            .field("stats", &self.stats())
            .finish_non_exhaustive()
    }
}

impl GpuContext {
    /// Returns the operational record policy chosen when this context was requested.
    pub fn resource_realization_policy(&self) -> GpuResourceRealizationPolicy {
        self.backend.resource_realization.policy()
    }

    /// Observes authoritative lookup counts. Per-kind counts are observations, not quotas.
    pub fn resource_realization_stats(&self) -> GpuResourceRealizationStats {
        self.backend.resource_realization.stats()
    }

    pub fn realize_buffer(
        &self,
        handle: &GpuBufferHandle,
    ) -> Result<GpuRealizedBuffer, GpuResourceRealizationError> {
        let identity = handle.diagnostic_identity();
        lowering::validate_resource_ownership(identity, handle.descriptor().common())?;
        self.backend
            .resource_realization
            .ensure_available(identity)?;
        let native_usage = lowering::lower_buffer(self, identity, handle.descriptor())?;
        let descriptor = handle.retained_descriptor();
        let mut registries = self.backend.resource_realization.registries(identity)?;
        if let Some(record) = registries.buffers.lookup(identity, &descriptor)? {
            return Ok(GpuRealizedBuffer::from_record(record));
        }
        registries.reject_other_kind(ResourceKind::Buffer, identity)?;
        registries.ensure_capacity(identity, self.backend.resource_realization.policy)?;

        let object = self
            .backend
            .resource_realization
            .create_backend_object(identity, || {
                self.backend.device.create_buffer(&BufferDescriptor {
                    label: Some(descriptor.common().label().as_str()),
                    size: descriptor.size_bytes(),
                    usage: native_usage,
                    mapped_at_creation: false,
                })
            })?;
        let record = Arc::new(BufferRealizationRecord {
            affinity: self.affinity(),
            logical_identity: identity,
            descriptor,
            object,
        });
        registries.buffers.insert(identity, Arc::clone(&record));
        Ok(GpuRealizedBuffer::from_record(record))
    }

    pub fn realize_texture(
        &self,
        handle: &GpuTextureHandle,
    ) -> Result<GpuRealizedTexture, GpuResourceRealizationError> {
        let identity = handle.diagnostic_identity();
        lowering::validate_resource_ownership(identity, handle.descriptor().common())?;
        self.backend
            .resource_realization
            .ensure_available(identity)?;
        let lowered = lowering::lower_texture(self, identity, handle.descriptor())?;
        let descriptor = handle.retained_descriptor();
        let mut registries = self.backend.resource_realization.registries(identity)?;
        if let Some(record) = registries.textures.lookup(identity, &descriptor)? {
            return Ok(GpuRealizedTexture::from_record(record));
        }
        registries.reject_other_kind(ResourceKind::Texture, identity)?;
        registries.ensure_capacity(identity, self.backend.resource_realization.policy)?;

        let paired = lowered.paired_view_format.into_iter().collect::<Vec<_>>();
        let object = self
            .backend
            .resource_realization
            .create_backend_object(identity, || {
                self.backend.device.create_texture(&TextureDescriptor {
                    label: Some(descriptor.common().label().as_str()),
                    size: lowered.size,
                    mip_level_count: descriptor.mip_level_count(),
                    sample_count: descriptor.sample_count(),
                    dimension: lowered.dimension,
                    format: lowered.format,
                    usage: lowered.usage,
                    view_formats: &paired,
                })
            })?;
        let record = Arc::new(TextureRealizationRecord {
            affinity: self.affinity(),
            logical_identity: identity,
            descriptor,
            object,
            permits_format_reinterpretation: lowered.permits_format_reinterpretation,
        });
        registries.textures.insert(identity, Arc::clone(&record));
        Ok(GpuRealizedTexture::from_record(record))
    }

    /// Realizes a view only within the exact already-realized parent texture.
    pub fn realize_texture_view(
        &self,
        handle: &GpuTextureViewHandle,
        parent: &GpuRealizedTexture,
    ) -> Result<GpuRealizedTextureView, GpuResourceRealizationError> {
        let identity = handle.diagnostic_identity();
        lowering::validate_resource_ownership(identity, handle.descriptor().common())?;
        validate_realized_input_affinity(self.affinity(), identity, parent.affinity())?;
        self.backend
            .resource_realization
            .ensure_available(identity)?;
        lowering::validate_texture_view(self, identity, handle.descriptor(), &parent.record)?;
        let descriptor = handle.retained_descriptor();
        let mut registries = self.backend.resource_realization.registries(identity)?;

        let authoritative_parent = registries
            .textures
            .lookup(parent.logical_identity(), parent.descriptor())?
            .ok_or_else(|| {
                GpuResourceRealizationError::new(
                    GpuResourceRealizationErrorCategory::UnknownLogicalResource,
                    Some(parent.logical_identity()),
                    "the parent texture is absent from this context's authoritative registry",
                )
            })?;
        if !Arc::ptr_eq(&authoritative_parent, &parent.record) {
            return Err(GpuResourceRealizationError::new(
                GpuResourceRealizationErrorCategory::UnknownLogicalResource,
                Some(parent.logical_identity()),
                "the supplied parent is not the authoritative texture realization record",
            ));
        }
        if let Some(record) = registries.texture_views.lookup(identity, &descriptor)? {
            return Ok(GpuRealizedTextureView::from_record(record));
        }
        registries.reject_other_kind(ResourceKind::TextureView, identity)?;
        registries.ensure_capacity(identity, self.backend.resource_realization.policy)?;

        let subresources = descriptor.subresources();
        let object = self
            .backend
            .resource_realization
            .create_backend_object(identity, || {
                authoritative_parent
                    .object
                    .create_view(&TextureViewDescriptor {
                        label: Some(descriptor.common().label().as_str()),
                        format: descriptor.format().map(lowering::map_texture_format),
                        dimension: Some(lowering::map_texture_view_dimension(&descriptor)),
                        usage: None,
                        aspect: lowering::map_texture_aspect(subresources.aspect()),
                        base_mip_level: subresources.base_mip_level(),
                        mip_level_count: Some(subresources.mip_level_count()),
                        base_array_layer: subresources.base_array_layer(),
                        array_layer_count: Some(subresources.array_layer_count()),
                    })
            })?;
        let record = Arc::new(TextureViewRealizationRecord {
            affinity: self.affinity(),
            logical_identity: identity,
            descriptor,
            object,
            parent: authoritative_parent,
        });
        registries
            .texture_views
            .insert(identity, Arc::clone(&record));
        Ok(GpuRealizedTextureView::from_record(record))
    }

    pub fn realize_sampler(
        &self,
        handle: &GpuSamplerHandle,
    ) -> Result<GpuRealizedSampler, GpuResourceRealizationError> {
        let identity = handle.diagnostic_identity();
        lowering::validate_resource_ownership(identity, handle.descriptor().common())?;
        self.backend
            .resource_realization
            .ensure_available(identity)?;
        lowering::validate_sampler(identity, handle.descriptor())?;
        let descriptor = handle.retained_descriptor();
        let mut registries = self.backend.resource_realization.registries(identity)?;
        if let Some(record) = registries.samplers.lookup(identity, &descriptor)? {
            return Ok(GpuRealizedSampler::from_record(record));
        }
        registries.reject_other_kind(ResourceKind::Sampler, identity)?;
        registries.ensure_capacity(identity, self.backend.resource_realization.policy)?;

        let (address_u, address_v, address_w) = descriptor.address_modes();
        let (mag_filter, min_filter, mipmap_filter) = descriptor.filters();
        let (lod_min_clamp, lod_max_clamp) = descriptor.lod_range();
        let object = self
            .backend
            .resource_realization
            .create_backend_object(identity, || {
                self.backend.device.create_sampler(&SamplerDescriptor {
                    label: Some(descriptor.common().label().as_str()),
                    address_mode_u: lowering::map_address_mode(address_u),
                    address_mode_v: lowering::map_address_mode(address_v),
                    address_mode_w: lowering::map_address_mode(address_w),
                    mag_filter: lowering::map_filter_mode(mag_filter),
                    min_filter: lowering::map_filter_mode(min_filter),
                    mipmap_filter: lowering::map_filter_mode(mipmap_filter),
                    lod_min_clamp,
                    lod_max_clamp,
                    compare: descriptor.compare().map(lowering::map_compare_function),
                    anisotropy_clamp: 1,
                    border_color: None,
                })
            })?;
        let record = Arc::new(SamplerRealizationRecord {
            affinity: self.affinity(),
            logical_identity: identity,
            descriptor,
            object,
        });
        registries.samplers.insert(identity, Arc::clone(&record));
        Ok(GpuRealizedSampler::from_record(record))
    }

    pub fn realize_query_set(
        &self,
        handle: &GpuQuerySetHandle,
    ) -> Result<GpuRealizedQuerySet, GpuResourceRealizationError> {
        let identity = handle.diagnostic_identity();
        lowering::validate_resource_ownership(identity, handle.descriptor().common())?;
        self.backend
            .resource_realization
            .ensure_available(identity)?;
        lowering::validate_query_set(self, identity, handle.descriptor())?;
        let descriptor = handle.retained_descriptor();
        let mut registries = self.backend.resource_realization.registries(identity)?;
        if let Some(record) = registries.query_sets.lookup(identity, &descriptor)? {
            return Ok(GpuRealizedQuerySet::from_record(record));
        }
        registries.reject_other_kind(ResourceKind::QuerySet, identity)?;
        registries.ensure_capacity(identity, self.backend.resource_realization.policy)?;

        let object = self
            .backend
            .resource_realization
            .create_backend_object(identity, || {
                self.backend.device.create_query_set(&QuerySetDescriptor {
                    label: Some(descriptor.common().label().as_str()),
                    ty: lowering::map_query_kind(descriptor.kind()),
                    count: descriptor.count(),
                })
            })?;
        let record = Arc::new(QuerySetRealizationRecord {
            affinity: self.affinity(),
            logical_identity: identity,
            descriptor,
            object,
        });
        registries.query_sets.insert(identity, Arc::clone(&record));
        Ok(GpuRealizedQuerySet::from_record(record))
    }
}

pub(super) fn validate_realized_input_affinity(
    expected: GpuContextAffinity,
    resource: GpuWorkResourceId,
    observed: GpuContextAffinity,
) -> Result<(), GpuResourceRealizationError> {
    match if observed.context() != expected.context() {
        Err(GpuContextAffinityError::ForeignContext)
    } else if observed.generation() != expected.generation() {
        Err(GpuContextAffinityError::StaleGeneration)
    } else {
        Ok(())
    } {
        Ok(()) => Ok(()),
        Err(GpuContextAffinityError::ForeignContext) => Err(GpuResourceRealizationError::affinity(
            GpuResourceRealizationErrorCategory::ForeignContext,
            Some(resource),
            expected,
            observed,
        )),
        Err(GpuContextAffinityError::StaleGeneration) => {
            Err(GpuResourceRealizationError::affinity(
                GpuResourceRealizationErrorCategory::StaleDeviceGeneration,
                Some(resource),
                expected,
                observed,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{GpuContextId, GpuDeviceGeneration};
    use std::num::NonZeroU64;

    fn backend_source() -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(std::io::Error::other("synthetic backend evidence"))
    }

    fn test_resource() -> GpuWorkResourceId {
        let mut allocator = crate::plugins::gpu::GpuWorkResourceIdAllocator::new();
        allocator.allocate().unwrap()
    }

    #[test]
    fn affinity_failure_order_distinguishes_foreign_context_and_stale_generation() {
        let context_one = GpuContextId::test_value(NonZeroU64::new(1).unwrap());
        let context_two = GpuContextId::test_value(NonZeroU64::new(2).unwrap());
        let generation_one = GpuDeviceGeneration::first();
        let generation_two = GpuDeviceGeneration::test_value(NonZeroU64::new(2).unwrap());
        let resource = test_resource();
        let expected = GpuContextAffinity::test_value(context_one, generation_one);

        let foreign = validate_realized_input_affinity(
            expected,
            resource,
            GpuContextAffinity::test_value(context_two, generation_two),
        )
        .unwrap_err();
        assert_eq!(
            foreign.category(),
            GpuResourceRealizationErrorCategory::ForeignContext
        );

        let stale = validate_realized_input_affinity(
            expected,
            resource,
            GpuContextAffinity::test_value(context_one, generation_two),
        )
        .unwrap_err();
        assert_eq!(
            stale.category(),
            GpuResourceRealizationErrorCategory::StaleDeviceGeneration
        );
    }

    #[test]
    fn device_health_keeps_uncaptured_validation_oom_and_internal_outcomes_distinct() {
        let cases = [
            (
                wgpu::Error::Validation {
                    source: backend_source(),
                    description: "validation".to_string(),
                },
                GpuResourceRealizationErrorCategory::UnexpectedBackendValidationRejection,
            ),
            (
                wgpu::Error::OutOfMemory {
                    source: backend_source(),
                },
                GpuResourceRealizationErrorCategory::BackendResourceExhaustion,
            ),
            (
                wgpu::Error::Internal {
                    source: backend_source(),
                    description: "backend internal failure".to_string(),
                },
                GpuResourceRealizationErrorCategory::ContextOrDeviceUnavailableOrLost,
            ),
        ];

        for (backend_error, expected) in cases {
            let health = DeviceHealth::new();
            let resource = test_resource();
            health.mark_uncaptured(backend_error);
            let error = health.ensure_available(resource).unwrap_err();
            assert_eq!(error.category(), expected);
            assert!(
                error.detail().is_some_and(|detail| detail.len() <= 256),
                "backend evidence must remain bounded"
            );
        }
    }

    #[test]
    fn synchronous_observed_backend_fault_rejects_constructor_candidate() {
        let context = GpuContextId::test_value(NonZeroU64::new(1).unwrap());
        let affinity = GpuContextAffinity::test_value(context, GpuDeviceGeneration::first());
        let state =
            ResourceRealizationState::new(affinity, GpuResourceRealizationPolicy::default());
        let resource = test_resource();

        let error = state
            .create_backend_object(resource, || {
                state.health.mark_uncaptured(wgpu::Error::Validation {
                    source: backend_source(),
                    description: "synchronous validation".to_string(),
                });
                7_u32
            })
            .unwrap_err();
        assert_eq!(
            error.category(),
            GpuResourceRealizationErrorCategory::UnexpectedBackendValidationRejection
        );
    }
}
