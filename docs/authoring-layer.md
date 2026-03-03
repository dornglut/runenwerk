# Authoring Layer

## Goal

Turn the current data-driven examples into a consistent engine authoring pipeline:

1. humans edit stable asset/config files
2. the engine parses and validates them into typed authoring models
3. the engine compiles those models into runtime ECS/resources/render registrations
4. hot reload updates the runtime with clear diagnostics

This keeps authored data separate from runtime data.

## Current Authoring Already In The Repo

### Scene templates

Current authored inputs:

- `engine/examples/scene_manager_ui/assets/scenes/*.ron`
- `engine/examples/scene_manager_ui/assets/components/*.ron`

Current behavior:

- scene files describe buttons, labels, actions, and component-template references
- runtime scene flow builds overlay/world behavior from those templates
- hot reload is already part of the intended workflow

This is already authoring.

### SDF renderer config

Current authored inputs:

- `engine/examples/sdf_renderer/assets/sdf_params.ron`
- `engine/examples/sdf_renderer/assets/input_bindings.ron`
- `engine/examples/sdf_renderer/assets/render_graph.ron`

Current behavior:

- params config defines camera/world/control defaults
- input config defines action bindings
- render graph config defines feature-owned resources, pipelines, executors, and passes
- runtime parses these into typed config structs and engine registrations

This is also already authoring.

## Current Problem

The repo has authoring data, but not yet a clear shared authoring layer.

Today, each feature mostly does this itself:

- find files
- deserialize `.ron`
- apply fallback defaults
- validate ad hoc
- convert directly into runtime state

That works for examples, but it does not scale well for:

- better validation errors
- shared hot reload behavior
- editor tooling
- reusable asset pipelines
- keeping runtime ECS models separate from authored schemas

## Recommended Model

Use a strict split:

### Authored model

Stable, human-edited types.

Examples:

- `SceneTemplateAsset`
- `UiComponentTemplateAsset`
- `SdfParamsAsset`
- `InputBindingsAsset`
- `RenderGraphAsset`

These are designed for readability, validation, and tooling.

### Compiled runtime model

Typed, validated, engine-usable data.

Examples:

- `CompiledSceneTemplate`
- `CompiledUiTemplateSet`
- `CompiledInputBindings`
- `RenderFeatureGraphSpec`
- feature-specific runtime init structs

These are designed for engine execution, not for direct editing.

### Live runtime state

ECS/resources/registries derived from compiled models.

Examples:

- spawned UI entities/components
- scene transition state
- input maps stored in runtime resources
- render graph registry entries
- feature-owned render frame resources

## Pipeline To Build

Every authored feature should follow the same pipeline:

1. Resolve asset path or asset id.
2. Load source bytes.
3. Deserialize into an authored schema type.
4. Validate with source-aware diagnostics.
5. Compile into a runtime-ready typed model.
6. Apply to the running engine.
7. On file change, repeat and swap atomically if validation succeeds.

That means "load" and "apply" should not be the same step.

## Missing Abstractions

### 1. Shared authoring asset loader

Add a small engine-level loader service instead of per-example file IO.

Suggested responsibilities:

- path resolution
- file loading
- modified-time tracking
- `ron` decode helpers
- structured diagnostics

Suggested shape:

- `engine/src/authoring/loader.rs`
- `engine/src/authoring/source.rs`
- `engine/src/authoring/diagnostics.rs`

### 1a. Stable asset identity

Do not rely on raw path strings alone as runtime identity.

Suggested model:

- `AssetId`: stable logical identity for authored assets
- normalized source path: where the asset came from on disk
- content/dependency revision: what compiled generation is currently active

This allows diagnostics, caches, reload, and editor tooling to refer to assets without coupling them to runtime entity ids or transient file system details.

### 1b. Dependency graph

The loader/reload layer should track authored dependencies explicitly.

Examples already present in the repo:

- scene files depend on component-template files
- render graph assets depend on shader paths, pipeline ids, executor ids, and logical resources

Required behavior:

- record direct dependencies during load/compile
- maintain reverse-dependency fanout for reload invalidation
- detect cycles and report them with source-aware diagnostics
- recompile dependents when a shared dependency changes

Without this, hot reload will keep stale compiled outputs alive.

### 2. Feature-local schema modules

Each feature/plugin should own its authored schema types.

Suggested pattern:

- `scene/domain/authoring/*.rs`
- `render/domain/authoring/*.rs`
- `ui/domain/authoring/*.rs`

The schema type should not be the same type as the runtime type unless that is genuinely correct.

### 3. Explicit compile step

Each authored type should compile into a runtime-facing model.

Examples:

- `SceneTemplateAsset -> CompiledSceneTemplate`
- `RenderGraphAsset -> RenderFeatureGraphSpec`
- `InputBindingsAsset -> CompiledInputBindings`

Suggested trait:

```rust
pub trait CompileAuthoring<Output> {
    type Error;

    fn compile(&self, ctx: &AuthoringCompileContext) -> Result<Output, Self::Error>;
}
```

### 4. Hot reload coordinator

Hot reload should be centralized enough to behave consistently.

Suggested responsibilities:

- detect changed assets
- reload and revalidate
- only apply successful updates
- retain last-known-good compiled state
- emit visible diagnostics for failures

