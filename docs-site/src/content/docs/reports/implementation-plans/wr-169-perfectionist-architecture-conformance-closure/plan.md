---
title: Perfectionist Architecture Conformance Closure Implementation Plan
status: active
type: implementation-plan
wr: WR-169
milestone: PM-UI-PROGRAM-ARCH-011
---

# Implementation Plan

PM-UI-PROGRAM-ARCH-011 is the correction milestone for the final UiProgram
architecture truth claim. Its purpose is not to repeat prior proof-slice
closeouts. It must make the concrete code truth match the accepted architecture
target, then certify that truth with a zero-finding verifier result.

Executable authority lives in `plan.contract.yaml`. The sidecar is now
`status: accepted`; product implementation may run only through the locked
execution harness, exact output scopes, declared validation commands, and
resolver-backed evidence records in that sidecar.

## Current State

- Track: `PT-UI-PROGRAM-ARCHITECTURE`
- Milestone: `PM-UI-PROGRAM-ARCH-011`
- Roadmap row: `WR-169`
- Current WR state: implementation candidate after WR-168 completion.
- Current implementation authority: accepted `plan.contract.yaml` sidecar only.
- Current truth verifier: `ui_program_architecture_conformance`
- Current truth result: failing until all verifier findings are repaired.

## Required Outcome

The milestone may close only when all of the following are true:

- `task truth:verify -- --track PT-UI-PROGRAM-ARCHITECTURE --claim ui-program-architecture-implementation` passes.
- `task truth:verify -- --track PT-UI-PROGRAM-ARCHITECTURE --claim ui-program-perfectionist-conformance` passes.
- `task truth:audit -- --track PT-UI-PROGRAM-ARCHITECTURE` reports no blocked verifier findings for UiProgram architecture claims.
- `task production:validate`, `task roadmap:validate`, `task docs:validate`, and `task planning:validate` pass.
- The closeout records concrete code, test, evidence, migration, diagnostics, source-map, and reproducibility subjects.
- Known gaps, known risks, and truth drift are empty.

## Repair Slices

The repair work must be split into accepted implementation authority before
product code is touched. A future accepted sidecar may cover one or more slices,
but only if it names exact files, evidence records, validations, and rollback
rules for each slice.

### Slice A: UiProgram Graph Semantics

Repair findings around generic graph string bags.

Required contracts:

- `ControlGraphNode`
- `LayoutGraphNode`
- `StateRequirement`
- `StyleRule`
- `InteractionHandler`
- `BindingEdge`
- `VisualOperator`
- `AccessibilityNode`
- `InspectionEntry`

Likely subject files:

- `domain/ui/ui_program/src/graphs/mod.rs`
- `domain/ui/ui_program/src/program.rs`
- focused tests named by the accepted sidecar

### Slice B: Route, Event, Schema, And Stable IDs

Repair missing route/event/schema ownership findings.

Required contracts:

- `UiEventPacket`
- `RouteId`
- `RouteSchemaVersion`
- `RouteCapability`
- `UiSchemaValue`
- event payload schema behavior
- source-map and diagnostics attachment contracts

Likely subject files:

- `domain/ui/ui_program/src/events/mod.rs`
- `domain/ui/ui_program/src/events/packet.rs`
- `domain/ui/ui_program/src/events/payload.rs`
- `domain/ui/ui_program/src/events/route.rs`
- `domain/ui/ui_schema/src/value.rs`
- `domain/ui/ui_schema/src/schema.rs`

### Slice C: Compiler And Runtime Artifact Semantics

Repair compiler pass-through and artifact table findings.

Required contracts:

- `PackageResolution`
- `CapabilityCheck`
- `ArtifactCacheKey`
- `CompiledSourceMap`
- `UiRuntimeArtifactManifest`
- `UiRuntimeArtifactTables`
- `StateTable`
- `BindingSnapshotTable`
- `CollectionDiffPlan`
- `TextLayoutRequestTable`

Likely subject files:

- `domain/ui/ui_compiler/src/lib.rs`
- `domain/ui/ui_artifacts/src/lib.rs`
- `domain/ui/ui_program/src/program.rs`

### Slice D: Evaluator, State, Binding, Host, And Testing Contracts

Repair evaluator pass-through and missing owner contract findings.

