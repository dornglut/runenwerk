---
title: Normalize Device-Level Input Observation Semantics
description: Durable ownership and normalization laws separating backend-neutral device-level input observations from application actions, UI interaction semantics, and host lifecycle.
status: accepted
owner: engine
layer: architecture
canonical: true
last_reviewed: 2026-09-13
related_adrs:
  - ./0013-app-neutral-ui-composition-clean-cutover.md
  - ./0017-cross-authority-consistency-and-graph-semantics.md
  - ./0018-semantic-federation-and-physical-realization.md
  - ./0019-batteries-included-application-composition.md
  - ./0023-normalize-app-runtime-host-lifecycle-and-capability-ownership.md
related_docs:
  - ../../design/accepted/runenwerk-physical-input-semantic-model.md
  - ../../architecture/runenwerk-platform-architecture.md
  - ../../architecture/ui-framework-architecture.md
  - ../../design/active/native-tablet-input-and-latency-contract.md
---

# ADR 0024: Normalize Device-Level Input Observation Semantics

## Context

Runenwerk currently has no single normalized semantic owner for backend-neutral input-bearing
observations below application and UI meaning.

The accepted RunenInput I0 investigation (#557) found that the current engine input surface
mixes concerns with different owners:

- backend-specific `winit` event, key, button, and device types;
- keyboard, pointer, scroll, touch, and relative-motion facts;
- accumulated held state and frame-local transitions;
- application bindings and concrete `ui.*`, `world.*`, `system.*`, and `scene.*` actions;
- typed text and editing commands;
- ECS hosting;
- product/editor behavior.

The native-tablet path adds a second input producer with device/tool identity, pressure,
orientation, twist, tangential pressure, timestamps, history/coalescing, prediction,
calibration, and backend-health concerns, but currently translates those facts through UI
vocabulary before a neutral owner exists.

Current source also loses backend distinctions before consumers can choose policy. In
particular the winit runner currently drops or collapses keyboard device identity, logical
key interpretation, key location, repeat and synthetic provenance; collapses two-dimensional
line/pixel scrolling into one vertical scalar; drops scroll phase/device identity; and drops
window-event device identity for pointer/touch delivery.

ADR 0023 establishes the complementary durable rule that a Runenwerk platform lifecycle
occurrence is not reusable input semantic authority and that current mixed `InputState` is
not App-core state.

RunenUI independently proves that a maintained UI consumer needs host-neutral keyboard
facts that keep physical key identity, logical key interpretation, key location, repeat,
composition state, device identity, and cancellation distinct. Those consumer requirements
must be satisfiable without making RunenUI the generic device-input owner.

## Decision

Runenwerk shall first establish one **backend-neutral device-level input observation
boundary** inside the current repository.

The governing law is:

```text
device-level input observation
!= application action meaning
!= RunenUI interaction/text meaning
!= host/window lifecycle
```

“Device-level” is a semantic layer, not a claim of raw hardware truth. Inputs may have
already been transformed, virtualized, synthesized, accelerated, calibrated, coalesced, or
predicted by an OS/backend. The normalized model must preserve those limitations and
provenance rather than relabeling the result as raw physical fact.

The boundary answers:

> What input-bearing fact did the backend/source report, with what identity, ordering,
> coordinate/unit meaning, evidence status, delivery role, and provenance?

It does not answer:

> What should the application, editor, camera, gameplay system, or UI do because of it?

The normalized flow is:

```text
backend / platform report
    -> backend adapter
        -> normalized device-level observations
            -> deterministic admission order
                -> explicit state reduction where required
                    -> Runenwerk product/action adaptation
                    -> RunenUI interaction adaptation
                    -> drawing/stylus consumers
                    -> later Render Lab camera consumers
```

This is an internal ownership normalization. It does not create or authorize a standalone
`dornglut/runen-input` repository and does not freeze a public Rust API.

## Observation and grouping law

A normalized **observation** is evidence about input in a source stream. It is not an
application action and it is not accumulated state.

A normalized **observation group** is one coherent source update whose facts must be
applied atomically to avoid impossible intermediate state. Grouping expresses source
coherence, not a claim of perfect physical simultaneity.

Chronologically distinct transitions whose intermediate state is semantically observable
must remain distinct ordered groups. Therefore:

```text
Down -> Up
```

must never disappear merely because the final held state equals the initial held state.
Backend callback/batch shape is transport shape, not permission to collapse chronology.

## Ordering law

Two ordering concepts remain distinct:

```text
source-local order
!= reducer/admission order
!= physical measurement time
```

Every source stream has deterministic source-local order.

Every observation admitted to one neutral reducer/stream domain also has deterministic
**admission order**. Admission order is the total processing/replay order used by that
domain after normalization and multi-source merge. It does not claim that unrelated
physical events happened in that order in real time.

Source timestamps are optional evidence scoped to their clock domain. Unrelated clocks
must not be compared as if synchronized merely because their values share a numeric unit.

## Evidence and delivery law

The model must not encode orthogonal concepts in one overloaded “sample role.” At minimum
keep distinct:

```text
evidence status
    observed / confirmed
    estimated / revisable, when a backend explicitly exposes that contract
    predicted / provisional

delivery role
    ordinary/current delivery
    historical/coalesced delivery

origin / reconciliation provenance
    backend/source report
    adapter-generated reconciliation or continuity evidence
```

Coalescing is delivery/batching history, not epistemic uncertainty. A coalesced sample may
be fully confirmed. A predicted sample remains provisional even when delivered in the
same backend callback as confirmed samples.

Estimated/revisable properties, when a backend supports them, must not be silently upgraded
to confirmed measurements. The exact correction API remains an implementation/design
question for the first consumer that requires it.

A normalizer must avoid double-counting one underlying sample when an API exposes both a
current aggregate and constituent history.

## State law

Accumulated neutral state is a deterministic projection from admitted evidence:

```text
State_n = reduce(State_(n-1), admitted_group_n)
```

The authoritative reducer uses confirmed state-changing observations plus explicit
reconciliation/invalidation semantics. Predicted samples must never silently mutate
confirmed held/contact state. Estimated/revisable analog values may be exposed to consumers
without becoming confirmed authority until their source contract confirms them.

Frame boundaries are host scheduling policy, not input semantics. `pressed_this_frame`,
`released_this_frame`, frame deltas, click counts, gestures, and action state are derived
consumer projections, not root neutral state.

## Transition, assertion, and cancellation law

Keep distinct:

```text
observed transition
!= state reconciliation/assertion
!= continuity loss/cancellation
```

A backend may synthesize state reconstruction around focus or source changes. Such reports
must not masquerade as fresh physical press/release edges.

For the current winit backend specifically, `WindowEvent::KeyboardInput::is_synthetic`
proves this distinction: synthetic focus-gain presses and focus-loss releases are
reconciliation evidence, not ordinary physical transitions.

A synthetic/reconciled “currently down” assertion may restore known held state without
emitting a fresh pressed edge. A synthetic release/continuity loss may invalidate or cancel
held-state confidence without fabricating a physical `Up` transition.

The same principle applies to button, tip, contact, or other stateful controls when a
backend exposes equivalent reconciliation semantics.

## Identity law

Identity is scoped to the invariant it actually owns. Keep distinct where available:

```text
input source identity
!= device instance identity
!= tool/transducer identity
!= contact/interaction identity
!= physical control identity
!= target surface correlation
!= application action identity
```

Source/device/tool/contact identities are runtime/session scoped by default. They must not
silently become persistent hardware fingerprints, globally unique IDs, or user identity.
A reconnect receives a new instance unless continuity is actually proven.

A contact ID is valid only for one contact lifetime and may be reused after termination if
the backend contract allows it.

## Keyboard law

Keep distinct:

```text
physical key/control identity
!= logical/layout-resolved key meaning
!= committed text
!= IME composition
```

Physical key identity is the held-state/control authority.

However the normalized keyboard observation boundary must preserve backend-provided
keyboard interpretation facts required by maintained consumers when available, including:

- physical key identity;
- logical/layout-resolved key meaning;
- key location;
- source/device identity;
- transition state;
- repeat provenance;
- synthetic/reconciliation provenance.

Logical key meaning is observation metadata, not physical held-state identity. It may vary
with layout and may change across repeat/reconciliation observations while the same physical
control remains held.

Committed text and IME remain separately owned; logical key is not committed text.

Unknown physical controls must preserve a source-local/native fallback capable of correlating
press/release when the backend provides one. They must not collapse into one universal
`Unknown`.

Repeat is metadata on a continued key-down lifetime, not another physical pressed edge.

## Pointer and motion law

Absolute cursor/pointer position and relative device/source motion are independent
observations:

```text
target-relative absolute position
!= relative source/device motion
```

The current winit cursor path is target-relative and OS-processed; its physical-pixel unit
does not make it raw hardware motion. The current raw `DeviceEvent::MouseMotion` path is
relative/unfiltered in unspecified device units and is therefore a different quantity.

A normalizer must not reconstruct one from the other when the backend exposes both.

Absolute spatial values require explicit coordinate-space/basis semantics. Relative motion
requires an explicit unit domain even when that unit is only “backend/device units.”

## Button law

Button/control observations preserve control identity, source/device scope, transition or
reconciliation semantics, and ordering.

Application/UI meaning such as `OrbitCamera`, `OpenContextMenu`, `EraseStroke`, or
`PrimaryAction` is downstream policy. A known physical left/right button must not be
silently renamed UI `Primary`/`Secondary` unless the source contract actually establishes
that semantic mapping.

## Scroll law

Scroll is a two-dimensional delta in an explicit measurement domain, not one universal
scalar.

Preserve, when the source provides them:

- horizontal and vertical components;
- line/row versus pixel/surface displacement;
- discrete steps or high-resolution fractions;
- angular wheel motion or another explicit continuous source domain;
- source mechanism where meaningful;
- gesture/scroll phase when the backend exposes a real lifecycle;
- device/source identity;
- platform-adjusted versus raw-source provenance when known.

No generic conversion between line, pixel, step, angular, or uncalibrated domains is
implied.

## Touch/contact law

Touch/contact identity is transient and lifecycle-bearing:

```text
begin -> update* -> end
                 -> cancel
```

`cancel` is terminal but is not an ordinary physical release/end. Multi-contact observations
retain independent contact identities and preserve source sample grouping when the backend
establishes it.

Position, pressure/force, geometry, and related facts are optional measured quantities with
explicit coordinate/unit/range semantics. Unavailable data must not be fabricated as zero.

## Stylus/tool law

Stylus semantics keep distinct where the backend proves them:

```text
device instance
!= tool/transducer instance
!= proximity
!= tip/surface contact
!= auxiliary buttons

position
pressure
orientation / tilt
twist
tangential pressure or auxiliary analog control
contact geometry
sample time
```

Proximity is not contact. Twist is not tilt/orientation. Unsupported analog fields are not
measured zero.

The semantic quantity matters more than a backend parameterization. Orientation conversion
requires a defined surface frame, axis direction, units, reference, and loss policy.
Calibration acquisition/configuration remains backend-owned; a normalized value records the
truthful resulting domain and relevant normalization provenance, not a per-event calibration
object by default.

## Availability law

The model distinguishes:

```text
measured zero
!= value omitted from this sample
!= known unsupported/unavailable
!= unknown capability/state
!= not applicable
```

Capability describes what a source/device/tool can establish; it does not imply every
sample contains every supported property.

No silent clamping/defaulting may turn absent or invalid data into a valid measurement.
Non-finite or otherwise invalid numeric data must be rejected or diagnosed according to the
quantity contract.

## Text and IME law

Committed text, preedit/composition, selection/caret semantics, and text editing remain
outside the device-state reducer.

The platform/backend may correlate keyboard observations and text/IME reports from one user
gesture, but the model must not derive Unicode text from physical key identity or use text as
held-state identity.

RunenUI retains reusable text/composition semantics.

## Consumer ownership law

Runenwerk applications retain action/binding policy, including camera controls, editor
commands, gameplay meaning, and `ui.*` / `world.*` / `system.*` / `scene.*` vocabularies.

RunenUI retains focus, routing, capture, semantic commands, shortcuts as UI behavior, text,
composition, selection, accessibility interaction, and widget semantics.

Runenwerk host/platform integration retains App lifecycle, windows, event-loop realization,
focus/window lifecycle, redraw, frame pacing, native hooks, and target lifecycle.

RunenECS may host neutral or derived state without becoming input semantic owner.

One explicit Runenwerk integration adapter may combine normalized device-level observations
with separately owned text/IME/context information to construct RunenUI ingress values. That
does not require RunenUI itself to depend directly on a future RunenInput package and does
not justify sharing types merely to avoid translation.

## Normalization and lossiness law

Backend-neutral does not mean erasing backend information until all inputs look alike.
Adapters must preserve maintained-consumer facts or explicitly declare loss.

They must not silently:

- collapse physical and logical keyboard identity;
- drop key location, repeat, device/source identity, or reconciliation provenance when a
  maintained consumer requires them;
- derive raw motion from cursor position when distinct raw motion exists;
- erase one scroll axis or scroll unit/source/phase distinctions;
- turn unsupported or omitted analog values into measured zero;
- merge distinct touch contacts;
- merge stylus proximity and tip contact;
- discard source time/history needed by drawing;
- classify coalescing as uncertainty;
- treat prediction or estimation as confirmed evidence;
- collapse chronologically distinct transitions;
- convert host/window lifecycle events into fake physical releases.

Backend-specific transport/API types terminate at the adapter boundary.

## Extraction law

This ADR does not accept RunenInput as a standalone framework.

The sequence remains:

```text
internal semantic/state repair
    -> backend-edge normalization
    -> product-action separation
    -> one explicit RunenUI adaptation path
    -> native-tablet convergence
    -> two structurally different real consumers
    -> fresh extraction decision
```

Only a later accepted decision may establish `dornglut/runen-input` after the boundary is
proven independently useful and cleanly separable under repository-family extraction rules.

## Rejected alternatives

### Extract current `InputState`

Rejected because it mixes backend, ECS, product action, text/editing, and device facts.

### Treat “physical input” as raw hardware truth

Rejected. Current backends include virtual/window streams and OS-transformed values. The
neutral contract describes truthful device-level/source observations and their provenance.

### Make RunenUI the generic input owner

Rejected. Camera/game/editor/drawing consumers exist independently of UI semantics.

### Exclude logical-key interpretation from all neutral keyboard observations

Rejected for the founding model. Current standalone RunenUI already requires a host-neutral
logical-key fact correlated with physical-key transitions. Preserving that interpretation
does not make it physical held-state authority and does not move text/IME into the reducer.

### Combine current/coalesced/predicted into one sample-role enum

Rejected because delivery/history and epistemic certainty are orthogonal. Coalescing does
not make confirmed evidence provisional.

### Use synthetic focus releases as physical `Up`

Rejected because reconciliation/continuity evidence is not proof of a physical transition.

### Use one giant backend-shaped event enum

Rejected because it preserves backend taxonomy and encourages backend-shaped consumers.

### Normalize every analog/scroll value into one universal range

Rejected because quantities have different units, domains, periodicity, and calibration.

### Generic event bus / service locator / plugin framework

Rejected. Input normalization does not justify a new meta-framework.

## Consequences

Positive consequences:

- I1 repair can target explicit semantics rather than rename current code;
- winit and native-tablet sources can converge without moving host or UI policy into input;
- current RunenUI keyboard requirements can be satisfied without duplicate direct-winit
  translation;
- drawing can preserve confirmed history, estimates/prediction, and high-rate samples without
  imposing drawing policy on other consumers;
- camera consumers can use raw relative motion without cursor reconstruction;
- synthetic/reconciliation behavior no longer creates false physical edges;
- future extraction has a testable host-neutral candidate boundary.

Accepted costs:

- current `InputState` cannot survive as the normalized root type;
- current `PlatformEvent` is intentionally too lossy to be the long-term input authority;
- adapters must represent availability, provenance, and reconciliation explicitly;
- exact Rust representation remains deferred to bounded I1 work.

## Fitness functions

This decision remains healthy only while:

1. backend API types do not appear in the neutral semantic model;
2. device-level observations remain usable without RunenUI, product actions, or ECS hosting;
3. the model never claims raw hardware truth it cannot prove;
4. every reducer domain has deterministic admission order distinct from physical time;
5. source-local order and source timestamps remain separately meaningful;
6. physical key, logical key, committed text, and IME remain distinct;
7. key location/repeat/device/reconciliation facts can be preserved without becoming held-state
   identity;
8. synthetic/reconciliation input does not create false physical edges;
9. absolute position and relative motion remain independent;
10. scroll axes and measurement domains are not silently collapsed;
11. contact cancellation remains distinguishable from ordinary end;
12. stylus proximity/contact/pressure/orientation/twist remain separately representable;
13. measured zero remains distinct from omitted/unavailable/unknown;
14. coalescing/history remains orthogonal to observed/estimated/predicted certainty;
15. prediction and estimation do not mutate confirmed authoritative state;
16. continuity loss does not fabricate physical releases;
17. RunenUI and Runenwerk host/application owners retain their accepted semantics;
18. no standalone RunenInput authority exists before a fresh extraction decision.

## Activation

This ADR authorizes no Rust implementation by itself.

After this ADR and the companion accepted semantic-model design are accepted on `main`, a
fresh exact-current census may derive exactly one bounded I1A implementation issue. That
issue must remain internal to Runenwerk and must not combine internal repair with standalone
extraction.
