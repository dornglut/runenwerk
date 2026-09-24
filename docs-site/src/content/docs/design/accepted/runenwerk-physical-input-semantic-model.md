---
title: Runenwerk Device-Level Input Observation Semantic Model
description: Accepted normalized semantic model for backend-neutral device-level observations, deterministic admission and state reduction, provenance, coordinates, keyboard meaning, reconciliation, and consumer adaptation before internal boundary repair.
status: accepted
owner: engine
layer: engine-runtime / input
canonical: true
last_reviewed: 2026-09-13
related_adrs:
  - ../../adr/accepted/0024-normalize-physical-input-observation-semantics.md
  - ../../adr/accepted/0023-normalize-app-runtime-host-lifecycle-and-capability-ownership.md
  - ../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
  - ../../adr/accepted/0018-semantic-federation-and-physical-realization.md
  - ../../adr/accepted/0019-batteries-included-application-composition.md
related_designs:
  - ../active/native-tablet-input-and-latency-contract.md
  - ./runenwerk-render-lab-product-design.md
  - ./runenwerk-app-runtime-and-composition-semantic-model.md
related_docs:
  - ../../architecture/runenwerk-platform-architecture.md
  - ../../architecture/ui-framework-architecture.md
---

# Runenwerk Device-Level Input Observation Semantic Model

## Status

This document is the accepted semantic target for Runenwerk's **internal input ownership
repair** following issue #557.

It is a semantic-model design. It is not a public Rust API specification, not an extraction
plan, and not a claim that current code already conforms.

Evidence and reconciliation revisions:

```text
INPUT_CENSUS_BASE=423eb85dbd46ce1106c725784775183c1663f02f
REVIEWED_RUNENWERK_MAIN=3282079f9c50865000c367aa4a8889372666547e
ENGINEERING_MAIN=dfae38c801600c591c38ae14c5418cfb0f89c8ac
RUNENUI_MAIN=ad75903d2d91910e1b105f1abcf50d15a2caa770
```

Between `INPUT_CENSUS_BASE` and `REVIEWED_RUNENWERK_MAIN`, accepted changes were RunenRender
#598 and App semantic normalization #613. Exact changed-file reconciliation found no input,
platform-input, editor-input, or native-tablet source changes, so the #557 input census remains
valid. ADR 0023 now additionally confirms that Runenwerk platform-event occurrence does not
become reusable input semantic authority and that current mixed input state is not App-core
semantic state.

Code and tests remain authority for current behavior. ADR 0024 and this design define target
semantics for later bounded I1 implementation work.

Normative words `MUST`, `MUST NOT`, `SHOULD`, and `MAY` are deliberate.

The historical file path retains `physical-input` only to avoid unnecessary authority-path churn
inside this bounded documentation slice. The normalized terminology in this design is
**device-level input observation**.

## Goal

Establish one normalized internal semantic boundary that can represent demonstrated keyboard,
pointer, button, scroll, touch, stylus, and relative-motion observations without importing:

- `winit` or another backend API into the semantic owner;
- Runenwerk application action vocabulary;
- RunenUI focus/routing/capture/widget/text-editing authority;
- application/window/event-loop lifecycle policy;
- RunenECS semantics merely because state is hosted in ECS;
- a speculative standalone RunenInput framework.

The target relation is:

```text
OS / platform / device / virtual source
    -> backend adapter
        -> ordered normalized device-level observations
            -> deterministic admission
                -> confirmed-state reduction where required
                    -> Runenwerk product/action adaptation
                    -> RunenUI interaction adaptation
                    -> drawing/stylus consumers
                    -> later Render Lab camera consumers
```

The normalized boundary answers:

> What input-bearing fact did the source/backend report or reconcile, with what identity,
> coordinate, unit, ordering, time, availability, certainty, delivery role, and origin?

It does not answer:

> What should the application, editor, camera, gameplay system, or UI do because of that fact?

## Terminology law: device-level does not mean raw hardware truth

“Device-level” describes semantic layer, not epistemic purity.

A source may be:

- physical hardware;
- an OS-transformed or accelerated stream;
- a virtualized device or pointer stream;
- a remote/compatibility source admitted by a backend;
- a backend prediction or estimate;
- adapter-generated reconciliation evidence.

The model MUST therefore preserve what the backend can truthfully establish. It MUST NOT relabel
platform-adjusted cursor motion as raw device motion, virtual source identity as stable hardware
identity, or synthetic reconciliation as physical transitions.

Use of terms such as “physical key” means layout-independent control identity where that semantic
identity is known. It does not assert direct electrical/hardware provenance.

## Current exact-source census

At `REVIEWED_RUNENWERK_MAIN`, `engine/src/plugins/input/state.rs` remains the principal mixed
predecessor. It combines:

- `winit::event::{WindowEvent, DeviceEvent, ElementState, MouseButton}`;
- `winit::keyboard::{KeyCode, PhysicalKey}`;
- ECS `Component` / `Resource` hosting;
- held keys/buttons;
- pointer position and relative motion;
- wheel accumulation;
- touch samples;
- text input and editing commands;
- application bindings;
- action down/pressed state;
- concrete UI/world/system/scene/editor-facing action flags;
- frame-local reset/finalization behavior;
- backend event handling.

`engine/src/plugins/input/actions_and_bindings.rs` still couples binding mechanics to winit
`KeyCode` plus concrete action names such as `ui.*`, `world.*`, `system.*`, and `scene.*`.

### Current platform normalization losses

`engine/src/runtime/platform.rs` currently exposes a backend-shaped `PlatformEvent` surface. The
winit runner maps into it before current consumers.

The current keyboard path preserves only:

```text
physical KeyCode
pressed/released state
optional text
```

and drops or fails to represent winit facts that are required by the current standalone RunenUI
keyboard ingress contract:

```text
device identity
logical/layout-resolved key
key location
repeat
synthetic/reconciliation provenance
unidentified physical-key identity
```

The current scroll path maps both line and pixel scroll to one scalar vertical `f32`, losing:

```text
horizontal component
measurement domain (line vs pixel)
device identity
phase/lifecycle where supplied
source/transformation provenance
```

