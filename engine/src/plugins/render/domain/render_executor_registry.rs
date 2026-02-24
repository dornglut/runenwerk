use super::PipelineKey;
use anyhow::{Result, bail};
use ecs::{Component, World};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};

pub trait RenderPassExecutor: Send + Sync {
    fn prepare(&self, _ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        Ok(())
    }

    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()>;
}

#[derive(Default)]
pub struct RenderFrameDataRegistry<'a> {
    by_type: HashMap<TypeId, &'a dyn Any>,
}

impl<'a> RenderFrameDataRegistry<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<T: 'static>(mut self, value: &'a T) -> Self {
        self.insert(value);
        self
    }

    pub fn insert<T: 'static>(&mut self, value: &'a T) {
        self.by_type.insert(TypeId::of::<T>(), value);
    }

    pub fn get<T: 'static>(&self) -> Option<&'a T> {
        self.by_type
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }
}

pub struct RenderPassPrepareContext<'a> {
    device: &'a Device,
    queue: &'a Queue,
    frame_data: &'a RenderFrameDataRegistry<'a>,
    surface_format: TextureFormat,
    surface_size: (u32, u32),
    builtin_dispatch: Option<&'a mut dyn FnMut(BuiltinRenderPassExecutor) -> Result<()>>,
}

impl<'a> RenderPassPrepareContext<'a> {
    pub fn new(
        device: &'a Device,
        queue: &'a Queue,
        frame_data: &'a RenderFrameDataRegistry<'a>,
        surface_format: TextureFormat,
        surface_size: (u32, u32),
    ) -> Self {
        Self {
            device,
            queue,
            frame_data,
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

    pub fn frame_data<T: 'static>(&self) -> Option<&'a T> {
        self.frame_data.get::<T>()
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
    frame_data: &'a RenderFrameDataRegistry<'a>,
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
        frame_data: &'a RenderFrameDataRegistry<'a>,
        surface_format: TextureFormat,
        surface_size: (u32, u32),
        pipeline: PipelineKey,
    ) -> Self {
        Self {
            device,
            encoder,
            frame_view,
            frame_data,
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

    pub fn frame_data<T: 'static>(&self) -> Option<&'a T> {
        self.frame_data.get::<T>()
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.surface_format
    }

    pub fn surface_size(&self) -> (u32, u32) {
        self.surface_size
    }

    pub fn pipeline(&self) -> &PipelineKey {
        &self.pipeline
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
    Compute,
    Compose,
    MeshOverlay,
    UiComposite,
}

impl BuiltinRenderPassExecutor {
    pub fn label(self) -> &'static str {
        match self {
            Self::Compute => "builtin_compute",
            Self::Compose => "builtin_compose",
            Self::MeshOverlay => "builtin_mesh_overlay",
            Self::UiComposite => "builtin_ui_composite",
        }
    }

    pub fn from_label(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "builtin_compute" => Some(Self::Compute),
            "builtin_compose" => Some(Self::Compose),
            "builtin_mesh_overlay" => Some(Self::MeshOverlay),
            "builtin_ui_composite" => Some(Self::UiComposite),
            _ => None,
        }
    }
}

#[derive(Clone)]
enum ExecutorBindingKind {
    Builtin(BuiltinRenderPassExecutor),
    Custom(Arc<dyn RenderPassExecutor>),
}

impl std::fmt::Debug for ExecutorBindingKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Builtin(value) => f.debug_tuple("Builtin").field(value).finish(),
            Self::Custom(_) => f.write_str("Custom(<dyn RenderPassExecutor>)"),
        }
    }
}

#[derive(Clone)]
struct RenderExecutorBindingComponent {
    executor_id: String,
    binding: ExecutorBindingKind,
}

impl Component for RenderExecutorBindingComponent {}

