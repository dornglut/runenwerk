// Owner: Engine Render Shader Registry - Types and Resources
pub const DEFAULT_SHADER_ASSET_ROOT: &str = "assets/shaders";

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ShaderHandle(usize);

impl ShaderHandle {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Default)]
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
    assets: Vec<ShaderAssetComponent>,
    by_id: HashMap<String, usize>,
    config: ShaderRegistryConfigResource,
    events: Vec<ShaderRegistryEvent>,
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
        let assets = self.assets.clone();
        Self {
            by_id: build_shader_index(&assets),
            assets,
            config: self.config.clone(),
            events: Vec::new(),
        }
    }
}

impl Default for ShaderRegistryResource {
    fn default() -> Self {
        Self::new()
    }
}

