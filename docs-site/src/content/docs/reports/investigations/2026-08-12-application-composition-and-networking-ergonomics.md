---
title: Application Composition and Networking Ergonomics Investigation
description: Current-main census, user-path pressure tests, external provenance, and alternatives analysis behind Runenwerk's batteries-included product-composition architecture.
status: active
owner: workspace
layer: investigation
canonical: false
last_reviewed: 2026-08-12
related_docs:
  - ../../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
  - ../../adr/accepted/0018-semantic-federation-and-physical-realization.md
  - ../../adr/accepted/0019-batteries-included-application-composition.md
  - ../../design/active/net-declarative-replication-authoring.md
  - ../../design/active/net-plugin-runtime-bridge.md
  - ../../net/multiplayer-replication-implementation-roadmap.md
  - ../../design/active/runenwerk-domain-workbench-north-star.md
---

# Application Composition and Networking Ergonomics Investigation

## Purpose

This report preserves the evidence behind ADR 0019.

The investigation began from a usability concern after ADR 0018:

> Can Runenwerk preserve explicit independent authorities and custom engine systems
> without making ordinary game, server, or tool authors manually assemble the complete
> internal architecture?

The result is:

```text
yes — if product convenience composes the existing owners transparently
rather than creating another runtime, registry, or configuration authority
```

This report is supporting evidence. ADR 0019 owns the durable decision.

## Evidence baseline

The current-source census was performed against accepted main:

```text
87241ef56b180bbd8f2d3357b0b901e78ad3eaa8
```

Important inspected surfaces include:

```text
engine/src/app/domain/app.rs
engine/src/app/domain/plugins.rs
engine/src/app/runtime/bootstrap.rs
engine/src/plugins/mod.rs
engine/src/plugins/scene/plugin.rs
engine/src/plugins/render/plugin.rs
engine/src/plugins/ui/plugin.rs
engine/src/plugins/world/plugin.rs
engine/src/plugins/net/plugin.rs
engine/Cargo.toml
apps/runenwerk_draw/src/runtime/app.rs
apps/runenwerk_editor/src/runtime/app.rs
apps/runenwerk_runtime_preview/src/lib.rs
engine/examples/runtime_minimal/main.rs
engine/examples/window_input_demo/main.rs
docs-site/src/content/docs/design/active/net-declarative-replication-authoring.md
docs-site/src/content/docs/design/active/net-plugin-runtime-bridge.md
docs-site/src/content/docs/net/multiplayer-replication-implementation-roadmap.md
```

Issue #246 contains the detailed checkpoint comments and exact source observations.

# Part I — current Runen composition

## App is already the composition root

`App` already provides the mechanics required for one application runtime:

```text
App::new
App::headless
add_plugin
add_plugins
add_systems
init_resource
insert_resource
set_runner
set_title
input bindings
render-flow registration
simulation profile/authority/seed controls
```

This strongly argues against creating another runtime abstraction merely for product
convenience.

The missing design question is how supported product capability sets are named,
inspected, modified, and composed.

## IntoPlugins is useful but not yet a full product-group model

Current `IntoPlugins` accepts:

```text
one Plugin
Box<dyn Plugin>
Vec<Box<dyn Plugin>>
tuples through arity 8
```

This is enough to execute a known composition.

It does not itself provide a first-class product-group model with:

```text
named group identity for debugging
effective member inspection
member replacement/configuration
member disable/removal
composition of named groups
explicit duplicate/incompatibility reporting
deterministic group-level ordering policy
```

Those differences matter for a batteries-included API because the user needs to be able
to start from an opinionated default without turning that default into hidden magic.

## Current default_plugins is a runtime baseline

At the evidence baseline, `default_plugins()` contains:

```text
TimePlugin
FixedStepPlugin
ReplayPlugin
InputFinalizePlugin
DiagnosticsPlugin
```

`default_plugins_with_diagnostics()` additionally installs scheduler diagnostics.

This is significantly narrower than a conventional complete game/tool stack. Scene,
rendering, UI, world, and networking are selected separately.

The function is therefore evidence that Runen already understands a distinction between
basic runtime services and larger product capabilities. Its final naming and future role
must not be changed merely by analogy with Bevy.

## App builtin state is broader than plugin defaults

`App` construction currently installs resources including:

```text
Time
InputState
WindowState / window registry / native-window hooks
frame pacing
SceneCatalog
StartupState
SceneRuntimeState
UiOverlayState
GameplayRuntimeConfig
fixed-time state and catchup budget
SimulationTick
product publication runtime
query snapshot runtime
runtime job executor
runtime product cache
SimulationProfileConfig
SimulationSessionId
SimulationSeed / SimulationRng
```

