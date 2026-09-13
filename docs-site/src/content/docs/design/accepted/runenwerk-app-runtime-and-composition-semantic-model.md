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

It defines the semantic categories and fitness rules that future App/runtime cleanup must
preserve. It is **not** a statement that current Rust already conforms, and it does not
select final Rust type names for every concept.

Current code/tests remain the source of truth for current behavior. Where this design
and current implementation differ, the difference is an implementation gap requiring a
separately accepted delivery issue.

## Scope

This model answers:

```text
What is App?
What may bare App construction own?
What is composition rather than runtime authority?
What belongs to a host?
What is advancement policy?
What is an App lifecycle occurrence?
How do owner-specific capabilities participate without transferring authority?
How should current builtin state be classified before cleanup?
What must future product/plugin groups lower into?
```

It does not define a universal Runen semantic model, Plan IR, generic DI framework,
service locator, dynamic plugin runtime, new scheduler, compile-time feature topology,
or the reusable semantics owned by peer frameworks.

## 1. Normalized top-level model

The target mental model is:

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
+-------------------------------------------------------+
|                         App                           |
|                                                       |
|  Runtime Container                                   |
|    - World                                            |
|    - RunenECS Runtime                                 |
|    - genuine App lifecycle state                      |
|                                                       |
|  accepted composition                                |
+-------------------------------------------------------+
            |                              |
            | hosted by                    | advanced by
            v                              v
          Host                      Advancement Policy
            |                              |
            +---------------+--------------+
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

The critical rule is:

> **Composition, hosting, advancement, lifecycle occurrence, and domain/framework state
> are different semantic roles even when one Rust object currently contains several of
> them.**

## 2. Semantic roles

### 2.1 App

`App` is the one live Runenwerk application/runtime composition root.

Its semantic responsibilities are limited to:

- containing the live RunenECS `World` used by this runtime instance;
- containing the live RunenECS `Runtime`/schedule registration used by this runtime
  instance;
- accepting explicit pre-run composition operations;
- owning genuine Runenwerk App lifecycle state;
- connecting the accepted composition to a selected host and advancement policy;
- invoking Runenwerk-owned lifecycle integration according to accepted architecture;
- exposing owner integration without taking owner semantics.

`App` is not:

- a universal semantic object database;
- an application-domain model;
- a replacement for owner frameworks;
- a generic service container;
- a capability registry whose entries themselves determine semantics;
- a product preset database;
- a universal executor.

### 2.2 Application Composition

Application Composition is the pre-run selection/configuration of owner capabilities and
product behavior.

Examples include:

```text
add owner plugin
add product plugin
insert owner configuration
register owner systems
install a cross-framework adapter
select a host configuration
select an advancement policy
```

Composition is authoritative only for **which integration has been selected for this App
instance**. It does not acquire the semantics of the selected owner.

### 2.3 Runtime Container

The Runtime Container is the live state needed to execute the accepted App composition.

The minimal demonstrated container is:

```text
World
RunenECS Runtime
App lifecycle state
```

Additional state enters the container through its semantic owner. Presence in `World`
does not make a resource App-owned.

### 2.4 Host

A Host realizes the App runtime in an execution environment.

Current demonstrated host classes are:

```text
NativeWindowHost
HeadlessHost
```

These names are conceptual. They do not require matching public Rust types.

The native-window host owns Runenwerk's Winit realization, native-window lifecycle,
platform/window event delivery, redraw policy, frame pacing, and event-loop control.

The headless host owns the absence of native-window lifecycle. It is not a window host
with synthetic sentinel windows.

A future host class must be justified by a real environment/consumer. This design does
not create a generic host-plugin framework in advance.

### 2.5 Advancement Policy

Advancement Policy decides how execution opportunities are supplied to the App runtime.

Current demonstrated needs include:

```text
native event-loop driven execution
bounded N-frame execution for tests/proofs/headless tools
bounded fixed-step/tick-oriented proof execution
```

Advancement is distinct from Host. A test driver controlling how many runtime
occurrences happen does not redefine the host's semantic environment.

### 2.6 Plugin

A Runenwerk `Plugin` is a composition action that installs/configures a capability or
integration in the App runtime.

A plugin is not automatically the semantic owner of the values it installs. Semantic
ownership follows the invariant owner.

A truthful plugin boundary has these properties:

