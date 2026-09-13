---
title: Runenwerk App Runtime and Composition Semantic Model
description: Normative normalized model for App composition, runtime containment, host realization, advancement, lifecycle occurrences, capability ownership, and current builtin-state disposition.
status: accepted
owner: engine
layer: architecture / runtime integration
canonical: true
last_reviewed: 2026-09-13
related_adrs:
  - ../../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
  - ../../adr/accepted/0018-semantic-federation-and-physical-realization.md
  - ../../adr/accepted/0019-batteries-included-application-composition.md
  - ../../adr/accepted/0022-runenwerk-owned-product-and-query-publication-phases.md
  - ../../adr/accepted/0023-normalize-app-runtime-host-lifecycle-and-capability-ownership.md
related_docs:
  - ../../architecture/runenwerk-platform-architecture.md
  - ./execution-fabric-and-product-jobs-design.md
  - ../../engine/reference/architecture.md
---

# Runenwerk App Runtime and Composition Semantic Model

## Status and role

This document is the detailed normative model under ADR 0023 and ADR 0019.

It owns current-source classification, disposition pressure, failure/admission semantics,
and mechanical fitness rules for later App/runtime cleanup. It does **not** claim current
Rust already conforms and does not select final Rust type names for every concept.

Current code/tests remain the authority for current behavior. Where this design differs
from current behavior, the difference is an implementation gap requiring a separately
accepted delivery issue.

The current-source census below was revalidated against accepted `main`:

```text
423eb85dbd46ce1106c725784775183c1663f02f
```

## 1. Exact current-source findings

The normalized design is driven by concrete current behavior rather than directory names.

### 1.1 Current `App` mixes independent roles

Current `App` stores:

```text
runen_ecs::World
runen_ecs::Runtime
Box<dyn AppRunner>
startup_ran
AppMode(Windowed | Headless)
title
winit::event_loop::ControlFlow
```

It also exposes both core composition operations and capability-specific scene, render,
input, simulation, replay, pacing, and direct-World operations.

### 1.2 Current construction installs broad state

`App::install_builtin_resources()` currently installs or initializes:

```text
Time
InputState
WindowState
FramePacingPolicyResource
FramePacingRuntimeStateResource
WindowStateRegistryResource
PlatformWindowEventQueueResource
NativeWindowHookRegistryResource
SceneCatalog
StartupState
SceneRuntimeState
UiOverlayState
GameplayRuntimeConfig
FixedTimeConfig
CatchupBudget
FixedTimeState
SimulationTick
ProductPublicationRuntimeResource
QuerySnapshotRuntimeResource
RuntimeJobExecutorResource
RuntimeProductCacheResource
SimulationProfileConfig
SimulationSessionId
SimulationSeed
SimulationRng
```

and installs the built-in product/query publication handlers.

Bootstrap placement is implementation evidence, not semantic ownership.

### 1.3 Current host and advancement semantics are conflated

`App::new()` records `AppMode::Windowed`; `App::headless()` records
`AppMode::Headless`. However:

```text
run_for_frames(...)
run_for_ticks(...)
```

always route through `run_headless()` regardless of stored mode, then return the same
`App` value to the caller on success.

Therefore current mode, host preparation, and bounded advancement are not one coherent
semantic axis.

### 1.4 Bounded advancement is demonstrably repeatable on success

`run_for_frames` and `run_for_ticks` consume and return `Self` on success. A caller can
receive the same runtime back and advance it again. `startup_ran` prevents successful
Startup from running again.

Therefore **normal bounded completion cannot semantically mean App termination**.

### 1.5 Current Startup state records success, not failed-attempt identity

`run_startup_if_needed` sets `startup_ran = true` only after the Startup schedule
returns success. A returned error leaves the boolean indistinguishable from
"Startup was never attempted."

Current public bounded-run helpers consume `App` and do not return it on failure, while
the current windowed runner records a fatal error and exits the event loop on Startup
failure. Therefore current source does **not** establish a maintained public same-instance
Startup retry path.

