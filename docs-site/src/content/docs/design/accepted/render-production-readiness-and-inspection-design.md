---
title: Render Production Readiness And Inspection Design
description: Accepted design for PM-RENDER-PG-008 renderer diagnostics, inspection, capture/replay policy, performance budgets, examples, and completion evidence.
status: accepted
owner: engine
layer: engine-runtime / render production readiness
canonical: true
last_reviewed: 2026-05-21
related_designs:
  - ./render-product-graph-platform-design.md
  - ./render-contract-ergonomics-design.md
  - ./feature-owned-render-contributions-design.md
  - ./render-execution-graph-compiler-maturity-design.md
  - ./product-surface-platform-hardening-design.md
  - ./render-fragment-data-driven-maturity-design.md
  - ./editor-native-multi-window-presentation-design.md
  - ../implemented/render-product-surface-foundation-bundle-design.md
related_roadmaps:
  - ../../engine/roadmaps/fully-featured-renderer-roadmap.md
  - ../../engine/plugins/render/docs/roadmap.md
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# Render Production Readiness And Inspection Design

## Status

This is the accepted design contract for `PM-RENDER-PG-008`.

It accepts the production-readiness and inspection direction before
implementation work starts. It does not authorize product code changes by
itself, does not mark `PM-RENDER-PG-008` complete, and does not assign
`completion_quality`. Implementation still requires a legal bounded WR row,
`task production:plan`, roadmap promotion or current-candidate selection,
focused validation, closeout evidence, and a rerun of
`task ai:goal -- --track PT-RENDER-PG`.

## Goal

Harden renderer production evidence around the already completed product-graph
platform slices:

```text
prepared products, surfaces, contributions, fragments, surfaces, and flows
  -> typed inspection and readiness reports
  -> capture/replay and budget policy evidence
  -> examples and documentation usable by product teams
  -> bounded closeout evidence
```

The renderer remains an execution and presentation layer over prepared inputs.
Production readiness must make renderer behavior understandable and testable
without moving product truth, product selection, freshness, fallback legality,
authority, rebuild policy, material truth, drawing truth, or residency policy
into renderer code.

## Current Baseline

The required platform pieces already exist as bounded contracts:

- PM-002 added return-only product-surface request helpers and typed duplicate
  prepared-frame request diagnostics.
- PM-003 added typed feature-owned contribution collector contracts and
  inspection.
- PM-004 added typed render execution graph diagnostics, prepared-frame
  preflight, capability profiles, resource lifetimes, and compiler inspection.
- PM-005 added shared product-surface manifests, product-surface diagnostics,
  UI binding intents, upload-backed surfaces, and product-surface inspection.
- PM-006 added render-surface identity, native-window identity, prepared-frame
  surface identity, and submit-time surface mismatch diagnostics.
- PM-007 added render fragment descriptors, validation, merge provenance,
  last-good reload state, and fragment inspection.
- Existing render inspection modules already expose capture, artifact, timing,
  prepared-frame, plan, pass-provenance, and resource inspection primitives.

The remaining gap is production readiness, not a new ownership model. PM-008
must connect those typed surfaces into a coherent readiness, capture/replay,
budget, example, and closeout story.

## Ownership

DDD bounded context owner:

- `engine/src/plugins/render` owns renderer readiness inspection, backend-neutral
  render execution diagnostics, capture/replay contracts over renderer-derived
  state, render budget reports, examples, and render closeout evidence.

Translation boundaries:

- Product domains and Product Jobs own product truth, product dependency truth,
  freshness, authority, fallback legality, rebuild policy, residency intent,
  source lineage, and product diagnostics.
- App and product-surface producers translate their product decisions into
  prepared views, dynamic targets, uploads, flow invocations, UI bindings,
  feature contributions, and producer diagnostics.
- Render inspection aggregates already prepared renderer-facing DTOs. It does
  not inspect mutable backend handles and does not query domains to decide
  product policy.
- Backend runtime exposes derived timings, captures, resources, and submission
  results only as read-only inspection data.

Team Topologies ownership:

- Complicated-subsystem render owner enabling stream-aligned product, editor,
  drawing, and future world/product teams.

No ADR is required while this remains an engine-runtime hardening slice over
existing accepted render contracts. Create an ADR only if implementation turns
capture/replay into a persisted cross-domain ABI, changes source-of-truth
ownership, or introduces a reusable contract outside the render plugin boundary.

## Vocabulary

- `readiness report`: an aggregate, read-only renderer inspection DTO that
  summarizes prepared inputs, graph/preflight diagnostics, product-surface
  status, fragment/reload state, multi-surface identity, timing, capture, and
  budget results.
- `diagnostic coverage matrix`: a documented and tested map from production
  readiness concerns to typed diagnostics and inspection fields.
- `capture manifest`: a backend-neutral description of what renderer-derived
  textures, passes, resources, and frame metadata were captured.
- `replay manifest`: a backend-neutral description of renderer inputs and
  artifact references required to rerun or dry-run a renderer scenario.
- `budget report`: a renderer-owned timing, memory, upload, capture, and
  inspection budget result. It does not own product rebuild or product
  freshness policy.
- `evidence pack`: closeout-ready collection of validation commands, example
  outputs, capture/replay policy results, budget reports, and known gaps.

## Locked Decisions

PM-008 accepts these decisions:

- Production readiness is hardening over the accepted product-graph platform,
  not a new renderer-owned product-policy layer.
- Inspection surfaces must be typed read-only DTOs. They must not expose
  mutable WGPU handles, backend caches, product source objects, or editor/app
  workflow internals.
- Readiness reports should aggregate existing typed reports instead of replacing
  prepared-frame, product-surface, graph/preflight, contribution, fragment,
  capture, timing, and multi-surface inspection contracts.
- Capture/replay policy records renderer execution inputs, selected renderer
  artifact references, diagnostics, and derived captures. It must not make
  renderer captures authoritative product source truth.
- Replay must fail closed when required prepared inputs, capability profiles,
  source artifact references, formats, or target identities are missing or
  incompatible.
- Performance budgets apply to renderer execution and evidence paths: prepare,
  graph/preflight, fragment validation/reload, dynamic target/upload handling,
  capture/readback, timing, and multi-surface presentation. Budget overruns
  produce diagnostics and reports; they do not choose semantic product fallback.
- Examples must demonstrate the production contract path through normal public
  APIs. They must not use renderer-private handles or bypass prepare/compile
  validation.
- PM-008 implementation needs its own bounded WR row. `WR-003` remains support
  context, `WR-009` remains PM-006 completion evidence, and `WR-010` remains
  PM-007 completion evidence.
- Completion quality starts at `bounded_contract`. Claim `runtime_proven` only
  if host-backed runtime capture/replay, budget, and multi-surface evidence
  actually pass. Claim `perfectionist_verified` only with a completed audit and
  no known quality gaps.

## Implementation Scope

Future PM-008 implementation may touch:

```text
engine/src/plugins/render/inspect
engine/src/plugins/render/backend
engine/src/plugins/render/runtime
engine/src/plugins/render/renderer
engine/src/plugins/render/frame
engine/src/plugins/render/graph
engine/src/plugins/render/composition
engine/examples
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/engine/plugins/render
docs-site/src/content/docs/engine/roadmaps/fully-featured-renderer-roadmap.md
```

Expected implementation concepts:

- a renderer readiness report DTO that references existing inspection reports;
- typed readiness diagnostics for missing evidence, failed preflight, failed
  capture, failed replay, budget overflow, incomplete fragment reload evidence,
  surface identity mismatch, and unsupported backend capability;
- a capture/replay manifest policy over renderer artifacts and prepared inputs;
- budget thresholds and reports for renderer-owned execution evidence;
- examples that cover product surfaces, graph/preflight diagnostics, fragments,
  capture/readback policy, and multi-surface identity;
- documentation that explains the public production inspection path without
  forcing product teams to read backend internals first.

## Out Of Scope

PM-008 must not implement:

- product truth, product selection, product freshness, fallback legality,
  authority class, rebuild policy, or dependency truth;
- material graph truth or material lowering;
- SDF/world brick/page-table, clipmap, raymarch, collision, query, or streaming
  products;
- asset catalog ownership, fragment authoring UI, migration UI, or project
  save/load workflows;
- new native multi-window shell behavior beyond readiness evidence over the
  PM-006 contracts;
- renderer-owned domain semantics for drawing, field visualizer, material lab,
  gameplay, world, prefab, VFX, water, atmosphere, or vegetation products;
- a new public persisted capture/replay ABI without a separate ADR or accepted
  design update.

## Diagnostics And Inspection Requirements

The readiness report must expose enough structure for tooling:

- frame, surface, native window, flow, pass, view, invocation, fragment package,
  producer, product family, target, and capture identities where relevant;
- typed diagnostic kind, severity, scope, message, and source report;
- product-surface status as producer-authored data, not renderer decisions;
- graph/preflight diagnostics and backend capability mismatches;
- fragment validation, merge, reload, and last-good status;
- capture selector status, artifact paths, terminal reasons, pixel probe
  results, and texture diff results;
- timing summaries and budget threshold results.

Diagnostics should be stable enough for examples, closeout evidence, and future
tooling, but they should remain backend-neutral unless an inspected value is
explicitly backend-derived execution state.

## Capture And Replay Policy

Capture/replay exists to prove renderer behavior and support debugging. It is
not a substitute for product source truth.

Implementation should define:

- which prepared renderer inputs and derived artifacts are captured;
- which fields are required for deterministic dry-run replay versus host-backed
  GPU replay;
- how unsupported formats, missing artifacts, missing backend capabilities, and
  stale manifests fail closed;
- how captures connect to pass/resource identities and artifact manifests;
- how closeout evidence records host limitations without overstating quality.

Renderer replay may reference product payload artifacts supplied by owning
producers, but it must not create or repair those product payloads.

## Performance Budgets

Renderer readiness budgets should cover:

- frame prepare and submit timing summaries;
- graph compile/preflight diagnostics and timing where available;
- dynamic target and upload counts, sizes, and unsupported format paths;
- capture/readback counts, skipped captures, failures, and artifact size;
- fragment validation/reload counts and failed-preserved revisions;
- multi-surface frame and submit mismatch diagnostics.

Budget reports are evidence and diagnostics. They do not decide whether a
product should rebuild, reuse last-good state, fall back, or change authority.

## Examples And Closeout Evidence

PM-008 implementation should add or harden examples that prove the production
contract path through normal render APIs:

- product-surface manifest and prepared-frame inspection;
- compiler/preflight failure inspection;
- fragment merge/reload inspection;
- capture/readback or capture-manifest policy;
- multi-surface identity and mismatch diagnostics where host support allows.

Closeout evidence must state exactly which validations passed, which examples
were run, whether host-backed GPU/multi-surface proof ran, and why the selected
`completion_quality` is honest.

## Validation Expectations

The future implementation contract should include at least:

```text
cargo test -p engine render_runtime_inspect
cargo test -p engine render_dynamic_targets
cargo test -p engine --test render_flow_v2
cargo test -p engine --test render_flow_fragments
cargo test -p engine --test render_multi_surface
cargo test -p engine --example render_fragment_compositor
cargo test -p engine --example render_readiness_inspection
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-RENDER-PG
```

Add narrower tests as implementation names stabilize. Do not use broad
validation to compensate for missing focused coverage.

## Acceptance Criteria

- The accepted design is linked from render platform, renderer roadmap, and
  production-track gates.
- PM-008 has a legal bounded WR row before implementation.
- Renderer readiness inspection aggregates existing typed contracts instead of
  replacing them.
- Capture/replay policy is fail-closed, backend-neutral at the contract layer,
  and does not make captures product source truth.
- Budget reports diagnose renderer execution evidence only.
- Examples and docs explain complex renderer behavior without exposing backend
  handles or product-domain internals.
- Completion evidence is honest about host-backed GPU, capture/replay, and
  multi-surface limits.
