#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NetRole {
    Client,
    Server,
    Host,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct NetPluginConfig {
    pub enable_diagnostics: bool,
}