The current raw-device path ignores the winit `DeviceId` and preserves only raw mouse-motion
numbers. Cursor and mouse-button window events similarly discard their input-device identity.
Touch preserves contact id, position, phase, and normalized force but not the winit device id.

These losses are current implementation facts to repair. They are not accepted semantic policy.

### Current editor translation defects

`apps/runenwerk_editor/src/runtime/composition/input.rs` independently translates
`PlatformEvent` into the old UI-input model and currently demonstrates several ownership defects:

- physical `KeyCode::KeyZ` is converted directly into logical character `"z"`;
- the same physical-to-character mapping is layout-wrong for layouts such as QWERTZ/AZERTY;
- scroll is projected to `(0, delta)` after upstream domain/x-axis loss;
- touch contact id and pressure are not represented in the produced UI pointer event;
- touch cancellation is converted directly into the UI semantic command `Cancel`;
- raw relative motion is ignored by this UI path.

I1D must remove this duplicate semantic translation only after the normalized replacement proves
owner-correct behavior.

### Current native-tablet pressure

The native tablet adapter demonstrates a second and richer producer, including:

- device/tool identity and capabilities;
- position;
- pressure and tangential pressure;
- tilt/orientation;
- twist;
- hover/proximity, eraser, barrel controls, and contact;
- source timestamps;
- historical/coalesced samples;
- predicted samples;
- calibration;
- backend health/control.

Its current implementation maps largely into old UI-input packet vocabulary. That packet model is
evidence of required facts, not the target neutral owner.

Notably, its current `PointerSampleRole::{Raw, Coalesced, Predicted}` shape conflates delivery
history with epistemic certainty. The normalized model fixes that category error rather than
copying it.

### Standalone RunenUI evidence

Current `runenui_core` already distinguishes:

```text
PhysicalKey
LogicalKey
KeyboardPhase
KeyLocation
KeyboardCompositionState
CommittedTextEvent
```

and its keyboard event keeps physical key, logical key, location, repeat, modifiers, and optional
device identity distinct.

Current `runenui_winit` additionally distinguishes synthetic native keyboard transitions and maps a
synthetic release to UI `Cancel` rather than ordinary `Up`.

This is important consumer evidence:

- logical-key meaning is already required by a real downstream framework;
- physical key remains the held-key identity;
- logical key is observation/interaction metadata, not committed text;
- backend-generated reconciliation cannot be flattened into ordinary physical edges.

The normalized Runenwerk input boundary must preserve enough backend-neutral information to build
that ingress through one explicit adapter without forcing RunenUI to re-read winit directly.

## Ownership model

### Neutral device-level input semantics own

Only backend-independent input meaning useful without RunenUI, product actions, window lifecycle,
or ECS hosting:

- source-local identity and ordering;
- deterministic admission order inside one normalized reducer domain;
- device/tool/contact/control identity when established;
- keyboard transition facts plus required backend-neutral logical/location/repeat/reconciliation
  metadata;
- pointer/spatial facts with explicit coordinate and unit semantics;
- raw/relative motion with truthful unit domain;
- scroll facts with axes, measurement domain, source mechanism, and provenance;
- touch-contact lifetime and optional measurements;
- demonstrated stylus/tool facts;
- source measurement time when available;
- evidence certainty, delivery role, and origin/reconciliation provenance;
- explicit omitted/unavailable/unknown semantics;
- explicit stream continuity loss;
- deterministic confirmed-state reduction where maintained consumers need state.

### Backend/platform adapters own

- winit, Win32/Windows Pointer/Ink, Wayland/libinput, Cocoa, web, or other native API types;
- native handles and event-loop attachment;
- backend-specific acquisition and decoding;
- backend clock extraction;
- backend-native identities before normalization;
- backend defaults/quirks;
- source-specific calibration acquisition;
- backend health/control APIs;
- prediction/history acquisition;
- translation into the normalized semantic model.

### Runenwerk host/application integration owns

- application lifecycle;
- native window creation/destruction/focus/redraw/event loop;
- target-surface lifecycle;
- ECS hosting wrappers;
- frame/tick scheduling;
- product/editor/game/camera action policy;
- action bindings and presets;
- host recovery decisions;
- deterministic merge/admission orchestration when multiple normalized sources feed one consumer
  domain.

### RunenUI owns

- UI focus;
- routing;
- capture;
- widget targeting;
- UI shortcut semantics;
- text editing/selection;
- IME/composition interaction behavior;
- accessibility interaction semantics;
- UI gesture meaning;
- UI pointer/key lifetime semantics after admission into RunenUI.

### Native-tablet owner retains

The current tablet backend retains backend acquisition, calibration, health/control, history and
prediction acquisition, native timestamps, and OS-specific recovery. The neutral model consumes
only normalized observations/provenance.

### Future standalone ownership remains undecided

This design does not authorize a `runen-input` repository/crate. I3 may accept extraction only
after internal repair and two real consumer families prove the contract independently of
Runenwerk lifecycle/product policy.

## Semantic principles

### SP-1 — Meaning before representation

The semantic distinctions are fixed before Rust enums/structs. One backend callback type must not
become one normalized event type by default.

### SP-2 — Observation is not state

```text
observation
!= accumulated confirmed state
!= transition projection
!= product action
```

An observation records source evidence. A reducer derives state. A product may derive frame-local
or action-local projections.

### SP-3 — Backend-neutral is not information-erasing

Normalization removes backend API ownership while retaining semantic differences required for
correctness.

### SP-4 — No invented device facts

Adapters/reducers MUST NOT create ordinary device transitions merely to make downstream state
convenient.

### SP-5 — Ordering is explicit and layered

Source order, admission order, and source time are different authorities.

### SP-6 — Provenance dimensions are orthogonal

Certainty, delivery/history, and observation origin MUST NOT be collapsed into one “sample role.”

### SP-7 — Unknown, unavailable, omitted, and zero are distinct

Backend defaults are not automatically measurements.

### SP-8 — Loss may be explicit, never silent

If a target adapter intentionally discards a fact, that disposition belongs to its contract. A
consumer requiring the fact blocks the lossy mapping rather than silently accepting degraded
meaning.

### SP-9 — Consumer policy remains above the neutral model

No product action, camera operation, widget target, UI command, text-edit operation, or window
lifecycle decision is a device-level input fact.