The architectural gap is narrower but real: lifecycle state itself cannot represent a
failed Startup attempt and therefore relies on surrounding runner destruction/exit to
prevent reuse. A future/specialized caller that retained the runtime after a returned
error, or a caller that caught an unwind around Startup execution, would need an explicit
failed/non-runnable state rather than interpreting `startup_ran == false` as retry
eligibility.

### 1.6 Current bounded tick advancement leaks simulation/timing ownership

`FixedTicksRunner` directly reads:

```text
SimulationTick
FixedTimeConfig
Time
```

and mutates `Time::delta_seconds` to drive progress.

That proves the current runner is not a neutral App advancement abstraction: it borrows
simulation identity and timing state from other semantic owners.

### 1.7 Current headless preparation still uses window-shaped state

`prepare_world_for_run` mutates `WindowState` to represent both headless and windowed
run preparation. Bare App bootstrap also installs the native window registry, platform
event queue, native-window hooks, and Winit pacing state for headless construction.

The normalized model must not require synthetic native-window state as the marker for a
headless host.

### 1.8 Some plugins are nominal rather than truthful activators

For example, `ScenePlugin` initializes `SceneRuntimeState`, `GameplayRuntimeConfig`, and
`UiOverlayState` that bare App bootstrap already installs.

Likewise current fixed/time/input plugin boundaries do not fully align with actual state
activation. A plugin name is therefore not ownership proof.

### 1.9 Publication is already separately governed

ADR 0022 and issue #591 own the active Product/Query publication lifecycle cutover.
That work preserves/refines narrow publication handler registration and currently keeps
built-in handler registration in App construction.

This design classifies semantic ownership without overriding #591's exact cutover.

## 2. Normalized top-level model

```text
Product / application intent
        |
        v
Application Composition
        |
        | selects/configures
        v
owner plugins / adapters / product behavior
        |
        v
+--------------------------------------------------------------+
|                            App                               |
|                                                              |
|  Runenwerk-owned application/integration lifecycle state     |
|                                                              |
|  contained owner runtimes / adapters                         |
|    current example: RunenECS World + Runtime                 |
|    owner semantics remain with the contained owner           |
|                                                              |
|  accepted composition                                        |
+--------------------------------------------------------------+
             |                                  |
             | hosted by                        | advanced by
             v                                  v
           Host                         Advancement Policy
             |                                  |
             +------------------+---------------+
                                |
                                v
                        App lifecycle occurrences
                                |
                 +--------------+--------------+
                 |              |              |
                 v              v              v
            owner adapter   owner plugin   product behavior
                 |              |              |
                 v              v              v
            owner-specific semantic/runtime state
```

The critical laws are:

```text
composition != semantic ownership
containment != semantic ownership
host != advancement policy
lifecycle occurrence != owner state
resource presence != authority
schedule position != domain ownership
```

## 3. Semantic roles

### 3.1 `App`

`App` is the one live Runenwerk application/runtime composition root.

It may:

- accept explicit composition operations while configuring;
- own genuine Runenwerk application/integration lifecycle state;
- contain/invoke selected owner runtimes and adapters;
- connect accepted composition to a selected Host and Advancement Policy;
- invoke Runenwerk-owned lifecycle integration;
- expose typed owner integration without taking foreign semantic ownership.

It is not:

- a universal semantic object database;
- an application-domain model;
- a replacement for owner frameworks;
- a generic service container;
- a persistent capability registry;
- a product preset database;
- a universal executor;
- the semantic owner of a runtime merely because it stores it.

### 3.2 Application Composition

Application Composition selects and configures what participates in one App instance.

Examples:

```text
install owner/product plugin
register systems
insert owner configuration
install a cross-framework adapter
select Host configuration
select Advancement Policy
set host-neutral application metadata
```

Composition owns **selection/integration facts**, not the semantics of the selected
owner.

Ordinary plugin/capability topology is mutable only while configuring. Once runtime
preparation/start begins, it is stable unless a separately accepted dynamic-topology
design explicitly says otherwise.

### 3.3 Integration / Runtime Container

The live container may hold:

```text
A. App-owned Runenwerk integration/lifecycle state
B. contained foreign-owner runtime instances/handles
C. owner capability state installed in a shared runtime such as World
```

Those categories remain semantically distinct even when one Rust object exposes them.

Current `runen_ecs::World` and `runen_ecs::Runtime` are category B: Runenwerk contains
and invokes them; RunenECS owns ECS semantics.

### 3.4 Host

A Host realizes the runtime in an execution environment.

Current demonstrated host classes are conceptually:

```text
NativeWindowHost
HeadlessHost
```

These are semantic roles, not required public type names.

The current native-window host owns Runenwerk's Winit realization, native-window
lifecycle, platform/window event delivery, redraw/event-loop policy, native-window hooks,
surface-host integration, and current Winit pacing behavior.

A headless host is a host with no native-window lifecycle. It is not represented by fake
native-window state merely to satisfy APIs designed around Winit.

### 3.5 Advancement Policy

Advancement Policy determines how execution opportunities are supplied, bounded, or
paced.

Current demonstrated needs include:

```text
native event-loop driven advancement
bounded N-frame advancement
bounded fixed-step-oriented proof advancement
```

A future generic cross-host pacing/throttling policy also belongs here if real consumers
prove it.

Advancement is not Host identity.

### 3.6 Owner Capability State

Owner Capability State belongs to the domain/framework/product/integration owner whose
invariants define it.

Examples include scene state, UI state, simulation state, native-window state,
publication state, product-job/cache state, networking state, and replay state.

Presence inside `World` or reachability through `App` does not transfer ownership.

### 3.7 Plugin

A `Plugin` is a Runenwerk composition/install mechanism.

A truthful plugin boundary means:

```text
plugin selected
    -> capability integration becomes installed/configured

plugin absent
    -> capability-owned state/systems are absent
       unless another explicit owner legitimately supplies the requirement
```

A plugin that only reinitializes already globally activated state is a boundary defect
unless the shared state is independently proven App-owned or explicitly supplied by
another owner.

### 3.8 Product/plugin group

A future Product/Plugin Group remains the ADR-0019 concept:

```text
ordered inspectable composition recipe
-> owner plugins/configuration
-> validation/admission
-> ordinary App composition
-> no independent runtime authority
```

Do not implement groups merely to hide current bootstrap debt.

## 4. App-owned runtime-state qualification

Every candidate bare App-owned runtime field/resource/integration state must pass all six
gates:

| Gate | Question |
|---|---|
| Ownership | Is the invariant genuinely Runenwerk App/runtime-integration semantics? |
| Universality | Does every currently supported App runtime path require it independent of optional capabilities and Host realization? |
| Independence | Is it meaningful with every optional domain/product capability absent? |
| Necessity | Would its absence invalidate App/runtime integration itself rather than one optional capability? |
| Authority safety | Does installation avoid manufacturing foreign/fake semantic authority? |
| Cost | Is universal installation independently acceptable? |

Failure of any gate means **not App-owned core runtime state**.

The test does not classify:

- host-neutral pre-run metadata such as a title;
- contained foreign-owner runtimes whose semantics remain with their owner;
- optional owner resources installed through composition.

## 5. Host/Advancement compatibility and stability

Host and Advancement Policy are independent semantic dimensions but not an unrestricted
Cartesian product.

Composition/preparation must reject unsupported combinations explicitly. Examples of
questions an implementation must answer include:

```text
Does this Host support externally bounded advancement?
Does this Advancement Policy require a host clock or event pump?
Does a selected capability require a native window before Startup?
Can this host realize the capability required by the product?
```

### 5.1 Host identity becomes stable before Startup

Once selected-host preparation or Startup begins, Host identity is stable for that
runtime instance unless a separately accepted host-transition contract explicitly allows
change.

Therefore a bounded runner must not silently implement itself by changing a Windowed
runtime into a Headless host, or vice versa.

Current `run_for_frames`/`run_for_ticks` routing through `run_headless()` regardless of
stored `AppMode` is predecessor behavior to normalize, not a target law.

### 5.2 Bounded advancement is repeatable

