---
title: Normalize App Runtime, Host, Lifecycle, and Capability Ownership
description: Durable application-runtime ownership laws separating Runenwerk App composition, runtime container state, host realization, advancement policy, lifecycle occurrences, and owner capability state.
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
accepted a target discipline: bare App construction should own only genuine
application-runtime invariants and cheap state required across supported App modes.
Capability/domain/product state should normally be installed by its owner.

That target was intentionally not implemented or fully classified by ADR 0019.
Current source still mixes several independent concerns in App construction and
execution:

- runtime container state (`World`, RunenECS `Runtime`);
- application lifecycle state;
- native-window/Winit host policy;
- bounded headless runner policy;
- fixed-step cadence and simulation identity;
- timing/input/scene/UI/gameplay state;
- product/query publication state;
- product-job executor/cache state;
- simulation profile/session/seed/RNG state.

Current `AppMode` and `AppRunner` also do not form one coherent semantic axis:
windowed execution is owned by the Winit path, while `run_for_frames` and
`run_for_ticks` use the headless runner path independently of the stored mode.

The defect is not merely that too many resources are installed in one function. The
current shape conflates **application composition, runtime containment, host
realization, advancement policy, lifecycle occurrences, and owner capability state**.
Moving resources mechanically would preserve the ambiguity.

ADR 0022 independently proves the same ownership principle for product/query
publication: Runenwerk may own an application lifecycle occurrence without thereby
owning the semantic state that participating domains publish at that occurrence.

This ADR establishes the missing durable model beneath ADR 0019. The detailed
current-source classification and implementation pressure are recorded in the
companion accepted semantic-model design.

## Decision

### 1. `App` remains the single live runtime composition root

Runenwerk uses exactly one live application/runtime composition root:

```text
App
```

`App` owns the assembly and execution of one Runenwerk runtime instance. It does not
become a semantic super-domain merely because owner plugins/resources/systems are
installed through it.

Rejected as parallel live authority:

```text
RunenApp runtime
persistent meta-application runtime
untyped service locator
hidden dependency-injection universe
generic capability database
second scheduler/meta-executor
```

Ephemeral helpers or future product/plugin groups may lower into ordinary `App`, owner
plugins, owner configuration, and resources exactly as allowed by ADR 0019. They do not
remain independent runtime truth.

### 2. Separate five semantic dimensions

Runenwerk normalizes application runtime around five distinct dimensions:

```text
Application Composition
    selected owners/capabilities/product behavior and configuration

Runtime Container
    live World + RunenECS Runtime + genuine App lifecycle state

Host
    environment realization: native-window/event-loop, headless process,
    or another separately proven host

Advancement Policy
    how the runtime is driven: host/event-driven, bounded frames,
    bounded fixed-step proof, or another separately accepted driver

Owner Capability State
    scene/render/UI/input/simulation/replay/network/product-execution/etc. state
```

Correspondence between these dimensions does not transfer semantic ownership.

In particular:

```text
host != advancement policy
lifecycle occurrence != domain state
composition root != semantic ownership
resource presence != capability authority
schedule position != domain ownership
```

### 3. Bare App state requires universal-invariant proof

A state item may be installed by bare App construction only when **all** of these are
true:

1. Runenwerk application/runtime semantics own the invariant.
2. Every valid supported App host/runtime requires it.
3. It remains meaningful when every optional domain/product capability is absent.
4. Its absence would make the App runtime itself invalid rather than merely disable a
   capability.
5. Installing it does not manufacture foreign semantic authority or a fake host or
   capability.
6. Universal installation has independently acceptable cost.

Convenience, historical placement, a broad prelude, or a current consumer that assumes
the resource exists are not evidence for bare-App ownership.

When this test fails, the state belongs to its owner plugin, host integration, product,
or another explicit integration capability.

### 4. Composition is explicit pre-run assembly

The ordinary application model is:

```text
construct App
-> install/configure owner capabilities and product behavior
-> admit/finalize composition
-> prepare host/runtime
-> start
-> run
```

Plugin/resource/system installation is pre-run composition. This ADR does not authorize
arbitrary live plugin addition/removal after startup. A future need for dynamic runtime
capability topology requires a separate semantic design.

Composition order is deterministic. Cross-plugin requirements must be explicit and
diagnosable rather than depending on accidental global builtins or registration order.

Illegal duplicate/incompatible capability selection must reject explicitly. Multiple
instances are allowed only where the owner contract actually defines multiple-instance
semantics.

### 5. A plugin installs capability integration; it is not semantic authority

A Runenwerk `Plugin` is a composition/install mechanism over `App`.

