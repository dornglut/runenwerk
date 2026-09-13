---
title: Normalize App Runtime, Host, Lifecycle, and Capability Ownership
description: Durable application-runtime ownership laws separating Runenwerk App composition, contained owner runtimes, host realization, advancement policy, lifecycle occurrences, and owner capability state.
status: accepted
owner: engine
layer: architecture
canonical: true
last_reviewed: 2026-09-13
related_adrs:
  - ./0014-repository-family-extraction-boundaries.md
  - ./0017-cross-authority-consistency-and-graph-semantics.md
  - ./0018-semantic-federation-and-physical-realization.md
  - ./0019-batteries-included-application-composition.md
  - ./0022-runenwerk-owned-product-and-query-publication-phases.md
related_docs:
  - ../../architecture/runenwerk-platform-architecture.md
  - ../../design/accepted/runenwerk-app-runtime-and-composition-semantic-model.md
  - ../../design/accepted/execution-fabric-and-product-jobs-design.md
---

# ADR 0023: Normalize App Runtime, Host, Lifecycle, and Capability Ownership

## Context

ADR 0019 established one `App` as Runenwerk's live runtime composition root and
accepted a target discipline: App construction should own only genuine application
runtime invariants, while domain/product capability state should normally be installed by
its owner.

Current source still mixes several distinct concerns in `App`, bootstrap, and runtime
execution: RunenECS containment, App lifecycle state, Winit/native-window host policy,
bounded runner policy, fixed cadence, simulation identity, input, scene/UI/gameplay
state, publication, product-job execution/cache state, and simulation configuration.

The defect is therefore not simply "too many resources in bootstrap". The current shape
partially conflates composition, containment, hosting, advancement, lifecycle placement,
and semantic ownership.

ADR 0022 supplies an important precedent: Runenwerk can own a lifecycle occurrence such
as product publication without owning the semantic payload participating in that
occurrence.

This ADR defines the durable ownership laws needed before implementation cleanup. The
companion accepted semantic-model design owns the detailed current-source census,
resource/API disposition matrix, and implementation fitness tests. Those details are
intentionally not duplicated here.

## Decision

### 1. Keep one live `App` composition root

Runenwerk uses one live application/runtime composition root:

```text
App
```

`App` owns Runenwerk application assembly and integration for one runtime instance. It
does not become a semantic super-domain merely because owner runtimes, plugins,
resources, systems, or adapters are reachable through it.

Rejected as parallel live authority:

```text
RunenApp runtime
persistent meta-application runtime
untyped service locator
hidden dependency-injection universe
generic capability database
second scheduler/meta-executor
```

Future convenience must lower into the same `App`/owner integration path, consistent
with ADR 0019.

### 2. Normalize five independent semantic dimensions

Runenwerk distinguishes:

```text
Application Composition
    selected owners/capabilities/product behavior and configuration

Integration / Runtime Container
    Runenwerk lifecycle/integration state plus contained owner runtimes/adapters

Host
    execution-environment realization such as native-window or headless

Advancement Policy
    how execution opportunities are supplied or bounded

Owner Capability State
    state whose semantic invariants belong to scene/render/UI/input/simulation/
    replay/network/product-execution/etc. owners
```

The governing inequalities are:

```text
host != advancement policy
containment != semantic ownership
lifecycle occurrence != owner state
composition root != semantic ownership
resource presence != capability authority
schedule position != domain ownership
```

### 3. Containing an owner runtime does not transfer its semantics

Current `App` mechanically contains a RunenECS `World` and `Runtime`. Their ECS semantics
remain owned by RunenECS. Runenwerk owns their application integration and invocation at
Runenwerk lifecycle points.

This ADR neither promotes ECS into a universal Runen ontology nor authorizes removing
the current RunenECS structural integration. Either broader topology decision requires
independent evidence.

General law:

> **`App` may contain or invoke an owner runtime without acquiring that owner's semantic
> invariants.**

### 4. App-owned runtime state requires universal-invariant proof

A state item may be installed as bare **App-owned runtime/integration state** only when
all of the following hold:

1. Runenwerk application/runtime-integration semantics own the invariant.
2. Every currently supported App host/runtime requires it.
3. It remains meaningful with every optional domain/product capability absent.
4. Its absence invalidates the App runtime itself rather than one optional capability.
5. Installing it does not manufacture foreign semantic authority or a fake host or
   capability.
6. Universal installation has independently acceptable cost.

Historical placement, convenience, or existing consumers that assume presence are not
ownership evidence.

This test applies to App-owned runtime state. It does not turn host-neutral pre-run
configuration/metadata or explicitly contained foreign-owner runtimes into App semantic
state.

### 5. Composition is explicit pre-run assembly

The ordinary path is conceptually:

```text
construct App
-> install/configure owner capabilities and product behavior
-> admit/finalize composition
-> prepare selected host/runtime integration
-> start
-> run
```

Runtime plugin/capability topology is not implicitly mutable after startup. A future need
for live dynamic plugin topology requires its own semantic design.

Composition must be deterministic. Required dependencies, illegal duplicates, and
incompatible selections must be diagnosable rather than relying on accidental global
builtins or registration order.

A plugin is a Runenwerk composition/install mechanism, not semantic authority. Capability
semantics remain with their owner.

### 6. Host and Advancement Policy are orthogonal

**Host** answers where/how the runtime is realized. **Advancement Policy** answers how
execution opportunities are supplied or bounded.

The current native-window/Winit realization owns native-window lifecycle, event-loop
control, window/platform event delivery, redraw policy, native hooks, and the current
Winit/event-loop pacing implementation. Those facts are not universal App-owned runtime
state.