It also installs product/query publication barrier handlers.

This matters because merely adding better plugin groups would not by itself make `App`
conceptually minimal. Some state is clearly application-runtime scaffolding; other state
belongs to richer domain/product concerns.

The investigation did not classify these resources one by one because moving them could
change runtime behavior. ADR 0019 therefore records a target ownership law and leaves
current-source cleanup separately gated.

## Runtime omission and compile-time omission differ

`engine/Cargo.toml` has no broad product-capability feature topology today. The engine
package directly depends on WGPU, Winit, networking/simulation/history, many UI crates,
world/spatial systems, material/render infrastructure, and other capabilities.

Therefore:

```text
App::headless + no RenderPlugin
```

can mean rendering is not activated at runtime without meaning:

```text
WGPU/render dependencies are absent from the build graph
```

The product ergonomics decision must not conflate these.

# Part II — current product examples

## Runtime Preview — proof that small composition can stay small

`build_preview_app` chooses `App::new()` or `App::headless()`, sets a title, and adds the
current default plugin baseline.

The preview host separately owns its editor-preview protocol and Quinn-based runtime.
That is legitimate product behavior rather than generic engine ceremony.

This example argues for preserving a very small path instead of making one giant
default group mandatory.

## Runenwerk Draw — repeated common capability assembly

Draw currently composes:

```text
App
+ default_plugins
+ SchedulerDiagnosticsPlugin
+ ScenePlugin
+ RenderPlugin
+ NativeTabletRuntimePlugin
+ drawing-specific render flow
+ drawing GPU ink flow
+ DrawingAppPlugin
```

Classification:

### Product intent

```text
window title
drawing render flow
drawing GPU ink flow
DrawingAppPlugin
NativeTabletRuntimePlugin
```

The tablet adapter is a product capability choice because not every game/tool needs it.

### Candidate generic product wiring

```text
default runtime baseline
scheduler diagnostics
scene
render
```

The fact that another substantial product repeats nearly the same prefix is evidence of
potential reusable product composition.

## Runenwerk Editor — a second product with the same prefix

Editor/workbench setup similarly composes:

```text
default_plugins
SchedulerDiagnosticsPlugin
ScenePlugin
RenderPlugin
editor render flows
EditorAppPlugin or UiGalleryPlugin
editor diagnostic policy
editor input bindings
```

Product-specific flows, workbench selection, diagnostics policy, and bindings remain
legitimate explicit product intent.

The repeated engine capability prefix is the important commonality.

## Runtime minimal example — no product stack required

The minimal runtime example can use:

```text
App::new
+ RuntimeMinimalPlugin
+ run_for_frames
```

This demonstrates that an application should not have to opt into a complete game stack
merely to use the runtime/ECS foundation.

## Window/input demo — composition policy is not standardized

The window/input example has its product plugin call `default_plugins()` internally,
whereas other products add defaults from their outer app composition function.

Both are mechanically valid. The inconsistency shows why a documented product
composition doctrine is useful:

```text
Who installs the baseline?
Can it be replaced?
Can duplicate install be diagnosed?
What should a tutorial teach?
```

# Part III — owner-plugin pressure

## RenderPlugin is substantial

`RenderPlugin` initializes a large renderer state surface and schedules render prepare
and submit work. It includes shader, flow, feature, world-visual, UI-prepared,
residency, pipeline, backend, surface, debug, inspection, and frame-planning resources.

This supports two conclusions:

1. application users should not routinely rebuild its internal integration manually;
2. putting RenderPlugin into every product by default would be inappropriate for
   headless/server workloads.

## ScenePlugin, UiPlugin, and WorldPlugin remain distinct for good reasons

`ScenePlugin` owns scene runtime resources/lifecycle.

`UiPlugin` owns the engine-side UI runtime publication foundation.

`WorldPlugin` owns a substantial world runtime including chunk/build/invalidation,
collision/nav, replication extraction, world authority mode, and render contributions.

The product layer therefore needs composition, not semantic merger.

# Part IV — networking current state

## Runen networking is already custom technology

The current architecture is not an empty transport wrapper.

Directionally:

```text
engine_net
  protocol/session/replication semantics

engine_net_quic
  Quinn-based QUIC realization

engine/src/plugins/net
  engine scheduling/resources/runtime integration

gameplay/application
  product-specific mapping and presentation policy
```

The networking roadmap records implemented substrate including snapshots/deltas, ACK
handling, baseline logic, runtime queues, input flow, prediction replay, checkpoints,
diagnostics, QUIC realization, history/replay, and declarative metadata.

