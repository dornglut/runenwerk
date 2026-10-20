use super::super::render_integration_is_active;
use super::super::renderer::Gfx;
use super::{RenderSurfaceId, RenderSurfaceLifecycleState, RenderSurfaceRegistryResource};
use crate::runtime::NativeWindowId;
use anyhow::{Context, Result, anyhow};
use std::sync::Arc;
use winit::window::Window;

pub(crate) fn initialize_primary(
    world: &mut runen_ecs::World,
    window: Arc<Window>,
    target_size_px: (u32, u32),
) -> Result<()> {
    if !render_integration_is_active(world) {
        return Ok(());
    }
    if world.has_resource::<Gfx>() {
        return Err(anyhow!(
            "preexisting runtime gfx cannot be proven attached to the newly created primary window"
        ));
    }

    let gfx = Gfx::new(window).context("failed to initialize runtime gfx")?;
    world.insert_resource(gfx);
    confirm_primary_attachment(world, target_size_px)
        .context("failed to confirm primary render surface attachment")
}

pub(crate) fn attach_secondary(
    world: &mut runen_ecs::World,
    native_window_id: NativeWindowId,
    window: Arc<Window>,
    requested_size_px: (u32, u32),
    realized_size_px: (u32, u32),
) -> Result<()> {
    if !render_integration_is_active(world) {
        return Ok(());
    }

    let render_surface_id = {
        let registry = world
            .resource_mut::<RenderSurfaceRegistryResource>()
            .context("render surface registry is unavailable")?;
        registry
            .surface_for_native_window(native_window_id)
            .unwrap_or_else(|| {
                registry.reserve_surface_for_native_window(native_window_id, requested_size_px)
            })
    };

    if let Err(err) = world
        .resource_mut::<Gfx>()
        .context("runtime gfx is unavailable")
        .and_then(|gfx| gfx.attach_surface(render_surface_id, window, realized_size_px))
    {
        try_rollback_secondary(world, native_window_id)
            .context("failed to rollback rejected secondary GPU attachment")?;
        return Err(anyhow!("GPU surface attachment failed: {err:#}"));
    }

    if let Err(err) = world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .context("render surface registry is unavailable")
        .and_then(|registry| {
            registry.confirm_surface_attachment(
                render_surface_id,
                native_window_id,
                realized_size_px,
            )
        })
    {
        try_rollback_secondary(world, native_window_id)
            .context("failed to rollback rejected secondary surface correlation")?;
        return Err(anyhow!("render surface correlation failed: {err:#}"));
    }

    Ok(())
}

pub(crate) fn project_primary_extent(
    world: &mut runen_ecs::World,
    target_size_px: (u32, u32),
) -> bool {
    if !render_integration_is_active(world) {
        return false;
    }
    world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .ok()
        .is_some_and(|registry| {
            registry
                .update_surface_extent_for_native_window(NativeWindowId::primary(), target_size_px)
        })
}

/// Host-observable outcome: an absent Render surface is a successful no-op.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SecondaryRetireOutcome {
    NoAttachment,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum SecondaryRetireError {
    #[error("primary native window cannot be retired as a secondary")]
    PrimaryWindow,
    #[error("attached secondary Render surface {surface:?} has no GPU attachment")]
    MissingGpuAttachment { surface: RenderSurfaceId },
    #[error("physical detach refused for secondary Render surface {surface:?}")]
    PhysicalDetachRefused { surface: RenderSurfaceId },
}

/// A failed physical detach MUST leave the logical Render reservation intact.
pub(crate) fn try_retire_secondary(
    world: &mut runen_ecs::World,
    native_window_id: NativeWindowId,
) -> std::result::Result<SecondaryRetireOutcome, SecondaryRetireError> {
    try_retire_secondary_with(world, native_window_id, |world, surface, lifecycle| {
        match world.resource_mut::<Gfx>() {
            Ok(gfx) => {
                if gfx.has_surface(surface) {
                    if !gfx.detach_surface(surface) {
                        return Err(SecondaryRetireError::PhysicalDetachRefused { surface });
                    }
                } else if lifecycle == RenderSurfaceLifecycleState::Attached {
                    return Err(SecondaryRetireError::MissingGpuAttachment { surface });
                }
            }
            Err(_) if lifecycle == RenderSurfaceLifecycleState::Attached => {
                return Err(SecondaryRetireError::MissingGpuAttachment { surface });
            }
            Err(_) => {}
        }
        Ok(())
    })
}