```text
plugin selected
    -> capability integration becomes installed/configured

plugin absent
    -> capability-owned state/systems are absent
       unless another explicit owner legitimately supplies the same requirement
```

A plugin that only reinitializes state already universally activated elsewhere is a
boundary defect unless the duplicated state is independently proven App-core.

### 2.7 Product/Plugin Group

A Product/Plugin Group is an ephemeral deterministic composition recipe accepted by ADR
0019.

Conceptually:

```text
Group
  -> ordered owner plugin/configuration selection
  -> validation/admission
  -> ordinary App composition
  -> no independent runtime authority remains
```

The future group mechanism may support only proven requirements such as deterministic
membership/order, legal replacement/removal/configuration, composition, and useful
incompatibility diagnostics.

### 2.8 Owner Capability State

Owner Capability State is state whose invariant belongs to a selected domain/framework,
product, or Runenwerk integration capability rather than universal App runtime.

Examples include scene state, UI state, simulation state, native-window state,
publication state, product-job execution/cache state, networking state, and replay
state.

The owner installs and validates such state directly or through a Runenwerk adapter.

## 3. Core-App qualification test

Every candidate bare-App field/resource must pass all six gates:

| Gate | Question |
|---|---|
| Ownership | Is the invariant genuinely Runenwerk App/runtime semantics? |
| Universality | Does every supported App host/runtime require it? |
| Independence | Is it meaningful with every optional capability absent? |
| Necessity | Would its absence invalidate App itself rather than one capability? |
| Authority safety | Does installation avoid manufacturing foreign/fake semantic authority? |
| Cost | Is universal installation independently acceptable? |

Failure of any gate means **not App-core**.

This test is deliberately stricter than "many products use it" and stricter than
"current code expects it".

## 4. Composition state machine

The conceptual App lifecycle is:

```text
Configuring
    |
    | composition admitted / sealed for ordinary execution
    v
Prepared
    |
    | host/runtime preparation succeeds
    v
Starting
    |
    | Startup schedule succeeds exactly once
    v
Running
    |
    | host termination / bounded driver completion / fatal error
    v
Terminating
    |
    v
Terminated
```

### Configuring

Allowed:

- plugins/groups;
- systems;
- owner resources/configuration;
- owner adapters;
- product declarations;
- host/advancement configuration.

Not authorized by this model:

- executing normal runtime schedules while composition is still changing;
- arbitrary live plugin add/remove after start;
- hidden autodiscovery that mutates composition behind the application.

### Prepared

Prepared means the selected composition and host/runtime prerequisites have passed the
required admission checks for startup.

The model does not require one monolithic `prepare()` function or one validation
registry. Owners may validate through their normal typed contracts. Runenwerk must make
composition failures diagnosable at the integration boundary.

### Starting

Runenwerk executes App Startup at most once per runtime instance.

Product/domain "startup", loading, warm-up, readiness, connection establishment, or
resource residency are separate owner state machines unless an accepted design says
otherwise.

### Running

Running contains lifecycle occurrences such as frame/update/fixed/render/publication
integration according to their accepted owners and placements.

A lifecycle occurrence is a point at which owner integration may run. It is not a claim
that all state touched there is App-owned.

### Terminating / Terminated

The App runtime has stopped accepting normal advancement. Concrete host behavior may
differ by platform; for example an event loop may not return normally on every target.

This model defines the semantic terminal transition without pre-authorizing a universal
`Shutdown` schedule or global teardown callback registry.

## 5. Lifecycle occurrences and authority

The central normalization law is:

> **Runenwerk may own when integration work occurs without owning the semantic meaning
> of the state that participates.**

### Fixed-step example

```text
Runenwerk fixed-step occurrence
    -> simulation adapter may advance SimulationTick
    -> replay may capture fixed-step evidence
    -> world systems may execute fixed-step behavior
```

The occurrence and each consumer's semantic state remain separate.

### Window/input example

```text
Winit WindowEvent / DeviceEvent
    -> Runenwerk native-host translation
    -> neutral/device facts when/if the accepted input repair provides them
    -> Runenwerk product action mapping and/or RunenUI adapter
```

Native-window lifecycle does not become physical-input semantic authority.

### Rendering example

```text
Runenwerk RenderPrepare / RenderSubmit lifecycle position
    -> renderer integration executes
```

The schedule position does not become renderer scene/material/lighting/image-formation
semantic authority.

### Publication example