## Core ontology

The concepts below are semantic roles, not required one-to-one Rust types.

### Input source

An **InputSource** is the minimum provenance/order domain for one normalized stream.

It may correspond to a physical device stream, winit/window stream, native tablet stream,
virtualized source, or another ingestion path.

A source is not automatically one hardware device. Source identity is runtime/session scoped by
default.

### Device instance

A **DeviceInstance** is an optional source/session-scoped identity for a distinguishable input
device instance when the backend can establish it.

It MUST NOT imply:

- persistent hardware serial identity;
- global uniqueness;
- cross-run continuity;
- user identity.

Reconnect/recreation yields a new instance unless continuity is explicitly proven.

### Tool instance

A **ToolInstance** is optional identity for a distinguishable stylus/transducer/tool. It is
separate from digitizer/device identity and need not be persistent.

### Contact / interaction lifetime

A **ContactId** distinguishes active direct-contact lifetimes such as touch contacts. Identity is
valid only within its documented scope and lifetime. Source numeric tokens may be reused after
terminal end/cancel.

### Control identity

A **ControlId** identifies a device-level digital/physical control whose identity matters to
consumers, such as a keyboard location, mouse button, or stylus barrel control.

Known semantic control identities and source-local opaque fallback identities are both allowed.
Unrecognized-but-distinguishable controls MUST NOT collapse to one undifferentiated `Unknown`.

### Target/spatial context

Some observations require correlation to a Runenwerk-owned target surface or coordinate basis.
The neutral model may carry an opaque correlation required to interpret spatial values. It does
not own the target lifecycle, focus, capture, or routing.

### Observation

An **InputObservation** records one device-level fact or coherent set of facts normalized from a
source report or reconciliation operation.

Examples:

- keyboard control reported down;
- logical key interpretation accompanying that transition;
- absolute pointer position;
- raw relative motion;
- 2D scroll delta in an explicit domain;
- contact begin/update/end/cancel;
- stylus pressure/orientation/twist;
- tool proximity;
- continuity loss;
- state reconciliation assertion.

### Observation group

An **ObservationGroup** is one atomic logical source update.

Facts that belong to one source sample and must not expose impossible intermediate state may share
a group. Chronologically distinct transitions never share a group merely because the backend
delivered them in one callback.

### SourceSequence

Every admitted group has a deterministic strictly ordered **SourceSequence** within its source
stream.

SourceSequence is not a timestamp, frame number, persistence identity, or global chronology.

### AdmissionSequence

Every group admitted into one normalized reducer/consumer domain receives a deterministic strictly
ordered **AdmissionSequence**.

AdmissionSequence is the authority for deterministic reduction/replay inside that admission
domain. It may merge multiple source streams deterministically.

It MUST NOT be interpreted as proof that physical event A occurred before physical event B in real
world time when their source clocks are unrelated.

The exact Rust representation and merge mechanism are deferred; the semantic requirement is not.

### SourceTime

An observation MAY carry source time with:

- numeric value;
- unit/resolution;
- clock domain;
- measurement/sample vs delivery meaning where known;
- wrap/monotonicity/precision limitations where known.

Unrelated clock domains MUST NOT be compared as if globally synchronized.

### Evidence status

Evidence certainty is one dimension:

```text
ObservedConfirmed
EstimatedRevisable   // only where backend exposes a real revision contract
PredictedProvisional
```

Exact names are deferred.

Observed/confirmed evidence may update authoritative confirmed state. Predicted evidence may not.
Estimated evidence requires an explicit correction/revision contract before it can be treated as
final confirmed fact.

### Delivery role

Delivery/history is a separate dimension:

```text
OrdinaryCurrent
HistoricalCoalesced
```

Coalescing means confirmed earlier samples were delivered in a batch/later report. It does not make
them uncertain or predicted.

### Observation origin

Origin/reconciliation is another independent dimension:

```text
SourceReport
BackendSyntheticReconciliation
AdapterGeneratedReconciliation
ContinuityMetaEvidence
```

Exact vocabulary is deferred. The invariant is that synthetic/reconciled state is not silently
represented as ordinary hardware truth.

## Observation grouping and chronology

### Atomic-group rule

A reducer applies one confirmed atomic group coherently. Facts that represent the same source sample
must not expose impossible intermediate states.

### Sequential-transition rule

If one backend delivery contains chronologically distinct transitions, they normalize to distinct
ordered groups.

```text
Down -> Up
```

must remain two transitions even if final held state equals initial held state.

### Batch rule

Backend callback/frame/batch boundaries are transport. One callback may contain:

- one atomic sample;
- several contacts from one sample time;
- historical/coalesced samples;
- multiple real transitions;
- current plus predicted samples.

The adapter classifies the semantics rather than copying callback shape.

### No-double-counting rule

If the backend exposes history plus an aggregate/current event that overlaps the same underlying
sample, normalization MUST prevent duplicate motion/state application.

## Deterministic state reduction

The model permits an explicit reducer for confirmed state needed by current consumers.

```text
prior confirmed state
+ next admitted observation group
    -> next confirmed state
       + derived transition/reconciliation outcomes
```

### Reducer laws

A conforming reducer MUST:

1. consume groups by deterministic AdmissionSequence;
2. preserve SourceSequence metadata independently;
3. apply one atomic group coherently;
4. preserve transition multiplicity/order;
5. use physical/control identity as held-state key where applicable;
6. treat repeat as repeat metadata, not a new pressed edge;
7. keep logical-key interpretation metadata separate from physical held identity;
8. never commit predicted evidence as confirmed state;
9. obey explicit estimated-value revision contracts rather than silently finalizing them;
10. distinguish ordinary transition, reconciliation assertion, and cancellation/invalidation;
11. terminate touch/contact lifetimes correctly;
12. invalidate affected assumptions on continuity loss without fabricating releases;
13. preserve unrelated source/device/contact/control state across scoped invalidation;
14. distinguish omitted per-sample values from capability unavailability;
15. be deterministic for semantically equivalent admitted sequences.

### State is not frame-local

Neutral confirmed state may persist across frames. `pressed_this_frame`,
`released_this_frame`, frame deltas, scroll accumulators, click counts, gestures, and similar
values are projections owned above the root input semantics.

