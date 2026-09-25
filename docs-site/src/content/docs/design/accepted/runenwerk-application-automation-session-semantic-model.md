---
title: Runenwerk Application Automation Session Semantic Model
description: Accepted semantic model for application automation sessions, typed owner adapters, execution fidelity, scenarios, traces, waits, results, and evidence without creating a universal command or replay authority.
status: accepted
owner: engine
layer: engine-runtime / integration
canonical: true
last_reviewed: 2026-09-25
publication: reference
pagefind: false
related_adrs:
  - ../../adr/accepted/0023-normalize-app-runtime-host-lifecycle-and-capability-ownership.md
  - ../../adr/accepted/0024-normalize-physical-input-observation-semantics.md
  - ../../adr/accepted/0025-normalize-editor-coordination-and-semantic-ownership.md
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
RUNENWERK_MAIN=8f25fffd4c59c3575c6b66dd7b8e8ceff6669872
ENGINEERING_MAIN=66234b3c1985d125ecc5019b690e2d96281703fc
RUNENINPUT_PIN=2751e19fa42255b86e786e7cd837198c917b7b25
~~~

Accepted PR #885 / commit 8f25fffd4c59c3575c6b66dd7b8e8ceff6669872 makes Host selection
explicit as native-window versus headless. This design uses those accepted semantic terms and does
not depend on the retired AppMode naming.

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

## Accepted normalized-input recording state

Runenwerk now exposes an explicit, opt-in admitted-input capture seam and a canonical App-frame trace
envelope.

Accepted A3 captures exact successfully admitted `InputObservationGroup` values at the Runenwerk
input-integration boundary. Accepted A4 partitions those groups by canonical App frame and preserves
idle frames plus explicit trailing groups that have no proven frame membership.

The accepted capture relation is:

~~~
backend or synthetic normalized producer
    -> InputObservationGroup
        -> validation and admission
            -> optional exact-group capture
            -> confirmed-state reduction and product projection
                -> canonical App FrameEnd
                    -> trace-local frame ordinal + admitted groups
~~~

The trace preserves atomic group boundaries, deterministic admission order, source/device/contact/
tool identity, origin and reconciliation provenance, evidence status, delivery role, relative
versus absolute motion, scroll axes/domain, continuity loss, source time where present, and
canonical App-frame partitioning.

A4 is not a complete platform-event transcript and does not snapshot pre-capture input state.

In particular:

- `PlatformEvent::TextInput` is a separate Runenwerk platform/input side channel and is not an
  `InputObservationGroup`; A3/A4 therefore do not record typed-text payloads;
- capture may begin while keys/buttons/contacts are already held or active;
- capture does not snapshot the absolute-pointer position that existed before its first captured
  absolute-position observation;
- capture does not snapshot Runenwerk frame-local projection state such as primary-touch ownership.

Those omissions are truthful recording boundaries, not permission to reconstruct missing facts
during replay.

Frame-local InputState projections, PlatformWindowEventQueueResource, and native-tablet product
staging remain non-authoritative as general recording surfaces.

Recorder ownership remains Runenwerk integration/tooling policy. RunenInput remains reusable input
meaning, validation, and reducer authority.

## Normalized observed-trace replay

Normalized observed-trace replay is an explicit caller-selected operation over an in-memory
Runenwerk automation input trace. It is not an implicit trace-to-scenario transformation and does
not make the observed trace an authored scenario.

The first replay contract is intentionally process-local and version-local. It does not claim a
persisted trace is portable across application versions, machines, window layouts, devices, or
product/editor/UI target identities.

### Replay identity mapping

Recorded RunenInput runtime identities are trace-local facts, not stable cross-run selectors.

A normalized replay MUST use caller-supplied replay-owned source identity mapping for every distinct
recorded `InputSourceId`. The mapping MUST preserve distinct recorded source equivalence classes
and MUST NOT silently map two recorded sources onto one replay source.