impl std::fmt::Debug for RenderExecutorBindingComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderExecutorBindingComponent")
            .field("executor_id", &self.executor_id)
            .field("binding", &self.binding)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPassExecutorRegistryEventKind {
    RegisteredBuiltin,
    RegisteredCustom,
    Removed,
    ClearedCustom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPassExecutorRegistryEvent {
    pub kind: RenderPassExecutorRegistryEventKind,
    pub executor_id: Option<String>,
    pub builtin: Option<BuiltinRenderPassExecutor>,
    pub revision: u64,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct RenderPassExecutorRegistryMetaResource {
    revision: u64,
}

pub struct RenderPassExecutorRegistryResource {
    world: World,
}

impl std::fmt::Debug for RenderPassExecutorRegistryResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderPassExecutorRegistryResource")
            .field("binding_count", &self.binding_count())
            .field("revision", &self.revision())
            .finish()
    }
}

impl Default for RenderPassExecutorRegistryResource {
    fn default() -> Self {
        let mut world = World::new();
        world.register_component::<RenderExecutorBindingComponent>();
        world.ensure_component_index::<RenderExecutorBindingComponent, String>(|binding| {
            binding.executor_id.clone()
        });
        world.ensure_event_channel::<RenderPassExecutorRegistryEvent>();
        world.insert_resource(RenderPassExecutorRegistryMetaResource::default());
        Self { world }
    }
}

impl RenderPassExecutorRegistryResource {
    pub fn revision(&self) -> u64 {
        self.meta().revision
    }

    pub fn binding_count(&self) -> usize {
        self.world
            .entities_with::<RenderExecutorBindingComponent>()
            .count()
    }

    pub fn register_builtin(
        &mut self,
        executor_id: impl Into<String>,
        builtin: BuiltinRenderPassExecutor,
    ) {
        let executor_id = normalize_executor_id(executor_id.into());
        if executor_id.is_empty() {
            return;
        }

        let mut changed = false;
        if let Some(entity) = self.binding_entity(&executor_id) {
            if let Some(binding) = self
                .world
                .get_component_mut::<RenderExecutorBindingComponent>(entity)
            {
                changed = !matches!(
                    binding.binding,
                    ExecutorBindingKind::Builtin(current) if current == builtin
                );
                if changed {
                    binding.binding = ExecutorBindingKind::Builtin(builtin);
                }
            }
        } else {
            let _ = self
                .world
                .spawn_entity_typed(RenderExecutorBindingComponent {
                    executor_id: executor_id.clone(),
                    binding: ExecutorBindingKind::Builtin(builtin),
                });
            changed = true;
        }

        if changed {
            let revision = self.bump_revision();
            self.world.emit_event(RenderPassExecutorRegistryEvent {
                kind: RenderPassExecutorRegistryEventKind::RegisteredBuiltin,
                executor_id: Some(executor_id),
                builtin: Some(builtin),
                revision,
                details: None,
            });
        }
    }

    pub fn register_custom(
        &mut self,
        executor_id: impl Into<String>,
        executor: Arc<dyn RenderPassExecutor>,
    ) {
        let executor_id = normalize_executor_id(executor_id.into());
        if executor_id.is_empty() {
            return;
        }

        let mut changed = false;
        if let Some(entity) = self.binding_entity(&executor_id) {
            if let Some(binding) = self
                .world
                .get_component_mut::<RenderExecutorBindingComponent>(entity)
            {
                changed = match &binding.binding {
                    ExecutorBindingKind::Custom(current) => !Arc::ptr_eq(current, &executor),
                    ExecutorBindingKind::Builtin(_) => true,
                };
                if changed {
                    binding.binding = ExecutorBindingKind::Custom(Arc::clone(&executor));
                }
            }
        } else {
            let _ = self
                .world
                .spawn_entity_typed(RenderExecutorBindingComponent {
                    executor_id: executor_id.clone(),
                    binding: ExecutorBindingKind::Custom(Arc::clone(&executor)),
                });
            changed = true;
        }

        if changed {
            let revision = self.bump_revision();
            self.world.emit_event(RenderPassExecutorRegistryEvent {
                kind: RenderPassExecutorRegistryEventKind::RegisteredCustom,
                executor_id: Some(executor_id),
                builtin: None,
                revision,
                details: None,
            });
        }
    }

