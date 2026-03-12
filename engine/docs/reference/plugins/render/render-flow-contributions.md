# Render Flow Contributions

## Purpose

`RenderFlowContribution` lets feature plugins author namespaced subflows that merge into a base `RenderFlow`.

## Pattern

1. App declares a base flow (`surface.color`, shared imports).
2. Features declare contributions (resources + passes).
3. App registers contributions with `App::add_render_flow_contribution(...)`.
4. Merge/namespace validation runs before bridging to execution.

## Rules

- Contribution namespace must be unique.
- Pass/resource IDs must be globally unique after merge.
- Cross-contribution dependencies must point to known passes.
- Use namespaced IDs (for example `boids.simulate`, `post.tonemap`).

## Example Shape

```rust
let boids = RenderFlowContribution::new("boids")
    .storage_buffer::<BoidInstance>("boids.instances")
    .compute_pass("boids.simulate")
    .writes("boids.instances")
    .finish();
```

See runnable example:

- `engine/examples/render_flow_contributions/main.rs`