The semantic owner of a capability remains the domain/framework/product that defines its
invariants. A plugin representing that capability must install/configure the state,
systems, and adapters the capability requires unless a dependency is independently
proven to be App-core.

A nominal plugin must not coexist indefinitely with globally pre-activated state that
already makes the plugin's capability present. Such cases require a clean disposition:
make the plugin the real activator/owner, or remove the meaningless plugin boundary when
the capability is genuinely universal App lifecycle.

### 6. Host realization is not App-domain state

A Host adapts the runtime to an execution environment.

The native-window host owns Runenwerk's Winit/event-loop integration, including as
applicable:

```text
native window lifecycle
window creation/destruction
platform/window event delivery
Winit ControlFlow
redraw policy
frame pacing
native-window hooks
surface-host integration
```

These are Runenwerk-owned integration semantics, but they are **not universal App
state**.

A headless host must not require synthetic native-window state, a native-window registry,
Winit `ControlFlow`, native-window hooks, or redraw/frame-pacing state merely to satisfy a
window-shaped API.

Application metadata such as a product title may remain application configuration; a
window host may project it into native-window state.

Winit is a contained host realization, not the normalized App semantic model.

### 7. Advancement policy is orthogonal to host

Host answers **where/how the runtime is hosted**. Advancement policy answers **how the
runtime is advanced**.

These must not be encoded as one mode axis.

A runtime may be advanced through a native event loop, a bounded-frame test/proof driver,
a bounded fixed-step proof driver, or another accepted driver without transferring host
state or domain ownership.

The current `AppMode`, `AppRunner`, `run_for_frames`, and `run_for_ticks` are
implementation evidence only. Future implementation may replace or refine them, but it
must preserve the semantic split.

A proof/test driver must not implement bounded execution by pretending a windowed host
became a headless window state.

### 8. App lifecycle and owner lifecycle are distinct

The normalized conceptual App lifecycle is:

```text
Configuring
    -> Prepared / composition admitted
        -> Starting
            -> Running
                -> Terminating
                    -> Terminated
```

This is a semantic model, not a requirement for one public Rust enum.

Required laws:

- configuration/installation happens before runtime start;
- host/runtime preparation happens before normal execution;
- the App Startup schedule executes at most once for one runtime instance;
- runtime lifecycle occurrences happen only while the runtime is in an appropriate
  running transition;
- termination is explicit even when a concrete host does not return normally;
- this ADR does not create a universal lifecycle event bus or require a new `Shutdown`
  ECS schedule.

Owner-specific readiness/state machines remain separate. A renderer warm-up/readiness
state named `StartupState`, for example, is not App Startup lifecycle merely because of
its name.

### 9. Lifecycle occurrence does not acquire foreign semantic authority

Runenwerk owns application integration/lifecycle points where already accepted, including
frame/fixed/render integration and ADR 0022 publication occurrences.

That ownership does not make state observed or advanced there Runenwerk App-core state.

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

Owner adapters/plugins translate an application lifecycle occurrence into owner-specific
state transitions where required.

### 10. Fixed cadence and simulation identity are separate

Runenwerk owns application fixed-step lifecycle/cadence and invocation of the accepted
`FixedUpdate` application lifecycle position.

`engine_sim::SimulationTick`, simulation profile, authority role, session identity,
seed, and RNG are simulation semantics. They are not universal App invariants solely
because current fixed-step code reads or advances them.

The clean target is:

```text
Runenwerk fixed-step cadence / occurrence
        |
        +--> simulation integration may advance SimulationTick
        +--> other fixed-step consumers may react under their own semantics
```

This ADR deliberately does not choose the final Rust representation of fixed cadence.
A later implementation must decide whether the current `FixedStepPlugin` becomes the
real activator of optional fixed-step machinery or disappears because the fixed cadence
is proven universal App lifecycle. It may not remain a nominal plugin duplicating
already-active global state.

### 11. Host progression and `Time` resource authority are separate

The Host/Advancement Policy determines when runtime occurrences happen.

A `Time` resource exposed to systems is a runtime timing projection/service. It does not
become the authority over host progression merely because systems read it.

If `TimePlugin` remains, it must truthfully own the resource/system behavior it exposes,
unless a later classification independently proves the `Time` state satisfies the
universal App-core test.

### 12. Input remains separately owned and separately repaired

The completed RunenInput I0 investigation concluded `INTERNAL_BOUNDARY_REPAIR_FIRST`.
This ADR preserves that result.

Runenwerk continues to own window/event-loop lifecycle and product integration. Physical
device facts, application action mapping, and RunenUI interaction semantics remain
separate ownership questions.