Recorded device identities MAY remain numerically unchanged beneath a remapped replay-owned source,
because device identity is scoped by `InputContext`. Contact and tablet tool IDs remain trace-local
payload identity under that remapped context; replay does not promote them to global selectors.

If a tablet observation contains source time, replay MUST rewrite only the embedded
`SourceTime.context` to the remapped enclosing `InputContext`. Timestamp value and unit remain
the recorded measurement facts. This rewrite is required by RunenInput validation and does not
claim the original backend clock exists in the replay execution.

The caller is responsible for supplying replay-owned source IDs that do not collide with unrelated
physical/current sources. Runenwerk MUST NOT introduce a global mutable input-ID allocator solely
for automation.

### Replay preflight and trace self-containment

Before mutating the target App, replay MUST validate that the trace is supported by the selected
replay contract.

The first replay contract assumes fresh replay-owned source identities and MUST NOT infer state that
predates capture.

Because A4 does not snapshot source state at capture start, trace shape alone cannot prove that the
first captured press/begin was the first active state for that source. The first replay proof
therefore also requires a recording-side precondition: for every replayed source/control/contact
used by the proof, the caller must establish that the relevant normalized state was neutral when
capture began. The Render Lab proof satisfies this by starting capture on a fresh headless
`InputState` before the first automation-owned button press.

This is an explicit precondition, not evidence encoded in `AutomationInputTrace`. A later recorder
extension may capture initial-state evidence if broader observed-human replay requires it.

A trace is not self-contained merely because every captured group is valid. The first replay
implementation therefore supports only a deliberately narrow subset whose required state can be
established from the trace plus that explicit recording precondition:

- pointer-button input when the explicit recording-side neutral-state precondition is established;
- relative motion;
- scroll;
- all-tablet atomic groups for exact neutral admission and native-tablet staging replay.

All-tablet support at this layer proves atomic normalized-input transport only. It does not by itself
prove that a Draw/tablet product stroke whose product state began before capture can be reproduced.
That stronger owner-level claim requires an owner-specific initial-state/assertion contract.

The first replay implementation MUST report UnsupportedTraceShape before target mutation for:

- absolute pointer-position groups, because the first Runenwerk cursor-motion projection depends on
  a pre-capture absolute-position baseline that A4 does not record;
- ordinary contact/touch groups whose first required contact state predates capture;
- keyboard replay as a claim of recorded text entry, because `PlatformEvent::TextInput` payloads are
  not present in A4;
- digital input whose required initial state is not covered by the explicit recording-side
  neutral-state precondition;
- continuity-loss behavior that would require state established before the trace;
- any other group whose Runenwerk projection depends on missing pre-capture state.

A later recorder extension may add explicit initial-state or text-input evidence. Replay MUST NOT
manufacture those facts in the meantime.

Normalized keyboard observations may eventually be replayed as keyboard observations, but that MUST
remain explicitly distinct from text-entry replay. A completed normalized keyboard replay would not
prove that original typed-text behavior was reproduced.

### Projection-preserving atomic replay ingress

Replay MUST exercise the accepted Runenwerk input integration, not only the neutral RunenInput
reducer.

For the first bounded implementation:

- supported pointer-button, relative-motion, and scroll groups route through their existing
  family-specific normalized Runenwerk ingress after identity remapping;
- supported all-tablet groups route through the existing whole-group device admission/staging path,
  so historical/current/predicted samples remain one atomic validation and product-staging unit;
- unsupported families or non-self-contained group shapes fail before replay begins.

A multi-observation group containing any non-tablet observation is Unsupported until Runenwerk has
an owner-correct projection contract for that group shape. Replay MUST NOT split such a group into
independent admissions.

The replay path MUST NOT add a second reducer, reconstruct missing initial state, synthesize text
input from keyboard observations, or rebuild frame-local projections after the fact.

### App-frame replay

For every completed trace frame, replay ordering is:

~~~
for frame in trace.frames:
    remap and admit/project each group in recorded admission order
    advance exactly one canonical App frame