A headless host must not require synthetic native-window state or Winit control-flow
state merely to satisfy a window-shaped API.

A bounded-frame/fixed-step proof driver must likewise not change the semantic host merely
because it controls advancement.

If future cross-host pacing/throttling is proven, that generic concern belongs to
Advancement Policy rather than being inferred from today's Winit pacing machinery.

### 7. App lifecycle and owner lifecycle are separate

The conceptual App lifecycle is:

```text
Configuring
-> Prepared
-> Starting
-> Running
-> Terminating
-> Terminated
```

This is semantic vocabulary, not a requirement for one public enum or a new lifecycle
event bus.

The successful App Startup schedule executes at most once per runtime instance.
Owner-specific loading, warm-up, connection, readiness, residency, and similar state
machines remain owned by those owners.

### 8. Lifecycle occurrence does not acquire foreign semantic authority

Runenwerk may own **when** integration work occurs without owning the semantic meaning of
state participating at that point.

Normative examples:

```text
Runenwerk fixed-step occurrence
    != simulation tick identity

Runenwerk native-window/platform event
    != reusable physical-input semantic authority

Runenwerk RenderPrepare/RenderSubmit lifecycle position
    != renderer semantic authority

Runenwerk ProductPublication occurrence
    != product/domain payload authority
```

Owner adapters/plugins translate Runenwerk lifecycle occurrences into owner-specific
state transitions where required.

### 9. Fixed cadence and simulation identity are separate

Runenwerk owns application fixed-step cadence/lifecycle placement. Simulation tick,
profile, authority role, session identity, seed, and RNG remain simulation semantics.

Current code directly coupling fixed-step execution to `engine_sim::SimulationTick` must
not become the long-term ownership model.

The later implementation must decide from then-current consumers whether fixed cadence
is universal App lifecycle or a selectable capability. It must not preserve a nominal
`FixedStepPlugin` that merely duplicates already-active global state.

### 10. Capability-specific convenience does not require a giant inherent `App` API

The one App root may expose owner-specific extension APIs for scene, render, input,
simulation, replay, or other capability authoring.

Exact Rust names are not selected here. The law is:

> **Convenience remains ergonomic, but semantic ownership stays with the capability
> whose invariants the convenience manipulates.**

Such extensions operate on the same App. They do not create a second runtime.

### 11. Effective composition inspection is derived state

Runenwerk may expose read-oriented effective-composition diagnostics such as selected
plugins/groups, deterministic installation order, host/advancement selection, and
rejected/incompatible choices.

That view must derive from the actual composition path. It is not a writable service
locator, mutable registry, or second runtime authority.

## Preserved owner boundaries

This decision does not absorb adjacent work:

- ADR 0022 remains authoritative for Product/Query publication lifecycle semantics.
- The Execution Fabric and Product Jobs design remains authoritative for product jobs,
  query snapshots, publication integration, and runtime product/cache behavior.
- The completed RunenInput I0 result remains `INTERNAL_BOUNDARY_REPAIR_FIRST`; this ADR
  does not invent `RunenInput` semantics or declare current mixed `InputState` App-core.
- Issue #281 remains authoritative for the unresolved broader Plan/semantic-programming
  architecture.
- RunenRender, RunenUI, RunenECS, RunenNet, and other owners retain their reusable
  semantics.

## Consequences

- App-owned runtime state becomes smaller and more defensible.
- Owner-runtime containment becomes explicit without semantic ownership transfer.
- Headless execution no longer needs native-window-shaped sentinel state as a design
  requirement.
- Host selection and bounded advancement stop being one overloaded mode concept.
- Plugins become truthful installers/integration boundaries rather than labels over
  globally preactivated capability state.
- Fixed cadence can be separated from simulation identity.
- Future product/plugin groups can compose real owner boundaries instead of wrapping
  bootstrap debt.
- Clean implementation cuts may intentionally break legacy convenience/API assumptions
  rather than preserving rejected ownership with aliases or mirrors.

## Rejected alternatives

### Treat current bootstrap placement as semantic authority

Rejected. File/resource placement is implementation evidence, not proof of ownership.

### Move every builtin to a nearby plugin mechanically

Rejected. That would preserve deeper conflations such as host vs advancement and fixed
cadence vs simulation identity.

### Treat contained owner runtimes as App semantics

Rejected. Containment is not an ownership-transfer mechanism.

### Make every engine concern a plugin for symmetry

Rejected. Some Runenwerk application/integration invariants may genuinely be App-owned.
The qualification is semantic, not aesthetic.

### Persistent capability registry / service locator / generic DI

Rejected because it creates parallel runtime truth and weakens typed owner boundaries.

### Preserve rejected ownership through compatibility aliases

Rejected by default. Future implementation uses clean consumer migration and deletion
unless an independently proven compatibility contract requires otherwise.

## Relationship to broader Plan work

Issue #281 may later accept a logical Plan/programming model. If so, application plans
may lower into ordinary `App` composition as ADR 0019 already allows. They must not
become a second live App runtime or override this ADR's Host, Advancement Policy,
containment, lifecycle, or owner-state boundaries.

## Implementation gate

This ADR authorizes no Rust/Cargo/runtime change by itself.

The companion accepted semantic-model design owns the detailed current-source
classification, disposition matrix, failure/admission rules, and mechanical fitness
tests.

After this architecture is accepted, derive exactly one first bounded implementation
issue from then-current `main` and active overlap. Do not pre-author a full migration
tree.

The active ADR-0022 publication-lifecycle cutover remains separately owned and must be
accepted or explicitly re-reconciled before App implementation edits overlapping runtime
surfaces.
