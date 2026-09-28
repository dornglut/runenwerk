use super::backend::{RenderSurfaceId, RenderSurfaceRegistryResource};
use super::render_integration_is_active;
use super::renderer::Gfx;
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
        rollback_secondary(world, native_window_id);
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
        rollback_secondary(world, native_window_id);
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
            registry.update_surface_extent_for_native_window(
                NativeWindowId::primary(),
                target_size_px,
            )
        })
}

pub(crate) fn retire_secondary(
    world: &mut runen_ecs::World,
    native_window_id: NativeWindowId,
) -> bool {
    if native_window_id == NativeWindowId::primary() {
        return false;
    }
    rollback_secondary(world, native_window_id)
}

pub(crate) fn rollback_secondary(
    world: &mut runen_ecs::World,
    native_window_id: NativeWindowId,
) -> bool {
    if native_window_id == NativeWindowId::primary() {
        return false;
    }

    let render_surface_id = world
        .resource::<RenderSurfaceRegistryResource>()
        .ok()
        .and_then(|registry| registry.surface_for_native_window(native_window_id));

    if let Some(render_surface_id) = render_surface_id
        && let Ok(gfx) = world.resource_mut::<Gfx>()
    {
        gfx.detach_surface(render_surface_id);
    }

    world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .ok()
        .and_then(|registry| registry.retire_surface_for_native_window(native_window_id))
        .is_some()
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
    use crate::plugins::render::backend::RenderSurfaceLifecycleState;

    fn secondary_window(raw: u64) -> NativeWindowId {
        NativeWindowId::try_from_raw(raw).expect("secondary native window id should be valid")
    }

    #[test]
    fn primary_extent_projection_is_absent_without_render_selection() {
        let mut app = App::new();

        assert!(!project_primary_extent(app.world_mut(), (1440, 900)));
        assert!(!app.world().has_resource::<Gfx>());
        assert!(
            !app.world()
                .has_resource::<RenderSurfaceRegistryResource>()
        );
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

        assert!(rollback_secondary(app.world_mut(), secondary));

        let registry = app
            .world()
            .resource::<RenderSurfaceRegistryResource>()
            .unwrap();
        assert_eq!(registry.surface_for_native_window(secondary), None);
        assert_eq!(
            registry.record(surface).map(|record| record.lifecycle_state),
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

        assert!(retire_secondary(app.world_mut(), secondary));

        let registry = app
            .world()
            .resource::<RenderSurfaceRegistryResource>()
            .unwrap();
        assert_eq!(registry.primary_surface_id(), Some(primary));
        assert_eq!(
            registry.record(primary).map(|record| record.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Attached)
        );
        assert_eq!(
            registry
                .record(secondary_surface)
                .map(|record| record.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Retired)
        );
    }
}
