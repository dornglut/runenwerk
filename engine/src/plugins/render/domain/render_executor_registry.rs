use super::PipelineKey;
use anyhow::{Result, bail};
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap};
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

    pub fn extend_from(&mut self, other: &RenderFrameDataRegistry<'a>) {
        self.by_type.extend(
            other
                .by_type
                .iter()
                .map(|(type_id, value)| (*type_id, *value)),
        );
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

pub struct RenderPassExecutorRegistryResource {
    bindings: BTreeMap<String, ExecutorBindingKind>,
    events: Vec<RenderPassExecutorRegistryEvent>,
    revision: u64,
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
        Self {
            bindings: BTreeMap::new(),
            events: Vec::new(),
            revision: 0,
        }
    }
}

impl RenderPassExecutorRegistryResource {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn binding_count(&self) -> usize {
        self.bindings.len()
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

        let changed = !matches!(
            self.bindings.get(&executor_id),
            Some(ExecutorBindingKind::Builtin(current)) if *current == builtin
        );
        if changed {
            self.bindings
                .insert(executor_id.clone(), ExecutorBindingKind::Builtin(builtin));
        }

        if changed {
            let revision = self.bump_revision();
            self.events.push(RenderPassExecutorRegistryEvent {
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

        let changed = match self.bindings.get(&executor_id) {
            Some(ExecutorBindingKind::Custom(current)) => !Arc::ptr_eq(current, &executor),
            Some(ExecutorBindingKind::Builtin(_)) | None => true,
        };
        if changed {
            self.bindings.insert(
                executor_id.clone(),
                ExecutorBindingKind::Custom(Arc::clone(&executor)),
            );
        }

        if changed {
            let revision = self.bump_revision();
            self.events.push(RenderPassExecutorRegistryEvent {
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
        if self.bindings.remove(&executor_id).is_none() {
            return false;
        }
        let revision = self.bump_revision();
        self.events.push(RenderPassExecutorRegistryEvent {
            kind: RenderPassExecutorRegistryEventKind::Removed,
            executor_id: Some(executor_id),
            builtin: None,
            revision,
            details: None,
        });
        true
    }

    pub fn resolve_builtin(&self, executor_id: &str) -> Option<BuiltinRenderPassExecutor> {
        match self.binding(executor_id)? {
            ExecutorBindingKind::Builtin(builtin) => Some(*builtin),
            ExecutorBindingKind::Custom(_) => None,
        }
    }

    pub fn resolve_custom(&self, executor_id: &str) -> Option<Arc<dyn RenderPassExecutor>> {
        match self.binding(executor_id)? {
            ExecutorBindingKind::Builtin(_) => None,
            ExecutorBindingKind::Custom(custom) => Some(Arc::clone(custom)),
        }
    }

    pub fn contains(&self, executor_id: &str) -> bool {
        self.binding(executor_id).is_some()
    }

    pub fn clear_custom(&mut self) {
        let custom_ids: Vec<_> = self
            .bindings
            .iter()
            .filter_map(|(executor_id, binding)| {
                if matches!(binding, ExecutorBindingKind::Custom(_)) {
                    Some(executor_id.clone())
                } else {
                    None
                }
            })
            .collect();
        let removed_any = !custom_ids.is_empty();
        for executor_id in custom_ids {
            self.bindings.remove(&executor_id);
        }
        if !removed_any {
            return;
        }
        let revision = self.bump_revision();
        self.events.push(RenderPassExecutorRegistryEvent {
            kind: RenderPassExecutorRegistryEventKind::ClearedCustom,
            executor_id: None,
            builtin: None,
            revision,
            details: Some("custom executor bindings cleared".to_string()),
        });
    }

    pub fn read_events(&self) -> &[RenderPassExecutorRegistryEvent] {
        &self.events
    }

    pub fn drain_events(&mut self) -> Vec<RenderPassExecutorRegistryEvent> {
        std::mem::take(&mut self.events)
    }

    fn binding(&self, executor_id: &str) -> Option<&ExecutorBindingKind> {
        let executor_id = normalize_executor_id(executor_id);
        if executor_id.is_empty() {
            return None;
        }
        self.bindings.get(&executor_id)
    }

    fn bump_revision(&mut self) -> u64 {
        self.revision = self.revision.saturating_add(1);
        self.revision
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
    fn frame_data_registry_supports_lookup() {
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