ADR 0022 defines explicit ProductPublication and QuerySnapshotPublication occurrences.
Those lifecycle classes own placement/provenance, while product/query semantics remain
with their actual owners.

## 6. Host model

### Native-window host

The following current concepts are host realization facts and must not be treated as
universal App-core solely because they currently live in App/bootstrap/runtime:

```text
winit::ControlFlow
WindowState
WindowStateRegistryResource
PlatformWindowEventQueueResource
NativeWindowHookRegistryResource
FramePacingPolicyResource
FramePacingRuntimeStateResource
native window lifecycle state
redraw requests
focus/resize/scale-factor projection
```

The host owns creation, destruction, resumption/suspension translation where applicable,
window event dispatch, pacing, and platform constraints.

A host may expose Runenwerk-owned normalized integration facts. That does not require
Winit types in the core App semantic contract.

### Headless host

A headless runtime has no native-window authority unless an explicitly selected
capability creates an offscreen/native surface for its own purpose.

Bare headless construction therefore does not need a synthetic `WindowState`, registry,
native hook registry, Winit `ControlFlow`, or redraw/pacing state merely as placeholders.

### App metadata

Application/product metadata such as a title can remain application configuration.
Host adapters may project that value into host-specific concepts such as a primary
window title.

The projection does not reverse ownership.

## 7. Advancement model

### Event-driven advancement

The native-window host may drive runtime work in response to its event-loop/redraw
semantics.

The event loop is host realization. App lifecycle scheduling remains Runenwerk-owned.

### Bounded-frame advancement

Tests, proof runners, headless tools, and deterministic harnesses may advance a runtime
for a bounded number of frame occurrences.

The bound belongs to the advancement policy. It must not force native-window state to
pretend to be headless or vice versa.

### Bounded fixed-step advancement

A proof may need to run until a number of Runenwerk fixed-step occurrences or an
owner-specific simulation condition has been reached.

Those are distinct predicates. A generic App advancement driver should not use
`SimulationTick` as the universal App fixed-step identity.

## 8. Fixed-step and simulation normalization

Current implementation directly increments `engine_sim::SimulationTick` from the
Runenwerk fixed-step executor. That is a concrete ownership leak.

The normalized target is:

```text
Runenwerk
  owns fixed cadence and FixedUpdate invocation

Simulation integration
  owns SimulationTick / profile / authority / session / seed / RNG
  observes/adapts Runenwerk lifecycle as required
```

### FixedStepPlugin disposition gate

The current `FixedStepPlugin` cannot remain permanently in its current semantic state:
it initializes resources App construction already installs while the frame lifecycle
already runs fixed-step work unconditionally.

A later implementation must choose exactly one truth:

1. **Universal fixed cadence** — fixed cadence is proven App lifecycle, so App owns only
   the minimal Runenwerk cadence state and the redundant `FixedStepPlugin` is removed;
   owner-specific simulation state remains separate.
2. **Selectable fixed cadence** — fixed cadence itself is optional, so a real
   fixed-step capability/plugin activates the cadence and owns its Runenwerk-local state;
   App does not silently run the capability when absent.

The decision must come from then-current maintained consumers and this semantic model,
not from preserving the existing plugin name.

### Simulation state

These current builtins do not pass the App-core ownership gate merely by existing in
bootstrap:

```text
SimulationTick
SimulationProfileConfig
SimulationSessionId
SimulationSeed
SimulationRng
```

Their types and invariants belong to simulation integration and must be installed by the
correct simulation owner/path after the clean cut.

## 9. Time normalization

Current `TimePlugin` installs a system while bare App construction installs the `Time`
resource.

The target distinguishes:

```text
host/advancement progression facts
!=
system-visible Time projection/service
```

If `TimePlugin` survives, it must truthfully own the resource/system contract it exposes.
If the final implementation instead proves a minimal timing state to be a universal App
invariant, that proof must satisfy all Core-App gates and must not merely cite current
usage.

Fixed-step policy may consume advancement timing without making `Time` the authority over
the host itself.

## 10. Input normalization boundary

The completed input investigation concluded:

```text
RUNENINPUT_I0 = INTERNAL_BOUNDARY_REPAIR_FIRST
```

Therefore this App design records only the boundary:

```text
platform/window lifecycle       Runenwerk Host
physical/device input facts     input boundary under separate repair
product action mapping          Runenwerk/product
UI interaction semantics        RunenUI
```