Normal bounded completion is an Advancement Policy result. It returns control without
terminating the live App.

Conceptually:

```text
Running
  -- advance N frames --> Running

Running
  -- advance until accepted bounded condition --> Running
```

A caller may advance the same started runtime again when the selected Host/Advancement
combination permits it.

### 5.3 Terminal host shutdown is different

Host/process termination follows the App terminal path and is not equivalent to a
bounded driver simply reaching its requested bound.

## 6. Composition and lifecycle state model

The conceptual lifecycle is:

```text
Configuring
    |
    | composition admitted / sealed
    v
Prepared
    |
    | selected host/runtime preparation succeeds
    v
Starting
    |
    | Startup succeeds once
    v
Running
    |
    | terminal host/process stop
    v
Terminating
    |
    v
Terminated
```

This is semantic vocabulary, not a required public enum.

### 6.1 Configuring

Allowed:

- plugins/groups;
- systems;
- owner resources/configuration;
- owner adapters;
- product declarations;
- Host/Advancement configuration;
- host-neutral application metadata.

Not authorized:

- normal runtime advancement while topology is still changing;
- arbitrary plugin add/remove after start;
- hidden autodiscovery that silently mutates composition.

### 6.2 Prepared

Prepared means the selected composition has been admitted far enough to begin concrete
host/runtime preparation.

The model does not require one universal validation registry or one monolithic error
type. Owners may validate through typed owner contracts; Runenwerk supplies integration
context and diagnostics.

### 6.3 Starting

Startup is a one-shot lifecycle attempt for one runtime instance.

Current public runners do not return the same App after Startup failure, so this design
does not claim a maintained public retry bug. The target nevertheless records a
**non-retry-eligible Startup-attempt state** before the first Startup system executes. If
a runtime survives a returned error or caught unwind, it cannot remain semantically
indistinguishable from "not attempted."

Conceptually:

```text
Prepared
  -> Starting / Attempting   # recorded before the first Startup system
      -> Running             # Startup succeeds
      -> Failed/NonRunnable  # Startup returns error or unwinds
```

Required failure law for a surviving runtime:

- no generic rollback is implied;
- already committed effects remain committed;
- Runenwerk does not implicitly retry Startup on the same surviving runtime;
- ordinary advancement of that surviving runtime must reject;
- recovery requires an explicit owner/host recovery contract or reconstruction.

### 6.4 Running

`Running` means Startup completed and the runtime is eligible for valid advancement.
It does not mean a host callback or bounded runner is continuously executing.

A successful bounded advancement operation leaves the App `Running`.

Owner-specific loading, warm-up, connection, readiness, residency, pause, and similar
state machines remain with their owners unless another accepted design says otherwise.

### 6.5 Terminating / Terminated

Terminal Host/process shutdown may transition through Terminating to Terminated.

This design does not create a universal `Shutdown` ECS schedule or teardown callback
registry.

## 7. Lifecycle occurrence is not owner state

Runenwerk may own **when** integration happens without owning the semantic meaning of
state participating there.

### Fixed step

```text
Runenwerk fixed-step occurrence
    -> simulation adapter may advance SimulationTick
    -> replay may capture fixed-step evidence
    -> world/product systems may execute fixed-step behavior
```

The occurrence and consumer state remain separate.

### Window/input

```text
Winit WindowEvent / DeviceEvent
    -> Runenwerk native-host translation
    -> physical/device input boundary when accepted
    -> Runenwerk product mapping and/or RunenUI adapter
```

Native-window lifecycle does not become reusable physical-input authority.

### Rendering

```text
Runenwerk RenderPrepare / RenderSubmit lifecycle position
    -> renderer integration executes
```

The position does not become renderer semantic authority.

### Publication

ADR 0022 defines ProductPublication and QuerySnapshotPublication occurrences. Those own
placement/provenance; product/query payload semantics remain with their actual owners.

## 8. Fixed cadence, timing, and simulation

### 8.1 Fixed cadence

Runenwerk owns the application fixed-step lifecycle/cadence and invocation of
`FixedUpdate` according to accepted application lifecycle semantics.

