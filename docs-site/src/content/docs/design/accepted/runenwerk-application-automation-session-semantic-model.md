---
title: Runenwerk Application Automation Session Semantic Model
description: Accepted semantic model for application automation sessions, typed owner adapters, execution fidelity, scenarios, traces, waits, results, and evidence without creating a universal command or replay authority.
status: accepted
owner: engine
layer: engine-runtime / integration
canonical: true
last_reviewed: 2026-09-25
related_adrs:
  - ../../adr/accepted/0023-normalize-app-runtime-host-lifecycle-and-capability-ownership.md
  - ../../adr/accepted/0024-normalize-physical-input-observation-semantics.md
  - ../../adr/accepted/0025-normalize-editor-coordination-and-collaboration-ownership.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
related_designs:
  - ./runenwerk-app-runtime-and-composition-semantic-model.md
  - ./runenwerk-physical-input-semantic-model.md
  - ./runenwerk-render-lab-product-design.md
  - ./runenwerk-editor-coordination-semantic-model.md
related_docs:
  - ../../engine/reference/plugins/input/architecture.md
  - ../../engine/reference/plugins/replay/architecture.md
---

# Runenwerk Application Automation Session Semantic Model

## Status

This document is the accepted semantic target derived from issues #731 and #888.

It defines application-automation ownership and behavior. It is not a public Rust API
specification, CLI syntax, IPC protocol, persisted scenario or trace format, or permission to
merge existing replay systems.

Reviewed authority at decision preparation:

~~~
RUNENWERK_MAIN=f5a31342c495a4f9cfa9b594b00b8d0b4292a622
ENGINEERING_MAIN=66234b3c1985d125ecc5019b690e2d96281703fc
RUNENINPUT_PIN=2751e19fa42255b86e786e7cd837198c917b7b25
~~~

PR #885 is an active Host-selection normalization from this base. Its target makes Host selection
explicit as native-window versus headless. This design uses those durable semantic terms and does
not depend on the predecessor AppMode naming.

Code and tests remain authority for current behavior. This document defines the target semantics
for later bounded automation implementation.

Normative words MUST, MUST NOT, SHOULD, and MAY are deliberate.

## Decision summary

Runenwerk SHOULD have one shared application automation session semantic layer, but that layer owns
orchestration only.

The accepted relation is:

~~~
operator / scenario
    -> automation session
        -> typed owner adapter
            -> selected execution mode
                -> owner-correct stimulus or command ingress
                    -> owner state/effect
                        -> typed owner query
                            -> assertion verdict
                                -> evidence
~~~

The session owns sequencing, execution-mode selection, target lifetime, waits and timeouts,
cancellation, result/evidence envelopes, and automation-owned held-state cleanup.

It does not own RunenInput observation meaning, product/editor/UI command meaning, Scene simulation
replay, RunenUI interaction replay, native OS event semantics, process/window lifecycle, renderer
semantics, or one universal object/action/widget registry.

A CLI, test harness, or future local controller is a caller of this semantic contract. It is never
the semantic owner.

## Existing replay systems remain separate

### Scene replay

The Engine Replay integration records and validates Scene simulation snapshots, command frames,
ticks, checkpoints, hashes, and deterministic replay.

It answers whether simulation or Scene state can be reconstructed and validated at the owning
simulation tick. It does not define physical-input, native Host, or application-target semantics.

Scene replay remains separate.

### RunenUI interaction replay

RunenUI interaction stories replay UI-owner interaction facts against mounted UI fixtures and
report target resolution, state transitions, semantic outcomes, suppression, focus, and related UI
facts.

That contract remains UI-owner-specific. It does not become application-wide product, editor,
physical-input, or Host authority.

### Device-level input

RunenInput owns backend-neutral input observations and confirmed-state reduction. Runenwerk owns
backend/Host translation and product integration.

Normalized input replay may exercise this path, but it is neither Scene replay nor RunenUI replay.

## Scenario, trace, and transformation

These concepts MUST remain distinct.

### Authored scenario

An authored scenario expresses executable operator intent.

A scenario can contain steps such as selecting a target, issuing a typed product command, injecting
normalized input, waiting for an owner condition, querying owner state, asserting owner state, and
requesting evidence.

A scenario is not evidence that the described effect occurred.

### Observed trace

An observed trace records facts from one execution.

A trace MAY contain admitted normalized input, owner query results, Host timing observations,
step/result records, and optional visual artifacts.

A trace is not automatically executable and MUST NOT silently become a scenario.