Current `InputState` mixes several of these. It must not be blessed as universal App-core
state merely to preserve current bootstrap/tests.

Future App cleanup must coordinate with the input repair rather than invent a local
replacement taxonomy.

## 11. Scene/product/readiness normalization

### SceneCatalog

Scene registration/catalog state exists because the Scene capability is selected.
Target owner: Scene integration/plugin.

### SceneRuntimeState

Scene runtime state exists because the Scene capability is selected.
Target owner: Scene integration/plugin.

### GameplayRuntimeConfig

Gameplay policy is product/game integration state, not a universal application-runtime
invariant. Its exact final owner may be product/world/scene integration according to the
then-current consumers, but bare App is not the owner.

### UiOverlayState

UI overlay state is UI/render/product integration state. Bare App is not its owner.
The exact final owner must follow the current UI/render integration architecture at the
implementation slice.

### StartupState

Current `StartupState` tracks loading/readiness based on renderer warm frames and timeout
behavior. That is not the invariant "App Startup schedule has executed".

The implementation should rename/rehome this state under its actual readiness owner when
the relevant consumer slice is active. Do not retain the generic name as implied App
lifecycle authority.

## 12. Publication and execution-fabric normalization

ADR 0022 and the Execution Fabric design already provide the owners.

### Product/query publication resources

`ProductPublicationRuntimeResource` and `QuerySnapshotRuntimeResource` are Runenwerk
integration state for those publication/product workflows. Their presence is not a
universal App invariant.

### Publication handlers

The narrow product/query handler registries are accepted Runenwerk integration
mechanisms. They should exist when the corresponding publication integration is installed
and used.

They must not be used to justify global bare-App product/query capability activation.

### RuntimeJobExecutorResource

The runtime job executor is execution-fabric capability state. It does not become App
core because Draw/Editor currently consume it or because bootstrap currently installs
it.

### RuntimeProductCacheResource

The runtime product cache is product execution/cache state. It is reconstructable
integration state, not App semantic authority.

## 13. Capability-specific App API normalization

Current inherent `App` methods mix core composition with capability authoring:

```text
core-ish
  add_plugin / add_plugins
  add_systems
  init_resource / insert_resource
  set_runner
  world / world_mut

capability-specific
  add_input_bindings
  add_render_flow
  update_render_debug_*
  set_simulation_profile
  set_authority_role
  set_simulation_seed
  start/stop/load/seek replay
  add_scene / add_scene_template
```

The normalized rule is not "delete convenience". It is:

> **Convenience must be owned by the capability whose semantics it manipulates.**

Owner-specific extension traits/modules over the same App root are the preferred
long-term shape when they improve discoverability without moving authority.

Directionally only:

```text
AppSceneExt
AppRenderExt
AppSimulationExt
AppReplayExt
AppInput/ProductActionExt
```

Names and exact trait boundaries require source-level implementation review. No
compatibility aliases are pre-authorized.

## 14. Composition admission and duplicate semantics

A correct composition model must support deterministic rejection of invalid assembly.

Potential invalid classes include:

```text
required owner capability absent
illegal duplicate plugin/capability
mutually incompatible host/capability choice
owner configuration invalid
required host capability unavailable
cross-owner adapter prerequisite unsatisfied
```

The design does not require a universal `CompositionError` type or registry. Diagnostics
may be composed from owner-local errors plus Runenwerk integration context.

Where failure can be known before Startup, fail before Startup rather than allowing a
later missing-resource panic to become the primary contract.

## 15. Effective composition inspection

Future tooling may inspect the effective composition selected for one App instance.

Useful derived facts may include:

```text
selected plugin/group names
expansion/membership order
owner configuration identity or summary where meaningful
host choice
advancement policy
rejected/duplicate/incompatible selection diagnostics
```

This is a read model over actual composition. It must not become a writable service
locator or independent persistence authority.

## 16. Current builtin/field disposition matrix

