use super::pipeline_key::PipelineKey;
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



mod contexts_and_builtin;
mod frame_data;
mod registry;

pub use contexts_and_builtin::*;
pub use frame_data::*;
pub use registry::*;

// Owner: Grotto Quest Engine - Render Domain
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