The question is ordinary authoring ergonomics, not whether Runen owns networking.

## NetPlugin already hides meaningful engine wiring

`NetPlugin<TDriver>` exposes client, server, and host constructors.

Its plugin installation handles:

```text
client/server role setup
simulation authority alignment
world runtime mode alignment
runtime bridge setup
replication setup
prediction setup
```

That is already a batteries-included composition idea at the networking subsystem level.

## The concrete ordinary-user gap is TDriver

The plugin still requires a driver implementing:

```text
ReplicationDriver
SnapshotApplyDriver
InputDriver
```

The canonical declarative-replication design states that current macros primarily
generate metadata and that normal gameplay still often needs a custom driver.

Its target is much simpler:

```text
mark replicated entities/components
register inputs and ownership
write ordinary authoritative ECS systems
let Runen perform standard extraction/snapshot/delta/apply/ACK/replay integration
```

The multiplayer roadmap already owns implementation convergence toward that target:

- Phase 4: standard ECS component extraction and apply;
- Phase 10: public low-boilerplate usage path.

The product-composition architecture should reinforce this direction rather than create
another networking API above or beside it.

# Part V — user-path pressure tests

## Minimal/headless

### Current quality

Good runtime ergonomics for very small applications.

### Product-specific decisions

```text
headless/windowed
runner/tick policy if non-default
actual application systems/resources
```

### Remaining problem

Builtin resource and compile-time dependency breadth is larger than the runtime API
suggests.

### Direction

Preserve the small path. Do not require a full product group.

## Ordinary local game

### Current quality

Composable, but no canonical batteries-included product stack.

### Product-specific decisions

```text
game rules/systems
input actions/bindings
world/procedural policy
product-specific render choices
product UI/assets
```

### Repeated ceremony

```text
runtime baseline
standard scene/render/UI/world integration as selected
owner ordering/adapter details
```

### Direction

A transparent game-oriented group should compose selected ordinary capabilities while
leaving game intent visible.

## Dedicated authoritative server

### Current quality

Strong semantic pieces, weak public product path/discoverability.

### Existing pieces

```text
App::headless
simulation authority profiles
WorldPlugin authority-sensitive mode
NetPlugin::server
engine_net/history/diagnostics
```

### Product-specific decisions

```text
endpoint/server identity/trust policy
game protocol/component/input registration
game authority rules
world/interest policy where game-specific
```

### Repeated ceremony to remove

```text
server role -> simulation/world mode wiring
standard net runtime/replication integration
standard diagnostics/history selection
render/UI exclusion
```

### Direction

A dedicated-server product group should be subtractive by design and should not activate
presentation behavior.

## Multiplayer client/server game

### Current quality

Strong internals; incomplete normal authoring path.

### Product-specific decisions

```text
replicated components/entities
input definitions
ownership routing
messages/channels where needed
game simulation/authority rules
prediction/interpolation policy where genuinely product-specific
```

### Generic plumbing to remove

For ordinary registered ECS state:

```text
manual standard snapshot extraction
manual standard delta formation
standard spawn/despawn/upsert/remove apply
ACK/baseline plumbing
transport glue
custom ReplicationDriver solely because no standard bridge exists
```

### Direction

Complete the existing registration-driven networking design. Retain custom drivers for
specialized representations.

## Tool/editor

### Current quality

Functional and explicit, but repeats common capability assembly.

### Product-specific decisions

```text
workbench selection
UI/tool behavior
product-specific render flows
native adapters
commands/keybindings
diagnostic policy
```

### Direction

Tool/workbench composition should make UI/render/diagnostic capability selection easy
without importing gameplay/networking defaults.

## Managed backend services

### Current quality

No Runen-wide product abstraction, which is appropriate given current evidence.

### Distinct concern

```text
auth
cloud persistence
object storage
presence/lobbies
hosted functions
payments
```

These involve provider, identity, security, persistence, deployment, and operational
contracts that differ from game-state replication.

### Direction

Remain application/provider integration until repeated concrete consumers prove stable
shared Runen contracts.

# Part VI — external inspiration and provenance

## Research method

External systems are used to pressure-test abstraction level and usability. They are not
Runen authority.

For each source this report distinguishes:

```text
established external mechanism
useful Runen lesson
explicit non-adoption
Runen-specific synthesis
```

## Bevy 0.19 — App, DefaultPlugins, MinimalPlugins, PluginGroup

### Primary sources