/// Creation rollback and final close use the same idempotent physical handoff.
pub(crate) fn try_rollback_secondary(
    world: &mut runen_ecs::World,
    native_window_id: NativeWindowId,
) -> std::result::Result<SecondaryRetireOutcome, SecondaryRetireError> {
    try_retire_secondary(world, native_window_id)
}

/// Test seam is local: only Render performs the real Gfx operation.
fn try_retire_secondary_with(
    world: &mut runen_ecs::World,
    native_window_id: NativeWindowId,
    detach: impl FnOnce(
        &mut runen_ecs::World,
        RenderSurfaceId,
        RenderSurfaceLifecycleState,
    ) -> std::result::Result<(), SecondaryRetireError>,
) -> std::result::Result<SecondaryRetireOutcome, SecondaryRetireError> {
    if native_window_id == NativeWindowId::primary() {
        return Err(SecondaryRetireError::PrimaryWindow);
    }
    let Some((surface, lifecycle)) = world
        .resource::<RenderSurfaceRegistryResource>()
        .ok()
        .and_then(|registry| {
            let surface = registry.surface_for_native_window(native_window_id)?;
            Some((surface, registry.record(surface)?.lifecycle_state))
        })
    else {
        return Ok(SecondaryRetireOutcome::NoAttachment);
    };
    detach(world, surface, lifecycle)?;
    // There is no async gap: Host and Render consume the same synchronous World.
    let retired = world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .expect("synchronous retirement must retain its Render registry")
        .retire_surface_for_native_window(native_window_id);
    assert_eq!(
        retired,
        Some(surface),
        "physical retirement must correspond to exactly one logical identity"
    );
    Ok(SecondaryRetireOutcome::Retired)
}

