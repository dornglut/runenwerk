use crate::utils::{
    ReloadStatusPayload, file_modified, should_poll, should_reload, watch_status_line,
};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ShaderId {
    UiRect,
    WorldComputeBasic,
    WorldComputeHighContrast,
    WorldComposeFullscreen,
}

impl ShaderId {
    pub fn label(self) -> &'static str {
        match self {
            Self::UiRect => "ui_rect",
            Self::WorldComputeBasic => "world_compute_basic",
            Self::WorldComputeHighContrast => "world_compute_high_contrast",
            Self::WorldComposeFullscreen => "world_compose_fullscreen",
        }
    }

    pub fn path(self) -> &'static str {
        match self {
            Self::UiRect => "assets/shaders/ui_rect.wgsl",
            Self::WorldComputeBasic => "assets/shaders/world_compute_basic.wgsl",
            Self::WorldComputeHighContrast => "assets/shaders/world_compute_high_contrast.wgsl",
            Self::WorldComposeFullscreen => "assets/shaders/world_compose_fullscreen.wgsl",
        }
    }
}

const ALL_SHADERS: [ShaderId; 4] = [
    ShaderId::UiRect,
    ShaderId::WorldComputeBasic,
    ShaderId::WorldComputeHighContrast,
    ShaderId::WorldComposeFullscreen,
];

#[derive(Debug, Clone, Default)]
struct ShaderAsset {
    source: Option<String>,
    modified: Option<SystemTime>,
    revision: u64,
    last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ShaderStatus {
    pub id: ShaderId,
    pub path: &'static str,
    pub revision: u64,
    pub loaded: bool,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ShaderManager {
    assets: [ShaderAsset; 4],
    watch_enabled: bool,
    force_reload: bool,
}

impl Default for ShaderManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderManager {
    pub fn new() -> Self {
        let mut manager = Self {
            assets: std::array::from_fn(|_| ShaderAsset::default()),
            watch_enabled: true,
            force_reload: true,
        };
        let _ = manager.poll_updates();
        manager
    }

    pub fn watch_enabled(&self) -> bool {
        self.watch_enabled
    }

    pub fn set_watch_enabled(&mut self, enabled: bool) {
        self.watch_enabled = enabled;
    }

    pub fn request_reload(&mut self) {
        self.force_reload = true;
    }

    pub fn source_or<'a>(&'a self, id: ShaderId, fallback: &'a str) -> &'a str {
        self.asset(id)
            .source
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(fallback)
    }

    pub fn revision(&self, id: ShaderId) -> u64 {
        self.asset(id).revision
    }

    pub fn status_lines(&self) -> Vec<String> {
        let mut lines = vec![watch_status_line(
            "shader",
            self.watch_enabled,
            "assets/shaders/*.wgsl",
        )];
        lines.extend(
            self.status_payloads()
                .into_iter()
                .map(|payload| payload.line()),
        );
        lines
    }

    pub fn status_payloads(&self) -> Vec<ReloadStatusPayload> {
        ALL_SHADERS
            .iter()
            .map(|id| {
                let asset = self.asset(*id);
                let state = if asset.last_error.is_some() {
                    "error"
                } else if asset.source.is_some() {
                    "loaded"
                } else {
                    "fallback"
                };
                ReloadStatusPayload::new(
                    "shader",
                    id.label(),
                    state,
                    id.path(),
                    asset.revision,
                    self.watch_enabled,
                    asset.modified,
                    asset.last_error.clone(),
                    None,
                )
            })
            .collect()
    }

    pub fn statuses(&self) -> Vec<ShaderStatus> {
        ALL_SHADERS
            .iter()
            .map(|id| {
                let asset = self.asset(*id);
                ShaderStatus {
                    id: *id,
                    path: id.path(),
                    revision: asset.revision,
                    loaded: asset.source.is_some(),
                    last_error: asset.last_error.clone(),
                }
            })
            .collect()
    }

    pub fn poll_updates(&mut self) -> Vec<String> {
        if !should_poll(self.watch_enabled, self.force_reload) {
            return Vec::new();
        }

        let force = self.force_reload;
        self.force_reload = false;
        let mut payloads = Vec::new();

        for id in ALL_SHADERS {
            let path = Path::new(id.path());
            let modified = file_modified(path);
            let idx = Self::index(id);
            let asset = &mut self.assets[idx];
            if !should_reload(self.watch_enabled, force, asset.modified, modified) {
                continue;
            }

            match fs::read_to_string(path) {
                Ok(source) if !source.trim().is_empty() => {
                    asset.source = Some(source);
                    asset.modified = modified;
                    asset.revision = asset.revision.saturating_add(1);
                    asset.last_error = None;
                    payloads.push(ReloadStatusPayload::new(
                        "shader",
                        id.label(),
                        "reloaded",
                        id.path(),
                        asset.revision,
                        self.watch_enabled,
                        asset.modified,
                        None,
                        None,
                    ));
                }
                Ok(_) => {
                    asset.last_error = Some("file is empty".to_string());
                    asset.modified = modified;
                    payloads.push(ReloadStatusPayload::new(
                        "shader",
                        id.label(),
                        "skipped_empty",
                        id.path(),
                        asset.revision,
                        self.watch_enabled,
                        asset.modified,
                        asset.last_error.clone(),
                        None,
                    ));
                }
                Err(err) => {
                    asset.last_error = Some(err.to_string());
                    asset.modified = modified;
                    payloads.push(ReloadStatusPayload::new(
                        "shader",
                        id.label(),
                        "failed",
                        id.path(),
                        asset.revision,
                        self.watch_enabled,
                        asset.modified,
                        Some(err.to_string()),
                        None,
                    ));
                }
            }
        }

        payloads.into_iter().map(|payload| payload.line()).collect()
    }

    fn asset(&self, id: ShaderId) -> &ShaderAsset {
        &self.assets[Self::index(id)]
    }

    fn index(id: ShaderId) -> usize {
        match id {
            ShaderId::UiRect => 0,
            ShaderId::WorldComputeBasic => 1,
            ShaderId::WorldComputeHighContrast => 2,
            ShaderId::WorldComposeFullscreen => 3,
        }
    }
}