The current `FixedStepPlugin` is not a truthful final boundary because fixed-step work is
already run by the frame lifecycle while bootstrap already installs its state.

A later implementation must choose one evidence-backed truth:

1. **Universal fixed cadence** — fixed cadence is proven App lifecycle; App owns only
   minimal Runenwerk cadence integration and the redundant plugin disappears.
2. **Selectable fixed cadence** — a real fixed-step capability activates cadence and its
   Runenwerk-local state; bare App does not silently run it when absent.

### 8.2 Simulation identity

These are simulation semantics, not App fixed-step identity:

```text
SimulationTick
SimulationProfileConfig
SimulationSessionId
SimulationSeed
SimulationRng
```

Simulation integration may observe a Runenwerk fixed-step occurrence and advance its own
identity. The occurrence does not acquire that identity.

### 8.3 Bounded tick-oriented advancement

Current `FixedTicksRunner` directly uses `SimulationTick`, `FixedTimeConfig`, and `Time`.
That is a concrete ownership leak.

A future bounded App advancement primitive must use a Runenwerk-owned advancement
predicate/occurrence count or explicitly be a **simulation-owned** convenience. It must
not make simulation tick identity the universal App runner contract.

### 8.4 Time

Current `TimePlugin` installs a system while bare App construction installs the `Time`
resource.

The target separates:

```text
Host/Advancement progression facts
!=
system-visible Time projection/service
```

If `TimePlugin` survives, it must truthfully own the resource/system behavior it exposes.
If a minimal timing state is later proven App-owned, that proof must satisfy the App-owned
runtime-state qualification gates rather than relying on current usage.

## 9. Host state normalization

The following current resources are native-host realization facts, not universal App
state:

```text
winit::ControlFlow
WindowState
WindowStateRegistryResource
PlatformWindowEventQueueResource
NativeWindowHookRegistryResource
FramePacingPolicyResource
FramePacingRuntimeStateResource
```

Current frame pacing is Winit/event-loop specific. A later generic cross-host pacing
policy, if demonstrated, would be Advancement Policy instead of an automatic expansion
of these host resources.

Headless construction must not require native-window sentinels, window registries, Winit
control flow, native-window hooks, or Winit pacing state merely to mark itself headless.

Host-neutral application metadata such as title may remain composition/configuration and
be projected by a host into its own native state.

## 10. Input boundary

Preserve the accepted RunenInput I0 result:

```text
INTERNAL_BOUNDARY_REPAIR_FIRST
```

The App model therefore records only this ownership split:

```text
platform/window lifecycle       Runenwerk Host
physical/device input facts     separate input-boundary repair
product action mapping          Runenwerk/product
UI interaction semantics        RunenUI
```

Current mixed `InputState` must not be promoted to App-owned core state merely to
preserve bootstrap/tests.

## 11. Scene, product, UI, and readiness state

### SceneCatalog

Scene registration/catalog state exists because the Scene capability is selected.
Target owner: Scene integration/plugin.

### SceneRuntimeState

Scene runtime state exists because the Scene capability is selected.
Target owner: Scene integration/plugin.

### GameplayRuntimeConfig

Gameplay policy is product/game integration state, not universal App state. Its final
owner must follow then-current product/world/scene consumers.

### UiOverlayState

UI overlay state is UI/render/product integration state. Its final owner follows
then-current UI/render integration authority.

### StartupState

Current `StartupState` models product/render readiness from renderer warm frames and
timeout behavior. It is not App Startup lifecycle.

It should be renamed/reowned when its consumer slice is active so its name cannot imply
application lifecycle authority.

## 12. Publication and execution-fabric boundary

ADR 0022, issue #591, and the accepted Execution Fabric design already own this area.
This model must not create competing authority.

### Publication lifecycle

`ProductPublication` and `QuerySnapshotPublication` are Runenwerk lifecycle classes under
ADR 0022. Product/query payload semantics remain owner-local.

Current predecessor types such as:

```text
ProductPublicationRuntimeResource
QuerySnapshotRuntimeResource
PublicationBoundary
```

