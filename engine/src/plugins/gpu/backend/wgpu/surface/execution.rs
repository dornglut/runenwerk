use super::super::WgpuDeviceHealth;
use super::{WgpuSurfaceLease, WgpuSurfaceRecord, WgpuSurfaceState, WgpuSurfaceStateInner};
use crate::plugins::gpu::{
    GpuContextAffinity, GpuSurfaceLeaseError, GpuSurfaceLeaseErrorCategory, GpuSurfaceResourceLease,
    GpuWorkResourceId,
};
use std::sync::MutexGuard;
use wgpu::{SurfaceTexture, Texture, TextureView, TextureViewDescriptor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WgpuSurfaceLeaseResource {
    Texture(GpuWorkResourceId),
    TextureView(GpuWorkResourceId),
}

impl WgpuSurfaceLeaseResource {
    const fn identity(self) -> GpuWorkResourceId {
        match self {
            Self::Texture(identity) | Self::TextureView(identity) => identity,
        }
    }
}

/// A short lexical borrow of the G7 surface owner for private execution.
///
/// Holding this guard pins active physical surface-image leases against concurrent abandon,
/// reconfiguration, or acquisition while one command-encoding interval resolves them. Logical
/// handles remain non-owning; the guard is backend-private and cannot escape RunenGPU.
pub(crate) struct WgpuSurfaceLeaseGuard<'a> {
    affinity: GpuContextAffinity,
    health: &'a WgpuDeviceHealth,
    inner: MutexGuard<'a, WgpuSurfaceStateInner>,
}

impl WgpuSurfaceState {
    pub(crate) fn validate_execution_lease(
        &self,
        lease: GpuSurfaceResourceLease,
        resource: WgpuSurfaceLeaseResource,
        health: &WgpuDeviceHealth,
    ) -> Result<(), GpuSurfaceLeaseError> {
        ensure_lease_health(health, lease)?;
        let mut inner = self.shared.inner.lock().map_err(|_| {
            lease_error(
                GpuSurfaceLeaseErrorCategory::ContextOrDeviceUnavailableOrLost,
                lease,
                "surface lease validation authority is unavailable",
            )
        })?;
        if let Some(record) = inner.records.get_mut(&lease.surface().id()) {
            super::release_abandoned_lease(record);
        }
        validate_lease(self.affinity, &inner, lease, resource)?;
        ensure_lease_health(health, lease)
    }

    pub(crate) fn execution_lease_guard(
        &self,
        health: &WgpuDeviceHealth,
    ) -> Result<WgpuSurfaceLeaseGuard<'_>, GpuSurfaceLeaseError> {
        let mut inner = self.shared.inner.lock().map_err(|_| {
            unavailable_without_lease(self.affinity, &self.shared)
        })?;
        for record in inner.records.values_mut() {
            super::release_abandoned_lease(record);
        }
        if let Some(fault) = health.terminal_fault() {
            return Err(unavailable_for_first_record(&inner, fault.detail));
        }
        Ok(WgpuSurfaceLeaseGuard {
            affinity: self.affinity,
            health,
            inner,
        })
    }
}

impl WgpuSurfaceLeaseGuard<'_> {
    pub(crate) fn texture(
        &self,
        lease: GpuSurfaceResourceLease,
        identity: GpuWorkResourceId,
    ) -> Result<&Texture, GpuSurfaceLeaseError> {
        ensure_lease_health(self.health, lease)?;
        let active = validate_lease(
            self.affinity,
            &self.inner,
            lease,
            WgpuSurfaceLeaseResource::Texture(identity),
        )?;
        Ok(&active.texture.texture)
    }

    pub(crate) fn create_default_view(
        &self,
        lease: GpuSurfaceResourceLease,
        identity: GpuWorkResourceId,
    ) -> Result<TextureView, GpuSurfaceLeaseError> {
        ensure_lease_health(self.health, lease)?;
        let active = validate_lease(
            self.affinity,
            &self.inner,
            lease,
            WgpuSurfaceLeaseResource::TextureView(identity),
        )?;
        Ok(active
            .texture
            .texture
            .create_view(&TextureViewDescriptor::default()))
    }

    pub(crate) fn take_for_present(
        &mut self,
        lease: GpuSurfaceResourceLease,
        resource: WgpuSurfaceLeaseResource,
    ) -> Result<SurfaceTexture, GpuSurfaceLeaseError> {
        ensure_lease_health(self.health, lease)?;
        validate_lease(self.affinity, &self.inner, lease, resource)?;
        let record = self
            .inner
            .records
            .get_mut(&lease.surface().id())
            .expect("validated surface lease retains its owner record");
        let active = record
            .active_lease
            .take()
            .expect("validated surface lease retains its active physical image");
        Ok(active.texture)
    }
}

