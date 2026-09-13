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

ADR 0019 establishes one `App` runtime composition root and the target rule that bare App
construction should contain only genuine application/runtime integration invariants.
Current source still mixes several independent concerns in `App`, bootstrap, and runtime
execution: RunenECS containment, App lifecycle state, Winit/native-window host policy,
bounded-runner policy, fixed cadence, simulation identity, input, scene/UI/gameplay
state, publication, product-job state, and simulation configuration.

The defect is therefore not simply "too many resources in bootstrap". The implementation
partially conflates composition, containment, host realization, advancement, lifecycle
placement, and semantic ownership.

ADR 0022 supplies the key precedent: Runenwerk may own a lifecycle occurrence without
owning the domain/framework state participating in that occurrence.

This ADR defines the durable laws needed before implementation cleanup. The companion
accepted semantic-model design owns the detailed current-source census, disposition
matrix, failure semantics, and implementation fitness tests.

## Decision

### 1. Keep one live `App` composition root

Runenwerk uses one live application/runtime composition root:

```text
App
```

`App` owns Runenwerk application assembly and integration for one runtime instance. It
does not become a semantic super-domain because owner runtimes, plugins, resources,
systems, or adapters are reachable through it.

Rejected as parallel live authority:

```text
RunenApp runtime
persistent meta-application runtime
untyped service locator
hidden dependency-injection universe
generic capability database
second scheduler/meta-executor
```

Future convenience must lower into the same `App` and owner-integration path, consistent
with ADR 0019.

### 2. Separate five semantic dimensions

Runenwerk distinguishes:

```text
Application Composition
    selected owners/capabilities/product behavior and configuration

Integration / Runtime Container
    Runenwerk lifecycle/integration state plus contained owner runtimes/adapters

Host
    execution-environment realization such as native-window or headless

Advancement Policy
    how execution opportunities are supplied, bounded, or paced

Owner Capability State
    state whose semantic invariants belong to scene/render/UI/input/simulation/
    replay/network/product-execution/etc. owners
```

The governing laws are:

```text
host != advancement policy
containment != semantic ownership
lifecycle occurrence != owner state
composition root != semantic ownership
resource presence != capability authority
schedule position != domain ownership
```

### 3. Containment does not transfer semantic ownership

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
2. Every currently supported App runtime path requires it, independent of optional
   capabilities and Host realization.
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

### 5. Composition is explicit and has a lifecycle boundary

The ordinary path is conceptually:

```text
construct App
-> install/configure owner capabilities and product behavior
-> admit/finalize composition
-> prepare selected host/runtime integration
-> start
-> run/advance
```

Ordinary plugin/capability topology is mutable while configuring and becomes stable once
runtime preparation/start begins. A future need for live dynamic plugin topology requires
its own semantic design.

Composition must be deterministic. Required dependencies, illegal duplicates, and
incompatible selections must be diagnosable rather than depending on accidental global
builtins or registration order.

A plugin is a Runenwerk composition/install mechanism, not semantic authority. Capability
semantics remain with their owner.

### 6. Host and Advancement Policy are distinct but compatibility-admitted

**Host** answers where/how the runtime is realized. **Advancement Policy** answers how
execution opportunities are supplied, bounded, or paced.

They are separate semantic dimensions, but that does **not** imply every Host ×
Advancement Policy combination is valid. Composition/preparation must admit only
combinations supported by the selected host and capabilities.

The current native-window/Winit realization owns native-window lifecycle, event-loop
control, window/platform event delivery, redraw policy, native hooks, and the current
Winit/event-loop pacing implementation. Those facts are not universal App-owned runtime
state.

A headless host must not require synthetic native-window state or Winit control-flow
state merely to satisfy a window-shaped API.

Once host preparation/start has begun, the host identity for that runtime instance is
stable unless a separately designed host-transition contract explicitly allows a change.
A bounded proof/test driver must therefore not silently switch a windowed runtime into a
headless host or vice versa merely because it controls advancement.

If future cross-host pacing/throttling is proven, that generic concern belongs to
Advancement Policy rather than being inferred from today's Winit pacing machinery.

### 7. Bounded advancement does not terminate a live App

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

A bounded advancement operation is an operation **within `Running`**:

```text
Running
  -- advance N frames / until accepted bounded condition --> Running
```

Normal bounded completion returns control to the caller and does not by itself imply
Host shutdown, App termination, or reconstruction. This preserves the real requirement
that one started runtime may be advanced more than once.

Terminal host/process shutdown follows the `Running -> Terminating -> Terminated` path.

### 8. Startup is one-shot and fail-stop for a surviving runtime instance

The App Startup lifecycle attempt is one-shot for one runtime instance.

Current public bounded-run helpers consume `App` and discard it on returned failure, and
the current windowed runner exits on Startup failure. Therefore current source does not
establish a maintained public same-instance Startup retry path. However, the underlying
`startup_ran: bool` records only successful completion; the shared Startup helper leaves
that flag indistinguishable from "not attempted" after error, and a caught unwind would
likewise have no explicit failed-attempt state if a future/specialized caller retained
the runtime.

The normalized contract must not rely on runner destruction or process/event-loop exit to
make retry impossible. The runtime records that Startup has been attempted **before** the
first Startup system can execute. If that runtime instance survives a returned error or
caught unwind, it is explicitly non-runnable rather than retry-eligible.