are implementation evidence under #591, not types frozen by this design.

### Publication handler registry

#591 explicitly retains/refines the narrow product/query handler-registration mechanism
and keeps built-in handler registration during App construction as part of deterministic
registration order.

That is valid Runenwerk integration ownership. It does not make product/query payloads
App-owned. App cleanup must not independently move or redesign this mechanism while #591
owns the cutover.

### RuntimeJobExecutorResource

Execution Fabric owns executor semantics. Current bare-App placement does not prove that
executor installation is universally App-owned or optional. Final installation requires
a then-current Execution Fabric consumer census.

### RuntimeProductCacheResource

Execution Fabric/product integration owns cache semantics. Final installation placement
must follow current consumers rather than bootstrap history.

## 13. Capability-specific App API and temporal contracts

Current inherent `App` APIs mix several roles:

```text
core/integration-shaped
  add_plugin / add_plugins
  add_systems
  init_resource / insert_resource
  set_runner
  world / world_mut

capability-specific configuration/authoring
  add_input_bindings
  add_render_flow
  set_simulation_profile
  set_authority_role
  set_simulation_seed
  add_scene / add_scene_template

capability-specific runtime control/query
  update_render_debug_*
  start/stop/load/seek replay
  current_tick
```

The normalized rule is:

> **Convenience is owned by the capability whose semantics it manipulates, and every
> operation retains an explicit temporal contract.**

Owner-specific extensions over the same App/runtime are a preferred direction when they
improve discoverability:

```text
AppSceneExt
AppRenderExt
AppSimulationExt
AppReplayExt
AppInput/ProductActionExt
```

Names are illustrative only.

Moving an operation to an owner extension must not silently reclassify it as pre-run
composition. Owner APIs must distinguish as applicable:

```text
composition/configuration command
runtime command
runtime query
runtime diagnostic/control operation
```

Whether a specific operation is legal before or after Startup follows its owner contract.

`world()`/`world_mut()` remain current expert escape hatches. This design does not decide
their final exposure; any restriction requires a separate current-consumer review.

## 14. Composition admission and inspection

A correct composition path rejects invalid assembly deterministically where the failure
is knowable before Startup.

Potential invalid classes include:

```text
required owner capability absent
illegal duplicate plugin/capability
mutually incompatible Host/Advancement combination
owner configuration invalid
required Host capability unavailable
cross-owner adapter prerequisite unsatisfied
```

The model does not require a universal `CompositionError` type, registry, or service
locator. Diagnostics may compose owner-local errors with Runenwerk integration context.

Future tooling may expose a **derived** effective-composition view containing facts such
as:

```text
selected plugins/groups
expansion/installation order
owner configuration identity/summary where meaningful
selected Host
selected Advancement Policy
rejected/duplicate/incompatible selections
```

That view is read-oriented derived state, not mutable runtime authority.

## 15. Failure semantics

Failure behavior is part of the boundary contract.

### 15.1 Composition rejection

Where invalid composition is known before preparation/Startup, reject there with
owner-aware diagnostics.

Do not intentionally defer a missing required capability until a missing-resource panic.

### 15.2 Host preparation failure

Host preparation failure belongs to the Host/integration boundary. Cleanup and retry are
allowed only according to that Host's explicit contract. This design introduces no
generic transaction or rollback.

### 15.3 Startup failure

Current consuming runners discard/exit on Startup failure; this model does not claim a
maintained public same-instance retry path. The normalized contract is nevertheless
explicit for any runtime instance that survives the failure:

```text
record Attempting/non-retry-eligible before executing Startup systems
-> success: Running
-> returned error / caught unwind: Failed/NonRunnable
```

No automatic retry and no generic rollback. Failure semantics must not depend on runner
destruction or event-loop exit for correctness.

### 15.4 Runtime owner/system failure

Owner/system/runtime failures retain their accepted owner semantics. The current
advancement call must surface failure; this design does not declare every runtime error
terminal and does not invent generic rollback/retry.

A retry/recovery path exists only when the relevant owner/Host contract defines it.

### 15.5 Bounded completion