Relative motion and scroll are impulses/observations; a reducer MUST NOT retain their last value as
if it were durable current state unless a separately defined projection explicitly does so.

## Transition, reconciliation, and cancellation

The model separates three classes of state-bearing evidence.

### Ordinary observed transition

The source/backend reports an ordinary state transition:

```text
Up -> Down
Down -> Up
ContactBegin -> ... -> ContactEnd
```

This may create ordinary transition edges.

### Reconciliation assertion

A backend may synthesize or assert current state to reconcile after focus/acquisition changes.
Current winit exposes this explicitly through `is_synthetic` keyboard events on supported hosts.

A reconciliation assertion MAY restore known state, but MUST NOT masquerade as a newly observed
physical press/release edge.

Example:

```text
backend synthetic press on focus gain
```

may establish “source currently reports this control down” without producing a normal user-press
edge.

### Cancellation/invalidation

A synthetic release on focus loss, source loss, backend reset, or adapter continuity loss may mean
that prior state can no longer be trusted rather than that physical release was observed.

The normalized result is cancellation/invalidation/reconciliation evidence. Consumers may map this
to action stop or UI Cancel under their own semantics.

The neutral model MUST NOT fabricate ordinary Up/End observations solely to clear held state.

## Keyboard semantics

### Required distinctions

```text
physical key/control identity
!= logical/layout-resolved key meaning
!= committed text
!= IME composition state/text
```

### Physical key

Physical key is layout-independent control/location identity where the backend can provide one. It
is the primary key for held-state correlation.

A backend enum such as winit `KeyCode`, SDL scancode, Windows scan code, or USB HID usage is not
itself the public semantic owner.

Unknown/unidentified keys use a source-local fallback capable of correlating press/release when the
backend supplies such a token.

### Logical key

Logical key is the backend/platform interpretation of key meaning under the current layout and
platform rules. It is device-level observation metadata useful to downstream consumers, including
current RunenUI.

Logical key MUST NOT replace physical key as held-state identity.

The normalized keyboard observation MUST preserve logical meaning when the backend supplies it and
a maintained consumer requires it. It may be absent/unknown on backends that cannot establish it.

### Key location

Left/right/numpad/standard location is preserved when backend-provided because current RunenUI
exposes it and because location can matter independently from logical text.

### Repeat

Repeat is metadata on a continued key-down lifetime. It is not a new physical press edge.

### Device/source identity

Keyboard observations preserve source identity and backend-neutral device-instance identity when
available. State correlation is conceptually by source/device plus physical key as required to avoid
aliasing independent devices.

### Synthetic/reconciliation provenance

Backend-generated synthetic keyboard events retain origin/reconciliation provenance. They are
processed according to the transition/reconciliation rules above, not flattened into ordinary
Down/Up.

### Modifiers

Modifier state may be projected from normalized physical keys or preserved as a backend snapshot
where a target consumer requires it. A backend snapshot MUST NOT overwrite independently tracked
left/right physical modifier identity.

### Text/IME remains separate

Committed text and IME composition do not enter physical held state. They remain a sibling
platform/RunenUI text path.

One Runenwerk integration adapter MAY combine:

```text
normalized keyboard observation
+ separately owned text/IME/composition context
+ RunenUI target context
    -> RunenUI keyboard/text ingress
```

This is explicit composition of owner-correct facts, not duplicate authority.

## Button and digital-control semantics

Digital controls preserve identity, ordinary transitions, repeat/reconciliation provenance where
applicable, and source/device scope.

A neutral button identity MUST NOT encode application meaning such as `OrbitCamera`, `EraseStroke`,
or `OpenContextMenu`.

Likewise, platform/UI notions such as Primary/Secondary button MUST NOT replace known physical
left/right/other identity unless the source contract actually reports semantic primary/secondary
rather than physical position.

## Absolute pointer-position semantics

Absolute pointer/cursor position is an observation in an explicit coordinate space.

A bare `(x, y)` without a defined basis/unit is incomplete semantic input.

The founding winit window path demonstrates:

```text
target-surface-local physical-pixel-like coordinates
```

but this space is OS/window processed and MUST NOT be called raw device position.

The model MUST NOT silently treat:

- screen coordinates as surface coordinates;
- UI logical coordinates as physical pixels;
- device/native coordinates as target coordinates;
- normalized tablet coordinates as window coordinates.

Target correlation may be carried without moving target/window lifecycle ownership into input.

## Relative-motion semantics

Relative motion is independent from absolute cursor position.

Current winit raw `DeviceEvent::MouseMotion` demonstrates a source-relative, unfiltered/raw motion
quantity in unspecified device units. It is suitable for camera-like consumers precisely because it
is not equivalent to OS-processed cursor movement.

The neutral model preserves the unit limitation. It does not invent pixels/metres.

A camera/product applies sensitivity, acceleration, dead zones, or other policy above this fact.

## Scroll semantics

A scroll observation preserves at least, when the source provides them:

```text
x component
y component
measurement domain
source mechanism
phase/lifecycle
source/device identity
raw/platform-adjusted provenance
```

Demonstrated measurement domains include:

- line/row-oriented displacement;
- physical/surface-pixel-like displacement;
- discrete wheel steps;
- high-resolution fractions of a wheel step;
- angular wheel motion;
- another explicitly defined continuous source domain.

Source mechanism and unit are distinct concepts.

Adapters MUST NOT silently:

- drop x;
- convert line to pixel or vice versa without a real conversion law;
- relabel platform-adjusted direction as raw finger/wheel direction;
- collapse continuous and discrete domains merely because one consumer wants a scalar zoom amount.

## Touch/contact semantics

A contact lifetime is:

```text
Begin -> Update* -> End
                 \-> Cancel
```

Rules:

- Begin creates one active contact lifetime;
- Update references an active contact;
- End terminates normally;
- Cancel terminates because ordinary completion is not established;
- Cancel is not ordinary End/Up;
- simultaneous contacts retain independent identities;
- source token reuse is allowed only after the prior lifetime terminates;
- contact id, device/source identity, position, pressure, and geometry stay independent facts.

A touch-frame containing several contacts MAY form one atomic group when the backend establishes one
real sample boundary.

Missing pressure/geometry remains missing or unavailable; it is not zero.