| Current item | Current placement | Normalized class | Target disposition |
|---|---|---|---|
| `World` | `App` | Runtime Container | Keep App-core |
| RunenECS `Runtime` | `App` | Runtime Container | Keep App-core |
| `startup_ran` | `App` | App Lifecycle | Keep semantic responsibility; representation may change |
| `title` | `App` | Application metadata | Keep/configure at App level; host projects it |
| `AppMode` | `App` | Host selection mixed with runtime | Replace/refine around Host concept as implementation warrants |
| `AppRunner` | `App` | Advancement policy | Normalize independently from Host |
| Winit `ControlFlow` | `App` | Native host policy | Move to native-window host |
| `WindowState` | bootstrap | Native host state | Native-window host only; no required headless sentinel |
| `WindowStateRegistryResource` | bootstrap | Native host state | Native-window host |
| `PlatformWindowEventQueueResource` | bootstrap | Host/platform integration | Native-window/platform host |
| `NativeWindowHookRegistryResource` | bootstrap | Host/platform integration | Native-window host |
| `FramePacingPolicyResource` | bootstrap | Native host policy | Host integration |
| `FramePacingRuntimeStateResource` | bootstrap | Native host state | Host integration |
| `Time` | bootstrap | Timing projection/service | Time owner/plugin unless separately proven App-core |
| `FixedTimeConfig` | bootstrap | Fixed cadence policy | App fixed lifecycle or real fixed capability; final implementation decision required |
| `CatchupBudget` | bootstrap | Fixed cadence policy | Same as fixed cadence owner |
| `FixedTimeState` | bootstrap | Fixed cadence runtime | Same as fixed cadence owner |
| `SimulationTick` | bootstrap | Simulation identity | Simulation integration owner |
| `SimulationProfileConfig` | bootstrap | Simulation policy | Simulation integration owner |
| `SimulationSessionId` | bootstrap | Simulation identity | Simulation integration owner |
| `SimulationSeed` | bootstrap | Simulation input/identity | Simulation integration owner |
| `SimulationRng` | bootstrap | Simulation runtime state | Simulation integration owner |
| `InputState` | bootstrap | Mixed input/product state | Separate input-boundary repair; not App-core |
| `SceneCatalog` | bootstrap | Scene capability | Scene plugin/owner |
| `SceneRuntimeState` | bootstrap + ScenePlugin | Scene capability | Scene plugin/owner; remove duplicate bootstrap authority |
| `GameplayRuntimeConfig` | bootstrap + ScenePlugin | Product/game policy | Actual product/world/scene owner; not App-core |
| `UiOverlayState` | bootstrap + ScenePlugin | UI/render/product integration | Actual UI/render integration owner; not App-core |
| `StartupState` | bootstrap | Product/render readiness | Rename/rehome under readiness owner; not App lifecycle |
| `ProductPublicationRuntimeResource` | bootstrap | Publication/product integration | ADR 0022 / execution-fabric owner; install where required |
| `QuerySnapshotRuntimeResource` | bootstrap | Publication/query integration | ADR 0022 / execution-fabric owner; install where required |
| publication handler registry | implicit via handlers | Runenwerk publication integration | Narrow owner mechanism; no universal capability implication |
| `RuntimeJobExecutorResource` | bootstrap | Execution Fabric | Execution-fabric capability owner |
| `RuntimeProductCacheResource` | bootstrap | Product execution/cache | Execution-fabric/product owner |

This table establishes semantic pressure, not permission to edit every row in one PR.
Each implementation slice must re-census exact current consumers and active overlap.

## 17. Product/application composition pressure

Current Draw and Editor both repeat a generic capability prefix around default runtime,
diagnostics, scene, and render before adding product-specific behavior. Runtime Preview
remains much smaller.

This confirms ADR 0019's future product-group pressure, but the normalized order is:

```text
first
  truthful App/plugin/host/capability ownership

then
  prove useful product/plugin group machinery over those real owners
```

Do not introduce groups early as a facade around duplicate global builtins.

## 18. Failure semantics

### Composition failures

Where determinable before Startup, invalid composition should fail during
composition/admission with owner-aware diagnostics.

Do not rely on a later panic from `world.resource_mut::<T>().expect(...)` as the intended
contract for a missing selected capability.

### Runtime failures

Owner/system/runtime failures retain their accepted owner semantics. This model does not
add generic transactional rollback.

### Host failures

Window/event-loop/window-creation failure is host realization failure. It must not be
reported as a scene/render/simulation semantic failure unless an owner adapter actually
failed.

### Advancement completion

Bounded runners completing normally is an advancement result, not an App semantic
failure and not necessarily a host shutdown event.

## 19. Determinism and ordering

