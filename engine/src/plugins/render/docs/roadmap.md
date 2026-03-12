# Render Hard Cutover Roadmap

Hard cutover plan for replacing the hybrid render architecture with the new canonical render architecture.

This document assumes:

- no slow migration
- no long-term legacy compatibility layer
- no parallel old/new render stacks
- no normal-path dependency on custom executors
- no no-op builtin compute/compose behavior
- no duplicate ownership boundaries

The goal is a clean switch to the new architecture with a single public authoring model, a single graph model, a single resource model, and a single backend execution path.

---

## Cutover Goal

After this cutover, the render plugin must have:

- one canonical public API
- one canonical graph/planning model
- one canonical declarative resource model
- one canonical backend execution path
- one canonical inspection/debugging surface

Normal flows must render through builtin compiled execution, not example-owned custom executor registration.

---

## Canonical Target Structure

## File
`engine/src/plugins/render/`

```text
render/
├── mod.rs
├── plugin.rs
├── README.md
│
├── api/
│   ├── mod.rs
│   ├── flow.rs
│   ├── ids.rs
│   ├── passes.rs
│   ├── resources.rs
│   └── bindings.rs
│
├── backend/
│   ├── mod.rs
│   ├── device.rs
│   ├── formats.rs
│   ├── surface.rs
│   ├── wgpu_ctx.rs
│   ├── pipeline_cache.rs
│   ├── resource_allocator.rs
│   └── execution.rs
│
├── composition/
│   ├── mod.rs
│   ├── contribution.rs
│   ├── fragments.rs
│   ├── hot_reload.rs
│   ├── integration.rs
│   └── namespaces.rs
│
├── graph/
│   ├── mod.rs
│   ├── flow_graph.rs
│   ├── merge.rs
│   ├── pass_graph.rs
│   ├── planning.rs
│   ├── resource_graph.rs
│   └── validation.rs
│
├── inspect/
│   ├── mod.rs
│   ├── graph_dump.rs
│   ├── resource_inspector.rs
│   ├── texture_view.rs
│   └── timings.rs
│
├── params/
│   ├── mod.rs
│   ├── gpu_params.rs
│   └── gpu_value.rs
│
├── pipelines/
│   ├── mod.rs
│   ├── cache.rs
│   ├── keys.rs
│   └── specialization.rs
│
├── renderer/
│   ├── mod.rs
│   ├── extract.rs
│   ├── frame_bindings.rs
│   ├── graph_execution.rs
│   ├── prepare.rs
│   └── submit.rs
│
├── resource/
│   ├── mod.rs
│   ├── descriptors.rs
│   ├── import.rs
│   ├── lifetime.rs
│   ├── transient.rs
│   └── usages.rs
│
└── shader/
    ├── mod.rs
    ├── helpers.rs
    ├── hot_reload.rs
    ├── registry.rs
    └── types.rs
```

## Modules to Remove

These paths must not survive as first-class architecture after the cutover.

### Remove entirely

engine/src/plugins/render/frame_graph/

engine/src/plugins/render/resources/

engine/src/plugins/render/domain/

engine/src/plugins/render/debug/

### Move out of core render plugin

engine/src/plugins/render/sdf/

Best destination:

feature-owned plugin module

or example-owned render support module

not core render architecture

## Canonical Ownership After Cutover

### `api/`

Owns:

public flow builders

pass builders

public resource declarations

public IDs

public binding helpers

### `graph/`

Owns:

validated flow graph

pass graph

resource graph

planning

merge

dependency ordering

### `resource/`

Owns:

declarative resource descriptors

declarative usages

import model

lifetime model

transient model

### `params/`

Owns:

ToGpuValue

GpuParams

derive-facing conversion contracts

### `backend/`

Owns:

actual GPU resource realization

imported resource realization

transient resource realization

pipeline cache

compiled pass execution

### `renderer/`

Owns:

orchestration between ECS/frame lifecycle and backend execution

### `composition/`

Owns:

RenderFlowContribution

