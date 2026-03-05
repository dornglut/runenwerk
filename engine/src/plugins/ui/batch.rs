// Owner: Grotto Quest Engine - UI Plugin

use crate::plugins::input::domain::InputState;
use crate::plugins::scene::domain::WorldDebugPosition;
use crate::plugins::scene::{SceneManager, SceneResource};
use crate::plugins::ui::domain::{
    ConsoleUiRuntimeState, UiBatchCmd, UiButton, UiInputField, UiInteraction, UiStyle, UiText,
    UiTransform, UiWorldHudStats,
};
use crate::runtime::{Res, ResMut};

use super::layout::*;

include!("legacy/batch_core.rs");
include!("legacy/layout_batch_systems.rs");