- <https://docs.rs/bevy/latest/bevy/prelude/struct.App.html>
- <https://docs.rs/bevy/latest/bevy/struct.DefaultPlugins.html>
- <https://docs.rs/bevy/latest/bevy/struct.MinimalPlugins.html>
- <https://docs.rs/bevy/latest/bevy/app/trait.PluginGroup.html>
- <https://docs.rs/crate/bevy/latest/source/examples/app/plugin_group.rs>

### Established mechanism

Bevy uses `App` as its primary application composition surface. Plugins extend the App.
`DefaultPlugins` groups the plugins normally expected by a windowed/presentation
application, while `MinimalPlugins` supplies a much smaller bare-bones runtime set.

Plugin groups lower to plugins and may be modified through group-builder operations such
as setting/configuring, disabling, enabling, composing, and ordering members.

Bevy's default group also obeys Cargo feature selection, demonstrating that runtime
composition and compile-time capability selection can cooperate while remaining
different mechanisms.

### Useful Runen lesson

The strongest pattern is:

```text
one application root
+ ordinary plugins
+ transparent named groups
+ group customization
```

A default can be both easy and inspectable.

### Explicit non-adoption

Runen does not adopt Bevy's ECS ownership model or copy its particular default plugin
list. Runen's independent-framework direction and owner boundaries remain different.

The Bevy feature model is not evidence that Runen's current engine package already has
appropriate compile-time modularity.

### Runen-specific synthesis

Use a small transparent product-group concept over the existing App/Plugin seam rather
than introducing another configuration runtime.

## Lightyear — grouped networking with application-owned protocol

### Primary sources

- <https://cbournhonesque.github.io/lightyear/book/tutorial/setup.html>
- <https://cbournhonesque.github.io/lightyear/book/tutorial/build_client_server.html>

### Established mechanism

Lightyear groups substantial networking machinery behind client/server plugin groups.
The application still defines the protocol: input, messages, replicated components and
channels. A component registry provides metadata for automatic World replication.

### Useful Runen lesson

```text
complex internal networking
!=
complex application setup
```

The engine/framework can own transport/replication machinery while the game declares
its actual protocol intent.

### Explicit non-adoption

Runen does not adopt Lightyear itself, Bevy World ownership, or its exact link/entity
runtime model.

### Runen-specific synthesis

Runen already has stronger custom networking semantics and an existing declarative
registry direction. Finish that ordinary path instead of replacing the network stack.

## Supabase — independently useful services with integrated product experience

### Primary sources

- <https://supabase.com/docs/guides/getting-started/architecture>
- <https://supabase.com/docs/guides/auth/architecture>
- <https://supabase.com/docs/guides/realtime>
- <https://supabase.com/docs/guides/realtime/presence>
- <https://supabase.com/docs/guides/functions/architecture>

### Established mechanism

A Supabase project integrates several independently meaningful systems around Postgres,
including Auth, Realtime, Storage, APIs, Functions, gateway/pooling and tooling.
Supabase explicitly values systems that can work in isolation while being integrated
and approachable.

Client SDKs remove repeated authentication/configuration/token-management work.
Supabase's convenience also includes operation of hosted infrastructure, not merely a
local abstraction.

Realtime offers Broadcast, Presence and database-change functionality. Current Presence
documentation explicitly warns against high-frequency presence updates and directs such
use cases toward Broadcast instead.

### Useful Runen lesson

Independent subsystems and a batteries-included product experience are compatible.
Good integration does not require pretending the underlying systems are one semantic
owner.

Also, hosted-service convenience has an operational dimension that an engine library
cannot reproduce through API shape alone.

### Explicit non-adoption

Supabase Realtime is not adopted as Runen's authoritative game simulation/replication
model. Its realtime features and Runen prediction/rollback/interest/history semantics
serve different contracts.

No generic Runen `OnlineServices` abstraction is inferred from this analogy.

### Runen-specific synthesis

Keep game-state networking built into Runen's semantic model. Treat hosted auth/storage/
persistence/lobby/backend systems as separate provider/product integrations until real
common contracts are proven.

# Part VII — alternatives

