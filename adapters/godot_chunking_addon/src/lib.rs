use godot::prelude::*;

mod bridge;
mod chunk_streaming_node;

struct RunenwerkGodotExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RunenwerkGodotExtension {}
