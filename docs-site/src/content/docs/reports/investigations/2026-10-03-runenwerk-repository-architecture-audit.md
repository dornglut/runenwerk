---
title: Runenwerk Repository Architecture and Structure Audit
description: Repository-wide audit of Runenwerk ownership, package boundaries, structural concentration, stale residue, and bounded follow-up work.
status: active
owner: workspace
layer: investigation
canonical: false
last_reviewed: 2026-10-03
publication: reference
pagefind: false
related_docs:
  - ./2026-10-03-runenwerk-repository-file-census.md
  - ./2026-10-03-runenwerk-workspace-package-census.md
  - ../../apps/runenwerk-editor/current-architecture.md
  - ../../adr/accepted/0025-normalize-editor-coordination-and-semantic-ownership.md
  - ../../guidelines/dependency-rules.md
  - ../../guidelines/module-structure-guidelines.md
---

# Runenwerk Repository Architecture and Structure Audit

## Purpose

Issue #1153 asks whether Runenwerk has become structurally too large, whether its crate/module
boundaries still reflect semantic ownership, and which corrections are justified by current source.

The answer is deliberately narrower than "split the monorepo":

> Runenwerk is not currently proven too large as one integration-product repository. Its
> cross-layer package direction is mostly healthy. The material structural debt is concentrated in
> a bounded set of Editor, Automation, Drawing, and documentation/residue areas.

This report is point-in-time investigation evidence. Current source/tests remain behavior authority;
accepted ADRs/designs remain durable architecture authority; active issues own delivery sequencing.

## Exact evidence baseline

```text
RUNENWERK_MAIN=34ad02d9b8fac925936ac0d5971abe4d293f1749
TRACKED_FILES=2461
RUST_FILES=1912
PRODUCTION_RUST_FILES≈1625
WORKSPACE_PACKAGES=71
RUST_SOURCE_BYTES=15826060
```

The complete tracked-file inventory is in
[the repository file census](./2026-10-03-runenwerk-repository-file-census.md).
The 71-package workspace census is in
[the package census](./2026-10-03-runenwerk-workspace-package-census.md).

The audit also incorporates the accepted CI/test-topology investigation #1147. That program reduced
canonical validation from the previous >10-minute class to an accepted T8 baseline around 229s,
while preserving the repository-owned `cargo validate` authority. Repository size is therefore no
longer, by itself, a CI-driven extraction argument.

## 1. Executive disposition

Keep the current repository role:

```text
standalone Runen frameworks
        |
        v
Runenwerk domain + integration contracts
        |
        v
Engine/runtime composition
        |
        v
apps / host / editor / product policy
```

Do **not**:

- split Runenwerk into more repositories from size alone;
- merge small semantic crates just to reduce package count;
- introduce a generic `runen-math` or other shared-core repository from similarity alone;
- duplicate active RunenRender/RunenUI/material/world/cutover authority;
- mass-split every file above an arbitrary line threshold.

Do:

- delete or quarantine retired predecessor code when it no longer serves a live compatibility
  contract;
- decompose a small number of responsibility-heavy live modules along already accepted ownership
  lines;
- reconcile stale documentation/repository residue;
- serialize structural cleanup behind active owner work where paths overlap.

## 2. Repository and package topology

### 2.1 Top-level concentration

Exact current Rust concentration:

| Area | Rust files | Approximate Rust bytes | Disposition |
| --- | ---: | ---: | --- |
| `domain/` | 1,026 | ~5.93 MB | Keep domain ownership; review only concrete hotspots. |
| `engine/` | 487 | ~4.70 MB | Keep runtime/integration owner; review bounded hotspots. |
| `apps/` | 330 | ~4.76 MB | Keep product/app composition; Editor contains most concentration. |
| `foundation/` | 56 | ~0.23 MB | Keep; dependency direction and size are healthy. |
| `adapters/` | 8 | ~0.13 MB | Keep explicit adapter role. |

This is a large integration product, but the size is distributed across real domains/apps rather
than one accidental package.

### 2.2 Workspace granularity

There are 71 root workspace packages. Thirty-four live under `domain/ui/**` and are separately
owned by the active RunenUI adoption/cutover program. This audit does not adjudicate that package
granularity.

Outside that separately owned UI cluster, small packages generally have concrete reasons to exist:
proc-macro separation, low-level foundation contracts, or narrow semantic domain boundaries.
Package size alone is not evidence for merging.

### 2.3 Dependency direction

The reviewed manifest graph follows the intended direction:

```text
foundation -> domain -> engine/runtime -> apps/adapters/tools
```

Observed disposition:

- no foundation package depends upward on domain, Engine, or applications;
- no reviewed domain package depends upward on Engine or applications;
- Engine composes domain/foundation contracts and accepted standalone-framework public APIs;
- applications compose Engine/domain/adapter contracts;
- the reviewed `runenwerk_editor -> runenwerk_runtime_preview` app-to-app edge is dev-only external
  preview evidence, not a production semantic dependency.

No repository split or package merge is justified by the current dependency graph.

## 3. Editor architecture — strongest structural pressure

The accepted Editor authority is ADR 0025 plus the current Editor architecture. It explicitly
separates editor coordination from foreign semantic state, structural `ui_composition`, native
presentation, persistence owners, and runtime host behavior.

### 3.1 Retired WorkspaceState predecessor — revise the earlier split hypothesis

Earlier hotspot review treated the old workspace files as large production split candidates.
Current source disproves that framing.

`domain/editor/editor_shell/src/workspace/mod.rs` gates the old persistence, projection, reducer,
and projection-ratification modules behind `#[cfg(test)]`. In `state.rs`, the actual
`WorkspaceState`, `WorkspaceStateError`, and their large implementation are also test-gated.
`persisted.rs` is entirely a test-only module containing the V1–V5 predecessor persistence model.

Current Editor authority states that:

- live structural authority is `ui_composition::CompositionState`;
- current startup/profile activation/self-authoring activation do not route through
  `WorkspaceState` or `import_legacy_workspace`;
- V1–V5 workspace files are unsupported compatibility input;
- the predecessor graph remains only as compatibility/parity/test evidence.

Therefore the correct structural finding is:

> **Do not split the predecessor into nicer production modules. Audit whether the retained
> test-only implementation is still necessary; delete or sharply quarantine it if equivalent
> current architecture evidence can be preserved without maintaining a second historical model.**

Live compatibility identities/profile metadata in `workspace/state.rs` remain separately reviewed;
the finding does not authorize deleting current stable keys, profile metadata, or structural
composition adapters.

### 3.2 Live shell state

`apps/runenwerk_editor/src/shell/state.rs` is 2,232 lines and one
`RunenwerkEditorShellState` currently owns several independently changing concerns:

- target-local UI runtime/projection caches;
- composition runtime, projection, restore, and structural coordination;
- native Editor-window / presentation-target bindings;
- workspace-profile activation;
- toolbar and tab-stack menu state;
- self-authoring and editor-definition activation;
- structural history and identity allocation;
- tab drag/drop, region compass, and split-resize interaction sessions.

These are all Editor-app concerns, so a new crate/repository is not justified. The file is a strong
**internal decomposition candidate** into explicit app-owned coordination components while keeping
one shell aggregate/public behavior boundary.

### 3.3 Shell command dispatch

`apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` is 2,213 lines and dispatches:

- Editor Lab/self-authoring operations;
- structural composition commands;
- toolbar/window/workspace-profile commands;
- scene save/load;
- workspace-layout save/load;
- migration-failure presentation.

This is a strong **command-domain decomposition candidate**. The public dispatch entry point may
remain one route, but handlers should be grouped by owner/command family rather than one giant
implementation file.

### 3.4 Self-authoring

`apps/runenwerk_editor/src/shell/self_authoring/mod.rs` is 3,685 lines and owns a real app-local
product boundary, but its implementation combines:

- draft document/catalog state;
- recipe editing and direct UI/layout mutation;
- CRUD/import/export;
- project-package IO state;
- preview formation;
- apply/review/reject/rollback;
- operation history;
- scenario/performance evidence formation;
- default fixture construction and diff utilities.

Keep `SelfAuthoringWorkspaceState` as app-owned authority, but decompose its implementation by
lifecycle/phase. Do not extract it as a reusable framework.

### 3.5 Editor runtime resources

`apps/runenwerk_editor/src/runtime/resources.rs` is 2,128 lines and combines:

- Editor host/workbench activation and theme policy;
- input-bridge state;
- viewport primitive/render packet contracts;
- material-selection packet formation;
- viewport diagnostics/branch traces;
- camera state and camera math;
- serialization/shader packet helpers.

These are visibly separate runtime responsibilities. Split internally by host/input/viewport-packet/
camera concerns while preserving existing resource types and ownership.

### 3.6 Editor input bridge

`apps/runenwerk_editor/src/runtime/systems/input_bridge.rs` is 2,119 lines. Its production path
combines global/active shortcut routing, UI-consumption admission, cursor intent, target-local
viewport resolution, capture/focus, camera tools, picking, and pointer dispatch.

This remains one app/runtime integration seam, but helper/phase decomposition is justified. Do not
move reusable input semantics out of RunenInput or create a second input reducer.

### 3.7 Material Lab handoff

`apps/runenwerk_editor/src/material_lab/renderer_handoff.rs` is 3,020 lines. Production code spans
preview scene-product formation, scene-material table/resource-layout resolution, compiler-binding
validation, parameter/texture preparation, and renderer handoff.

The boundary remains app-local Material Lab integration. A pipeline-phase split is reasonable, but
active Material P3/P3A authority should settle first; this audit does not activate overlapping work.

## 4. Engine

### 4.1 Automation persistence

`engine/src/automation/persistence.rs` is ~1,565 lines of production code and combines:

- public V1 envelope/provenance/error contracts;
- persisted schema types;
- header/provenance validation;
- export builder and identity tables;
- import builder and materialization;
- every normalized-input value conversion.

V2 already has its own module. V1 should remain one public persistence contract, but the physical
implementation is a strong **schema / export / import decomposition candidate**.

Active Automation A18 owns nearby input/recording delivery. Structural persistence work must
serialize behind overlapping Automation writers.

### 4.2 Runtime

`engine/src/runtime/winit_runner.rs` is large in raw lines but only about ~790 production lines
before tests and remains one coherent Winit event-loop/runtime adapter. **Keep** unless repeated
change pressure proves a real split.

`engine/src/automation.rs` is similarly inflated by inline tests; its production body is roughly
half the raw file. It is review-worthy but not currently a high-priority structural defect.

### 4.3 Render

`engine/src/plugins/render/**` is explicitly excluded from duplicate delivery planning here.
Issue #1146 and the accepted render-runtime decomposition own that structure. This audit retains the
Runenwerk host/product integration boundary and does not reopen renderer ownership.

### 4.4 Net

Engine networking integration remains under `engine/src/plugins/net/**`; reusable networking
semantics remain standalone RunenNet-owned. No new networking repository/package boundary is
justified here.

## 5. Core domains and foundation

### 5.1 Drawing

`domain/drawing/src/tile/formation.rs` is ~1,200 production lines and combines:

- tile formation policy/contracts/diagnostics;
- committed and preview invalidation;
- affected-tile geometry;
- stroke-to-dab generation;
- rasterization and blending;
- lineage/cache/determinism identity.

This is a strong internal decomposition candidate, for example around formation/invalidation,
rasterization, and identity/hash responsibilities. Keep the existing `drawing` crate and public
semantic owner.

`domain/drawing/src/ratification/ratifier.rs` is ~836 production lines but is substantially more
cohesive as one ratifier traversing the drawing document. Keep unless owner-specific change pressure
emerges.

### 5.2 Foundation and other core domains

Foundation is small and dependency-clean. Other non-UI/non-Editor domain files mostly fall into a
reasonable production size after discounting inline tests. No blanket foundation/domain package
merge is justified.

## 6. Applications

### 6.1 Render Lab

Render Lab's apparently large files include substantial test/evidence tails and generally cohesive
product-evidence phases. `native.rs` and `native/temporal_quality.rs` may admit later local
decomposition, but there is no current reason for a size-only refactor.

### 6.2 Arena, Draw, runtime preview

These applications remain bounded product/integration owners. No repository or crate extraction is
justified by current evidence. Their small target-count/CI opportunities were intentionally not
pursued after #1147 because they no longer target the measured critical path.

## 7. Documentation and repository hygiene

Concrete inconsistencies:

1. `.idea/modules.xml` and `.idea/vcs.xml` are tracked even though `.gitignore` ignores
   `.idea/`. `modules.xml` references an untracked `Runenwerk.iml`. Delete these as stale IDE
   residue unless a current contributor contract explicitly requires them.
2. Top-level `net/` contains only `architecture.puml` and `architecture-target.puml`; it is not
   a workspace/code owner.
3. `docs-site/src/content/docs/guidelines/architecture.md` still describes top-level `net/` as
   if it owned remaining simulation/history/network-authoring code. Current code places realtime
   integration in Engine and simulation/replay under their domain crates.
4. Canonical long-form architecture already lives under `docs-site/`. The two top-level network
   diagrams should be checked for current value and either moved into the owning docs hierarchy or
   deleted if historical/stale.
5. `logs/.gitkeep` is intentional output-directory scaffolding and should remain unless a separate
   repository contract says otherwise.

## 8. Findings by severity

### High — structural maintenance debt, not runtime correctness defects

- Editor test-only `WorkspaceState` / V1–V5 predecessor implementation remains a large second
  historical model. Audit for deletion/quarantine rather than further investment.
- Live Editor shell state and shell command dispatch each combine multiple independently changing
  app-owned responsibilities.
- Editor self-authoring combines several lifecycle/persistence/evidence phases in one module.

### Medium

- Editor runtime resources combine host, input, viewport packet, diagnostics, camera, and shader
  packet helpers.
- Editor input bridge combines several routing phases inside one integration seam.
- Engine Automation V1 persistence combines schema, validation, export, import, and conversion.
- Drawing tile formation combines formation/invalidation, rasterization, and identity/hash.
- Material Lab renderer handoff combines several pipeline phases, but should defer to active Material
  owner work.

### Low / hygiene

- tracked `.idea` residue;
- stale top-level networking-document placement and architecture guideline wording.

## 9. Code-quality versus architecture findings

Architecture is broadly healthy:

- repository role is correct;
- package-layer direction is correct;
- standalone framework boundaries are respected;
- small semantic crates are not inherently a defect;
- no new shared-core repository is justified.

Code-quality/maintainability debt is local:

- several very large modules have real internal phase boundaries;
- predecessor test-only implementation remains larger than its current value may justify;
- some physical documentation/layout no longer matches current ownership.

The audit did not find evidence that broad weak typing, global mutable truth, or cross-layer cycles
are repository-wide problems.

## 10. Target responsibility model

Retain:

```text
foundation
  stable low-level vocabulary
        |
        v
domain
  Runenwerk-owned engine-agnostic semantics
        |
        v
engine/runtime
  lifecycle + framework/domain integration
        |
        v
apps/adapters/tools
  product policy, native host, editor, IO, presentation
```

Standalone RunenECS/RunenGPU/RunenRender/RunenUI/RunenInput/RunenNet/RunenSpatial/RunenShader/
RunenGraph/RunenSDF retain their accepted reusable semantics.

Within the Editor:

```text
semantic/domain owners
       |
       v
editor coordination / provider contracts
       |
       +---- ui_composition structural authority
       |
       v
Runenwerk Editor app/runtime integration
       |
       v
native windows / renderer / IO / product policy
```

Structural cleanup must make this model easier to see; it must not invent new semantic owners.

## 11. Minimal successor graph

Do not launch a mass refactor. Use bounded cuts with cold review and exact-head validation.

```text
S0 repository/docs hygiene
   independent, low-risk
   delete tracked .idea residue
   reconcile top-level net diagrams + stale architecture guideline

E0 Editor predecessor cleanup investigation
   domain/editor/editor_shell workspace predecessor only
   prove exactly which test-only WorkspaceState / reducer / projection / V1-V5
   persistence evidence is still necessary
   -> delete/quarantine obsolete implementation rather than modularizing it
        |
        v
E1 Editor live shell decomposition
   app shell state + command-domain boundaries
   preserve RunenwerkEditorShellState public behavior and ADR-0025 ownership
        |
        +-----------------------------+
        |                             |
        v                             v
E2 self-authoring phase split     E3 runtime host/input/viewport split
   app-local only                    app runtime only

D0 Drawing tile-formation split
   independent of Editor
   preserve drawing public contracts/determinism

A0 Automation V1 persistence split
   serialize behind active Automation A18 / any overlapping writer

M0 Material Lab handoff split
   defer behind active Material P3/P3A owner work

Render #1146 and RunenUI #994/#1110 remain separate authorities.
```

### Parallelism / serialization

- S0 and D0 are independent of Editor/Automation/Material and can proceed in parallel.
- E0 should precede broad Editor shell restructuring because it removes or clearly quarantines the
  predecessor model that still shapes tests/imports.
- E1/E2/E3 should be sequenced by their actual shared `shell/mod.rs`, `runtime/mod.rs`, and test
  write sets; do not assume parallelism merely because primary files differ.
- A0 is blocked by overlapping Automation delivery until writer reconciliation is clean.
- M0 is deferred to Material owner sequencing.
- Render and UI work remain outside this successor program.

## 12. Rejected refactors

Rejected by current evidence:

- split Runenwerk into several repositories;
- create `runen-math` as a generic substrate;
- merge foundation/domain crates because they are small;
- move app-specific Editor self-authoring into a reusable framework;
- refactor Render Lab solely from raw line counts;
- duplicate #1146 Render work;
- restructure `domain/ui/**` while RunenUI cutover authority is active;
- preserve the retired Editor workspace predecessor merely by making it prettier.

## 13. Acceptance decision

The repository is **large but not structurally too large as one repository**.

The correct next architecture work is local simplification:

1. remove/reconcile stale repository/docs residue;
2. determine whether the test-only Editor workspace predecessor can be retired;
3. decompose live Editor app hotspots by already accepted responsibilities;
4. independently split Drawing tile formation;
5. defer Automation/Material structural work until their active owners permit it.

No new repository extraction is authorized by this audit.