## Stylus/tool semantics

Stylus semantics preserve, where established:

```text
device/digitizer identity
tool/transducer identity
proximity lifetime
tip/surface contact
absolute position
pressure
tool-axis orientation / tilt
axial twist
tangential/auxiliary analog control
barrel/auxiliary buttons
eraser/inversion fact
contact geometry
source time
certainty
delivery role
origin/reconciliation provenance
```

### Proximity vs contact

Proximity means the source/digitizer currently observes the tool for hover/axis reporting. It is not
tip contact.

### Pressure

Pressure is a measurement only when supported/reported. A normalized fraction may be used only
under an explicit range/calibration contract. A backend default for unsupported pressure is not a
measurement.

### Orientation

The semantic quantity is tool-axis orientation relative to a defined surface coordinate frame.
Backends may express this as x/y tilt, altitude/azimuth, or another equivalent representation.
Conversion requires defined axes, units, handedness/reference, and controlled loss.

Exact Rust representation remains deferred.

### Twist

Twist is periodic axial rotation around the tool axis and remains distinct from tilt/orientation.

### Tangential/auxiliary analog

Tangential pressure/airbrush slider is distinct from tip pressure. Signed/unsigned/normalized/raw
range semantics are preserved truthfully rather than forced to `[0,1]`.

## Analog-value semantics

There is no universal analog scalar contract.

Each normalized analog quantity defines as relevant:

```text
quantity meaning
valid range
unit
neutral point
periodicity
coordinate/basis context
normalization/calibration provenance
per-sample presence
capability availability
```

A generic `axis: f32` bag is not an accepted neutral semantic API.

## Availability/presence semantics

These states are distinct:

```text
measured value = 0
field omitted/not reported in this sample
known unsupported/unavailable capability
unknown capability/state
not applicable to this source/tool/control
```

A source may support pressure while omitting a pressure value in one sample. That omission MUST NOT
be interpreted as capability loss or measured zero.

Likewise, unknown capability is not equivalent to known unsupported.

## Time and ordering semantics

### Source order

SourceSequence gives deterministic order inside one source stream even if timestamps are missing,
equal, wrapped, or non-monotonic.

### Admission order

AdmissionSequence gives deterministic total processing order inside one reducer/consumer domain,
including across multiple sources.

The admission merge policy is owner/integration behavior and must itself be deterministic. It does
not invent a global physical chronology.

### Source timestamps

SourceTime retains its clock domain and measurement/delivery meaning where known. It is diagnostic
or semantic timing evidence, not a substitute for AdmissionSequence.

### Frames are not source time

Frame/tick ids may be correlated downstream but do not become root measurement time or input
identity.

## Evidence certainty, history, and prediction

### Observed/confirmed

Confirmed evidence reports source observations that the backend considers occurred.

### Historical/coalesced delivery

Coalesced/history is delivery role, not certainty. Historical samples remain confirmed observations
when the backend says they occurred. They retain chronological order/time when available.

### Estimated/revisable

Some platforms may mark current/past properties as estimates that can later receive a correction.
Such values MUST retain estimated/revisable status and a usable correlation mechanism if the model
accepts them.

The exact generic correction API is deferred until a maintained consumer requires it. Until then,
an adapter MUST NOT silently publish revisable estimates as stronger confirmed facts.

### Predicted/provisional

Predicted samples describe possible future input. They:

- remain provisional;
- do not update authoritative confirmed held/contact state;
- may be consumed only by consumers explicitly opting into provisional data;
- are replaceable/discardable without altering confirmed history;
- are superseded/invalidated according to source prediction contract when new confirmed input
  arrives.

### Batching is not provenance

A vector/packet/batch is transport/container shape. It is not itself evidence certainty or delivery
role.

## Capability semantics

Capability knowledge is scoped to source/device/tool as appropriate and distinguishes:

```text
known supported
known unsupported
unknown / not established
```

Capability says what a source can establish, not what every sample contains.

No global machine capability registry is introduced.

## Device/source lifecycle and continuity

Backend-provided device/source add/remove/reset evidence may be normalized where real.

Keep distinct:

```text
device/source availability
!= tool proximity
!= contact lifetime
!= window focus
!= UI pointer capture
```

Source/device removal or reset typically causes scoped continuity loss for state derived from that
source.

### Continuity loss

Continuity loss means prior reducer assumptions are no longer trustworthy. It is meta-evidence, not
an ordinary release observation.

The reducer invalidates/drops/marks unknown affected state according to its contract. It does not
emit fake per-control Up/End observations.

Higher owners decide how to stop actions, cancel UI interaction, end drawing previews, or halt
camera motion.

## Coordinate-space model

Every absolute spatial value names or unambiguously implies:

- coordinate basis/space;
- origin;
- axis orientation/handedness where relevant;
- unit;
- target/device correlation needed to interpret it.

Device/native space and target-surface space are distinct. Conversion belongs to the adapter/host
integration and must be known/controlled.

RunenUI logical coordinates remain a RunenUI/platform adaptation concern, not the universal neutral
space.

Relative motion has no absolute origin and carries its own unit domain.

## Information-preservation and lossiness

### Required preservation at current evidence base

A backend normalization path MUST preserve, when supplied and needed by maintained consumers:

- keyboard physical key identity and transitions;
- logical key interpretation;
- key location;
- repeat metadata;
- source/device identity;
- synthetic/reconciliation provenance;
- unrecognized-but-distinguishable key identity;
- absolute pointer position + target/space;
- raw relative motion independently;
- physical/button identity;
- 2D scroll + measurement domain;
- touch contact identity/lifetime;
- stylus device/tool/proximity/contact/buttons;
- stylus pressure/orientation/twist/tangential pressure;
- source sample time where supplied/required;
- historical/coalesced delivery role;
- prediction certainty;
- continuity/reconciliation evidence.

### Explicitly lossy target adapters

A consumer-specific mapping may discard facts only when:

1. the target contract does not require them;
2. the loss is explicit at the adapter boundary;
3. discarded facts are not replaced by fabricated semantics;
4. future consumers that require them extend the boundary rather than consume silent degradation.

### Forbidden silent losses

Do not silently:

- convert physical key into logical character;
- drop backend logical/location/repeat/device/reconciliation facts required by RunenUI;
- collapse unknown keys/buttons;
- replace raw motion with cursor deltas;
- collapse scroll x/y or unit domains;
- discard contact identity;
- map touch cancel directly to a product/UI semantic command in the neutral layer;
- conflate proximity and contact;
- substitute zero for missing analog measurements;
- conflate sample omission with unsupported capability;
- conflate coalescing with prediction;
- publish estimated/revisable facts as confirmed;
- duplicate current/history samples;
- fabricate release transitions on continuity loss.

## Backend normalization requirements

For each emitted normalized group the adapter must be able to answer enough of:

1. Which source produced it?
2. Which device/tool/contact/control identities are actually known?
3. What is its SourceSequence?
4. What AdmissionSequence did integration assign?
5. What source time/clock domain is known?
6. What is the evidence status: confirmed, estimated, predicted?
7. What is the delivery role: ordinary/current or historical/coalesced?
8. What is the origin: source report, backend synthetic reconciliation, adapter reconciliation,
   or continuity meta-evidence?
9. Which values are present in this sample versus absent?
10. Which capabilities are supported/unsupported/unknown?
11. What coordinate space/unit applies?
12. What analog domain/range applies?
13. Is any source information intentionally lost?

If the adapter cannot answer enough to make a value truthful/unambiguous, it must not publish a
stronger normalized claim.

## Founding backend disposition

### winit window/event path

Retain winit as Runenwerk host/backend realization. Normalize input-bearing facts before semantic
consumers; winit types do not survive into the neutral owner.

Keyboard normalization must use full `KeyEvent`/window-event evidence instead of the current
reduced `PlatformEvent` shape, preserving maintained-consumer facts listed above.

Text/IME remains a sibling path.

Window focus is host lifecycle evidence. It may trigger continuity/reconciliation handling but is
not itself a device transition.

### winit raw device path

Preserve raw `MouseMotion` independently for camera-like consumers and preserve its source/device
scope when winit provides it.

Do not assume raw-device identity equals a window pointer/device identity.

### native tablet path

Normalize physical/device-level tablet observations before UI/drawing adaptation while leaving
native API realization, calibration acquisition, health/control, history/prediction acquisition,
and OS recovery in the tablet backend owner.

## Consumer adaptation

### Runenwerk action mapping

```text
confirmed normalized input/state
+ product binding policy
    -> application action
```

Action vocabulary and defaults remain Runenwerk/application-owned.

Bindings may intentionally choose physical-key semantics or a separately admitted logical-key
semantic contract. The choice is product policy, not neutral input meaning.

### RunenUI

One explicit Runenwerk adapter should construct RunenUI ingress from owner-correct facts.

For keyboard, that adapter may consume:

```text
normalized keyboard observation
    physical key
    logical key
    location
    repeat
    device/source
    reconciliation/cancel provenance
+ separately owned modifier/composition/text context where required
    -> RunenUI KeyboardEvent / CommittedText / composition ingress
```

For pointer/touch/stylus it maps normalized spatial/contact facts into RunenUI's admitted
interaction protocol plus Runenwerk-owned target/scale context.

RunenUI continues to own focus, capture, routing, widget targeting, selection, text editing,
semantic actions, and accessibility interaction.

This design does not require RunenUI itself to depend directly on a future RunenInput crate. Similar
value types across frameworks are acceptable where their invariants differ; explicit adapter
mapping is preferable to a dependency introduced only to avoid translation.

### Drawing/tablet

Drawing may consume high-rate confirmed history, predictions, pressure, orientation, twist,
controls, time, and contact/proximity state. Stroke formation/correction remains drawing/product
policy.

### Render Lab camera

RL2 or another independent real-time consumer is expected to use physical-key held state, raw
relative motion, button state, scroll, and possibly target-relative cursor position.

Camera orbit/pan/zoom/sensitivity remain product semantics.

## Text, logical keys, and IME

The sibling-path rule is:

```text
keyboard device-level observation
    -> neutral input

committed text / IME composition stream
    -> text/RunenUI owner
```

Logical key belongs with the keyboard observation when backend-provided and required by a consumer,
but it remains distinct from committed text and does not become held-state identity.

Do not derive Unicode text from physical key. Do not reconstruct logical key from physical key. Do
not treat IME composition as a device transition.

## Gestures

Backend-provided gestures are not founding neutral device-level semantics merely because an API
exposes them. A real consumer must establish whether a gesture belongs to RunenUI, product policy,
or a separately accepted reusable gesture layer.

Do not synthesize fake contacts from a gesture API that does not expose underlying contacts.

## Gamepad and generic axes

Gamepad/controller semantics remain deferred because #557 did not establish current consumer
pressure sufficient to define a clean neutral model.

Generic axes likewise remain backend-local until control identity, domain, range, neutral point,
units, lifecycle, provenance, and consumer need are proven.

## Security/privacy and persistence

Runtime source/device/tool identity is session-scoped by default. The neutral model does not require
persistent hardware serial/vendor identity.

Persisted calibration/device-profile keys, replay formats, or cross-process identity require their
own owner/version/privacy/lifecycle design.

## Comparative external evidence

This design was compared against established models only for semantic pressure:

- W3C Pointer Events Level 3 demonstrates scoped pointer identity, pressure/tilt/twist/contact
  geometry, coalesced history, and predictions;
- current winit distinguishes physical key, logical key, text, key location, repeat, device identity,
  synthetic keyboard events, processed cursor motion, and raw relative device motion;
- SDL3 distinguishes scancode-like physical identity from layout-derived key meaning/text;
- libinput distinguishes tool/device identity, proximity, tip contact, capabilities, timestamps,
  physical axes, and multiple scroll domains;
- Windows pointer/pen APIs expose pressure, tilt, twist, history, and defaults that cannot always be
  interpreted as measured values.

Those systems are evidence, not Runenwerk authority. Where a backend uses a synthetic default, this
model retains explicit availability/provenance rather than upgrading it to a measurement.

## Current-source disposition matrix