fn confirm_primary_attachment(
    world: &mut runen_ecs::World,
    target_size_px: (u32, u32),
) -> Result<()> {
    let surface = RenderSurfaceId::primary();
    if !world
        .resource::<Gfx>()
        .context("runtime gfx is unavailable")?
        .has_surface(surface)
    {
        return Err(anyhow!(
            "runtime gfx does not own the primary render surface after initialization"
        ));
    }
    world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .context("render surface registry is unavailable")?
        .confirm_surface_attachment(surface, NativeWindowId::primary(), target_size_px)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::plugins::RenderPlugin;
    use crate::plugins::render::host::RenderSurfaceLifecycleState;

    fn secondary_window(raw: u64) -> NativeWindowId {
        NativeWindowId::try_from_raw(raw).expect("secondary native window id should be valid")
    }

    #[test]
    fn no_render_and_missing_secondary_are_idempotent_successful_noops() {
        let mut app = App::new();
        let secondary = secondary_window(2);
        assert_eq!(
            try_retire_secondary(app.world_mut(), secondary),
            Ok(SecondaryRetireOutcome::NoAttachment)
        );
        assert_eq!(
            try_rollback_secondary(app.world_mut(), secondary),
            Ok(SecondaryRetireOutcome::NoAttachment)
        );
        app.init_resource::<RenderSurfaceRegistryResource>();
        assert_eq!(
            try_retire_secondary(app.world_mut(), secondary),
            Ok(SecondaryRetireOutcome::NoAttachment)
        );
        assert_eq!(
            try_retire_secondary(app.world_mut(), NativeWindowId::primary()),
            Err(SecondaryRetireError::PrimaryWindow)
        );
    }

    #[test]
    fn injected_detach_refusal_preserves_precommit_and_committed_identity() {
        for attached in [false, true] {
            let mut app = App::new();
            app.init_resource::<RenderSurfaceRegistryResource>();
            let secondary = secondary_window(3);
            let surface = app
                .world_mut()
                .resource_mut::<RenderSurfaceRegistryResource>()
                .unwrap()
                .reserve_surface_for_native_window(secondary, (640, 480));
            if attached {
                app.world_mut()
                    .resource_mut::<RenderSurfaceRegistryResource>()
                    .unwrap()
                    .confirm_surface_attachment(surface, secondary, (640, 480))
                    .unwrap();
            }
            let refused = try_retire_secondary_with(app.world_mut(), secondary, |_, id, _| {
                assert_eq!(id, surface);
                Err(SecondaryRetireError::PhysicalDetachRefused { surface })
            });
            assert_eq!(
                refused,
                Err(SecondaryRetireError::PhysicalDetachRefused { surface })
            );
            let registry = app.world().resource::<RenderSurfaceRegistryResource>().unwrap();
            assert_eq!(registry.surface_for_native_window(secondary), Some(surface));
            assert_eq!(
                registry.record(surface).map(|record| record.lifecycle_state),
                Some(if attached {
                    RenderSurfaceLifecycleState::Attached
                } else {
                    RenderSurfaceLifecycleState::Requested
                })
            );
            assert_eq!(
                try_retire_secondary_with(app.world_mut(), secondary, |_, _, _| Ok(())),
                Ok(SecondaryRetireOutcome::Retired)
            );
            assert_eq!(
                try_retire_secondary(app.world_mut(), secondary),
                Ok(SecondaryRetireOutcome::NoAttachment)
            );
        }
    }

    #[test]
    fn primary_extent_projection_is_absent_without_render_selection() {
        let mut app = App::new();

        assert!(!project_primary_extent(app.world_mut(), (1440, 900)));
        assert!(!app.world().has_resource::<Gfx>());
        assert!(!app.world().has_resource::<RenderSurfaceRegistryResource>());
    }

    #[test]
    fn primary_extent_projection_updates_only_selected_render_state() {
        let mut app = App::new();
        app.add_plugin(RenderPlugin);
        let surface = app
            .world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()
            .expect("RenderPlugin should install surface registry")
            .reserve_surface_for_native_window(NativeWindowId::primary(), (1280, 720));
        app.world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()
            .expect("RenderPlugin should install surface registry")
            .confirm_surface_attachment(surface, NativeWindowId::primary(), (1280, 720))
            .expect("primary registry state should attach");

        assert!(project_primary_extent(app.world_mut(), (1440, 900)));
        assert_eq!(
            app.world()
                .resource::<RenderSurfaceRegistryResource>()
                .unwrap()
                .record(surface)
                .map(|record| record.target_size_px),
            Some((1440, 900))
        );
    }

    #[test]
    fn rollback_retires_reserved_secondary_surface_before_logical_commit() {
        let mut app = App::new();
        app.init_resource::<RenderSurfaceRegistryResource>();
        let secondary = secondary_window(2);
        let surface = app
            .world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()
            .unwrap()
            .reserve_surface_for_native_window(secondary, (900, 600));

        assert_eq!(
            try_rollback_secondary(app.world_mut(), secondary),
            Ok(SecondaryRetireOutcome::Retired)
        );

        let registry = app
            .world()
            .resource::<RenderSurfaceRegistryResource>()
            .unwrap();
        assert_eq!(registry.surface_for_native_window(secondary), None);
        assert_eq!(
            registry
                .record(surface)
                .map(|record| record.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Retired)
        );
    }

    #[test]
    fn retiring_secondary_surface_preserves_primary_surface_state() {
        let mut app = App::new();
        app.init_resource::<RenderSurfaceRegistryResource>();
        let primary = app
            .world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()
            .unwrap()
            .reserve_surface_for_native_window(NativeWindowId::primary(), (1280, 720));
        app.world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()
            .unwrap()
            .confirm_surface_attachment(primary, NativeWindowId::primary(), (1280, 720))
            .unwrap();

        let secondary = secondary_window(2);
        let secondary_surface = app
            .world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()
            .unwrap()
            .reserve_surface_for_native_window(secondary, (900, 600));
        app.world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()
            .unwrap()
            .confirm_surface_attachment(secondary_surface, secondary, (900, 600))
            .unwrap();

        assert_eq!(
            try_retire_secondary(app.world_mut(), secondary),
            Err(SecondaryRetireError::MissingGpuAttachment {
                surface: secondary_surface
            })
        );

        let registry = app
            .world()
            .resource::<RenderSurfaceRegistryResource>()
            .unwrap();
        assert_eq!(registry.primary_surface_id(), Some(primary));
        assert_eq!(
            registry
                .record(primary)
                .map(|record| record.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Attached)
        );
        assert_eq!(
            registry
                .record(secondary_surface)
                .map(|record| record.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Attached)
        );
    }
}