### Trace to scenario transformation

A trace-to-scenario transformation is explicit and reviewable. It may remove incidental timing,
replace runtime identities with resolvable selectors, remove unrelated observations, introduce
condition-based waits, or parameterize values.

If the transformed scenario is claimed to derive from a trace, provenance to that trace MUST remain
available.

## Execution modes

Every stimulus-bearing step MUST select an execution mode. The session MUST NOT silently fall back
to a lower-fidelity mode.

### Product-semantic

A typed owner adapter invokes an owner command through that owner's accepted API.

This exercises product/editor/UI semantics. It does not prove physical-input or native Host
acquisition.

### Normalized-input

The session injects backend-neutral input through the accepted Runenwerk and RunenInput admission
path.

This exercises:

~~~
normalized input
    -> RunenInput validation and reduction
        -> Runenwerk consumer projection
            -> product behavior
~~~

It does not exercise native winit acquisition, OS event generation, physical-device scheduling, or
backend-specific decoding.

### Native/OS

A platform-specific driver produces stimulus through the actual operating-system and windowing
path.

Native mode is supported only when the selected platform mechanism is proven to reach the required
ingress. A synthetic cursor capability MUST NOT imply raw-device-motion capability.

### Observed human/native capture

Human/native capture records an actual execution. It is a capture source, not a replay mode.

A captured fact is not automatically reproducible through an automation driver.

## Current physical-input recording gap

Current Runenwerk does not expose one lossless general physical-input trace.

The existing mechanisms are insufficient individually:

- PlatformWindowEventQueueResource observes normalized window events used by maintained consumers,
  but raw winit DeviceEvent MouseMotion follows a separate Host path;
- ordinary keyboard, pointer, scroll, and contact admissions are not uniformly retained as
  replayable InputObservationGroup values;
- native-tablet integration temporarily retains observation groups for its product projection;
- frame-local InputState aggregate deltas, pressed/released flags, touch samples, and actions are
  consumer projections and lose source/group/provenance information.

A future normalized recorder MUST therefore capture at one explicit admitted-observation boundary,
not by serializing frame projections or one partial event queue.

The conceptual capture point is:

~~~
backend or synthetic normalized producer
    -> InputObservationGroup
        -> validation and admission
            -> admitted-observation capture
            -> confirmed-state reduction and product projection
~~~

The capture MUST preserve atomic group boundaries, deterministic admission order, source/device/
contact/tool identity, origin and reconciliation provenance, evidence status, delivery role,
relative versus absolute motion, scroll axes/domain, continuity loss, and source time where
present.

This design does not move recorder ownership into RunenInput. RunenInput remains reusable semantic
and reducer authority. Application recording is Runenwerk integration/tooling policy until
independent reusable-framework pressure proves otherwise.

## Target identity and lifetime

Automation target identity belongs to the owning domain. There is no universal target ID.

Examples include native-window identity for Host operations, product-local Render Lab identity,
Editor mounted-unit/tool-surface/document identity, RunenUI widget/focus identity, and RunenInput
source/device/contact identity.

A typed adapter MUST resolve a scenario target to a current owner identity before executing a step.

Target resolution may report Resolved, Missing, Stale, Ambiguous, or Unsupported. Stale and
ambiguous targets fail closed. The session MUST NOT silently redirect to another active window,
document, tool surface, or widget.

Runtime identities recorded in traces are session-local by default. A later persisted format would
need explicit stable-selector and version semantics.

## Session lifetime

An automation session is explicit and bounded.

It has an execution-local identity, selected Host context, selected owner adapters, cancellation
state, automation-owned held-input state, and current-run step/result history.

### Launch and attach

The model distinguishes:

- launch-owned session: the controller creates the application execution it controls;
- attached-local session: the controller joins an application that explicitly enabled an
  automation endpoint/session.

Attach is not required for the first implementation.

A future attach path MUST be explicit opt-in, local-only by default, session-scoped, and bounded by
clear controller ownership. This design does not choose TCP, HTTP, Unix sockets, named pipes,
stdio, or another transport.

### Concurrency

A typed adapter MAY allow concurrent read-only queries. Mutation/control SHOULD default to one
active automation controller per target unless the owner defines a stronger contract.

The session MUST not hide conflicting operator control. It may report a truthful busy,
inconclusive, or cancelled result rather than pretending exclusive control.

## Automation-owned input state

Injected held input is owned by the automation session that created it.

