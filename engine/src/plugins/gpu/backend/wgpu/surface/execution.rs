use super::super::WgpuDeviceHealth;
use super::{WgpuSurfaceLease, WgpuSurfaceRecord, WgpuSurfaceState, WgpuSurfaceStateInner};
use crate::plugins::gpu::{
    GpuContextAffinity, GpuSurfaceLeaseError, GpuSurfaceLeaseErrorCategory, GpuSurfaceResourceLease,
    GpuWorkResourceId,
};

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

impl WgpuSurfaceState {
    /// Validates one surface-acquired logical resource against the sole G7 physical lease owner.
    ///
    /// This is intentionally separate from G4C1 realization: surface-acquired resources are
    /// transient presentation leases and never become ordinary resource-registry records.
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