Required contracts:

- `InputEvaluationPass`
- `LayoutEvaluationPass`
- `StateEvaluationPass`
- `BindingEvaluationPass`
- `InteractionEvaluationPass`
- `VisualEvaluationPass`
- `AccessibilityEvaluationPass`
- `InspectionEvaluationPass`
- `UiOutput`
- state contracts: transient, preview, committed, focus, hover, drag, animation, host-fed, package-owned
- binding contracts: snapshots, dirty propagation, collection diffs, host data, authorization
- host contracts: editor, game, world-space, headless
- testing contracts: headless fixture, source-map assertion, diagnostic assertion, reproducibility assertion

Likely subject files:

- `domain/ui/ui_evaluator/src/lib.rs`
- `domain/ui/ui_state/src/lib.rs`
- `domain/ui/ui_binding/src/lib.rs`
- `domain/ui/ui_hosts/src/lib.rs`
- `domain/ui/ui_testing/src/lib.rs`

### Slice E: Final Owner Map Reconciliation

Repair missing final owner map findings. This slice must decide whether each
owner is implemented now or explicitly reconciled by accepted architecture
policy.

Owners:

- `domain/ui/ui_controls`
- `domain/ui/ui_accessibility`
- `domain/ui/ui_geometry`

Rules:

- No placeholder folders.
- New crates require exact `new: .../Cargo.toml` scope and explicit crate creation authority.
- If an owner is reconciled instead of created, the accepted sidecar must name the design/report file that carries that reconciliation and the verifier must inspect it.

### Slice F: Evidence And Truth Certification

Repair weak or self-referential architecture evidence.

Evidence records must name concrete non-self subject paths for:

- runtime tests
- fixtures
- diagnostics
- source maps
- artifacts
- migration
- reproducibility
- visual/render boundary

Likely subject paths:

- `docs-site/src/content/docs/reports/execution-evidence/pt-ui-program-architecture/pm-ui-program-arch-011/*.yaml`
- truth certificates under `docs-site/src/content/docs/reports/truth-certificates/pt-ui-program-architecture/`
- final closeout under `docs-site/src/content/docs/reports/closeouts/pm-ui-program-arch-011-perfectionist-architecture-conformance-closure/closeout.md`

## Forbidden Scope

The accepted PM-011 contract must continue to forbid:

- MaterialProgram implementation.
- `foundation/meta` extraction.
- renderer-owned UI product truth.
- ECS-owned UI semantics.
- generic node soup.
- giant `UiSemanticEvent` enum growth.
- placeholder owner folders.
- broad retained UI rewrites.
- unscoped crate creation.

## Accepted Product Implementation Authority

PM-011 may run product implementation only while all of these remain true:

- `WR-169` remains promoted to an implementation-ready roadmap state according to the roadmap workflow.
- `plan.contract.yaml` remains `status: accepted`.
- The accepted sidecar names exact existing outputs and exact `new:` file outputs.
- Every new crate is represented by exact `new: <crate>/Cargo.toml` authority and explicit `crate_creation` permission.
- Every runtime evidence requirement references an actual typed validation command id from `validation_commands`.
- No evidence requirement uses placeholder ids such as `missing:*`.
- Product implementation permissions in the sidecar are covered by the manifest and WR state.
- `task production:audit-track -- --track PT-UI-PROGRAM-ARCHITECTURE --full-automation` passes if full automation is requested.

## Validation

Minimum validation for accepted implementation authority:

- `task truth:verify -- --track PT-UI-PROGRAM-ARCHITECTURE --claim ui-program-architecture-implementation`
- `task truth:verify -- --track PT-UI-PROGRAM-ARCHITECTURE --claim ui-program-perfectionist-conformance`
- focused Rust tests named by the accepted sidecar
- `task production:validate`
- `task roadmap:validate`
- `task docs:validate`
- `task planning:validate`

## Stop Conditions

- Stop while any truth verifier finding remains.
- Stop if product authority would be broader than the active WR and accepted sidecar.
- Stop if evidence is generated paperwork without concrete code/test/artifact subjects.
- Stop if MaterialProgram, `foundation/meta`, renderer-owned UI truth, or ECS-owned UI semantics would start.
- Stop if validation fails.