| Approach | Ergonomics | Explicitness | Replaceability | Inspectability | Framework complexity | Primary failure | Disposition |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Explicit low-level composition only | low-medium | excellent | excellent | excellent | low | internal wiring repeated by every app | expert path only |
| Monolithic default-everything App | excellent initially | low | low-medium | low | deceptively low | hidden behavior and server/tool/headless baggage | reject |
| Transparent typed product/plugin groups | excellent | high | high | high | moderate | group proliferation/order rules if undisciplined | preferred mechanism |
| Persistent capability/preset config runtime | excellent | medium | high | medium | medium-high | parallel configuration authority | reject without new evidence |
| Thin ephemeral helper lowering to groups | excellent | high after lowering | high | high | low-medium | may be unnecessary syntax | allow if later proven useful |
| Convention/autodiscovery | excellent | low | medium | low | medium | hidden startup magic | reject as primary authority |
| Scaffolding/code generation | high initial | high after generation | high | high | tooling cost | boilerplate ages/drifts | optional authoring aid only |
| Service locator/global registry | superficially high | low | medium | medium | medium | dependency/authority hiding | reject |

## Why transparent groups are preferred

The existing App/Plugin seam already performs execution and ownership integration.
Therefore a group can remain very small in conceptual scope:

```text
selection
ordering
configuration
inspection
validation
```

It does not need to invent another runtime.

A persistent product-configuration engine would have to synchronize its own state with
plugins/resources and would recreate exactly the parallel truth ADR 0018 rejects.

# Part VIII — architecture distinctions

## Semantic authority

Owns domain invariants and truth.

Examples include RunenRender image-formation semantics or engine_net replication
protocol state.

## Owner/framework plugin

Installs an owner's runtime resources, systems and integration into App.

Examples:

```text
RenderPlugin
ScenePlugin
UiPlugin
WorldPlugin
NetPlugin<TDriver>
```

## Product/plugin group

Selects and configures existing plugins for a supported application shape.

It owns defaults and composition policy, not subsystem semantic truth.

## Product plugin

Contains product/application behavior.

Examples include `DrawingAppPlugin` and `EditorAppPlugin`.

## Low-level realization

Provides implementation beneath an owner contract.

Examples include WGPU, Quinn/QUIC, OS window/input APIs, and potentially a future
provider-specific hosted-service client.

This distinction is what allows Runen to remain a custom engine without requiring every
primitive to be custom-built from hardware/network packets upward.

# Part IX — risks and mitigations

## Risk: product groups become another ontology

Mitigation:

- groups contain existing plugins/configs;
- no new semantic identity/revision system;
- groups disappear into App composition;
- group names describe supported product paths, not domain truth.

## Risk: bundle combinatorics

Mitigation:

- small orthogonal groups;
- compose capabilities rather than enumerate cross-products;
- product plugin remains the place for application-specific behavior.

## Risk: opaque defaults

Mitigation:

- effective membership/order inspectable;
- member configuration/removal explicit where legal;
- invalid combinations diagnose rather than silently change behavior.

## Risk: product group duplicates App builtin resources

Mitigation:

- ADR 0019 establishes builtin-state discipline;
- resource movement requires a separate current-source review;
- product groups must not paper over duplicate authority by adding more hidden state.

## Risk: false headless minimality

Mitigation:

- documentation distinguishes runtime activation from Cargo/build dependency topology;
- compile-time modularity is separately designed only when justified.

## Risk: networking convenience erases meaningful game protocol

Mitigation:

- game registers components/input/ownership/protocol intent;
- Runen handles generic replication plumbing;
- custom driver remains for genuinely different representations.

## Risk: managed backend becomes fake universal abstraction

Mitigation:

- no `OnlineServices` framework without multiple real providers/consumers and security/
  persistence/operational proof;
- provider-specific integration is acceptable where product needs justify it.

## Risk: architecture vocabulary leaks into ordinary APIs

Mitigation:

- ordinary examples teach product concepts first;
- ADR 0017/0018 machinery remains for owner/integration/tooling layers;
- common adapters/admission are performed by selected Runen integration where safe.

# Part X — final assessment

The current architecture does **not** need to be simplified into a monolith to obtain
Bevy-like ergonomics.

The evidence supports:

```text
App
  remains the one runtime composition root

owner plugins
  remain explicit executable integration units

product/plugin groups
  provide transparent batteries-included compositions

product plugin/domain declarations
  preserve application intent

expert access
  reaches the same owners directly
```

For networking:

```text
Runen keeps its custom replication/prediction/history/interest semantics
+ ordinary ECS replication becomes registration-driven
+ NetPlugin keeps role integration
+ custom ReplicationDriver remains expert path
+ Quinn remains a contained transport realization
```

For hosted services:

```text
possible future provider integrations
!=
authoritative game networking
!=
currently proven universal Runen service abstraction
```

The most important synthesis is therefore:

> **Strong internal decomposition and batteries-included product ergonomics are not
> competing goals. Product composition should hide repeated generic wiring while
> preserving the same typed owners underneath.**

That synthesis is the evidence basis for ADR 0019.