### 4a. Atomic reload bundles

Reload should apply compatible compiled artifacts as a bundle, not as unrelated per-file side effects.

Examples:

- scene asset + referenced component templates
- render graph asset + referenced shader/pipeline/executor declarations

Recommended behavior:

- compute a reload set from the dependency graph
- compile the full affected set first
- if any member fails, keep the previous live bundle
- if all succeed, swap the bundle as one runtime generation

This avoids mixed-generation runtime state during hot reload.

### 5. Runtime apply layer

Compilers should not mutate the engine directly.

Prefer:

- compile authored data first
- apply compiled data second

That keeps reload safer and easier to test.

### 6. Structured diagnostics contract

Authoring failures should produce a shared diagnostic shape rather than ad hoc log strings.

Suggested fields:

- severity
- machine-readable code
- asset id
- source path
- source span if available
- import/dependency chain
- human-readable message
- optional hint

This gives the same diagnostic payload to logs, overlay UI, tests, and future editor tools.

### 7. Schema versioning

Authored files should support explicit schema versioning once formats become stable enough to evolve.

Suggested policy:

- top-level `version` field in authored roots
- migration or compatibility handling at load/compile boundaries
- clear diagnostic when an unsupported version is encountered

This is especially important for scene templates and render graph assets.

### 8. Testing strategy

Authoring code should have dedicated tests, not only runtime integration coverage.

Recommended test layers:

- deserialize tests
- validation failure tests
- compile tests
- dependency invalidation tests
- hot reload integration tests

This is where authoring-specific regressions should be caught.

## What This Means For Current Features

### Scene template flow

Authored:

- scene `.ron`
- reusable component-template `.ron`

Compile into:

- a fully resolved scene template with actions, labels, and component refs resolved

Apply into:

- overlay/world ECS entities
- scene transition hooks
- typed emitted events

Recommended next cleanup:

- move scene-template schema and validation into a dedicated `authoring` module under `scene`
- keep runtime scene flow code focused on applying compiled templates, not parsing files
- treat each scene plus referenced component templates as one atomic reload unit
- emit diagnostics that include both the scene asset and the component-template include chain

### SDF example

Authored:

- params
- input bindings
- render graph declarations

Compile into:

- `SdfParamsCompiled`
- `CompiledInputBindings`
- `RenderFeatureGraphSpec`

Apply into:

- render resources
- input registry/resource state
- render graph and executor registry state

Recommended next cleanup:

- move config structs and loading helpers out of the example entrypoint
- keep `main.rs` focused on plugin wiring
- track dependencies from render graph assets to shader and executor references
- reload render authoring as an atomic bundle so graph/resource/executor state stays consistent

## What Not To Do

- Do not make raw runtime ECS component structs your editing schema by default.
- Do not let examples keep owning their own bespoke load/validate/reload logic forever.
- Do not couple authored asset identity to runtime entity ids.
- Do not require reflection in the ECS core just to support typed authoring files.

## Reflection Guidance

Reflection is useful here, but as a layer above the ECS core.

Good uses:

- editor inspectors
- schema-driven tooling
- generic property editing
- scene/template previews

Bad use:

- making ECS storage/query internals depend on reflection

Recommended split:

- `ecs_v2`: typed runtime core
- `engine/authoring`: schemas, loaders, compilers, reload
- optional reflection/tooling layer on top

## Dependency And Reload Rules

Use the same rules for every authored domain:

1. Every authored root declares or discovers its direct dependencies.
2. The engine records a reverse-dependency graph.
3. Reload invalidation is graph-driven, not only file-driven.
4. Compilation produces a generation-tagged compiled artifact.
5. Application swaps only fully compiled bundles into runtime state.
6. Failed reloads keep the last-known-good runtime generation active.

These rules are the minimum bar for reliable hot reload.

## First Implementation Phases

### Phase 1: Standardize the pipeline

- add `engine/src/authoring/`
- add shared `ron` load + diagnostics helpers
- define `CompileAuthoring` trait
- define `AssetId`, dependency, and diagnostic core types
- move SDF config loading behind the shared loader

### Phase 2: Scene authoring cleanup

- extract scene-template schema types from runtime flow code
- compile scene assets into resolved templates before runtime apply
- make template reload use last-known-good compiled outputs
- add dependency invalidation for shared component templates

### Phase 3: Render authoring cleanup

- define authored render graph schema separate from runtime graph spec
- validate resource/pass/pipeline/executor references before registration
- keep example/plugin setup code as a thin apply layer
- add dependency tracking for shader/config inputs used by render graph authoring

### Phase 4: Tooling

- add source-aware diagnostics in overlay/editor/log output
- add asset validation commands/tests
- add preview/inspect hooks for authored assets

## Success Criteria

The authoring layer is in good shape when:

- examples no longer own bespoke file parsing in `main.rs`
- authored schemas are clearly distinct from runtime ECS/resources
- hot reload applies only validated compiled outputs
- failures show actionable diagnostics with asset paths
- dependency changes invalidate and recompile all affected authored roots
- runtime state never observes mixed-generation authoring bundles
- adding a new authored feature follows the same pipeline as scene and render config
