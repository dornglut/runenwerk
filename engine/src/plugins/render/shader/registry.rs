use super::helpers::{
    derive_shader_id_for_root, derive_shader_id_from_path, discover_shader_files, normalize_roots,
    normalize_shader_id, shader_event_state_label,
};
use super::*;

// Owner: Engine Render Shader Registry - Core Registry Implementation
impl ShaderRegistryResource {
    pub fn new() -> Self {
        Self::with_roots([DEFAULT_SHADER_ASSET_ROOT])
    }

    pub fn with_roots<I, S>(roots: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            assets: Vec::new(),
            by_id: HashMap::new(),
            config: ShaderRegistryConfigResource {
                roots: normalize_roots(roots),
                ..ShaderRegistryConfigResource::default()
            },
            events: Vec::new(),
        }
    }

    pub fn revision(&self) -> u64 {
        self.config.revision
    }

    pub fn shader_count(&self) -> usize {
        self.assets.len()
    }

    pub fn roots(&self) -> &[String] {
        &self.config.roots
    }

    pub fn set_roots<I, S>(&mut self, roots: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config.roots = normalize_roots(roots);
        self.config.force_reload = true;
    }

    pub fn add_root(&mut self, root: impl Into<String>) {
        let root = root.into();
        let root = root.trim();
        if root.is_empty() {
            return;
        }
        if self.config.roots.iter().any(|value| value == root) {
            return;
        }
        self.config.roots.push(root.to_string());
        self.config.force_reload = true;
    }

    pub fn watch_enabled(&self) -> bool {
        self.config.watch_enabled
    }

    pub fn set_watch_enabled(&mut self, enabled: bool) {
        self.config.watch_enabled = enabled;
    }

    pub fn request_reload(&mut self) {
        self.config.force_reload = true;
    }

    pub fn register_shader(&mut self, path: impl Into<String>) -> ShaderHandle {
        let path = path.into();
        let id = derive_shader_id_from_path(Path::new(&path));
        self.register_shader_with_id(id, path)
    }

    pub fn register_shader_with_id(
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
            if let Some(asset) = self.asset_mut(existing)
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
                self.config.force_reload = true;
                self.events.push(event);
            }
            return existing;
        }

        let handle = ShaderHandle(self.assets.len());
        self.assets.push(ShaderAssetComponent {
            id: id.clone(),
            path: path.clone(),
            ..ShaderAssetComponent::default()
        });
        self.by_id.insert(id.clone(), handle.index());
        self.config.force_reload = true;
        self.events.push(ShaderRegistryEvent {
            kind: ShaderRegistryEventKind::Registered,
            id,
            path,
            revision: 0,
            error: None,
            details: None,
        });
        handle
    }

    pub fn handle(&self, id: impl AsRef<str>) -> Option<ShaderHandle> {
        let id = normalize_shader_id(id.as_ref());
        self.by_id.get(&id).copied().map(ShaderHandle)
    }

    pub fn source_or<'a>(&'a self, id: &str, fallback: &'a str) -> &'a str {
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

    pub fn revision_for(&self, id: &str) -> u64 {
        self.handle(id)
            .map(|handle| self.revision_for_handle(handle))
            .unwrap_or(0)
    }

    pub fn revision_for_handle(&self, handle: ShaderHandle) -> u64 {
        self.asset(handle).map(|asset| asset.revision).unwrap_or(0)
    }

    pub fn statuses(&self) -> Vec<ShaderStatus> {
        let mut statuses: Vec<ShaderStatus> = self
            .assets
            .iter()
            .enumerate()
            .map(|(index, asset)| {
                let handle = ShaderHandle(index);
                ShaderStatus {
                    handle,
                    id: asset.id.clone(),
                    path: asset.path.clone(),
                    revision: asset.revision,
                    loaded: asset.source.is_some(),
                    modified: asset.modified,
                    last_error: asset.last_error.clone(),
                }
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
        &self.events
    }

    pub fn drain_events(&mut self) -> Vec<ShaderRegistryEvent> {
        std::mem::take(&mut self.events)
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
        self.events.clear();
    }

    pub fn poll_updates(&mut self) -> Vec<String> {
        let watch_enabled = self.watch_enabled();
        let force_reload = self.config.force_reload;
        if !should_poll(watch_enabled, force_reload) {
            return Vec::new();
        }
        self.config.force_reload = false;

        let mut lines = self.discover_from_roots();
        let mut changed = false;

        let handles: Vec<_> = (0..self.assets.len()).map(ShaderHandle).collect();
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
                    self.events.push(ShaderRegistryEvent {
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
                    self.events.push(ShaderRegistryEvent {
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
                    self.events.push(ShaderRegistryEvent {
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
            self.config.revision = self.config.revision.saturating_add(1);
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
                        self.events.push(ShaderRegistryEvent {
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

                let handle = ShaderHandle(self.assets.len());
                self.assets.push(ShaderAssetComponent {
                    id: id.clone(),
                    path: path_string.clone(),
                    ..ShaderAssetComponent::default()
                });
                self.by_id.insert(id.clone(), handle.index());
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
                self.events.push(ShaderRegistryEvent {
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

    fn asset(&self, handle: ShaderHandle) -> Option<&ShaderAssetComponent> {
        self.assets.get(handle.index())
    }

    fn asset_mut(&mut self, handle: ShaderHandle) -> Option<&mut ShaderAssetComponent> {
        self.assets.get_mut(handle.index())
    }
}