This ADR does not authorize a `RunenInput` repository/API or declare today's mixed
`InputState` to be App-core.

### 13. Publication and product-job semantics keep their accepted owners

ADR 0022 remains authoritative for `ProductPublication` and
`QuerySnapshotPublication` lifecycle semantics. The Execution Fabric and Product Jobs
design remains authoritative for product-job execution, query snapshots, publication,
and runtime product/cache behavior.

Those are Runenwerk integration capabilities, not therefore universal bare-App state.
They must be installed where their maintained semantic producer/consumer paths require
them.

This ADR must not interfere with the active ADR-0022 implementation cutover.

### 14. One App root does not require one giant inherent API

Capability-specific authoring may use owner-specific extension APIs over the same App
root.

Directionally:

```text
App core API
    composition/runtime-container/lifecycle primitives

owner App extensions
    scene authoring/configuration
    render flows/debug configuration
    input/action mapping
    simulation configuration
    replay control
    other owner-local integration
```

Exact trait/type names are not selected here.

Moving capability-specific operations out of inherent `App` methods does not create a
second runtime. It narrows the semantic surface while preserving ergonomic progressive
disclosure.

### 15. Effective composition may be inspected, but inspection is derived state

Runenwerk may expose a read-oriented effective-composition projection containing facts
such as selected plugins/groups, deterministic installation order, owner configuration
identity where meaningful, and rejected/incompatible selections.

That projection is diagnostics/tooling evidence, not a mutable semantic registry. It
must derive from the actual composition path and must not become a second source of
runtime truth.

## Consequences

Bare App construction becomes intentionally smaller and more stable. Optional domains
and product capabilities become absent by default unless selected through their owner or
a transparent product composition recipe.

Headless applications stop carrying native-window-shaped sentinel state merely because
windowed execution exists elsewhere.

Application lifecycle becomes easier to reason about because Host, Advancement Policy,
fixed cadence, simulation identity, and product/domain state no longer masquerade as one
mode/runtime object.

Plugins become truthful capability installers rather than labels placed over globally
preinstalled state.

Future batteries-included product groups become safer to implement because they compose
real owner boundaries instead of wrapping global App bootstrap debt.

This decision may require breaking cleanup of current convenience APIs/tests. Preserve
behavior only when the behavior remains semantically valid; do not retain rejected
ownership through aliases, synthetic sentinel state, or compatibility mirrors.

## Relationship to broader Plan work

Issue #281 owns the unresolved Runen semantic Plan/programming architecture.

This ADR neither accepts nor rejects that Plan direction. If Plan is accepted later, an
application Plan may lower into the ordinary `App` composition/runtime root as already
allowed by ADR 0019. It must not become a second live App runtime or override these host,
lifecycle, and owner-state boundaries.

## External pressure

Current Bevy demonstrates that one App can provide plugin and plugin-group composition
while complex capabilities remain plugin-installed. Runenwerk borrows only the product
pressure that internal decomposition need not leak into ordinary application setup; it
does not adopt Bevy's ECS-centered ownership model.

Current Winit exposes `ControlFlow` and native lifecycle through its event-loop APIs and
`ApplicationHandler`. That supports treating Winit policy as a host realization rather
than universal App-domain state. Runenwerk retains its own host/lifecycle semantics.

## Rejected alternatives

### Keep broad App builtins and document them as core

Rejected. Current placement includes scene, UI/gameplay, simulation, publication,
product-job, input, and native-window state whose owners are not universal App semantics.
Documentation cannot turn incidental bootstrap placement into authority.

### Move each resource to a nearby plugin without a semantic model

Rejected. This would preserve deeper conflations such as fixed cadence versus simulation
tick, host versus runner, and product readiness versus App Startup lifecycle.

### Make all engine features plugins mechanically

Rejected as a universal rule. Some lifecycle/container invariants may genuinely belong
to App core. The qualification rule is semantic ownership and universality, not a desire
for symmetrical APIs.

### Persistent capability registry / DI container

Rejected. It creates parallel runtime truth and weakens typed owner boundaries.

### Preserve old APIs through compatibility aliases

Rejected by default. The eventual implementation should use clean cuts after consumer
migration unless a separately proven external compatibility requirement exists.

## Implementation gate

This ADR authorizes no Rust change by itself.

The companion semantic-model design records the exact current builtin/API disposition
pressure and mechanical fitness tests. After this architecture is accepted, derive
exactly one first bounded implementation issue from then-current `main` and active
source overlap.

Do not pre-author the full migration tree from this decision branch.

Active publication-lifecycle work remains separately owned and must be accepted or
explicitly reconciled before an App implementation slice edits overlapping runtime
surfaces.
