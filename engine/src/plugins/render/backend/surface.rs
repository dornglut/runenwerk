use crate::runtime::NativeWindowId;
use anyhow::{Result, anyhow};
use id_macros::id;
use runen_gpu::{
    GpuSurfaceCapabilities, GpuSurfaceConfiguration, GpuSurfacePresentMode, GpuTextureFormat,
    GpuTextureUsage,
};
use std::collections::BTreeMap;

#[id]
pub struct RenderSurfaceId;

impl RenderSurfaceId {
    pub fn primary() -> Self {
        Self::try_from_raw(1).expect("primary render surface id must be non-zero")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSurfaceLifecycleState {
    Requested,
    Attached,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSurfaceRecord {
    pub render_surface_id: RenderSurfaceId,
    pub native_window_id: NativeWindowId,
    pub target_size_px: (u32, u32),
    pub lifecycle_state: RenderSurfaceLifecycleState,
}

impl RenderSurfaceRecord {
    fn new(
        render_surface_id: RenderSurfaceId,
        native_window_id: NativeWindowId,
        target_size_px: (u32, u32),
        lifecycle_state: RenderSurfaceLifecycleState,
    ) -> Self {
        Self {
            render_surface_id,
            native_window_id,
            target_size_px: normalized_surface_extent(target_size_px),
            lifecycle_state,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSurfaceDiagnostic {
    pub render_surface_id: Option<RenderSurfaceId>,
    pub native_window_id: Option<NativeWindowId>,
    pub message: String,
}

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct RenderSurfaceRegistryResource {
    primary_surface_id: Option<RenderSurfaceId>,
    next_surface_raw: u64,
    records: BTreeMap<RenderSurfaceId, RenderSurfaceRecord>,
    surfaces_by_native_window: BTreeMap<NativeWindowId, RenderSurfaceId>,
    diagnostics: Vec<RenderSurfaceDiagnostic>,
}

impl RenderSurfaceRegistryResource {
    pub fn reserve_surface_for_native_window(
        &mut self,
        native_window_id: NativeWindowId,
        target_size_px: (u32, u32),
    ) -> RenderSurfaceId {
        if let Some(surface_id) = self
            .surfaces_by_native_window
            .get(&native_window_id)
            .copied()
        {
            if let Some(record) = self.records.get_mut(&surface_id) {
                record.target_size_px = normalized_surface_extent(target_size_px);
            }
            return surface_id;
        }
        let render_surface_id = if native_window_id == NativeWindowId::primary() {
            RenderSurfaceId::primary()
        } else {
            self.allocate_secondary_surface_id()
        };
        self.next_surface_raw = self
            .next_surface_raw
            .max(render_surface_id.raw().saturating_add(1));
        self.records.insert(
            render_surface_id,
            RenderSurfaceRecord::new(
                render_surface_id,
                native_window_id,
                target_size_px,
                RenderSurfaceLifecycleState::Requested,
            ),
        );
        self.surfaces_by_native_window
            .insert(native_window_id, render_surface_id);
        if native_window_id == NativeWindowId::primary() {
            self.primary_surface_id = Some(render_surface_id);
        }
        render_surface_id
    }

    pub fn confirm_surface_attachment(
        &mut self,
        render_surface_id: RenderSurfaceId,
        native_window_id: NativeWindowId,
        target_size_px: (u32, u32),
    ) -> Result<()> {
        let render_is_primary = render_surface_id == RenderSurfaceId::primary();
        let native_is_primary = native_window_id == NativeWindowId::primary();
        if render_is_primary != native_is_primary {
            return Err(anyhow!(
                "render surface {} and native window {} disagree on primary identity",
                render_surface_id.raw(),
                native_window_id.raw()
            ));
        }
        if let Some(existing) = self.surfaces_by_native_window.get(&native_window_id)
            && *existing != render_surface_id
        {
            return Err(anyhow!(
                "native window {} is already correlated with render surface {}",
                native_window_id.raw(),
                existing.raw()
            ));
        }
        match self.records.get_mut(&render_surface_id) {
            Some(record) => {
                if record.native_window_id != native_window_id {
                    return Err(anyhow!(
                        "render surface {} is reserved for native window {}, not {}",
                        render_surface_id.raw(),
                        record.native_window_id.raw(),
                        native_window_id.raw()
                    ));
                }
                if record.lifecycle_state == RenderSurfaceLifecycleState::Retired {
                    return Err(anyhow!(
                        "retired render surface {} cannot be reattached",
                        render_surface_id.raw()
                    ));
                }
                record.target_size_px = normalized_surface_extent(target_size_px);
                record.lifecycle_state = RenderSurfaceLifecycleState::Attached;
            }
            None => {
                self.records.insert(
                    render_surface_id,
                    RenderSurfaceRecord::new(
                        render_surface_id,
                        native_window_id,
                        target_size_px,
                        RenderSurfaceLifecycleState::Attached,
                    ),
                );
            }
        }
        self.next_surface_raw = self
            .next_surface_raw
            .max(render_surface_id.raw().saturating_add(1));
        self.surfaces_by_native_window
            .insert(native_window_id, render_surface_id);
        if native_is_primary {
            self.primary_surface_id = Some(render_surface_id);
        }
        Ok(())
    }

    pub fn update_surface_extent_for_native_window(
        &mut self,
        native_window_id: NativeWindowId,
        target_size_px: (u32, u32),
    ) -> bool {
        let Some(render_surface_id) = self
            .surfaces_by_native_window
            .get(&native_window_id)
            .copied()
        else {
            return false;
        };
        let Some(record) = self.records.get_mut(&render_surface_id) else {
            return false;
        };
        if record.lifecycle_state == RenderSurfaceLifecycleState::Retired {
            return false;
        }
        record.target_size_px = normalized_surface_extent(target_size_px);
        true
    }

    pub fn primary_surface_id(&self) -> Option<RenderSurfaceId> {
        self.primary_surface_id
    }
    pub fn record(&self, render_surface_id: RenderSurfaceId) -> Option<&RenderSurfaceRecord> {
        self.records.get(&render_surface_id)
    }
    pub fn surface_for_native_window(
        &self,
        native_window_id: NativeWindowId,
    ) -> Option<RenderSurfaceId> {
        self.surfaces_by_native_window
            .get(&native_window_id)
            .copied()
    }
    pub fn records(&self) -> impl Iterator<Item = &RenderSurfaceRecord> {
        self.records.values()
    }

    pub fn retire_surface_for_native_window(
        &mut self,
        native_window_id: NativeWindowId,
    ) -> Option<RenderSurfaceId> {
        let render_surface_id = self.surfaces_by_native_window.remove(&native_window_id)?;
        if let Some(record) = self.records.get_mut(&render_surface_id) {
            record.lifecycle_state = RenderSurfaceLifecycleState::Retired;
        }
        if self.primary_surface_id == Some(render_surface_id) {
            self.primary_surface_id = None;
        }
        Some(render_surface_id)
    }
    pub fn diagnostics(&self) -> &[RenderSurfaceDiagnostic] {
        &self.diagnostics
    }
    pub fn record_diagnostic(&mut self, diagnostic: RenderSurfaceDiagnostic) {
        self.diagnostics.push(diagnostic);
    }
    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    fn allocate_secondary_surface_id(&mut self) -> RenderSurfaceId {
        loop {
            let raw = self
                .next_surface_raw
                .max(RenderSurfaceId::primary().raw().saturating_add(1));
            self.next_surface_raw = raw.saturating_add(1);
            if let Ok(id) = RenderSurfaceId::try_from_raw(raw)
                && !self.records.contains_key(&id)
            {
                return id;
            }
        }
    }
}

fn normalized_surface_extent(target_size_px: (u32, u32)) -> (u32, u32) {
    (target_size_px.0.max(1), target_size_px.1.max(1))
}

pub fn build_surface_config(
    width: u32,
    height: u32,
    format: GpuTextureFormat,
    capabilities: &GpuSurfaceCapabilities,
) -> Result<GpuSurfaceConfiguration> {
    let alpha_mode = capabilities
        .alpha_modes()
        .first()
        .copied()
        .ok_or_else(|| anyhow!("render surface reports no supported alpha mode"))?;
    let mut usages = vec![GpuTextureUsage::ColorAttachment];
    for usage in [
        GpuTextureUsage::CopySource,
        GpuTextureUsage::CopyDestination,
    ] {
        if capabilities.supports_usage(usage) {
            usages.push(usage);
        }
    }
    if !capabilities.supports_present_mode(GpuSurfacePresentMode::Fifo) {
        return Err(anyhow!("render surface does not support FIFO presentation"));
    }
    if !capabilities.supports_alpha_mode(alpha_mode) {
        return Err(anyhow!(
            "render surface alpha-mode selection became invalid"
        ));
    }
    Ok(GpuSurfaceConfiguration::new(
        width.max(1),
        height.max(1),
        format,
        usages,
        GpuSurfacePresentMode::Fifo,
        alpha_mode,
        2,
        [format],
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservation_allocates_stable_identity_without_claiming_attachment() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let surface =
            registry.reserve_surface_for_native_window(NativeWindowId::primary(), (1280, 720));
        assert_eq!(surface, RenderSurfaceId::primary());
        assert_eq!(registry.primary_surface_id(), Some(surface));
        assert_eq!(
            registry.record(surface).map(|r| r.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Requested)
        );
    }

    #[test]
    fn attachment_confirmation_promotes_the_exact_reserved_identity() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let window = NativeWindowId::try_from_raw(2).unwrap();
        let surface = registry.reserve_surface_for_native_window(window, (640, 480));
        registry
            .confirm_surface_attachment(surface, window, (800, 600))
            .unwrap();
        assert_eq!(registry.surface_for_native_window(window), Some(surface));
        assert_eq!(
            registry
                .record(surface)
                .map(|r| (r.lifecycle_state, r.target_size_px)),
            Some((RenderSurfaceLifecycleState::Attached, (800, 600)))
        );
    }

    #[test]
    fn reservation_keeps_primary_id_available_when_secondary_is_requested_first() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let window = NativeWindowId::try_from_raw(2).unwrap();
        let secondary = registry.reserve_surface_for_native_window(window, (640, 480));
        let primary =
            registry.reserve_surface_for_native_window(NativeWindowId::primary(), (1280, 720));
        assert_ne!(secondary, RenderSurfaceId::primary());
        assert_eq!(primary, RenderSurfaceId::primary());
    }

    #[test]
    fn attachment_confirmation_rejects_mismatched_primary_identity() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let window = NativeWindowId::try_from_raw(2).unwrap();
        let secondary = registry.reserve_surface_for_native_window(window, (640, 480));
        assert!(
            registry
                .confirm_surface_attachment(RenderSurfaceId::primary(), window, (640, 480))
                .is_err()
        );
        assert_eq!(
            registry.record(secondary).map(|r| r.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Requested)
        );
    }

    #[test]
    fn extent_updates_never_promote_requested_identity() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let window = NativeWindowId::try_from_raw(2).unwrap();
        let surface = registry.reserve_surface_for_native_window(window, (640, 480));
        assert!(registry.update_surface_extent_for_native_window(window, (900, 600)));
        assert_eq!(
            registry
                .record(surface)
                .map(|r| (r.lifecycle_state, r.target_size_px)),
            Some((RenderSurfaceLifecycleState::Requested, (900, 600)))
        );
    }

    #[test]
    fn retiring_surface_removes_window_lookup_and_preserves_auditable_record() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let window = NativeWindowId::try_from_raw(2).unwrap();
        let surface = registry.reserve_surface_for_native_window(window, (640, 480));
        assert_eq!(
            registry.retire_surface_for_native_window(window),
            Some(surface)
        );
        assert_eq!(registry.surface_for_native_window(window), None);
        assert_eq!(
            registry.record(surface).map(|r| r.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Retired)
        );
    }
}