contribution integration

namespaces

data-driven fragments

hot reload

### `inspect/`

Owns:

graph inspection

resource inspection

texture inspection

timing inspection

## Hard Cutover Rules

### Rule 1

No no-op builtin rendering paths.

### Rule 2

No normal render flow path may depend on executor registry.

### Rule 3

No duplicated ownership boundaries remain after the delete pass.

### Rule 4

Examples must prove the public API is real.

### Rule 5

If a builtin pass kind is declared and not implemented, fail loudly.

### Rule 6

The canonical architecture is:

api/

graph/

resource/

params/

backend/

renderer/

composition/

inspect/

pipelines/

shader/

No other top-level render architecture domains should compete with these.

## Functional Acceptance Criteria

The cutover is successful only if all of the following are true.

### Rendering

compute_pass(...) executes through builtin backend execution

fullscreen_pass(...) executes through builtin backend execution

graphics_pass(...) executes through builtin backend execution once implemented

copy_pass(...) executes through builtin backend execution once implemented

present_pass(...) executes through builtin backend execution once implemented

builtin_ui_composite_pass(...) overlays correctly

### Authoring

examples use RenderFlow

multi-plugin examples use RenderFlowContribution

examples do not register custom executors for normal paths

examples do not mutate low-level registries directly in the normal path

### Planning

graph planning produces typed compiled pass descriptors

graph planning does not rely on pass-id-as-executor-id assumptions

### Resources

declarative resources live under resource/

runtime realization lives under backend/resource_allocator.rs

### Structure

no duplicate graph/resource/debug/domain ownership trees remain

## End-State Public API Direction

The hard cutover exists to support this public model.

### Target authoring example
```rust
app.add_input_bindings([
    (ACTION_TOGGLE_PAUSE, KeyCode::Space),
    (ACTION_SINGLE_STEP, KeyCode::Enter),
    (ACTION_SPEED_UP, KeyCode::PageUp),
    (ACTION_SPEED_DOWN, KeyCode::PageDown),
]);

app.add_render_flow(
    RenderFlow::new("game_of_life_sdf")
        .ecs_resource::<GameOfLifeSdfState>()
        .uniform_buffer::<GameOfLifeComputeParams>("gol.params")
        .uniform_buffer::<GameOfLifeComposeParams>("gol.compose_params")
        .storage_buffer::<GameOfLifeCell>("gol.cells")
        .sampled_texture("surface.color")
        .sampled_texture("ui.draw_list")
        .compute_pass("gol.compute")
            .shader("assets/shaders/game_of_life_sdf.wgsl")
            .uniform_state(GameOfLifeSdfState::compute_params)
            .writes("gol.cells")
            .workgroup_size(8, 8, 1)
            .finish()
        .fullscreen_pass("gol.compose")
            .shader("assets/shaders/game_of_life_sdf.wgsl")
            .uniform_state_with_surface(GameOfLifeSdfState::compose_params)
            .reads("gol.cells")
            .writes("surface.color")
            .clear_color([0.03, 0.045, 0.042, 1.0])
            .depends_on("gol.compute")
            .finish()
        .builtin_ui_composite_pass("ui.composite")
            .reads("ui.draw_list")
            .writes("surface.color")
            .depends_on("gol.compose")
            .finish(),
);
```

## Hard Cutover Roadmap

### Phase H0 — Freeze the new architecture as canonical
Goal

Declare the target architecture as the only valid architecture before code deletion begins.

Files

engine/src/plugins/render/README.md

engine/src/plugins/render/docs/README.md

engine/src/plugins/render/docs/index.md

engine/src/plugins/render/docs/roadmap.md

engine/docs/reference/plugins/render/render-target-architecture.md

Actions

state explicitly that the canonical render architecture is the target tree in this doc

state explicitly that frame_graph/, resources/, domain/, and debug/ are scheduled for removal

state that normal flows must not require custom executors

state that no-op builtin execution is forbidden

state that input bindings belong to input/app API, not render flow

