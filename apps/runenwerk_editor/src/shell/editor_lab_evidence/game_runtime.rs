//! App-owned game-runtime descriptor fixtures for UI Designer compatibility evidence.

use super::{
    EditorLabDescriptorCompatibility, EditorLabReadOnlyFixtureBindingDescriptor,
    EditorLabValidatedIntentDescriptor,
};

pub const GAME_RUNTIME_TARGET_PROFILE: &str = "game.runtime";

pub fn descriptor_fixture_bindings() -> Vec<EditorLabReadOnlyFixtureBindingDescriptor> {
    vec![
        EditorLabReadOnlyFixtureBindingDescriptor::new(
            "fixture.game-runtime.safe-area",
            "binding.game-runtime.hud-data",
            GAME_RUNTIME_TARGET_PROFILE,
            EditorLabDescriptorCompatibility::Compatible,
            "read-only game.runtime safe-area fixture descriptor owned by app evidence",
        ),
        EditorLabReadOnlyFixtureBindingDescriptor::new(
            "fixture.game-runtime.input-modality",
            "binding.game-runtime.input-state",
            GAME_RUNTIME_TARGET_PROFILE,
            EditorLabDescriptorCompatibility::Compatible,
            "read-only game.runtime input-modality fixture descriptor owned by app evidence",
        ),
    ]
}

pub fn validated_intent_descriptors() -> Vec<EditorLabValidatedIntentDescriptor> {
    vec![
        EditorLabValidatedIntentDescriptor::new(
            "intent.game-runtime.open-hud-preview",
            GAME_RUNTIME_TARGET_PROFILE,
            "validated descriptor only; no game-runtime command is executed",
        ),
        EditorLabValidatedIntentDescriptor::new(
            "intent.game-runtime.inspect-safe-area",
            GAME_RUNTIME_TARGET_PROFILE,
            "validated descriptor only; no game-runtime command is executed",
        ),
    ]
}
