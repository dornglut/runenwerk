use super::{BuiltinRenderPassExecutor, RenderPassExecutor};
use std::collections::BTreeMap;
use std::sync::Arc;

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

#[derive(ecs::Component)]
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