Exit criteria

There is no ambiguity about which architecture is canonical.

### Phase H1 — Add the missing backend core
Goal

Create the missing backend modules that make the new architecture executable.

Add files

engine/src/plugins/render/backend/pipeline_cache.rs

engine/src/plugins/render/backend/resource_allocator.rs

engine/src/plugins/render/backend/execution.rs

File

engine/src/plugins/render/backend/pipeline_cache.rs

Owns

builtin compute pipeline cache

builtin fullscreen pipeline cache

builtin graphics pipeline cache

bridge/wrapper around existing pipeline cache if necessary

File

engine/src/plugins/render/backend/resource_allocator.rs

Owns

descriptor -> GPU resource realization

imported resource realization

persistent flow-owned resource realization

transient resource realization

runtime resource binding realization

File

engine/src/plugins/render/backend/execution.rs

Owns

compiled compute pass execution

compiled fullscreen pass execution

compiled graphics pass execution later

compiled copy/present execution later

builtin UI composite execution

Exit criteria

The backend has clear homes for allocation, pipelines, and execution.

### Phase H2 — Replace executor-label planning with compiled pass planning
Goal

Planning must compile to typed executable pass descriptors, not executor label assumptions.

Files

engine/src/plugins/render/graph/planning.rs

engine/src/plugins/render/graph/pass_graph.rs

engine/src/plugins/render/graph/flow_graph.rs

Remove

pass-id-as-executor-id assumptions

non-UI pass kinds mapped to executor labels

any reliance on external executor registration for builtin pass kinds

Add

Typed compiled pass descriptors such as:

CompiledComputePass

CompiledFullscreenPass

CompiledGraphicsPass

CompiledCopyPass

CompiledPresentPass

CompiledUiCompositePass

Compiled pass descriptors must include

pass ID

pass kind

shader/pipeline identity

resolved resource bindings

resolved read/write targets

clear/load/store metadata where relevant

workgroup size or draw mode

validated dependency order position

Exit criteria

Planning produces typed executable pass descriptors consumable directly by backend execution.

### Phase H3 — Wire renderer to builtin compiled execution
Goal

The renderer must execute compiled passes through backend execution.

Files

engine/src/plugins/render/renderer/graph_execution.rs

engine/src/plugins/render/renderer/mod.rs

engine/src/plugins/render/renderer/prepare.rs

engine/src/plugins/render/renderer/submit.rs

engine/src/plugins/render/backend/execution.rs

Actions

route compiled passes to backend execution

resolve uniform_state(...) and uniform_state_with_surface(...) through ECS resource lookup and param projection

upload generated GPU params through allocator/binding bridge

execute compute passes through builtin backend execution

execute fullscreen passes through builtin backend execution

execute builtin UI composite through builtin backend execution

Exit criteria

Public-API-only flows produce visible pixels without custom executors.

### Phase H4 — Delete no-op builtin execution
Goal

Remove silent-success behavior.

Files

engine/src/plugins/render/renderer/mod.rs

any files defining no-op builtin compute/compose behavior

Remove

BuiltinComputeNoopPassExecutor

BuiltinComposeNoopPassExecutor

any equivalent silent “do nothing” builtin execution path

any “skip if unresolved” behavior for builtin pass kinds

Replace with

actual builtin execution

or hard error if a builtin pass kind is declared but not implemented

Exit criteria

No validated builtin flow can silently render black because execution is a no-op.

### Phase H5 — Remove executor-registry dependency from the normal flow path
Goal

Normal flow execution must not depend on custom executor registry bindings.

Files

engine/src/plugins/render/renderer/graph_execution.rs

engine/src/plugins/render/composition/integration.rs

any executor-registry bridge modules still influencing the normal path

Actions

ensure standard pass kinds execute through compiled builtin execution only

keep custom executors only as advanced internal/escape-hatch path

make sure examples do not depend on executor registration

Exit criteria

Normal RenderFlow / RenderFlowContribution usage renders without executor registry involvement.

