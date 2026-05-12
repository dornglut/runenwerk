//! Runtime resources for the drawing app shell.

use crate::app::RunenwerkDrawApp;

#[derive(Debug, Default, ecs::Resource)]
pub struct DrawingHostResource {
    pub app: RunenwerkDrawApp,
}