- plugin/group expansion order must be deterministic for the same explicit composition;
- owner system ordering continues to use its accepted schedule/owner contracts;
- composition order must not substitute for semantic system ordering;
- lifecycle schedule order must not be encoded through cross-schedule RunenECS ordering
  references;
- physical executor order must not create semantic capability ownership;
- derived effective-composition inspection must report actual accepted order rather than
  a separately maintained expected order.

## 20. Mechanical fitness tests for future implementation

A future clean cut must eventually prove all of the following.

### Minimal App / headless

- bare headless construction has no native-window state;
- no scene/UI/render/network/replay/product-job/simulation state appears merely because
  App exists;
- no synthetic `WindowState` is required as a headless marker;
- no Winit `ControlFlow` is App-core.

### Host

- windowed host creates and owns native-window/event-loop/pacing state;
- headless host remains valid without it;
- title/application metadata can project into the window host without ownership reversal.

### Fixed lifecycle / simulation

- fixed cadence does not require `engine_sim::SimulationTick` as App identity;
- simulation integration advances its own tick semantics from the accepted lifecycle;
- `FixedStepPlugin` has one truthful disposition rather than duplicate activation.

### Time

- Time state is installed by its truthful owner;
- host progression does not depend semantically on resource presence alone.

### Scene/product/UI

- selecting Scene installs required scene state exactly once;
- omitting Scene leaves scene state absent;
- gameplay/UI/readiness state is not globally manufactured by bare App.

### Input

- App cleanup does not bless the current mixed `InputState` as universal authority;
- later input repair can supply its neutral/product/UI adapters without fighting an App
  ownership claim.

### Publication/execution fabric

- ADR 0022 publication behavior remains explicit and independent of generic schedule
  wrapping;
- product/query/job/cache state exists only where selected/required;
- #591 behavior is not regressed or duplicated.

### Composition

- direct expert plugins and future groups lower to the same runtime owners;
- illegal duplicate/incompatible selection is rejected deterministically;
- effective-composition diagnostics are derived rather than separately authored.

### API

- capability-specific authoring remains ergonomic;
- App's core API does not need to own render/scene/simulation/replay/input semantics;
- no forwarding aliases are required solely to preserve rejected ownership.

## 21. Migration doctrine

Implementation follows clean cuts, not indefinite dual authority.

For each delivery slice:

```text
re-resolve current main
-> census exact owner + consumers + tests
-> select one coherent semantic boundary
-> move/install owner state and migrate consumers atomically
-> delete predecessor ownership/aliases in the same accepted cut where feasible
-> focused tests + cargo validate + exact-head CI
-> accepted-main proof
-> derive next slice only then
```

Do not open the full implementation tree in advance. Current work such as the ADR-0022
publication cutover may change the correct first implementation boundary.

## 22. Explicit non-goals

This model does not authorize:

```text
RunenApp / RunenCore
universal capability registry
service locator / generic DI
live dynamic plugin runtime
universal lifecycle bus
second scheduler
Plan / AppProgram / semantic database
product/plugin group implementation
RunenInput extraction
RunenRender or other framework semantic redesign
Cargo feature/binary modularity redesign
publication implementation owned by #591
compatibility forwarding layers
```

## 23. External comparison

### Bevy

Useful pressure:

- one application composition root;
- individual plugins and customizable plugin groups;
- explicit duplicate-plugin behavior;
- ordinary composition need not expose every internal subsystem boundary.

Not adopted:

- Bevy's ECS-centered ownership model;
- its exact App/PluginGroup API or default membership;
- the assumption that every Runen capability must be a plugin.

### Winit

Useful pressure:

- event-loop `ControlFlow` belongs to the event-loop host;
- native resume/suspend/window events are host/platform lifecycle;
- window creation has platform lifecycle constraints.

Runen adaptation:

- contain Winit under the Runenwerk native host;
- do not make its types the normalized App semantic vocabulary.

## Completion condition

This model is implementation-ready when a reviewer can classify any App field/resource
or convenience API by answering, in order:

```text
Who owns the invariant?
Is it universal App state?
Is it host realization?
Is it advancement policy?
Is it merely a lifecycle occurrence?
Which capability installs it?
What explicit dependency/admission makes it valid?
What state must be absent when the capability is absent?
What clean predecessor authority is deleted during migration?
```

If those questions do not yield one owner, the implementation slice is not yet
decision-complete and must stop rather than preserve ambiguity.