    pub fn remove(&mut self, executor_id: &str) -> bool {
        let executor_id = normalize_executor_id(executor_id);
        if executor_id.is_empty() {
            return false;
        }
        let Some(entity) = self.binding_entity(&executor_id) else {
            return false;
        };
        self.world.remove_entity(entity);
        let revision = self.bump_revision();
        self.world.emit_event(RenderPassExecutorRegistryEvent {
            kind: RenderPassExecutorRegistryEventKind::Removed,
            executor_id: Some(executor_id),
            builtin: None,
            revision,
            details: None,
        });
        true
    }

    pub fn resolve_builtin(&self, executor_id: &str) -> Option<BuiltinRenderPassExecutor> {
        let binding = self.binding(executor_id)?;
        match binding.binding {
            ExecutorBindingKind::Builtin(builtin) => Some(builtin),
            ExecutorBindingKind::Custom(_) => None,
        }
    }

    pub fn resolve_custom(&self, executor_id: &str) -> Option<Arc<dyn RenderPassExecutor>> {
        let binding = self.binding(executor_id)?;
        match &binding.binding {
            ExecutorBindingKind::Builtin(_) => None,
            ExecutorBindingKind::Custom(custom) => Some(Arc::clone(custom)),
        }
    }

    pub fn contains(&self, executor_id: &str) -> bool {
        self.binding(executor_id).is_some()
    }

    pub fn clear_custom(&mut self) {
        let entities: Vec<_> = self
            .world
            .entities_with::<RenderExecutorBindingComponent>()
            .collect();
        let mut removed_any = false;
        for entity in entities {
            let remove = self
                .world
                .get_component::<RenderExecutorBindingComponent>(entity)
                .is_some_and(|binding| matches!(binding.binding, ExecutorBindingKind::Custom(_)));
            if remove {
                self.world.remove_entity(entity);
                removed_any = true;
            }
        }
        if !removed_any {
            return;
        }
        let revision = self.bump_revision();
        self.world.emit_event(RenderPassExecutorRegistryEvent {
            kind: RenderPassExecutorRegistryEventKind::ClearedCustom,
            executor_id: None,
            builtin: None,
            revision,
            details: Some("custom executor bindings cleared".to_string()),
        });
    }

    pub fn read_events(&self) -> &[RenderPassExecutorRegistryEvent] {
        self.world.read_events::<RenderPassExecutorRegistryEvent>()
    }

    pub fn drain_events(&mut self) -> Vec<RenderPassExecutorRegistryEvent> {
        self.world.drain_events::<RenderPassExecutorRegistryEvent>()
    }

    fn binding(&self, executor_id: &str) -> Option<&RenderExecutorBindingComponent> {
        let executor_id = normalize_executor_id(executor_id);
        if executor_id.is_empty() {
            return None;
        }
        self.world
            .entities_with::<RenderExecutorBindingComponent>()
            .find_map(|entity| {
                let binding = self
                    .world
                    .get_component::<RenderExecutorBindingComponent>(entity)?;
                if binding.executor_id == executor_id {
                    Some(binding)
                } else {
                    None
                }
            })
    }

    fn binding_entity(&mut self, executor_id: &str) -> Option<ecs::EntityHandle> {
        let executor_id = normalize_executor_id(executor_id);
        if executor_id.is_empty() {
            return None;
        }
        self.world
            .find_entity_by_index::<RenderExecutorBindingComponent, String>(&executor_id)
    }

    fn meta(&self) -> &RenderPassExecutorRegistryMetaResource {
        self.world
            .get_resource::<RenderPassExecutorRegistryMetaResource>()
            .expect("render executor registry meta resource should exist")
    }

    fn meta_mut(&mut self) -> &mut RenderPassExecutorRegistryMetaResource {
        self.world
            .get_resource_mut::<RenderPassExecutorRegistryMetaResource>()
            .expect("render executor registry meta resource should exist")
    }

