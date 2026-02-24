use super::{PipelineKey, WorldRenderFrame};
use anyhow::{Result, bail};
use std::collections::BTreeMap;
use std::sync::Arc;
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};

pub trait RenderPassExecutor: Send + Sync {
    fn prepare(&self, _ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        Ok(())
    }

    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()>;
}

pub struct RenderPassPrepareContext<'a> {
    device: &'a Device,
    queue: &'a Queue,
    world_frame: &'a WorldRenderFrame,
    surface_format: TextureFormat,
    surface_size: (u32, u32),
    builtin_dispatch: Option<&'a mut dyn FnMut(BuiltinRenderPassExecutor) -> Result<()>>,
}

impl<'a> RenderPassPrepareContext<'a> {
    pub fn new(
        device: &'a Device,
        queue: &'a Queue,
        world_frame: &'a WorldRenderFrame,
        surface_format: TextureFormat,
        surface_size: (u32, u32),
    ) -> Self {
        Self {
            device,
            queue,
            world_frame,
            surface_format,
            surface_size,
            builtin_dispatch: None,
        }
    }

    pub fn with_builtin_dispatch(
        mut self,
        dispatch: &'a mut dyn FnMut(BuiltinRenderPassExecutor) -> Result<()>,
    ) -> Self {
        self.builtin_dispatch = Some(dispatch);
        self
    }

    pub fn device(&self) -> &'a Device {
        self.device
    }

    pub fn queue(&self) -> &'a Queue {
        self.queue
    }

    pub fn world_frame(&self) -> &'a WorldRenderFrame {
        self.world_frame
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.surface_format
    }

    pub fn surface_size(&self) -> (u32, u32) {
        self.surface_size
    }

    pub fn run_builtin(&mut self, builtin: BuiltinRenderPassExecutor) -> Result<()> {
        let Some(dispatch) = &mut self.builtin_dispatch else {
            bail!("builtin prepare dispatch is not available in this context");
        };
        dispatch(builtin)
    }
}

pub struct RenderPassEncodeContext<'a> {
    device: &'a Device,
    encoder: &'a mut CommandEncoder,
    frame_view: &'a TextureView,
    world_frame: &'a WorldRenderFrame,
    surface_format: TextureFormat,
    surface_size: (u32, u32),
    pipeline: PipelineKey,
    builtin_dispatch:
        Option<&'a mut dyn FnMut(&mut CommandEncoder, BuiltinRenderPassExecutor) -> Result<()>>,
    ui_dispatch: Option<&'a mut dyn FnMut(&mut CommandEncoder) -> Result<()>>,
}

impl<'a> RenderPassEncodeContext<'a> {
    pub fn new(
        device: &'a Device,
        encoder: &'a mut CommandEncoder,
        frame_view: &'a TextureView,
        world_frame: &'a WorldRenderFrame,
        surface_format: TextureFormat,
        surface_size: (u32, u32),
        pipeline: PipelineKey,
    ) -> Self {
        Self {
            device,
            encoder,
            frame_view,
            world_frame,
            surface_format,
            surface_size,
            pipeline,
            builtin_dispatch: None,
            ui_dispatch: None,
        }
    }

    pub fn with_builtin_dispatch(
        mut self,
        dispatch: &'a mut dyn FnMut(&mut CommandEncoder, BuiltinRenderPassExecutor) -> Result<()>,
    ) -> Self {
        self.builtin_dispatch = Some(dispatch);
        self
    }

    pub fn with_ui_dispatch(
        mut self,
        dispatch: &'a mut dyn FnMut(&mut CommandEncoder) -> Result<()>,
    ) -> Self {
        self.ui_dispatch = Some(dispatch);
        self
    }