A bounded driver reaching its requested bound is success-shaped Advancement Policy
completion and leaves the live App `Running`. It is not App termination or Host shutdown.

## 16. Current field/resource disposition matrix

This matrix is normative ownership pressure, not permission to move every row in one PR.

| Current item | Normalized class | Target disposition |
|---|---|---|
| `World` | contained RunenECS runtime | retain current containment until separately redesigned; ECS semantics remain RunenECS-owned |
| RunenECS `Runtime` | contained RunenECS runtime | retain current containment until separately redesigned; schedule semantics remain RunenECS-owned |
| `startup_ran` | predecessor App lifecycle marker | replace/refine only when implementing explicit attempting/failed/non-runnable lifecycle semantics; current consuming runners already discard/exit on Startup failure |
| `title` | host-neutral application metadata | composition/configuration; Host projects it as needed |
| `AppMode` | Host selection mixed with run mode | replace/refine around explicit Host selection |
| `AppRunner` | Advancement Policy realization | normalize independently from Host and foreign owner state |
| Winit `ControlFlow` | native-host policy | native-window Host |
| `WindowState` | native-host state | native-window Host only; no required headless sentinel |
| `WindowStateRegistryResource` | native-host state | native-window Host |
| `PlatformWindowEventQueueResource` | Host/platform integration | native-window/platform Host |
| `NativeWindowHookRegistryResource` | native-host integration | native-window Host |
| `FramePacingPolicyResource` | current Winit/event-loop pacing | native Host now; future generic cross-host pacing would be Advancement Policy |
| `FramePacingRuntimeStateResource` | current Winit/event-loop pacing state | same owner as current pacing policy |
| `Time` | timing projection/service | truthful Time owner/plugin unless separately proven App-owned |
| `FixedTimeConfig` | fixed cadence policy | App lifecycle or real fixed-step capability; separate implementation decision |
| `CatchupBudget` | fixed cadence policy | same fixed-cadence owner |
| `FixedTimeState` | fixed cadence runtime state | same fixed-cadence owner |
| `SimulationTick` | simulation identity | simulation integration owner |
| `SimulationProfileConfig` | simulation policy | simulation integration owner |
| `SimulationSessionId` | simulation identity | simulation integration owner |
| `SimulationSeed` | simulation input/identity | simulation integration owner |
| `SimulationRng` | simulation runtime state | simulation integration owner |
| `InputState` | mixed input/product state | separate input repair; not App-owned core state |
| `SceneCatalog` | scene capability | Scene plugin/owner |
| `SceneRuntimeState` | scene capability | Scene plugin/owner; remove duplicate bootstrap authority |
| `GameplayRuntimeConfig` | product/game policy | actual product/world/scene owner; not App-owned core state |
| `UiOverlayState` | UI/render/product integration | actual UI/render integration owner; not App-owned core state |
| current `StartupState` | product/render readiness | rename/rehome under readiness owner; not App lifecycle |
| current product/query publication resources | ADR-0022 predecessor integration state | exact replacement/removal/placement remains #591-owned |
| publication handler registries | Runenwerk publication integration | preserve/refine per #591; App registration does not transfer payload authority |
| `RuntimeJobExecutorResource` | Execution Fabric subsystem | Execution Fabric owns semantics; final installation universality needs consumer proof |
| `RuntimeProductCacheResource` | product execution/cache | Execution Fabric/product owner; final placement follows current consumers |

If an implementation census cannot resolve one final owner from this matrix and current
consumers, that row is not decision-complete enough to move.

## 17. Ordering and determinism

- composition/group expansion must be deterministic for the same explicit input;
- composition order does not substitute for semantic system ordering;
- Host/Advancement compatibility must be admitted explicitly;
- owner system ordering continues to use accepted owner/schedule contracts;
- lifecycle schedule order must not be reconstructed from physical executor order;
- effective-composition inspection reports actual accepted order rather than a separately
  maintained expected order.

## 18. Mechanical fitness tests for future implementation

A future clean cut must eventually prove the applicable subset below.

### Minimal/headless