~~~

An empty trace frame advances one canonical App frame with no injected group.

The first normalized replay executor is in-process and headless-only. Current canonical explicit
multi-frame advancement is an `App::headless()` capability; native-window Host execution has a
different event-loop owner and may admit concurrent physical/native input. The first replay slice
MUST NOT pretend that headless App-frame advancement proves or controls that native Host path.

A later native/attached normalized replay path requires its own concurrency and Host orchestration
decision.

Replay MUST NOT substitute simulation ticks, render-frame counters, Host redraw count, sleeps,
source time, wall-clock time, or presentation intervals for the recorded App-frame boundary.

### Trailing groups

A trace with non-empty `trailing_groups` has observations whose App-frame membership was not
established.

The first normalized replay contract MUST reject such a trace before mutating the target App.
It MUST NOT drop the groups, append them to the previous frame, or fabricate another frame.

A later explicit trace-to-scenario transformation may decide how to handle those observations.

### Initial state, exclusivity, and cleanup

Normalized replay requires explicit caller control of the target App and fresh replay-owned source
IDs.

The caller MUST choose replay-owned source identities that do not already carry target-App input
state. The first replay implementation does not clear those identities to manufacture a clean
baseline and does not snapshot/restore unrelated input.

The first executor is headless-only, but a headless App may still contain input state established by
earlier programmatic activity. Runenwerk's current pointer compatibility projection asks whether a
button is held **anywhere**, across all admitted sources. Source remapping therefore does not by
itself isolate replay semantics.

Before the first replay input mutation, preflight MUST reject `TargetStateConflict` when any
pointer button used by the supported trace is already held by target-App input state. It MUST NOT
clear that other source to make replay proceed.

The target's frame-local input projection must also be quiescent before the first replay frame.
Pending relative-motion delta, scroll delta, pointer transitions/edges, or native-tablet staging
from earlier programmatic activity would otherwise be combined with the first replay frame.
The executor MUST reject that state as `TargetStateConflict`; it MUST NOT call `clear_frame()` to
erase unrelated evidence merely to make replay proceed.

Replay also MUST reject a target App whose admitted-input capture is already active or whose
automation input trace recorder is active. Replay must not steal, reset, or silently contaminate
another capture/evidence owner.

Trace-shape, source-mapping, and trailing-group preflight MUST complete before App startup or replay
input mutation. Target-state conflict checks MUST run before replay input mutation and SHOULD be
repeated after startup if startup can establish relevant input state.

A trace that requires pre-capture held/contact/absolute-pointer state is Unsupported under the first
replay contract rather than being silently approximated.

Replay-owned held state is cleaned through existing source-scoped continuity semantics on
cancellation or explicit replay teardown. Cleanup MUST NOT synthesize physical Up/Cancel
observations for unrelated sources.

Product/owner assertions that need the post-replay held state MUST run before replay teardown.
Teardown is an orchestration lifecycle action, not part of the recorded trace.

### Replay result knowledge

Replay result knowledge MUST distinguish at least:

- Completed;
- UnsupportedTraceShape;
- InvalidSourceMapping;
- InvalidOrRejectedInput;
- UnframedTrailingGroups;
- TargetStateConflict;
- Cancelled;
- InfrastructureFailure.

A source map is invalid when it omits a recorded source or maps distinct recorded sources onto the
same replay source. This is detected before target mutation.

A failure after earlier frames were applied MUST report partial progress, including the last
completed frame or the failing frame/group location. It MUST NOT report the whole trace as
successfully replayed merely because iteration started.

`Completed` means all supported normalized groups were replayed and all recorded canonical App
frames were advanced. It does not by itself prove that every original product effect was reproduced.
Product equivalence requires an owner-specific query/assertion after replay.

### Ownership and fidelity

Normalized observed-trace replay proves only the normalized-input path:

~~~
recorded normalized group
    -> explicit replay identity remap
        -> RunenInput validation/reduction
            -> Runenwerk consumer projection
                -> canonical App advancement
                    -> product behavior