| Current surface | Current role/defect | Target disposition |
| --- | --- | --- |
| `engine/src/plugins/input/state.rs` device facts | observations + state + backend types | split into internal neutral observation/state owner |
| `InputState` actions/bindings/product flags | product policy | remain above neutral input |
| `InputState` text/edit commands | text/UI policy | leave neutral confirmed state; sibling text path |
| ECS derives/hosting | integration | wrappers above semantic types |
| `actions_and_bindings.rs` `KeyCode` coupling | backend leak into product binding | consume normalized key identity after repair |
| `runtime/platform.rs` keyboard | drops logical/location/repeat/device/synthetic/unidentified facts | platform remains host owner; input-bearing facts normalize before loss |
| `runtime/platform.rs` scroll | scalar-y collapses x/domain/device/phase | preserve 2D/domain/provenance before target adaptation |
| `winit_runner.rs` raw motion | drops `DeviceId` | preserve source/device scope with raw motion |
| `winit_runner.rs` cursor/button/touch | loses some device identity | preserve available device/source correlation |
| editor `composition/input.rs` physical key -> character | layout-invalid semantic conversion | remove after normalized RunenUI adapter is proven |
| editor touch cancel -> UI semantic Cancel | neutral/device lifetime becomes UI command | cancel stays device/input evidence; UI decides semantic command |
| editor touch id/pressure drop | lossy target mapping | preserve upstream, make target loss explicit if still allowed |
| old `ui_input` sample role | `Raw/Coalesced/Predicted` conflates axes | replace with orthogonal certainty + delivery semantics in neutral model |
| standalone RunenUI keyboard | real consumer of physical/logical/location/repeat/device/cancel | explicit neutral-input -> RunenUI mapping |
| native tablet backend | native acquisition + tablet facts + UI adaptation | retain backend owner; normalize device-level observations before UI/drawing |
| native tablet calibration/health/control | operations/backend policy | retain native-tablet owner |
| product/editor/camera actions | application meaning | remain Runenwerk/app owned |

Exact files/types are re-censused for each implementation issue.

## Implementation sequencing doctrine

The accepted #557 sequence remains authoritative. Boundaries are intentionally serialized.

### I1A — internal observation/state seam

Create the smallest internal backend-neutral semantic representation and deterministic reducer seam
needed to separate device-level facts from current action state.

I1A may initially be populated through current integration paths. It does not need to complete the
full winit edge cutover, UI adapter, or tablet migration.

I1A MUST NOT create/extract a standalone repository/crate.

### I1B — winit edge normalization

Move full winit conversion to platform/backend edges. Stop exposing winit types through normalized
semantic types and stop dropping required keyboard/scroll/device facts before normalization.

### I1C — action/binding separation

Move concrete action vocabulary, bindings, presets, and frame/action conveniences fully above
neutral confirmed input state. Do not invent a reusable action framework for symmetry.

### I1D — one RunenUI adapter

Establish one explicit normalized-input-to-RunenUI adaptation path. Remove duplicate app-local
winit-to-UI translation only after behavior/ownership equivalence is proven.

Committed text/IME remains separately owned and may join at the adapter boundary.

### I1E — native tablet convergence

Make native-tablet observations enter the same neutral semantic authority before UI/drawing
adaptation. Preserve historical/predicted/estimated provenance without copying old packet category
errors.

### I2 — two structurally different consumers

Require at least:

1. drawing/tablet;
2. Render Lab RL2 camera or another equally independent real-time non-UI consumer.

The shared boundary must remain useful without consumer-specific policy leakage.

### I3 — fresh extraction decision

Only after I1/I2:

```text
NO_EXTRACTION
or
EXTRACTION_DESIGN_JUSTIFIED
```

If justified, follow ADR 0014 clean source-transfer/extraction rules. No alias, mirror, forwarding,
source include, duplicate implementation, or parallel authority.

## I1A minimum semantic proof

The future I1A issue must require executable proof for the subset it implements, including where
applicable:

1. normalized semantic types contain no winit API types;
2. every reducer domain has deterministic AdmissionSequence;
3. SourceSequence remains separately available from AdmissionSequence;
4. two sequential transitions delivered together remain two transitions;
5. atomic grouped facts cannot expose impossible intermediate reducer state;
6. repeat does not create a second pressed edge;
7. synthetic/reconciliation input cannot masquerade as ordinary device edges;
8. unknown-but-distinguishable controls do not alias;
9. physical key remains held identity while logical key/location metadata can be preserved;
10. absolute cursor position and raw relative motion remain independent;
11. scroll preserves both axes and founding measurement domains;
12. multiple simultaneous touch contacts retain identity;
13. touch cancel remains distinct from ordinary end;
14. measured zero, per-sample omission, unsupported/unavailable, and unknown remain distinct;
15. certainty and delivery/history provenance are orthogonal;
16. predicted/estimated evidence cannot silently mutate confirmed state;
17. continuity loss prevents stale state without fabricating release observations;
18. current application action vocabulary remains outside the neutral owner;
19. ECS hosting does not enter semantic identity;
20. no standalone RunenInput package/repository/dependency appears.

Do not pre-authorize exact Rust names or module layout in this design.

## Conformance model

### Representation conformance

Internal types can represent every accepted semantic distinction without backend API leakage.

### Adapter conformance

Backend translation preserves identity, ordering, unit, availability, keyboard interpretation,
certainty, delivery, reconciliation, and explicit lossiness.

### Admission conformance

Multi-source admission establishes one deterministic reducer order without pretending it is global
physical time.

### Reducer conformance

Confirmed state reduction is deterministic, transition-preserving, reconciliation-aware,
prediction-safe, and continuity-aware.

### Consumer conformance

Runenwerk actions, RunenUI, drawing, and later camera derive their own semantics without
mutating/redefining device observations.

A green test at one layer does not prove the others.

## Semantic examples

### QWERTZ/QWERTY-safe keyboard observation

```text
backend report:
    physical key = normalized key position K
    logical key = "Z" under current layout
    transition = Down
    repeat = false
    location = Standard
    device = D

neutral confirmed observation:
    physical K Down
    logical meaning "Z"
    location Standard
    device D

later text/IME path:
    committed text "z"   // if/when text is committed
```

Physical identity, logical meaning, and committed text remain distinct. A product binding may choose
physical K independently.

### Synthetic focus reconciliation

```text
K was physically held before focus changes
backend later emits synthetic press/assertion after focus gain
```

