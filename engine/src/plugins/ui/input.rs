// Owner: Grotto Quest Engine - UI Plugin

use crate::plugins::input::domain::InputState;
use crate::plugins::scene::SceneManager;
use crate::plugins::scene::{SceneResource, domain::OverlaySubmitMessage};
use crate::plugins::time::domain::Time;
use crate::plugins::ui::domain::{
    UiButtonRuntimeClickEvent, UiInputField, UiInteraction, UiRenderShaderConfig, UiText,
    UiTransform, reload_console_template_if_changed,
};
use crate::runtime::{Res, ResMut};

use super::layout::*;

include!("legacy/input_core.rs");
include!("legacy/preupdate_input_systems.rs");