~~~

It does not prove native OS event generation, winit acquisition, physical-device timing, raw-device
motion delivery, stable cross-run product targeting, or persisted trace portability.

Scene replay and RunenUI replay remain separate owners.

## Persisted normalized replay-trace artifact

A6 proves that one supported in-memory `AutomationInputTrace` can replay through the normalized
input path and reproduce an owner-observed Render Lab camera result. Persistence is a separate
contract.

The first durable automation artifact is a **persisted normalized replay trace** owned by Runenwerk
automation/integration policy. It is not a scenario, product target selector, product assertion,
native-input recording, Scene replay, RunenUI replay, or CLI protocol.

ADR 0014 applies: the persisted trace format requires an explicit owner, identifier, version,
validation policy, and migration policy. Runtime IDs MUST NOT silently become persisted identity.

### Artifact identity and first encoding

The first format is:

~~~
artifact_kind = "runenwerk.automation.normalized-replay-trace"
schema_version = 1
encoding = RON
~~~

RON is the first local handoff encoding because the data is variant-rich and human-reviewable,
Runenwerk already uses validated RON for durable authored formats, and Engine already depends on
`serde` and `ron`. This does not make RON a future network or IPC protocol.

The persisted schema MUST be a dedicated storage representation. Runenwerk MUST NOT make
`AutomationInputTrace` itself the durable schema, derive persistence on RunenInput runtime types
merely for this format, or treat Rust enum layout as an undocumented compatibility promise.

The conversion boundary is explicit:

~~~
AutomationInputTrace
    -> persisted replay-trace schema V1
        -> RON
        -> parse + validate V1
            -> AutomationInputTrace
                -> accepted A6 replay
~~~

### V1 replayable scope

V1 represents only the normalized-input subset already accepted by A6:

- canonical App-frame order, including idle frames;
- pointer-button observations;
- relative motion with its unit domain;
- scroll axes, domain, and phase;
- supported all-tablet atomic groups;
- exact group order and atomicity.

Export MUST fail closed rather than silently omit or rewrite:

- non-empty trailing groups;
- keyboard observations;
- absolute-pointer observations;
- ordinary contact observations;
- continuity-loss observations;
- mixed multi-observation non-tablet groups;
- empty groups;
- any other shape rejected by A6 first-slice replay preflight.

A later format version may expand this scope only through an explicit compatibility/migration
decision.

### Trace-local persisted identity

`InputSourceId`, `InputDeviceId`, `ContactId`, and `ToolId` are runtime/session identities.
Their raw numeric values are not durable identity.

V1 projects runtime identity into deterministic trace-local slots.

Source slots are assigned by first appearance in canonical frame/group order. Device slots are
scoped beneath a source. Tablet contact/tool slots preserve equality, distinction, and lifetime
relationships inside the artifact without claiming cross-run hardware identity.

Import MAY materialize these slots into deterministic in-memory runtime IDs solely to reconstruct
one `AutomationInputTrace`. Those materialized IDs remain trace-local tokens. A6 still maps the
reconstructed recorded sources to fresh replay-owned sources before target mutation.

Tablet source time persists its recorded value and unit plus trace-local source/device context.
Import MUST reconstruct source-time context consistently with the enclosing group.

### Provenance is not compatibility

V1 SHOULD preserve inspectable provenance such as the producing Runenwerk revision, producing
RunenInput revision or contract provenance, capture Host class when known, and bounded
human-readable label/description metadata.

Those values are evidence. Exact Git revision equality is not the V1 parsing rule.

Compatibility is established by:

1. supported artifact kind and schema version;
2. persisted-schema validation;
3. successful conversion into current normalized runtime semantics;
4. current A6 replay preflight.

Product identity, fixture identity, owner target identity, product queries/assertions, and expected
product outcomes do not belong in the generic normalized trace artifact. Those are later
scenario/caller concerns.

