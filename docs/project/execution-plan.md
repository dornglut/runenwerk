# Execution Plan

## Purpose

Track near-term implementation steps and architecture decisions for engine/runtime systems.

## Current Focus

1. Stabilize plugin-level runtime data flow.
2. Remove hardcoded cross-system wiring where dynamic behavior is required.
3. Make hot-reload behavior predictable, observable, and testable.

## Proposed Architecture Update: ECS Resources + Event Core for Hot Reload

Suggestion:
Use the ECS resource system as the canonical state for hot-reloadable subsystems, and add a small event core element to propagate reload intents/results across plugins.

### Rationale

1. Centralized mutable state in resources avoids hidden plugin-local state.
2. Event-driven propagation decouples file watchers/reload triggers from consumers.
3. Enables deterministic testing of reload flows (emit event -> process -> assert state).
4. Scales across input/UI/scene/render without introducing direct plugin dependencies.

### Plan

1. Add shared ECS resources:
   - `HotReloadRegistryResource` (tracked assets/configs, revision counters, timestamps).
   - `HotReloadStatusResource` (latest outcome per key: `reloaded`, `fallback`, `failed`).
2. Add a lightweight event core:
   - `EngineEventQueue<HotReloadEvent>` resource.
   - Initial events:
     - `HotReloadRequested { key, source }`
     - `HotReloadCompleted { key, revision }`
     - `HotReloadFailed { key, error }`
3. Update plugin systems to publish/consume events:
   - Producers: file-change detection and manual reload commands.
   - Consumers: scene/ui/render/input systems that rebuild from changed data.
4. Standardize ordering in scheduler:
   - detect changes -> enqueue events -> apply reload -> publish result -> render/overlay status.
5. Add tests:
   - unit tests for event queue/resource transitions.
   - integration tests for one end-to-end reload path (change detected to visible runtime update).

### Goal API / Usage Example

Target shape:

1. Systems publish data-only remap or reload intents into a resource/event queue.
2. A dedicated apply system consumes those intents and mutates the authoritative resource.
3. Consumers react to status/result events instead of direct method calls.

Example (input remap payload flow):

```rust
// producer system
event_queue.push(InputBindingChange::MapKey {
    action: "world.move_left".to_string(),
    key: KeyCode::ArrowLeft,
});

// applier system
while let Some(change) = event_queue.pop() {
    input_state.apply_binding_change(change);
}
```

Same pattern should be used for hot-reload:

1. publish `HotReloadRequested`.
2. applier loads/parses and updates resource state.
3. publish `HotReloadCompleted` or `HotReloadFailed`.

### Acceptance Criteria

1. Hot-reload state is queryable from ECS resources only (no hidden singleton/plugin-local authority).
2. Reload consumers react to events instead of direct caller coupling.
3. Reload success/failure is surfaced consistently in runtime status and logs.
4. At least one integration test validates full event-driven reload flow.