    pub fn device(&self) -> &'a Device {
        self.device
    }

    pub fn encoder(&mut self) -> &mut CommandEncoder {
        self.encoder
    }

    pub fn frame_view(&self) -> &'a TextureView {
        self.frame_view
    }

    pub fn world_frame(&self) -> &'a WorldRenderFrame {
        self.world_frame
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.surface_format
    }

    pub fn surface_size(&self) -> (u32, u32) {
        self.surface_size
    }

    pub fn pipeline(&self) -> PipelineKey {
        self.pipeline
    }

    pub fn run_builtin(&mut self, builtin: BuiltinRenderPassExecutor) -> Result<()> {
        let Some(dispatch) = &mut self.builtin_dispatch else {
            bail!("builtin encode dispatch is not available in this context");
        };
        dispatch(self.encoder, builtin)
    }

    pub fn run_ui(&mut self) -> Result<()> {
        let Some(dispatch) = &mut self.ui_dispatch else {
            bail!("ui encode dispatch is not available in this context");
        };
        dispatch(self.encoder)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinRenderPassExecutor {
    WorldCompute,
    WorldCompose,
    MeshOverlay,
    UiComposite,
}

impl BuiltinRenderPassExecutor {
    pub fn label(self) -> &'static str {
        match self {
            Self::WorldCompute => "world_compute",
            Self::WorldCompose => "world_compose",
            Self::MeshOverlay => "mesh_overlay",
            Self::UiComposite => "ui_composite",
        }
    }

    pub fn from_label(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "world_compute" => Some(Self::WorldCompute),
            "world_compose" => Some(Self::WorldCompose),
            "mesh_overlay" => Some(Self::MeshOverlay),
            "ui_composite" => Some(Self::UiComposite),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct RenderPassExecutorRegistryResource {
    builtin_bindings: BTreeMap<String, BuiltinRenderPassExecutor>,
    custom_bindings: BTreeMap<String, Arc<dyn RenderPassExecutor>>,
    revision: u64,
}

impl Default for RenderPassExecutorRegistryResource {
    fn default() -> Self {
        let mut builtin_bindings = BTreeMap::new();
        let defaults = [
            BuiltinRenderPassExecutor::WorldCompute,
            BuiltinRenderPassExecutor::WorldCompose,
            BuiltinRenderPassExecutor::MeshOverlay,
            BuiltinRenderPassExecutor::UiComposite,
        ];
        for builtin in defaults {
            builtin_bindings.insert(builtin.label().to_string(), builtin);
        }
        Self {
            builtin_bindings,
            custom_bindings: BTreeMap::new(),
            revision: 0,
        }
    }
}

impl RenderPassExecutorRegistryResource {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn register_builtin(
        &mut self,
        executor_id: impl Into<String>,
        builtin: BuiltinRenderPassExecutor,
    ) {
        let executor_id = executor_id.into();
        if executor_id.trim().is_empty() {
            return;
        }
        let replaced = self.builtin_bindings.insert(executor_id.clone(), builtin);
        self.custom_bindings.remove(&executor_id);
        if replaced != Some(builtin) {
            self.revision = self.revision.saturating_add(1);
        }
    }

    pub fn register_custom(
        &mut self,
        executor_id: impl Into<String>,
        executor: Arc<dyn RenderPassExecutor>,
    ) {
        let executor_id = executor_id.into();
        if executor_id.trim().is_empty() {
            return;
        }
        let removed_builtin = self.builtin_bindings.remove(&executor_id).is_some();
        let replaced = self
            .custom_bindings
            .insert(executor_id, Arc::clone(&executor));
        let custom_changed = replaced
            .as_ref()
            .map_or(true, |previous| !Arc::ptr_eq(previous, &executor));
        if removed_builtin || custom_changed {
            self.revision = self.revision.saturating_add(1);
        }
    }

    pub fn remove(&mut self, executor_id: &str) -> bool {
        let removed_builtin = self.builtin_bindings.remove(executor_id).is_some();
        let removed_custom = self.custom_bindings.remove(executor_id).is_some();
        let removed = removed_builtin || removed_custom;
        if removed {
            self.revision = self.revision.saturating_add(1);
        }
        removed
    }

    pub fn resolve_builtin(&self, executor_id: &str) -> Option<BuiltinRenderPassExecutor> {
        self.builtin_bindings.get(executor_id).copied()
    }

    pub fn resolve_custom(&self, executor_id: &str) -> Option<Arc<dyn RenderPassExecutor>> {
        self.custom_bindings.get(executor_id).cloned()
    }

    pub fn contains(&self, executor_id: &str) -> bool {
        self.builtin_bindings.contains_key(executor_id)
            || self.custom_bindings.contains_key(executor_id)
    }

    pub fn clear_custom(&mut self) {
        let defaults = [
            BuiltinRenderPassExecutor::WorldCompute,
            BuiltinRenderPassExecutor::WorldCompose,
            BuiltinRenderPassExecutor::MeshOverlay,
            BuiltinRenderPassExecutor::UiComposite,
        ];
        let mut next = BTreeMap::new();
        for builtin in defaults {
            next.insert(builtin.label().to_string(), builtin);
        }
        if next == self.builtin_bindings && self.custom_bindings.is_empty() {
            return;
        }
        self.builtin_bindings = next;
        self.custom_bindings.clear();
        self.revision = self.revision.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BuiltinRenderPassExecutor, RenderPassEncodeContext, RenderPassExecutor,
        RenderPassExecutorRegistryResource,
    };
    use anyhow::Result;
    use std::sync::Arc;

    struct TestExecutor;

    impl RenderPassExecutor for TestExecutor {
        fn encode(&self, _ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn defaults_include_builtin_executor_ids() {
        let registry = RenderPassExecutorRegistryResource::default();
        assert_eq!(
            registry.resolve_builtin("world_compute"),
            Some(BuiltinRenderPassExecutor::WorldCompute)
        );
        assert_eq!(
            registry.resolve_builtin("world_compose"),
            Some(BuiltinRenderPassExecutor::WorldCompose)
        );
        assert_eq!(
            registry.resolve_builtin("mesh_overlay"),
            Some(BuiltinRenderPassExecutor::MeshOverlay)
        );
        assert_eq!(
            registry.resolve_builtin("ui_composite"),
            Some(BuiltinRenderPassExecutor::UiComposite)
        );
    }

    #[test]
    fn register_builtin_alias_updates_binding() {
        let mut registry = RenderPassExecutorRegistryResource::default();
        registry.register_builtin("sdf.compute", BuiltinRenderPassExecutor::WorldCompute);
        registry.register_builtin("sdf.compose", BuiltinRenderPassExecutor::WorldCompose);

        assert_eq!(
            registry.resolve_builtin("sdf.compute"),
            Some(BuiltinRenderPassExecutor::WorldCompute)
        );
        assert_eq!(
            registry.resolve_builtin("sdf.compose"),
            Some(BuiltinRenderPassExecutor::WorldCompose)
        );
    }

    #[test]
    fn register_custom_overrides_builtin_slot() {
        let mut registry = RenderPassExecutorRegistryResource::default();
        registry.register_custom("world_compute", Arc::new(TestExecutor));
        assert!(registry.resolve_builtin("world_compute").is_none());
        assert!(registry.resolve_custom("world_compute").is_some());
    }

    #[test]
    fn clear_custom_keeps_builtin_defaults() {
        let mut registry = RenderPassExecutorRegistryResource::default();
        registry.register_builtin("sdf.compute", BuiltinRenderPassExecutor::WorldCompute);
        registry.register_custom("custom.compute", Arc::new(TestExecutor));
        assert!(registry.contains("sdf.compute"));
        assert!(registry.contains("custom.compute"));
        registry.clear_custom();
        assert!(!registry.contains("sdf.compute"));
        assert!(!registry.contains("custom.compute"));
        assert!(registry.contains("world_compute"));
    }
}
