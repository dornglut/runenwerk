use super::executor_contexts::{RenderPassEncodeContext, RenderPassPrepareContext};
use anyhow::Result;

pub trait RenderPassExecutor: Send + Sync {
    fn prepare(&self, _ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        Ok(())
    }

    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::super::{
        BuiltinRenderPassExecutor, RenderPassEncodeContext, RenderPassExecutorRegistryEventKind,
        RenderPassExecutorRegistryResource,
    };
    use super::RenderPassExecutor;
    use anyhow::Result;
    use std::sync::Arc;

    struct TestExecutor;

    impl RenderPassExecutor for TestExecutor {
        fn encode(&self, _ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
            Ok(())
        }
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
