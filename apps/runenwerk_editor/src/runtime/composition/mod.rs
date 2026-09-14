mod input;
mod input_adapter;
mod transition;

pub use input::dispatch_editor_target_input_system;
pub use input_adapter::EditorTargetInputRuntimeResource;
pub(crate) use input_adapter::translate_platform_event;
pub use transition::{
    EditorCompositionTransitionRuntimeResource, sync_editor_composition_transitions_system,
};