fn validate_lease<'a>(
    expected: GpuContextAffinity,
    inner: &'a WgpuSurfaceStateInner,
    lease: GpuSurfaceResourceLease,
    resource: WgpuSurfaceLeaseResource,
) -> Result<&'a WgpuSurfaceLease, GpuSurfaceLeaseError> {
    validate_affinity(expected, lease)?;
    let record = inner.records.get(&lease.surface().id()).ok_or_else(|| {
        lease_error(
            GpuSurfaceLeaseErrorCategory::UnknownSurface,
            lease,
            "surface identity is absent from this context-local G7 owner",
        )
    })?;
    if record.generation != lease.surface().generation() {
        return Err(lease_error(
            GpuSurfaceLeaseErrorCategory::StaleGeneration,
            lease,
            "surface-acquired resource belongs to a stale surface generation",
        ));
    }
    let active = record.active_lease.as_ref().ok_or_else(|| {
        inactive_lease_error(record, lease, "the surface has no active acquired-image lease")
    })?;
    if active.id != lease.lease_id() {
        return Err(inactive_lease_error(
            record,
            lease,
            "surface-acquired resource does not name the current active lease",
        ));
    }
    let identity_matches = match resource {
        WgpuSurfaceLeaseResource::Texture(identity) => active.texture_identity == identity,
        WgpuSurfaceLeaseResource::TextureView(identity) => active.view_identity == identity,
    };
    if !identity_matches {
        return Err(lease_error(
            GpuSurfaceLeaseErrorCategory::InvalidLease,
            lease,
            format!(
                "logical {:?} identity {:?} is not owned by this surface lease",
                resource,
                resource.identity()
            ),
        ));
    }
    Ok(active)
}

fn validate_affinity(
    expected: GpuContextAffinity,
    lease: GpuSurfaceResourceLease,
) -> Result<(), GpuSurfaceLeaseError> {
    let observed = lease.surface().affinity();
    if observed.context() != expected.context() {
        return Err(lease_error(
            GpuSurfaceLeaseErrorCategory::ForeignContext,
            lease,
            "surface-acquired resource belongs to a different GPU context",
        ));
    }
    if observed.generation() != expected.generation() {
        return Err(lease_error(
            GpuSurfaceLeaseErrorCategory::StaleGeneration,
            lease,
            "surface-acquired resource belongs to a stale device generation",
        ));
    }
    Ok(())
}

fn inactive_lease_error(
    record: &WgpuSurfaceRecord,
    lease: GpuSurfaceResourceLease,
    detail: &'static str,
) -> GpuSurfaceLeaseError {
    let category = if record
        .last_lease_id
        .is_some_and(|last| lease.lease_id() <= last)
    {
        GpuSurfaceLeaseErrorCategory::AlreadyConsumed
    } else {
        GpuSurfaceLeaseErrorCategory::InvalidLease
    };
    lease_error(category, lease, detail)
}

fn ensure_lease_health(
    health: &WgpuDeviceHealth,
    lease: GpuSurfaceResourceLease,
) -> Result<(), GpuSurfaceLeaseError> {
    if let Some(fault) = health.terminal_fault() {
        Err(lease_error(
            GpuSurfaceLeaseErrorCategory::ContextOrDeviceUnavailableOrLost,
            lease,
            fault.detail,
        ))
    } else {
        Ok(())
    }
}

fn lease_error(
    category: GpuSurfaceLeaseErrorCategory,
    lease: GpuSurfaceResourceLease,
    detail: impl Into<String>,
) -> GpuSurfaceLeaseError {
    GpuSurfaceLeaseError::new(category, lease.surface().id(), lease.lease_id(), detail)
}

fn unavailable_without_lease(
    affinity: GpuContextAffinity,
    shared: &super::WgpuSurfaceShared,
) -> GpuSurfaceLeaseError {
    let inner = shared
        .inner
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    unavailable_for_first_record(
        &inner,
        format!(
            "surface lease execution authority is unavailable for context {:?}",
            affinity.context()
        ),
    )
}

fn unavailable_for_first_record(
    inner: &WgpuSurfaceStateInner,
    detail: impl Into<String>,
) -> GpuSurfaceLeaseError {
    let (surface, lease_id) = inner
        .records
        .iter()
        .find_map(|(surface, record)| {
            record
                .active_lease
                .as_ref()
                .map(|lease| (*surface, lease.id))
                .or_else(|| record.last_lease_id.map(|lease| (*surface, lease)))
        })
        .expect("surface lease authority errors require an existing surface lease");
    GpuSurfaceLeaseError::new(
        GpuSurfaceLeaseErrorCategory::ContextOrDeviceUnavailableOrLost,
        surface,
        lease_id,
        detail,
    )
}