### Phase H6 — Collapse duplicate module trees
Goal

Remove parallel ownership boundaries completely.

H6.1 Remove frame_graph/
Path

engine/src/plugins/render/frame_graph/

Actions

move any still-valid declarative graph concepts into graph/

move any execution-ready per-pass concepts into backend/ or renderer/

delete frame_graph/

Exit criteria

graph/ is the only graph architecture owner.

H6.2 Remove resources/
Path

engine/src/plugins/render/resources/

Actions

move declarative resource concepts into resource/

move runtime allocation/binding concepts into backend/resource_allocator.rs or renderer/

delete resources/

Exit criteria

resource/ is the only declarative resource owner and backend/resource_allocator.rs is the runtime realization owner.

H6.3 Remove domain/
Path

engine/src/plugins/render/domain/

Actions

Redistribute remaining files:

pass-like declarative types -> graph/ or api/

pipeline-related types -> pipelines/

timing-related types -> inspect/

frame/view runtime helpers -> renderer/

broad re-export façade -> delete

Then delete domain/.

Exit criteria

There is no generic catch-all domain/ bucket left in render.

H6.4 Merge debug/ into inspect/
Path

engine/src/plugins/render/debug/

Actions

Move:

debug/graph_dump.rs -> inspect/graph_dump.rs

debug/texture_inspector.rs -> inspect/texture_view.rs or inspect/resource_inspector.rs

debug/timings.rs -> inspect/timings.rs

overlay-specific inspect/debug support -> inspect/

Then delete debug/.

Exit criteria

There is one canonical inspect/debug surface: inspect/.

H6.5 Move sdf/ out of core render plugin
Path

engine/src/plugins/render/sdf/

Actions

Move to:

a feature/plugin-owned module

or example-owned support module

Do not keep SDF as a core render architecture domain.

Exit criteria

Core render architecture is renderer-agnostic and SDF support is feature-owned.

### Phase H7 — Normalize mod.rs and top-level exports
Goal

Top-level module exports must reflect only the new architecture.

Files

engine/src/plugins/render/mod.rs

engine/src/plugins/render/plugin.rs

Export only

api

backend

composition

graph

inspect

params

pipelines

renderer

resource

shader

Do not export

removed legacy modules

compatibility aliases for removed trees

broad façade re-exports that hide ownership boundaries

Exit criteria

The top-level module surface matches the canonical architecture only.

### Phase H8 — Update all examples to the gold path
Goal

Examples must prove the new architecture is real.

Target examples

engine/examples/render_flow_fullscreen_minimal/

engine/examples/render_flow_postprocess_compositor/

engine/examples/render_flow_contributions/

engine/examples/render_flow_debug_inspect/

engine/examples/game_of_life_sdf/

legacy `engine/examples/sdf_renderer/` is removed from the canonical example set

Rules

no custom executor registration in normal examples

no raw registry mutation in normal examples

no legacy frame-graph APIs

use app.add_input_bindings(...) for key mappings

use RenderFlow / RenderFlowContribution only

Exit criteria

All canonical examples render through the new builtin execution path.

### Phase H9 — Update tests to only validate the new architecture
Goal

Tests must reinforce the cutover.

Target tests

engine/tests/render_gpu_params.rs

engine/tests/render_resource_model.rs

engine/tests/render_flow_graph.rs

engine/tests/render_flow_bindings.rs

engine/tests/render_flow_graphics.rs

engine/tests/render_flow_bridge.rs

engine/tests/render_flow_contributions.rs

engine/tests/render_resource_lifetime.rs

engine/tests/render_flow_advanced.rs

engine/tests/render_inspect.rs

engine/tests/render_flow_fragments.rs

Remove or rewrite

Any tests that validate:

executor-label planning assumptions

old frame-graph behavior

legacy resource tree assumptions

no-op builtin behavior

Exit criteria

CI validates only the new architecture.

### Phase H10 — Final delete pass
Goal

Physically remove all dead architecture after the new path is green.

Delete paths