    fn bump_revision(&mut self) -> u64 {
        let meta = self.meta_mut();
        meta.revision = meta.revision.saturating_add(1);
        meta.revision
    }
}

fn normalize_executor_id(value: impl AsRef<str>) -> String {
    value.as_ref().trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        BuiltinRenderPassExecutor, RenderFrameDataRegistry, RenderPassEncodeContext,
        RenderPassExecutor, RenderPassExecutorRegistryEventKind,
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
    fn frame_data_registry_supports_typed_lookup() {
        let value = 42_u32;
        let registry = RenderFrameDataRegistry::new().with(&value);
        assert_eq!(registry.get::<u32>(), Some(&42_u32));
        assert!(registry.get::<u64>().is_none());
    }

    #[test]
    fn defaults_start_empty() {
        let registry = RenderPassExecutorRegistryResource::default();
        assert!(!registry.contains("builtin_compute"));
        assert!(!registry.contains("builtin_compose"));
        assert!(!registry.contains("builtin_mesh_overlay"));
        assert!(!registry.contains("builtin_ui_composite"));
    }

    #[test]
    fn register_builtin_alias_updates_binding() {
        let mut registry = RenderPassExecutorRegistryResource::default();
        registry.register_builtin("sdf.compute", BuiltinRenderPassExecutor::Compute);
        registry.register_builtin("sdf.compose", BuiltinRenderPassExecutor::Compose);

        assert_eq!(
            registry.resolve_builtin("sdf.compute"),
            Some(BuiltinRenderPassExecutor::Compute)
        );
        assert_eq!(
            registry.resolve_builtin("sdf.compose"),
            Some(BuiltinRenderPassExecutor::Compose)
        );
    }

    #[test]
    fn register_custom_overrides_builtin_slot() {
        let mut registry = RenderPassExecutorRegistryResource::default();
        registry.register_custom("builtin_compute", Arc::new(TestExecutor));
        assert!(registry.resolve_builtin("builtin_compute").is_none());
        assert!(registry.resolve_custom("builtin_compute").is_some());
    }

    #[test]
    fn clear_custom_keeps_registered_builtins() {
        let mut registry = RenderPassExecutorRegistryResource::default();
        registry.register_builtin("sdf.compute", BuiltinRenderPassExecutor::Compute);
        registry.register_custom("custom.compute", Arc::new(TestExecutor));
        assert!(registry.contains("sdf.compute"));
        assert!(registry.contains("custom.compute"));
        registry.clear_custom();
        assert!(registry.contains("sdf.compute"));
        assert!(!registry.contains("custom.compute"));
    }

    #[test]
    fn registry_emits_events_for_mutations() {
        let mut registry = RenderPassExecutorRegistryResource::default();
        assert!(registry.read_events().is_empty());

        registry.register_builtin("sdf.compute", BuiltinRenderPassExecutor::Compute);
        registry.register_custom("custom.compute", Arc::new(TestExecutor));
        assert!(registry.remove("custom.compute"));
        registry.register_custom("custom.compose", Arc::new(TestExecutor));
        registry.clear_custom();

        let events = registry.drain_events();
        assert!(events.iter().any(|event| {
            event.kind == RenderPassExecutorRegistryEventKind::RegisteredBuiltin
                && event.executor_id.as_deref() == Some("sdf.compute")
        }));
        assert!(events.iter().any(|event| {
            event.kind == RenderPassExecutorRegistryEventKind::RegisteredCustom
                && event.executor_id.as_deref() == Some("custom.compute")
        }));
        assert!(events.iter().any(|event| {
            event.kind == RenderPassExecutorRegistryEventKind::RegisteredCustom
                && event.executor_id.as_deref() == Some("custom.compose")
        }));
        assert!(events.iter().any(|event| {
            event.kind == RenderPassExecutorRegistryEventKind::Removed
                && event.executor_id.as_deref() == Some("custom.compute")
        }));
        assert!(events.iter().any(|event| {
            event.kind == RenderPassExecutorRegistryEventKind::ClearedCustom
                && event.executor_id.is_none()
        }));
    }
}