On explicit release, scenario completion, cancellation, controller disconnect, timeout, or session
teardown, the session MUST release or invalidate only automation-owned injected state.

It MUST NOT synthesize releases for unrelated physical input held by a human or another source.

Normalized-input mode SHOULD use a distinct automation source identity so cleanup can be scoped
through existing continuity semantics.

## Clocks and ordering

These domains remain separate:

| Domain | Owner | Meaning |
| --- | --- | --- |
| source time | input producer/backend | producer measurement or event time when available |
| input admission order | RunenInput and Runenwerk input integration | deterministic normalized reducer order |
| Host monotonic time | Host | elapsed local process or Host time |
| Host frame/redraw boundary | Host | application advancement or presentation opportunity |
| simulation tick | simulation owner | deterministic simulation progression |
| presentation interval | presentation owner | observed frame presentation timing |

No one clock substitutes for another.

A scenario MAY express duration or timeout requirements, but MUST NOT claim deterministic OS
scheduling, global physical chronology, identical frame timing, or identical GPU pixels.

## Waiting

Arbitrary sleeps are not the default synchronization primitive.

A step SHOULD wait on a typed owner condition with an explicit timeout.

Dispatch means the adapter received a request. A later typed query establishes whether the owning
effect became observable.

A fixed delay MAY be used when delay itself is part of the behavior being tested. Such a delay is
timing input, not proof of completion.

## Result knowledge

The automation system MUST distinguish transport from semantic effect knowledge.

The conceptual result model contains at least:

- Dispatched;
- AdmittedOrDelivered;
- EffectConfirmed;
- AssertionPassed;
- Unsupported;
- Inconclusive;
- Cancelled;
- InfrastructureFailure.

Concrete Rust representation is deferred.

Unsupported MUST NOT silently become another execution mode.

## Evidence

Evidence may include typed owner observations, input admission records, Host timing observations,
product diagnostics, structured assertion reports, screenshots, video, and renderer captures.

Screenshot/video/capture is opt-in and MUST remain separate from capture-free performance timing.

An artifact link or whole-screen hash alone does not prove a product-semantic effect unless the
owning assertion explicitly defines that relation.

## Ownership matrix

| Invariant | Owner |
| --- | --- |
| normalized input meaning and reducer laws | RunenInput |
| backend/native to normalized input translation | Runenwerk input integration |
| optional application admitted-input capture | Runenwerk automation/input integration |
| process/window/event-loop/lifecycle facts | Runenwerk Host |
| scenario sequencing, mode, waits, session lifetime, result envelope | automation session |
| product commands/queries/assertions | typed product adapter and product owner |
| Editor document/tool-surface targeting | Editor owner |
| RunenUI interaction replay and widget semantics | RunenUI |
| Scene simulation replay/checkpoints | Scene/simulation replay owner |
| native event-injection fidelity | OS/native driver |
| terminal UX | CLI/tooling caller |

## Capability discovery

The automation layer MUST NOT create one mutable registry mirroring every product command, widget,
document, device, or action.

A selected typed adapter MAY expose a bounded descriptor of supported modes, typed commands,
queries/assertions, target-selection shape, evidence kinds, and Host requirements.

Discovery is a projection of owner contracts, not new semantic authority.

## Security and operator control

Automation requires explicit opt-in.

The architecture requires application-scoped capture/control rather than OS-wide keylogging,
local-only control by default for future out-of-process sessions, validation of untrusted scenario
input, bounded payload/rate/resource use, capability versus permission distinction, cancellation
and manual takeover, cleanup limited to automation-owned state, and no hidden persistent
remote-control service.

A persistent or remotely reachable control service requires a separate security decision.

## External mechanism lessons

External systems are mechanism evidence, not Runenwerk authority.

### W3C WebDriver Actions

Useful lessons are stateful input sources, execution ticks, pauses, and explicit held-state cleanup.

Runenwerk does not adopt browser DOM identity, browser event semantics, or the WebDriver wire
protocol.

### Playwright

Useful lessons are generated editable scenario code, separate trace evidence, owner-semantic target
resolution, and condition-based assertions.

Browser locators do not become a universal Runenwerk target model.

### Robot Framework

A generic sequencer can compose owner/library keywords without owning the semantics of every
target system. Runenwerk follows the same ownership principle through typed adapters.

## Native fidelity and macOS raw motion

Current winit defines DeviceEvent MouseMotion as raw, unfiltered physical motion distinct from
window cursor movement and warns that raw-device IDs and window-event virtual-device IDs may differ.