engine/src/plugins/render/frame_graph/

engine/src/plugins/render/resources/

engine/src/plugins/render/domain/

engine/src/plugins/render/debug/

engine/src/plugins/render/sdf/ from core render plugin

Also remove

dead imports

dead re-exports

stale docs

stale example references

stale comments referencing the removed architecture

Exit criteria

The old architecture is gone, not merely ignored.

## File-by-File Add / Move / Delete Summary
Add

engine/src/plugins/render/backend/pipeline_cache.rs

engine/src/plugins/render/backend/resource_allocator.rs

engine/src/plugins/render/backend/execution.rs

Keep

engine/src/plugins/render/api/*

engine/src/plugins/render/backend/device.rs

engine/src/plugins/render/backend/formats.rs

engine/src/plugins/render/backend/surface.rs

engine/src/plugins/render/backend/wgpu_ctx.rs

engine/src/plugins/render/composition/*

engine/src/plugins/render/graph/*

engine/src/plugins/render/inspect/*

engine/src/plugins/render/params/*

engine/src/plugins/render/pipelines/*

engine/src/plugins/render/renderer/extract.rs

engine/src/plugins/render/renderer/frame_bindings.rs

engine/src/plugins/render/renderer/graph_execution.rs

engine/src/plugins/render/renderer/prepare.rs

engine/src/plugins/render/renderer/submit.rs

engine/src/plugins/render/resource/*

engine/src/plugins/render/shader/*

Reevaluate / merge

engine/src/plugins/render/renderer/render_flow.rs

engine/src/plugins/render/renderer/setup.rs

These should remain only if they still have a clear orchestration role after backend execution is introduced. Otherwise merge into:

renderer/graph_execution.rs

renderer/prepare.rs

plugin.rs

backend/execution.rs

Delete

engine/src/plugins/render/frame_graph/*

engine/src/plugins/render/resources/*

engine/src/plugins/render/domain/*

engine/src/plugins/render/debug/*

Move out of core render plugin

engine/src/plugins/render/sdf/*

## Verification Plan
Minimum required checks after cutover

cargo test -p engine --test render_flow_graph

cargo test -p engine --test render_flow_bindings

cargo test -p engine --test render_flow_bridge

cargo test -p engine --test render_flow_contributions

cargo test -p engine --test render_resource_model

cargo test -p engine --test render_resource_lifetime

cargo test -p engine --test render_inspect

cargo test -p engine --example render_flow_fullscreen_minimal

cargo test -p engine --example render_flow_postprocess_compositor

cargo test -p engine --example render_flow_contributions

cargo test -p engine --example game_of_life_sdf

Hard acceptance checks

At least these examples must produce visible rendering without custom executors:

render_flow_fullscreen_minimal

render_flow_postprocess_compositor

game_of_life_sdf

## Major Checkpoints
Checkpoint H-A

After Phase H3:

does a public-API-only flow render visible pixels?

Checkpoint H-B

After Phase H5:

does any normal flow still depend on executor registry?

Checkpoint H-C

After Phase H6:

are there any duplicate ownership boundaries left?

Checkpoint H-D

After Phase H8:

do examples prove the public API is real without legacy plumbing?

Checkpoint H-E

After Phase H10:

is the old architecture physically gone?

## Risks
Biggest risks

leaving frame_graph/ partially alive

keeping runtime resource logic split between resource/ and resources/

keeping no-op builtin execution around as silent fallback

allowing normal examples to keep custom executor plumbing

leaving domain/ as a catch-all bucket

Risk handling

The hard cutover intentionally chooses deletion over compatibility to reduce these risks.

## Final Recommendation

Follow this roadmap as a hard cut, not a migration.

Specifically:

add the missing backend core first

replace executor-label planning with compiled pass planning

wire builtin execution

remove no-op builtin paths

remove executor-registry dependency from the normal path

collapse duplicate trees aggressively

update examples and tests

delete the legacy render architecture completely

This is the cleanest way to make the new render architecture real.