If Startup succeeds, the runtime enters `Running` and Startup is never executed again.
If Startup returns an error or panics after execution has begun:

- the runtime does not enter `Running`;
- already committed owner/system effects are not generically rolled back;
- Runenwerk does not implicitly retry Startup on the same surviving runtime;
- ordinary advancement of that surviving runtime instance must reject;
- recovery requires an explicit owner/host recovery contract or reconstruction of the
  runtime instance.

Preparation failures before Startup retain the failure/retry contract of the owning host
or capability; no generic transaction or rollback is introduced here.

### 9. Lifecycle occurrence does not acquire foreign semantic authority

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

### 10. Fixed cadence and simulation identity are separate

Runenwerk owns application fixed-step cadence/lifecycle placement. Simulation tick,
profile, authority role, session identity, seed, and RNG remain simulation semantics.

Current code directly coupling bounded tick advancement and fixed-step execution to
`engine_sim::SimulationTick` must not become the long-term App ownership model.

A later implementation must decide from then-current consumers whether fixed cadence is
universal App lifecycle or a selectable capability. It must not preserve a nominal
`FixedStepPlugin` that merely duplicates already-active global state.

### 11. Capability APIs keep both owner and temporal semantics

The one App root does not require one giant inherent `App` API. Capability-specific
operations may live on owner-specific extension APIs over the same App/runtime instance.

Moving an API to an owner extension does **not** imply that every operation is
composition-time. Each owner contract must preserve the operation's actual temporal
role, for example:

```text
composition/configuration command
runtime command
runtime query
runtime diagnostic/control operation
```

Scene/render/input/simulation/replay convenience stays ergonomic, but its semantic owner
and legal lifecycle phase remain explicit.

### 12. Effective composition inspection is derived state

Runenwerk may expose read-oriented effective-composition diagnostics such as selected
plugins/groups, deterministic installation order, host/advancement selection, and
rejected/incompatible choices.

That view must derive from the actual composition path. It is not a writable service
locator, mutable registry, persistence authority, or second runtime truth.

## Preserved owner boundaries

This decision does not absorb adjacent work:

- ADR 0022 remains authoritative for Product/Query publication lifecycle semantics.
- Issue #591 owns the active ADR-0022 publication cutover, including its narrow handler
  registry decisions and current overlapping App/runtime source changes.
- The Execution Fabric and Product Jobs design remains authoritative for product jobs,
  query snapshots, publication integration, and runtime product/cache behavior.
- The completed RunenInput I0 result remains `INTERNAL_BOUNDARY_REPAIR_FIRST`; this ADR
  does not invent `RunenInput` semantics or declare current mixed `InputState` App-core.
- Issue #281 remains authoritative for the unresolved broader Plan/semantic-programming
  architecture. A future logical Plan may lower into ordinary App composition but must
  not become a second live App runtime.
- RunenRender, RunenUI, RunenECS, RunenNet, and other owners retain their reusable
  semantics.

## Consequences

- App-owned runtime state becomes smaller and more defensible.
- Owner-runtime containment becomes explicit without semantic ownership transfer.
- Headless execution no longer needs native-window-shaped sentinel state as a design
  requirement.
- Host selection and bounded advancement stop being one overloaded mode concept.
- Host stability prevents Startup under one host followed by silent continuation under a
  different host.
- Bounded advancement becomes repeatable without implying App termination.
- Startup failure semantics become explicit rather than relying on current runner
  destruction/exit to prevent reuse.
- Plugins become truthful installers/integration boundaries rather than labels over
  globally preactivated capability state.
- Fixed cadence can be separated from simulation identity.
- Capability APIs can be owner-local without conflating composition-time configuration
  and runtime control.
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

### Encode Host and Advancement as one mode enum

Rejected. Their invariants differ, even though compatibility between them must be
validated explicitly.

### Let bounded runners silently choose a different host

Rejected. Bounded execution controls advancement; it does not implicitly rewrite the
runtime's host identity.

### Leave Startup failure implicit in runner control flow

Rejected. Current consuming runners discard/exit on failure, but the lifecycle contract
must remain correct if a runtime survives a returned error or caught unwind. Failure must
not become retry eligibility by accident.

### Make every engine concern a plugin for symmetry

Rejected. Some Runenwerk application/integration invariants may genuinely be App-owned.
The qualification is semantic, not aesthetic.

### Persistent capability registry / service locator / generic DI

Rejected because it creates parallel runtime truth and weakens typed owner boundaries.

### Preserve rejected ownership through compatibility aliases

Rejected by default. Future implementation uses clean consumer migration and deletion
unless an independently proven compatibility contract requires otherwise.

## Implementation gate

This ADR authorizes no Rust/Cargo/runtime change by itself.

The companion accepted semantic-model design owns the exact-current source evidence,
disposition matrix, failure/admission details, and mechanical fitness tests.

After this architecture is accepted, derive exactly one first bounded implementation
issue from then-current `main` and then-current overlap. Do not pre-author a full
migration tree.

The active ADR-0022 publication-lifecycle cutover remains separately owned and must be
accepted or explicitly re-reconciled before App implementation edits overlapping runtime
surfaces.