Apple Core Graphics supports posting synthetic Quartz mouse events into the event stream.

The reviewed primary documentation does not establish that posting a synthetic Quartz cursor or
mouse event produces winit raw DeviceEvent MouseMotion.

Therefore macOS synthetic cursor/mouse replay of the raw-device path is Unsupported/Unproven.

A native driver MUST report unsupported rather than silently replay normalized motion. This may be
revised only by direct execution evidence on the maintained backend or an explicit platform
contract.

## Pressure tests

### Render Lab orbit, pan, and zoom

Normalized-input mode MAY inject button, motion, and scroll observations through the accepted input
path. Render Lab owns the camera query/assertion.

This proves normalized input through RunenInput and Runenwerk to Render Lab camera behavior. It does
not prove native winit raw-motion acquisition.

### Idle to pointer movement to idle frame pacing

A normalized-input scenario cannot establish that a physical/native raw event woke winit or caused
the observed event-loop/presentation behavior. That claim requires native or physical Host
evidence.

Performance measurement SHOULD avoid recording/capture overhead inside the measured interval unless
that overhead is itself the subject.

### Headless

The same automation vocabulary MAY run under a headless Host only for adapters and modes that are
actually available headlessly.

It MUST NOT manufacture native-window identity, display, GPU presentation, OS pointer target, or
native raw-input capability.

### Draw/tablet

Normalized tablet observations can test neutral-input-to-Drawing behavior. Driver acquisition,
calibration, backend health, and device-specific timing require native evidence.

### Editor

Editor automation resolves targets through current mounted-unit/tool-surface/document ownership.

It MUST NOT assume one global active document, one universal widget ID, one global command registry,
or one permanently valid target after composition changes.

## First implementation slice

After this design is accepted, create exactly one bounded implementation issue proving the semantic
shape before adding a production CLI, IPC transport, persisted format, or recorder.

The first implementation SHOULD provide an in-process automation session proof with:

1. one execution-local session identity;
2. typed step/result envelopes;
3. explicit ProductSemantic and NormalizedInput modes;
4. one automation-owned normalized input source with scoped cleanup;
5. condition-based wait/query support with timeout;
6. two structurally different typed owner adapters:
   - Render Lab camera query/assertion;
   - Editor target/query/assertion through current mounted-unit/tool-surface ownership;
7. headless execution where the selected operation is genuinely supported;
8. explicit Unsupported for Native/OS mode in the proof rather than fake native execution;
9. no persistence;
10. no CLI or IPC;
11. no Scene replay or RunenUI replay modification.

The proof succeeds when the same session model can drive the two owners without leaking either
owner's semantic vocabulary into the orchestration core.

Recording follows as a later slice only after this proof demonstrates pressure for the admitted-
observation capture seam.

## Why no new ADR is required

This decision does not transfer semantic authority between repositories or domains. It composes
already accepted App/Host, RunenInput/input integration, Editor, RunenUI, Scene replay, and
product-local owners.

An accepted design is sufficient durable authority for the new orchestration layer. A later new
repository boundary, remotely reachable service, persistence authority, or cross-framework
semantic transfer may require its own ADR.

## Rejected abstractions

The following are rejected:

- one universal application command enum;
- one global object/widget/document/device registry;
- Scene replay repurposed as physical-input replay;
- RunenUI replay repurposed as application-wide replay;
- frame-local InputState projections relabeled as lossless trace;
- PlatformWindowEventQueueResource relabeled as complete physical-input capture;
- automatic normalized-input to native-input fallback;
- synthetic cursor movement claimed as raw-device motion without proof;
- one clock used as source, simulation, Host, and presentation time;
- sleeps used as universal readiness proof;
- dispatch treated as effect confirmation;
- screenshot existence treated as semantic assertion success;
- hidden remote-control daemon;
- OS-wide recorder/keylogger;
- cleanup that releases human-owned physical input;
- a CLI as semantic authority.

## Acceptance and successor gate

This design is accepted only when current App/Host terminology is reconciled, owner/target/clock/
mode/result/security boundaries remain explicit, all five pressure tests remain truthful, macOS
raw-motion injection remains unsupported without new proof, no existing replay owner is merged or
duplicated, repository documentation validation and exact-head CI pass, and the complete diff
contains only this design authority and its index update.

After accepted-main verification, create one implementation issue for the first in-process proof
above.

Do not pre-authorize CLI, IPC, recording persistence, remote attach, or native OS automation in
that implementation slice.