Neutral result:

```text
state may reconcile to K = held/known
no ordinary user Pressed edge is created
origin = backend synthetic reconciliation
```

A synthetic release/cancellation likewise does not claim physical Up unless the backend contract
actually proves one.

### Raw camera motion versus cursor position

```text
window cursor:
    target S
    absolute position (x, y) in surface physical-pixel-like units

raw device:
    device/source D
    relative delta (dx, dy) in unspecified device units
```

Neither is reconstructed from the other.

### Touch cancellation

```text
contact C Begin
contact C Update
source cancels C
```

Neutral result is terminal Cancel for contact C, not ordinary End and not UI semantic `Cancel`.

### Coalesced confirmed history

```text
one backend delivery:
    t1 observed confirmed historical
    t2 observed confirmed historical
    t3 observed confirmed current
```

Certainty for all three is confirmed. Delivery role distinguishes history from current. The current
sample is not applied twice.

### Prediction

```text
confirmed t3
predicted p1, p2
```

Predictions may be previewed by an opting-in consumer. They never become confirmed history merely by
being delivered.

### Estimated property

If a backend says pressure at sample S is estimated and will later update it, neutral evidence keeps
that revisable status. If the current neutral implementation has no accepted correction contract,
it must not silently advertise the estimate as confirmed pressure.

### Missing pressure

```text
contact active
pressure field omitted this sample
capability = known supported
```

is distinct from:

```text
pressure = measured 0
```

and from:

```text
pressure capability = known unsupported
```

### Scroll

A touchpad supplies 2D platform-adjusted pixel-like displacement; a wheel supplies line/step-like
movement. Both are scroll observations, but their axes/domain/source provenance remain distinct
until a UI/camera product maps them to behavior.

## Invalid abstractions

The following are explicitly rejected for I1 unless later accepted authority revises this design:

- one global god `InputState` owning device facts, product actions, text, UI, and host lifecycle;
- backend `WindowEvent`/`DeviceEvent` as semantic API;
- “raw input” terminology that promises unprocessed hardware truth;
- one giant backend-shaped `Keyboard|Mouse|Touch|Tablet` event enum copied mechanically;
- one `SampleRole::{Raw,Coalesced,Predicted}` axis that conflates certainty/history;
- one source-local order used as a false global order;
- reducer logic with no deterministic multi-source AdmissionSequence;
- logical character reconstructed from physical key identity;
- committed text reconstructed from physical/logical key;
- synthetic/reconciliation events treated as ordinary device edges;
- universal `axis: f32`;
- universal `[0,1]` analog normalization;
- raw relative motion reconstructed from cursor position;
- scalar/unitless scroll;
- one global persistent hardware id;
- synthetic release events used to hide continuity loss;
- predicted/estimated observations written into confirmed state as if final;
- UI focus/capture/routing/semantic commands in neutral input;
- product action names in neutral input;
- IME/text editing in held-key state;
- ECS identity as input semantic identity;
- window/event-loop lifecycle as input semantic authority;
- generic event bus/provider registry introduced only to transport input;
- sharing types across frameworks solely to avoid an explicit adapter;
- standalone RunenInput extraction before I3.

## Deferred questions

Deliberately deferred until real consumer pressure:

- exact normalized physical-key vocabulary and opaque fallback representation;
- exact Rust representation of source/device/tool/contact/control ids;
- exact AdmissionSequence type and merge implementation;
- exact coordinate-space token representation;
- exact stylus orientation representation;
- generic estimated-property correction protocol;
- richer device hotplug/status contract;
- gamepad/controller taxonomy;
- generic joystick/axis semantics;
- haptics/output feedback;
- generic gesture framework;
- persisted device profiles/calibration identity;
- replay/serialization format;
- networking/cross-process input forwarding;
- universal HID exposure;
- gaze/voice/motion-controller families;
- extension ABI/provider registry;
- `no_std`/embedded portability;
- standalone RunenInput package/repository;
- stable public Rust API.

Deferred does not mean forbidden. It means current evidence does not authorize the concept to shape
the founding boundary.

## Architecture fitness review

The model remains acceptable only while all of the following hold:

1. The core describes source/device-level observations, not application/UI meaning.
2. It never claims raw hardware truth without evidence.
3. Backend API types terminate at adapters.
4. Observation and confirmed state remain distinct.
5. Atomic grouping does not collapse chronological transitions.
6. SourceSequence, AdmissionSequence, and SourceTime remain distinct.
7. Multi-source reduction is deterministic without false global physical chronology.
8. Certainty, delivery/history, and origin/reconciliation are orthogonal.
9. Physical key, logical key, committed text, and IME remain distinct.
10. Logical key/location/repeat/device metadata can reach RunenUI without redefining held state.
11. Synthetic/reconciliation events cannot create false physical edges.
12. Absolute pointer position and relative motion remain distinct.
13. Spatial values have explicit coordinate spaces/units.
14. Scroll preserves both axes and measurement domain.
15. Touch supports multiple contacts and terminal cancellation.
16. Stylus proximity/contact/pressure/orientation/twist/auxiliary controls remain separable.
17. Zero, omitted, unsupported/unavailable, unknown, and not-applicable remain distinguishable.
18. Coalesced history remains confirmed while prediction remains provisional.
19. Estimated/revisable evidence cannot silently become confirmed.
20. Continuity loss never fabricates physical releases.
21. RunenUI retains interaction/text authority.
22. Runenwerk retains host lifecycle and product action policy.
23. ECS hosting does not become semantic ownership.
24. A future extraction decision follows proven consumers and ADR 0014 rather than repository symmetry.

## Acceptance and successor gate

This design is documentation authority only.

After this design and ADR 0024 are merged and accepted-main validation is green:

1. re-resolve Runenwerk and Engineering exact `main` revisions;
2. re-census current device/action/UI/backend ownership;
3. reconcile intervening App/runtime/tablet/UI changes;
4. create exactly one decision-complete I1A internal boundary-repair issue;
5. implement only that bounded seam;
6. require focused tests, `cargo validate`, exact-head CI, complete-diff review, and accepted-main
   evidence;
7. derive later I1 slices one at a time from accepted state.

Do not combine this documentation slice with Rust implementation or extraction.