- headless construction has no native-window sentinel state;
- no scene/UI/render/network/replay/simulation state appears merely because App exists;
- no Winit `ControlFlow` is App-owned core state;
- contained RunenECS runtime remains semantically RunenECS-owned.

### Host/Advancement

- Host and Advancement are represented/validated independently;
- unsupported Host × Advancement combinations reject clearly;
- bounded advancement does not silently switch Host;
- bounded completion leaves the live runtime eligible for later advancement;
- title/metadata projects into a window Host without ownership reversal.

### Lifecycle

- composition topology is sealed before preparation/Startup;
- if a runtime can survive Startup failure, a non-retry-eligible `Starting/Attempting`
  state is recorded before Startup systems execute;
- successful Startup executes once;
- returned error / caught unwind transitions any surviving runtime to a failed/non-runnable
  state rather than restoring retry eligibility;
- partial Startup effects are not claimed to be generically rolled back;
- terminal shutdown is distinct from bounded completion.

### Fixed/simulation/time

- App fixed cadence does not use `SimulationTick` as universal identity;
- simulation integration advances its own tick semantics from accepted lifecycle;
- `FixedStepPlugin` receives one truthful disposition;
- bounded App advancement is not defined by simulation identity unless the API is
  explicitly simulation-owned;
- Time is installed by its truthful owner.

### Scene/UI/input

- selecting Scene installs required scene state once;
- omitting Scene leaves scene state absent;
- gameplay/UI/readiness state is not globally manufactured by bare App;
- App cleanup does not bless mixed `InputState` as universal authority.

### Publication/execution fabric

- ADR 0022 and #591 behavior remains intact;
- App cleanup does not resurrect predecessor publication-boundary semantics;
- product/query payload authority is not transferred by handler placement;
- executor/cache placement changes only after Execution Fabric consumer proof.

### API/composition

- capability-specific APIs retain owner and temporal semantics;
- direct expert composition and future groups lower to the same owners;
- illegal duplicate/incompatible selection rejects deterministically;
- composition inspection is derived rather than separately authored;
- no compatibility alias is retained solely to preserve rejected ownership.

## 19. Migration doctrine

Implementation proceeds one semantic boundary at a time:

```text
re-resolve current main
-> census exact owner + consumers + tests
-> select one coherent boundary
-> reconcile active overlapping work
-> migrate owner state/Host/Advancement/API consumers atomically
-> delete predecessor ownership/aliases where the clean cut permits
-> focused tests + cargo validate + exact-head CI
-> accepted-main proof
-> only then derive the next slice
```

Do not pre-open the full migration tree. #591 may materially change the correct first
implementation boundary.

## 20. Explicit non-goals

This model does not authorize:

```text
RunenApp / RunenCore
universal capability registry
service locator / generic DI
live dynamic plugin runtime
universal lifecycle bus
second scheduler
Plan / AppProgram implementation or semantic database
product/plugin group implementation
RunenInput extraction
RunenRender or other framework semantic redesign
RunenECS semantic redesign/removal from current App
Cargo feature/binary modularity redesign
publication implementation owned by #591
compatibility forwarding layers
```

Issue #281 may later accept a logical Plan/programming model. If so, Plan may lower into
ordinary App composition but must not become a second live runtime or override the Host,
Advancement, lifecycle, containment, and owner-state boundaries defined here.

## Completion condition

This model is implementation-ready when a reviewer can classify any App field/resource,
contained runtime, lifecycle operation, or convenience API by answering, in order:

```text
Who owns the invariant?
Is App merely containing/invoking another owner?
Is it App-owned runtime integration state?
Is it Host realization?
Is it Advancement Policy?
Is this Host/Advancement combination valid?
Is it host-neutral composition/configuration metadata?
Is it only a lifecycle occurrence?
Which capability installs it?
At what lifecycle phase is the operation legal?
What failure/retry contract applies?
What state must be absent when the capability is absent?
What predecessor authority is deleted during migration?
```

If those questions do not yield one coherent ownership/disposition result, the
implementation slice is not decision-complete and must stop rather than preserve
ambiguity.