### Validation and untrusted input

Persisted V1 validation MUST reject at least:

- wrong artifact kind;
- unsupported schema version;
- malformed or missing required fields;
- non-contiguous frame ordinals;
- invalid, duplicate, or contradictory trace-local identity references;
- non-finite numeric values;
- invalid measurement domains;
- invalid tablet source-time unit/context;
- unsupported A6 observation/group shapes;
- a release-first pointer state for one trace-local source/device/button;
- trailing/unframed input;
- structurally empty groups.

Where practical, conversion SHOULD finish by exercising existing RunenInput and A6 validation rather
than copying reducer laws into the persistence layer.

A persisted trace is untrusted input. The first loader MUST enforce explicit practical bounds for
input bytes, frame count, group count, observations per group, and human-readable metadata length.
Over-limit input fails with a typed load/validation result.

### Migration policy

V1 has no predecessor migration.

Unknown future schema versions fail closed. A future V2+ decision MUST explicitly choose whether to
continue accepting V1, provide a reviewed V1-to-V2 migration, or reject V1 with an unsupported-
version diagnostic. Old artifacts MUST NOT be silently reinterpreted under changed runtime enum
meaning.

### Trace versus authored scenario

The persisted replay trace states only that normalized input facts were recorded in one canonical
App-frame order.

It does not state which product to launch, which fixture or owner target to resolve, which product
query to issue, which assertion must pass, or which evidence to request.

A later authored scenario/caller may reference a persisted trace and compose it with product-owned
target/query/assertion semantics. The trace format itself MUST NOT create a universal product
command, target, or assertion vocabulary.

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

## Accepted implementation progression and next slice

The accepted automation sequence is now:

1. A2 proves typed in-process automation sessions, ProductSemantic/NormalizedInput mode separation,
   scoped automation-owned input cleanup, condition waits, and independent Render Lab and Editor
   owner adapters.
2. A3 adds opt-in exact admitted-`InputObservationGroup` capture at the Runenwerk input boundary.
3. A4 partitions that capture by canonical App frame while preserving idle frames and explicit
   trailing groups.
4. A5 defines truthful normalized observed-trace replay semantics.
5. A6 implements bounded in-memory headless replay and proves record -> replay -> typed Render Lab
   camera equivalence for the accepted first-slice input families.

After this persisted-artifact design is accepted, create exactly one bounded implementation issue
for persisted replay-trace V1.

That proof SHOULD:

1. define a dedicated V1 storage schema rather than serializing runtime trace types directly;
2. export one A6-replayable in-memory trace to pretty RON;
3. parse and validate it back under explicit resource bounds;
4. reconstruct equivalent trace-local source/device/contact/tool relationships without preserving
   raw runtime ID values as durable identity;
5. reject unsupported A6 shapes during export and malformed/unsupported artifacts during import;
6. preserve idle frames, group order, supported tablet atomicity/source-time semantics, motion units,
   scroll semantics, and pointer transitions;
7. prove semantic round-trip equivalence modulo runtime-ID remapping;
8. feed the imported trace through accepted A6 replay;
9. prove Render Lab record -> persist -> parse -> replay -> typed camera equality;
10. require no new serialization dependency while current Engine `serde`/`ron` ownership remains
    sufficient.

Do not add a production CLI, generic scenario AST, IPC, remote attach, native OS automation, Scene
replay changes, or RunenUI replay changes in that slice.

After persisted V1 is accepted, reassess the remaining #731 terminal-handoff gap. In particular,
decide from demonstrated pressure whether the next layer should be a product-local terminal caller,
a shared authored scenario representation, trace-to-scenario transformation, step/result history,
or another owner-specific integration. Their order is not pre-authorized here.

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
contains only this already-indexed design authority.

After accepted-main verification, create exactly one implementation issue for the persisted
normalized replay-trace V1 proof above.

Do not pre-authorize a production CLI, authored scenario persistence, IPC, remote attach, or native
OS automation in that implementation slice.
