use crate::plugins::shared::{
    ReloadStatusPayload, file_modified, should_poll, should_reload, watch_status_line,
};
use ecs::{EntityHandle, World};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub const DEFAULT_SHADER_ASSET_ROOT: &str = "assets/shaders";

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ShaderHandle(EntityHandle);

impl ShaderHandle {
    pub fn entity(self) -> EntityHandle {
        self.0
    }
}

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct ShaderAssetComponent {
    pub id: String,
    pub path: String,
    pub source: Option<String>,
    pub modified: Option<SystemTime>,
    pub revision: u64,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
struct ShaderRegistryConfigResource {
    roots: Vec<String>,
    watch_enabled: bool,
    force_reload: bool,
    revision: u64,
}

impl Default for ShaderRegistryConfigResource {
    fn default() -> Self {
        Self {
            roots: vec![DEFAULT_SHADER_ASSET_ROOT.to_string()],
            watch_enabled: true,
            force_reload: true,
            revision: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderRegistryEventKind {
    Discovered,
    Registered,
    PathUpdated,
    DuplicateId,
    Reloaded,
    SkippedEmpty,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderRegistryEvent {
    pub kind: ShaderRegistryEventKind,
    pub id: String,
    pub path: String,
    pub revision: u64,
    pub error: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ShaderStatus {
    pub handle: ShaderHandle,
    pub id: String,
    pub path: String,
    pub revision: u64,
    pub loaded: bool,
    pub modified: Option<SystemTime>,
    pub last_error: Option<String>,
}

pub struct ShaderRegistryResource {
    world: World,
}

impl std::fmt::Debug for ShaderRegistryResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderRegistryResource")
            .field("shader_count", &self.shader_count())
            .field("roots", &self.roots())
            .field("watch_enabled", &self.watch_enabled())
            .field("revision", &self.revision())
            .finish()
    }
}

impl Clone for ShaderRegistryResource {
    fn clone(&self) -> Self {
        // ECS world clone is intentionally shallow-by-state here: use statuses + config replay.
        let mut next = Self::with_roots(self.roots().to_vec());
        next.set_watch_enabled(self.watch_enabled());
        for status in self.statuses() {
            let _ = next.register_shader(status.id, status.path);
        }
        next
    }
}

impl Default for ShaderRegistryResource {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderRegistryResource {
    pub fn new() -> Self {
        Self::with_roots([DEFAULT_SHADER_ASSET_ROOT])
    }

    pub fn with_roots<I, S>(roots: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut world = World::new();
        world.register_component::<ShaderAssetComponent>();
        world.ensure_component_index::<ShaderAssetComponent, String>(|asset| asset.id.clone());
        world.ensure_event_channel::<ShaderRegistryEvent>();
        world.insert_resource(ShaderRegistryConfigResource {
            roots: normalize_roots(roots),
            ..ShaderRegistryConfigResource::default()
        });
        Self { world }
    }

    pub fn revision(&self) -> u64 {
        self.config().revision
    }

    pub fn shader_count(&self) -> usize {
        self.world.entities_with::<ShaderAssetComponent>().count()
    }

    pub fn roots(&self) -> &[String] {
        &self.config().roots
    }

    pub fn set_roots<I, S>(&mut self, roots: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let config = self.config_mut();
        config.roots = normalize_roots(roots);
        config.force_reload = true;
    }

    pub fn add_root(&mut self, root: impl Into<String>) {
        let root = root.into();
        let root = root.trim();
        if root.is_empty() {
            return;
        }
        let config = self.config_mut();
        if config.roots.iter().any(|value| value == root) {
            return;
        }
        config.roots.push(root.to_string());
        config.force_reload = true;
    }

    pub fn watch_enabled(&self) -> bool {
        self.config().watch_enabled
    }

    pub fn set_watch_enabled(&mut self, enabled: bool) {
        self.config_mut().watch_enabled = enabled;
    }

    pub fn request_reload(&mut self) {
        self.config_mut().force_reload = true;
    }

    pub fn register_shader(
        &mut self,
        id: impl Into<String>,
        path: impl Into<String>,
    ) -> ShaderHandle {
        let mut id = normalize_shader_id(id.into());
        let path = path.into();
        if id.is_empty() {
            id = derive_shader_id_from_path(Path::new(&path));
        }

        if let Some(existing) = self.handle(&id) {
            let mut event = None;
            if let Some(asset) = self
                .world
                .get_component_mut::<ShaderAssetComponent>(existing.entity())
                && asset.path != path
            {
                asset.path = path.clone();
                asset.modified = None;
                event = Some(ShaderRegistryEvent {
                    kind: ShaderRegistryEventKind::PathUpdated,
                    id: id.clone(),
                    path: path.clone(),
                    revision: asset.revision,
                    error: None,
                    details: None,
                });
            }
            if let Some(event) = event {
                self.config_mut().force_reload = true;
                self.world.emit_event(event);
            }
            return existing;
        }

        let handle = ShaderHandle(self.world.spawn_entity_typed(ShaderAssetComponent {
            id: id.clone(),
            path: path.clone(),
            ..ShaderAssetComponent::default()
        }));
        self.config_mut().force_reload = true;
        self.world.emit_event(ShaderRegistryEvent {
            kind: ShaderRegistryEventKind::Registered,
            id,
            path,
            revision: 0,
            error: None,
            details: None,
        });
        handle
    }

    pub fn handle(&mut self, id: impl AsRef<str>) -> Option<ShaderHandle> {
        let id = normalize_shader_id(id.as_ref());
        self.world
            .find_entity_by_index::<ShaderAssetComponent, String>(&id)
            .map(ShaderHandle)
    }

    pub fn source_or<'a>(&'a mut self, id: &str, fallback: &'a str) -> &'a str {
        self.handle(id)
            .and_then(|handle| self.asset(handle))
            .and_then(|asset| {
                asset
                    .source
                    .as_deref()
                    .filter(|source| !source.trim().is_empty())
            })
            .unwrap_or(fallback)
    }

    pub fn source_or_handle<'a>(&'a self, handle: ShaderHandle, fallback: &'a str) -> &'a str {
        self.asset(handle)
            .and_then(|asset| {
                asset
                    .source
                    .as_deref()
                    .filter(|source| !source.trim().is_empty())
            })
            .unwrap_or(fallback)
    }

    pub fn revision_for(&mut self, id: &str) -> u64 {
        self.handle(id)
            .map(|handle| self.revision_for_handle(handle))
            .unwrap_or(0)
    }

    pub fn revision_for_handle(&self, handle: ShaderHandle) -> u64 {
        self.asset(handle).map(|asset| asset.revision).unwrap_or(0)
    }

    pub fn statuses(&self) -> Vec<ShaderStatus> {
        let mut statuses: Vec<ShaderStatus> = self
            .world
            .entities_with::<ShaderAssetComponent>()
            .filter_map(|entity| {
                let handle = ShaderHandle(entity);
                self.asset(handle).map(|asset| ShaderStatus {
                    handle,
                    id: asset.id.clone(),
                    path: asset.path.clone(),
                    revision: asset.revision,
                    loaded: asset.source.is_some(),
                    modified: asset.modified,
                    last_error: asset.last_error.clone(),
                })
            })
            .collect();
        statuses.sort_by(|a, b| a.id.cmp(&b.id));
        statuses
    }

    pub fn status_payloads(&self) -> Vec<ReloadStatusPayload> {
        self.statuses()
            .into_iter()
            .map(|status| {
                let state = if status.last_error.is_some() {
                    "error"
                } else if status.loaded {
                    "loaded"
                } else {
                    "fallback"
                };
                ReloadStatusPayload::new(
                    "shader",
                    status.id,
                    state,
                    status.path,
                    status.revision,
                    self.watch_enabled(),
                    status.modified,
                    status.last_error,
                    None,
                )
            })
            .collect()
    }

    pub fn status_lines(&self) -> Vec<String> {
        let root_summary = self.roots().join(",");
        let source = if root_summary.is_empty() {
            "(none)"
        } else {
            root_summary.as_str()
        };
        let mut lines = vec![watch_status_line("shader", self.watch_enabled(), source)];
        lines.extend(
            self.status_payloads()
                .into_iter()
                .map(|payload| payload.line()),
        );
        lines
    }

    pub fn read_events(&self) -> &[ShaderRegistryEvent] {
        self.world.read_events::<ShaderRegistryEvent>()
    }

    pub fn drain_events(&mut self) -> Vec<ShaderRegistryEvent> {
        self.world.drain_events::<ShaderRegistryEvent>()
    }

    pub fn drain_event_lines(&mut self) -> Vec<String> {
        let watch_enabled = self.watch_enabled();
        self.drain_events()
            .into_iter()
            .map(|event| {
                ReloadStatusPayload::new(
                    "shader",
                    event.id,
                    shader_event_state_label(event.kind),
                    event.path,
                    event.revision,
                    watch_enabled,
                    None,
                    event.error,
                    event.details,
                )
                .line()
            })
            .collect()
    }

    pub fn finish_event_frame(&mut self) {
        self.world.finish_event_frame();
    }

    pub fn poll_updates(&mut self) -> Vec<String> {
        let watch_enabled = self.watch_enabled();
        let force_reload = self.config().force_reload;
        if !should_poll(watch_enabled, force_reload) {
            return Vec::new();
        }
        self.config_mut().force_reload = false;

        let mut lines = self.discover_from_roots();
        let mut changed = false;

        let handles: Vec<_> = self
            .world
            .entities_with::<ShaderAssetComponent>()
            .map(ShaderHandle)
            .collect();
        for handle in handles {
            let Some(snapshot) = self.asset(handle).map(|asset| {
                (
                    asset.id.clone(),
                    asset.path.clone(),
                    asset.modified,
                    asset.revision,
                )
            }) else {
                continue;
            };
            let (id, path, previous_modified, previous_revision) = snapshot;
            let modified = file_modified(Path::new(&path));
            if !should_reload(watch_enabled, force_reload, previous_modified, modified) {
                continue;
            }

            match fs::read_to_string(&path) {
                Ok(source) if !source.trim().is_empty() => {
                    if let Some(asset) = self.asset_mut(handle) {
                        asset.source = Some(source);
                        asset.modified = modified;
                        asset.revision = asset.revision.saturating_add(1);
                        asset.last_error = None;
                    }
                    let revision = self.revision_for_handle(handle);
                    changed = true;
                    lines.push(
                        ReloadStatusPayload::new(
                            "shader",
                            &id,
                            "reloaded",
                            path.clone(),
                            revision,
                            watch_enabled,
                            modified,
                            None,
                            None,
                        )
                        .line(),
                    );
                    self.world.emit_event(ShaderRegistryEvent {
                        kind: ShaderRegistryEventKind::Reloaded,
                        id,
                        path,
                        revision,
                        error: None,
                        details: None,
                    });
                }
                Ok(_) => {
                    if let Some(asset) = self.asset_mut(handle) {
                        asset.last_error = Some("file is empty".to_string());
                        asset.modified = modified;
                    }
                    lines.push(
                        ReloadStatusPayload::new(
                            "shader",
                            &id,
                            "skipped_empty",
                            path.clone(),
                            previous_revision,
                            watch_enabled,
                            modified,
                            Some("file is empty".to_string()),
                            None,
                        )
                        .line(),
                    );
                    self.world.emit_event(ShaderRegistryEvent {
                        kind: ShaderRegistryEventKind::SkippedEmpty,
                        id,
                        path,
                        revision: previous_revision,
                        error: Some("file is empty".to_string()),
                        details: None,
                    });
                }
                Err(err) => {
                    if let Some(asset) = self.asset_mut(handle) {
                        asset.last_error = Some(err.to_string());
                        asset.modified = modified;
                    }
                    lines.push(
                        ReloadStatusPayload::new(
                            "shader",
                            &id,
                            "failed",
                            path.clone(),
                            previous_revision,
                            watch_enabled,
                            modified,
                            Some(err.to_string()),
                            None,
                        )
                        .line(),
                    );
                    self.world.emit_event(ShaderRegistryEvent {
                        kind: ShaderRegistryEventKind::Failed,
                        id,
                        path,
                        revision: previous_revision,
                        error: Some(err.to_string()),
                        details: None,
                    });
                }
            }
        }

        if changed {
            self.config_mut().revision = self.config().revision.saturating_add(1);
        }

        lines
    }

    fn discover_from_roots(&mut self) -> Vec<String> {
        let mut lines = Vec::new();
        let watch_enabled = self.watch_enabled();
        let roots = self.roots().to_vec();
        for root in roots {
            let root_path = Path::new(&root);
            for path in discover_shader_files(root_path) {
                let id = derive_shader_id_for_root(root_path, &path);
                let path_string = path.to_string_lossy().to_string();
                if let Some(handle) = self.handle(&id) {
                    if let Some(existing) = self.asset(handle)
                        && existing.path != path_string
                    {
                        lines.push(
                            ReloadStatusPayload::new(
                                "shader",
                                &id,
                                "duplicate_id",
                                path_string.clone(),
                                existing.revision,
                                watch_enabled,
                                existing.modified,
                                None,
                                Some(format!(
                                    "retaining existing path '{}' for this id",
                                    existing.path
                                )),
                            )
                            .line(),
                        );
                        self.world.emit_event(ShaderRegistryEvent {
                            kind: ShaderRegistryEventKind::DuplicateId,
                            id: id.clone(),
                            path: path_string,
                            revision: existing.revision,
                            error: None,
                            details: Some(format!(
                                "retaining existing path '{}' for this id",
                                existing.path
                            )),
                        });
                    }
                    continue;
                }

                let _ = self.world.spawn_entity_typed(ShaderAssetComponent {
                    id: id.clone(),
                    path: path_string.clone(),
                    ..ShaderAssetComponent::default()
                });
                lines.push(
                    ReloadStatusPayload::new(
                        "shader",
                        &id,
                        "discovered",
                        path_string.clone(),
                        0,
                        watch_enabled,
                        None,
                        None,
                        None,
                    )
                    .line(),
                );
                self.world.emit_event(ShaderRegistryEvent {
                    kind: ShaderRegistryEventKind::Discovered,
                    id,
                    path: path_string,
                    revision: 0,
                    error: None,
                    details: None,
                });
            }
        }

        lines
    }

    fn config(&self) -> &ShaderRegistryConfigResource {
        self.world
            .get_resource::<ShaderRegistryConfigResource>()
            .expect("shader registry config resource should exist")
    }

    fn config_mut(&mut self) -> &mut ShaderRegistryConfigResource {
        self.world
            .get_resource_mut::<ShaderRegistryConfigResource>()
            .expect("shader registry config resource should exist")
    }

    fn asset(&self, handle: ShaderHandle) -> Option<&ShaderAssetComponent> {
        self.world
            .get_component::<ShaderAssetComponent>(handle.entity())
    }

    fn asset_mut(&mut self, handle: ShaderHandle) -> Option<&mut ShaderAssetComponent> {
        self.world
            .get_component_mut::<ShaderAssetComponent>(handle.entity())
    }
}

fn normalize_roots<I, S>(roots: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut out = Vec::<String>::new();
    for root in roots {
        let root = root.into();
        let root = root.trim();
        if root.is_empty() {
            continue;
        }
        if !out.iter().any(|value| value == root) {
            out.push(root.to_string());
        }
    }
    if out.is_empty() {
        out.push(DEFAULT_SHADER_ASSET_ROOT.to_string());
    }
    out
}

fn discover_shader_files(root: &Path) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("wgsl"))
            {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn derive_shader_id_for_root(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).ok().unwrap_or(path);
    let no_ext = relative.with_extension("");
    normalize_shader_id(no_ext.to_string_lossy())
}

fn derive_shader_id_from_path(path: &Path) -> String {
    let no_ext = path.with_extension("");
    normalize_shader_id(no_ext.to_string_lossy())
}

fn normalize_shader_id(value: impl AsRef<str>) -> String {
    let raw = value.as_ref();
    let mut id = String::new();
    let mut pending_separator = false;

    for ch in raw.chars() {
        if ch == '/' || ch == '\\' || ch == '.' {
            pending_separator = !id.is_empty() && !id.ends_with('.');
            continue;
        }

        if pending_separator {
            id.push('.');
            pending_separator = false;
        }

        if ch.is_ascii_alphanumeric() {
            id.push(ch.to_ascii_lowercase());
        } else if !id.ends_with('_') && !id.ends_with('.') {
            id.push('_');
        }
    }

    id.trim_matches(|ch| ch == '_' || ch == '.').to_string()
}

fn shader_event_state_label(kind: ShaderRegistryEventKind) -> &'static str {
    match kind {
        ShaderRegistryEventKind::Discovered => "discovered",
        ShaderRegistryEventKind::Registered => "registered",
        ShaderRegistryEventKind::PathUpdated => "path_updated",
        ShaderRegistryEventKind::DuplicateId => "duplicate_id",
        ShaderRegistryEventKind::Reloaded => "reloaded",
        ShaderRegistryEventKind::SkippedEmpty => "skipped_empty",
        ShaderRegistryEventKind::Failed => "failed",
    }
}

#[cfg(test)]
mod tests {
    use super::{ShaderRegistryEventKind, ShaderRegistryResource, normalize_shader_id};
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> String {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{unique}"));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir.to_string_lossy().to_string()
    }

    #[test]
    fn normalize_id_uses_relative_style_segments() {
        let id = normalize_shader_id("assets/shaders/ui/panel_text.wgsl");
        assert_eq!(id, "assets.shaders.ui.panel_text.wgsl");
    }

    #[test]
    fn poll_updates_discovers_and_loads_shader_files() {
        let root = temp_dir("shader_registry_discovery");
        let file = Path::new(&root).join("ui_rect.wgsl");
        fs::write(
            &file,
            "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(); }",
        )
        .expect("shader should be written");

        let mut registry = ShaderRegistryResource::with_roots([root.clone()]);
        let lines = registry.poll_updates();
        assert!(!lines.is_empty());
        assert_eq!(registry.shader_count(), 1);
        assert!(registry.handle("ui_rect").is_some());
        let src = registry.source_or("ui_rect", "fallback");
        assert_ne!(src, "fallback");

        let events = registry.drain_events();
        assert!(
            events
                .iter()
                .any(|event| event.kind == ShaderRegistryEventKind::Discovered)
        );
        assert!(
            events
                .iter()
                .any(|event| event.kind == ShaderRegistryEventKind::Reloaded)
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn register_shader_returns_stable_handle_for_same_id() {
        let mut registry = ShaderRegistryResource::with_roots(["assets/shaders"]);
        let first = registry.register_shader("custom.main", "assets/shaders/custom_main.wgsl");
        let second = registry.register_shader("custom.main", "assets/shaders/custom_main_v2.wgsl");
        assert_eq!(first, second);
        assert_eq!(registry.shader_count(), 1);
    }
}
