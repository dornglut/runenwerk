// Owner: Grotto Quest Engine - UI Plugin

use crate::plugins::input::domain::InputState;
use crate::plugins::scene::SceneResource;
use crate::plugins::ui::domain::{
    UiEditorNode, UiNode, UiTransform, reload_console_template_if_changed,
    save_console_template_to_disk,
};
use crate::runtime::{Res, ResMut};

use super::layout::*;

include!("legacy/editor_core.rs");
