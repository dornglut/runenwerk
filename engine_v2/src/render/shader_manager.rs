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
        let mut lines = vec![format!(
            "shader_watch={} (auto file polling)",
            if self.watch_enabled { "on" } else { "off" }
        )];
        for id in ALL_SHADERS {
            let asset = self.asset(id);
            let loaded = if asset.source.is_some() { "loaded" } else { "fallback" };
            if let Some(err) = &asset.last_error {
                lines.push(format!(
                    "shader {} rev={} {} error={}",
                    id.label(),
                    asset.revision,
                    loaded,
                    err
                ));
            } else {
                lines.push(format!(
                    "shader {} rev={} {} path={}",
                    id.label(),
                    asset.revision,
                    loaded,
                    id.path()
                ));
            }
        }
        lines
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
        if !self.watch_enabled && !self.force_reload {
            return Vec::new();
        }

        let force = self.force_reload;
        self.force_reload = false;
        let mut messages = Vec::new();

        for id in ALL_SHADERS {
            let path = Path::new(id.path());
            let modified = fs::metadata(path).ok().and_then(|meta| meta.modified().ok());
            let idx = Self::index(id);
            let asset = &mut self.assets[idx];
            let changed = force || modified != asset.modified;
            if !changed {
                continue;
            }

            match fs::read_to_string(path) {
                Ok(source) if !source.trim().is_empty() => {
                    asset.source = Some(source);
                    asset.modified = modified;
                    asset.revision = asset.revision.saturating_add(1);
                    asset.last_error = None;
                    messages.push(format!(
                        "shader {} reloaded rev={} ({})",
                        id.label(),
                        asset.revision,
                        id.path()
                    ));
                }
                Ok(_) => {
                    asset.last_error = Some("file is empty".to_string());
                    messages.push(format!(
                        "shader {} reload skipped: empty file ({})",
                        id.label(),
                        id.path()
                    ));
                }
                Err(err) => {
                    asset.last_error = Some(err.to_string());
                    messages.push(format!(
                        "shader {} reload failed: {} ({})",
                        id.label(),
                        err,
                        id.path()
                    ));
                }
            }
        }

        messages
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
