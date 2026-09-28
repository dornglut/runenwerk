---
title: Runenwerk Native Normalized Input Recording And Close-Time Persistence Design
description: Accepted design for bounded fresh-launch native product recording, in-App trace finalization, and Render Lab A8 persistence.
status: accepted
owner: engine
layer: engine-runtime / integration
canonical: true
last_reviewed: 2026-09-28
publication: reference
pagefind: false
related_designs:
  - ./runenwerk-application-automation-session-semantic-model.md
  - ./runenwerk-terminal-automation-orchestration-and-authored-scenario-design.md
  - ./runenwerk-persisted-authored-automation-scenario-handoff-design.md
---

# Runenwerk Native Normalized Input Recording And Close-Time Persistence Design

## Status and scope

This design is the A16 decision after accepted A15.

Reviewed accepted state:

```text
RUNENWERK_MAIN=715a9bc169fc5d69c50a5ed1e4b3f86917b86258
A15_MERGE=715a9bc169fc5d69c50a5ed1e4b3f86917b86258
A15_POST_MERGE_CI=36412135507
RUNENINPUT_PIN=2751e19fa42255b86e786e7cd837198c917b7b25
```

A15 completes persisted authored scenario handoff. The remaining first-stage A0 user workflow gap is
production recording:

```text
fresh real product launch
  -> actual native input acquisition
    -> normalized admission
      -> exact admitted-group capture
        -> bounded persisted normalized trace V2
          -> later normalized replay / A15 scenario reference
```

This design fixes that lifecycle. It does not add native input injection, global recording,
mid-session attach, trace-to-scenario generation, or a central automation terminal.

## Current mechanism disposition

| Mechanism | Disposition | Reason |
| --- | --- | --- |
| A3/A4 admitted-group capture | retain | canonical normalized recording seam |
| A8 trace V1 | retain unchanged and supported | existing replay artifacts remain valid |
| A8 trace V2 | add as prerequisite | represent native absolute-pointer observations without weakening V1 |
| A9/A12 persisted replay | retain unchanged | normalized execution authority |
| A15 scenario trace reference | retain unchanged | authored handoff can consume the recorded artifact |
| `AppAutomationInputTraceExt` | retain | synchronous App/test control remains useful |
| private `AutomationInputTraceRecorder` | retain private | products must not couple to recorder internals |
| Render Lab native App | first product pressure | real Winit ingress + existing close approval |
| Winit runner | retain unchanged | native Host must not become automation persistence owner |
| Render Lab close approval | adapt | product-owned place to delay close until evidence finalizes |
| trace-to-scenario | defer | real production recording comes first |

## Exact fidelity claim

The first production recorder records the normalized observations that were successfully admitted
after real native acquisition:

```text
Winit/native acquisition
  -> Runenwerk platform adapter
    -> InputObservationGroup validation/admission
      -> A3 exact admitted-group capture
        -> A4 App-frame partition
          -> A8 persisted normalized trace
```

The persisted artifact is **not**:

- a Winit event transcript;
- a raw OS event log;
- text/IME capture;
- proof that later replay re-enters Winit;
- proof of identical OS scheduling or GPU presentation.

Replay remains `NormalizedInput` fidelity.

`capture_host = NativeWindow` is provenance for where observations were acquired, not a replay
mode promise.

## Native cursor compatibility prerequisite

A normal native Render Lab mouse stream is not representable by A8 V1.

Current Winit integration admits:

```text
WindowEvent::CursorMoved
  -> InputObservation::AbsolutePointerPosition

DeviceEvent::MouseMotion
  -> InputObservation::RelativeMotion
```

Render Lab prefers raw relative motion when available but deliberately retains absolute cursor
motion as its fallback projection.

A6/A8 V1 rejects absolute-pointer observations. Therefore an exact admitted-input trace from an
ordinary native drag commonly contains an observation family that V1 cannot persist/replay.

first production recording MUST NOT:

- silently drop absolute-pointer groups;
- rewrite them into relative motion;
- claim raw relative motion makes the absolute evidence irrelevant;
- alter Render Lab native input policy merely to fit V1.

Those approaches would weaken exact admitted-input evidence and can change the product fallback
path.

The prerequisite is an explicit persisted/replay compatibility extension.

### Persisted normalized trace V2

Keep the existing artifact kind:

```text
runenwerk.automation.normalized-replay-trace
```

Add schema version 2.

V2 contains all V1 replayable semantics plus:

```text
AbsolutePointerPosition {
    position: Point2 {
        x,
        y,
        coordinate_space,
    }
}
```

The coordinate-space domain remains the already accepted normalized point domain used by current
Runenwerk/tablet persistence:

- `UnspecifiedTargetUnits`;
- `WindowPhysicalPixels`.

V2 stores the exact finite normalized point. It does not store Winit cursor-event objects.

V2 retains the explicit
`AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart` requirement and the
same provenance/witness distinction.

V1 remains a strict immutable schema. Do not add the new observation variant to V1 under
`schema_version = 1`.

### Version-aware import

Add one version-aware normalized-trace import path that performs the existing bounded envelope probe
and dispatches explicitly:

```text
schema_version = 1 -> strict V1 decode
schema_version = 2 -> strict V2 decode
other              -> UnsupportedSchemaVersion
```

A12 persisted replay, A9 direct Render Lab replay, and A15 scenario replay should consume this
version-aware path.

Existing explicit V1 import/export APIs may remain for conformance/tests and must retain their exact
behavior.

No implicit V1 -> V2 mutation is required. Both versions materialize the same in-memory
`AutomationInputTrace` type after their own strict decode.

### Absolute-pointer replay law

A6 normalized replay may accept a single-observation
`InputObservation::AbsolutePointerPosition` group only under the existing explicit
`RecordedAndReplaySourcesPristine` state assumption.

The accepted fresh-state relation is:

```text
recording side:
  fresh source has no retained absolute position
  -> first recorded absolute point establishes the source baseline

replay side:
  fresh mapped replay source has no retained absolute position
  -> same first absolute point establishes the same baseline
```

Current Runenwerk cursor projection computes motion from the source's retained previous absolute
position and uses the canonical no-prior-position baseline when none exists. With both sides fresh,
the first absolute observation is therefore self-contained.

Replay preflight MUST continue to reject a mapped replay source that already has retained absolute
pointer state.

Replay ingress MUST route the recorded absolute point through the existing Runenwerk
cursor-position admission/projection. It MUST NOT synthesize relative motion from persisted points
in the automation layer.

V2 does not add keyboard, ordinary contact, continuity-loss, text/IME, or arbitrary mixed
non-tablet replay. Those remain unsupported.

An arbitrary mid-session capture is still not made self-contained merely because V2 can encode an
absolute point.

## First product boundary

first production recording is Render Lab only.

Target CLI:

```text
runenwerk-render-lab --record-trace <trace.ron>
```

The mode launches the normal fresh native Render Lab interaction product.

Do not add Editor/Draw recording merely for symmetry. Reassess additional recorders only after this
path demonstrates stable product pressure.

## Fresh-launch-only recording

The first supported recorder begins capture on a **fresh launch-owned App before native product input
can be admitted**.

Required sequence:

```text
construct fresh Render Lab App
  -> install normal input/native/render integration
    -> install AutomationInputTracePlugin
      -> explicitly start automation trace
        -> enter app.run() / Winit event loop
```

Interactive mid-session start is not exposed by first production recording.

This restriction is what lets the product caller explicitly supply:

```text
AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart
```

without weakening A8.

The exporter still does not infer or default the witness.

If captured input later proves incompatible with the witness, such as an invalid release-first
shape, normal A8 validation rejects export. The recorder must not rewrite or invent missing
pre-capture state.

## Why App start remains separate from persistence witness

A3/A4 capture may legitimately start mid-session for non-persisted/test purposes.

Therefore V1 does **not** change `start_automation_input_trace()` into a globally
pristine-only operation.

The production Render Lab recording mode owns the stronger fresh-launch precondition and passes the
typed A8 witness explicitly.

## In-App recording control

Native `App::run()` consumes the App into the Winit runner. The terminal caller cannot recover the
App afterward and call synchronous `stop_automation_input_trace()`.

Do not change Winit/App lifetime merely for recording.

Instead, extend Engine automation with a narrow **in-App trace control resource** owned by
`AutomationInputTracePlugin`.

Conceptual state:

```text
Idle
Recording
StopRequested
Completed(AutomationInputTrace)
Failed(AutomationInputTraceRuntimeFailure)
```

The public product-facing capability is limited to:

- request finalization;
- inspect recording status/counters;
- take one completed trace or terminal failure.

It does not expose the private recorder.

It owns no:
- filesystem path;
- product close policy;
- provenance;
- A8 witness;
- scenario semantics.

Existing synchronous App start/stop APIs remain and should share one internal recorder lifecycle
implementation rather than diverge.

## FrameEnd finalization law

The Engine plugin owns ordering between frame capture and asynchronous stop.

Within one canonical frame:

```text
input admission
  -> normal FrameEnd
    -> admitted groups drained into trace frame
      -> trace-control finalizer observes StopRequested
        -> admitted capture stops
          -> completed trace published to control resource
```

The finalizer MUST run after the existing trace frame-capture system.

Products do not order against the private recorder function.

A product request made too late for the current FrameEnd may complete on the next frame. An
additional idle frame is acceptable and remains explicit trace evidence.

Close is not approved until the product observes completion or failure.

This ordering should make close-time completion produce no unframed trailing groups. A8 export
remains the final authority and still rejects any trailing groups if the invariant is violated.

## Close-time progress guarantee

Asynchronous finalization is allowed to complete on the frame after the user close intent, but the
product MUST guarantee progress while close remains pending.

Current Render Lab native interaction uses continuous capped frame pacing at 60 Hz. That policy
already guarantees another App frame and therefore satisfies the first implementation without new
Host behavior.

This dependency must remain explicit:

- while trace finalization or recording output is pending, at least one further App frame must be
  scheduled;
- if Render Lab later changes to event-driven or on-demand pacing, recording close policy must
  explicitly request redraw/wake rather than wait indefinitely;
- Engine trace control does not own native event-loop wake policy.

A future pacing change that removes this progress guarantee is a recording-mode compatibility
change and must update the product close integration.

## Final Host teardown is outside the recording

Trace finalization occurs before product close approval and before native-window destruction/source
retirement.

Therefore continuity-loss or source-retirement observations caused solely by final Host teardown
after close approval are **not** part of the persisted trace.

This is intentional:

- A8 records admitted interaction up to the explicit recording stop point;
- it is not a complete native-window lifetime transcript;
- if the final recorded state leaves a control held, replay may retain that state until normal
  A6/A12 replay teardown releases replay-owned input;
- recording must not fabricate synthetic release/continuity observations merely to make the trace
  look closed.

Focus loss, device removal, or other continuity loss that occurs during the actual recorded
interaction **before** finalization remains ordinary admitted input evidence and is captured when
supported by A3/A4.

`capture_host = NativeWindow` therefore means native acquisition was the producing Host class. It
does not mean that Host teardown is encoded or that replay exercises native-window lifetime events.

## Runtime recording bounds

Production recording must be bounded **before** export.

These ceilings are an explicit **production recording policy**, not new global A3/A4 capture
semantics.

The existing synchronous `AppAutomationInputTraceExt::start_automation_input_trace()` remains
available with its current in-memory/test semantics. The first implementation should introduce an
explicit bounded policy/configuration used by the in-App production recording path. Render Lab
`--record-trace` selects the persisted normalized-trace production policy.

The private recorder may share lifecycle/counter machinery between synchronous and bounded capture,
but it MUST NOT silently apply production persistence ceilings to unrelated existing in-memory
capture callers.

A8's persisted bounds remain unchanged. The first production runtime recording policy is no larger than:

```text
retained frames                <= 16,384
total groups                   <= 65,536
groups per frame               <= 4,096
observations per group         <= 256
total retained observations    <= 262,144
```

The first four match or narrow existing A8 structural limits.

`262,144` is a recording-runtime memory policy, not a new A8 schema limit. It keeps the first
interactive recorder bounded even though A8 has no independent total-observation cardinality.

The 8 MiB persisted normalized-trace encoded-byte limit remains authoritative at export for both
supported schema versions.

Source/device/contact/tool cardinality remains validated by A8. Runtime group/observation bounds
already prevent unbounded identity accumulation before that validation.

## Admission-time bound enforcement

The current admitted-input capture buffer is populated inside `InputState::admit_group`: every
successfully admitted group is cloned into `captured_admitted_groups` before
`AutomationInputTracePlugin` drains that buffer at `FrameEnd`.

Therefore checking production limits only when the automation recorder closes a frame would be too
late. One frame could already have accumulated beyond the promised memory policy.

The explicit production recording policy MUST extend the admitted-capture seam with bounded mode.

While bounded capture is active, the input-integration capture owner tracks at least:

- groups admitted in the current frame;
- observations in the incoming group;
- total groups captured in the active recording;
- total observations captured in the active recording;
- terminal capture-limit status.

Before cloning the next admitted group into the capture buffer, bounded capture rejects recording
of that group and enters terminal limit failure if the addition would exceed:

- groups per frame;
- observations per group;
- total groups;
- total retained observations.

This bound affects only the optional capture copy. Normal input admission/reduction for the running
product remains owned by the existing input path and MUST NOT be rejected merely because automation
recording exhausted its capture budget.

On first capture-limit failure:

```text
product input admission remains normal
  -> automation capture stops cloning
    -> terminal capture-limit status becomes observable
      -> FrameEnd automation control observes failure
        -> admitted-capture ownership is released/reset
          -> incomplete trace prefix is discarded
```

Retained-frame count remains automation trace/FrameEnd authority because frame partitioning is owned
there. If the next frame would exceed the retained-frame ceiling, the trace layer follows the same
terminal-failure/release path.

Existing unbounded/test `start_admitted_input_capture()` behavior remains unchanged. The bounded
mode is an explicit Runenwerk input-integration capture policy selected only by the production
automation recorder.

This is not a RunenInput reducer semantic change.

## Limit exhaustion

Hard recording-limit exhaustion is **not successful completion**.

When adding the next captured frame/group/observation would exceed a production recording limit:

1. transition the recording control to a typed terminal limit failure;
2. immediately stop/release the admitted-input capture owned by automation;
3. stop accepting additional automation recording data;
4. discard the incomplete retained prefix as non-exportable result state;
5. do not publish that prefix as a successful completed trace;
6. Render Lab exits recording mode nonzero;
7. no A8 output artifact is written.

Other terminal recorder-control failures that make continued capture invalid follow the same
capture-lease release rule. A failed recorder MUST NOT keep
`InputState::admitted_input_capture_active()` owned until native process teardown.

Do not silently truncate.

A later product UX may add an explicit user-requested duration/frame limit whose completion is
successful. That is not required for the first implementation and must not be confused with the
safety ceiling.

## Render Lab close-time state machine

Render Lab already delays native close while product evidence is finalized.

Recording mode extends that product-owned close policy:

```text
user native close intent
  -> if recording active:
       request trace finalization
       keep close pending
  -> wait until trace control yields Completed or Failed
  -> Completed:
       export A8 with explicit fresh-launch witness
       write requested output
       approve close
  -> Failed:
       return product/runtime error
       native runner exits nonzero
```

Render Lab may need one additional frame between close request and final approval.

The Engine control layer does not request or approve windows.

## Recording before user close

If the Engine runtime recorder enters terminal failure before user close, Render Lab recording mode
must fail the run rather than continue accumulating or later report success.

The product may request native close as part of surfacing that terminal failure, but the process
result remains nonzero and no trace is persisted.

## Output file policy

Filesystem identity remains product/terminal policy.

First production recording rules:

- the user supplies exactly one output path;
- parent directory must already exist;
- the final output path must not already exist;
- no implicit overwrite;
- encode the complete A8 artifact in memory before opening the final output;
- create the final path with create-new semantics;
- write the complete encoded bytes and flush/sync;
- on write/sync failure, remove the incomplete file best-effort and return nonzero.

V1 does not claim crash-atomic persistence if the process or machine dies during the write.

A partial file left by abrupt process death is not successful evidence and should fail A8 import.

No filesystem path enters the generic trace schema.

## Provenance

The first native recording sets only provenance it can prove.

Required:

```text
capture_host = NativeWindow
```

`runenwerk_revision` and `runen_input_revision` remain absent unless the running build already
has a trustworthy immutable build-provenance source.

Do not synthesize revision strings from source assumptions.

A product-local bounded label may identify Render Lab if useful, but it is evidence only.

## Terminal success summary

Successful recording output may report:

- output path;
- frame count;
- total group count;
- total observation count;
- artifact kind/schema version.

No persisted generic recording-result artifact is introduced.

## Security and privacy boundary

The recorder is application-scoped.

It captures only normalized observations admitted by the selected Runenwerk App while that App is
running in explicit recording mode.

It is not:
- an OS-wide recorder;
- a background service;
- a global keyboard logger.

A3/A4 still omit committed text/IME payloads.

No native injection capability is added.

## Automated evidence versus physical native evidence

Hosted CI can prove:

- trace-control request/result lifecycle;
- FrameEnd capture-before-finalize ordering;
- bounded-memory failure behavior;
- completed traces have no unexpected trailing groups;
- A8 export/import from the completed trace;
- Render Lab recording close-state logic;
- output-file success/failure logic;
- existing A9/A12 replay consumes the artifact;
- native Render Lab still passes current native-window smoke lanes.

Hosted CI must **not** claim it observed a human physical mouse produce Winit
`DeviceEvent::MouseMotion` unless a proven hardware/native injection source exists.

The statement:

> a human orbit/pan/zoom interaction was actually acquired from physical native mouse input

requires manual/hardware execution evidence.

That evidence is supplementary to source acceptance. Lack of such hardware evidence must be stated;
it must not be replaced by normalized injection.

## Representative user workflow

Recording:

```text
runenwerk-render-lab --record-trace orbit-pan-zoom.ron
```

User performs normal native Render Lab orbit/pan/zoom, then closes the window.

Replay:

```text
runenwerk-render-lab --replay-trace orbit-pan-zoom.ron
```

Authored handoff:

```text
runenwerk-render-lab --automation-scenario scenario.ron
```

where the A15 scenario may reference `orbit-pan-zoom.ron` and add a product-owned camera
assertion.

Recording, trace, and authored scenario remain distinct artifacts/operations.

## Trace-to-scenario ordering

Do not implement trace-to-scenario transformation before this real recording producer exists.

After the native recorder is accepted, reassess whether the next concrete user friction is:

- generating an editable product-owned scenario draft from a recorded trace;
- adding another product recorder;
- consolidating repeated terminal recording plumbing;
- attach/native automation.

Do not pre-authorize any of them.

## First implementation successor

A16 selects one prerequisite implementation successor before native recording is implemented.

### A17 — absolute-pointer normalized replay and persisted trace V2

A17 should:

1. keep A8 V1 import/export and conformance behavior unchanged;
2. add strict persisted normalized replay-trace schema V2 import/export under the existing artifact kind;
3. add V2 absolute-pointer position DTO/conversion using finite `Point2` and the accepted
   coordinate-space domain;
4. retain the explicit recording-side pristine witness, provenance rules, strict unknown-field/variant behavior, and existing metadata/resource limits;
5. add a bounded version-aware importer that accepts V1 and V2 and rejects unknown versions;
6. switch A12 persisted replay to that version-aware importer without changing session lifecycle;
7. extend A6 preflight to accept absolute-pointer groups only under the existing fresh-state
   assumption;
8. preserve/reuse the existing retained-absolute-pointer target-state conflict check;
9. extend Runenwerk automation ingress to route absolute positions through the existing cursor
   projection;
10. prove fresh-source absolute position replay reproduces cursor projection and Render Lab camera
    behavior;
11. prove dirty replay-source absolute state remains rejected before target mutation;
12. prove V1 artifacts still import/replay unchanged and V1 export still rejects absolute-pointer
    observations;
13. add no keyboard/text/contact/continuity replay, native injection, capture filtering, A8 V1
    reinterpretation, or native recording CLI.

A17 intentionally does **not** touch Render Lab native lifecycle or file output. At A16 publication,
PR #1038 writes `apps/runenwerk_render_lab/src/native.rs`; A17 should remain path-disjoint.

After A17 is accepted and accepted-main validation is green, re-resolve the repository and this
design's native-recording assumptions. If still valid, activate one bounded native recording
implementation successor (expected A18) for the in-App control/bounds/Render Lab close/file work
defined below. Do not pre-merge those concerns into A17.

## Expected A17 implementation write boundary

Likely:

```text
engine/src/automation/persistence.rs
engine/src/automation/persistence/tests.rs
engine/src/automation/replay.rs
engine/src/automation/replay/tests.rs
engine/src/automation.rs
engine/src/plugins/input/state.rs
engine input/automation focused tests
apps/runenwerk_render_lab/src/automation.rs or focused headless proof only if needed
```

No Render Lab `native.rs` or production recording CLI change is expected in A17.

No Cargo/lock change is expected.

## Expected native-recorder implementation boundary after A17

Likely:

```text
engine/src/automation.rs
engine automation focused tests
engine/src/plugins/input/state.rs
engine input-capture focused tests
apps/runenwerk_render_lab/src/native.rs
apps/runenwerk_render_lab/src/main.rs
Render Lab focused recording tests
```

A separate narrow Engine automation module is acceptable if it reduces `automation.rs` growth
without changing ownership.

No Cargo/lock change is expected.

## Deletion/replacement rule

Do not create a product-local second recorder.

Any temporary Render Lab recording glue may own only:

- CLI parsing;
- product recording configuration;
- close policy;
- A8 provenance;
- file output.

Recorder state transitions, frame capture, limits, and completion remain Engine automation
authority.

If A17 needs product code to duplicate recorder lifecycle internals, stop and repair the Engine seam
instead.

## ADR disposition

No new ADR is required.

This design extends Runenwerk's already accepted automation/integration authority and product-native
close policy. It does not transfer semantic ownership to RunenInput, Winit, another repository, or a
new service.

A future global recorder, attach protocol, or native injection authority would require a separate
decision.

## Non-goals

A16 does not authorize:

- OS-wide/global capture;
- arbitrary mid-session recording;
- native OS input injection;
- IPC/attach/remote control;
- central `runenctl`;
- trace-to-scenario generation;
- automatic semantic assertion inference;
- text/IME recording;
- screenshots/video;
- generic persisted recording-result schema;
- multi-product recording sessions;
- mutation/reinterpretation of A8 V1;
- persisted trace schema changes beyond the explicitly selected V2 absolute-pointer extension;
- RunenInput ownership changes;
- Scene/RunenUI replay absorption;
- new service/repository.

## Acceptance gate

The design is acceptable only if cold review confirms:

- fresh-launch witness ownership is truthful and explicit;
- App-only start/stop semantics remain available;
- Engine finalization cannot race frame capture;
- production recording memory is bounded before export through an explicit opt-in production policy rather than a silent global A3/A4 restriction;
- group/observation budgets are enforced at the admitted-capture seam before capture cloning, not only after FrameEnd drain;
- capture-limit failure does not reject otherwise valid product input admission;
- hard-limit/terminal recorder failure releases automation-owned admitted capture and cannot produce a successful truncated artifact;
- product close remains pending until recording completion/failure;
- close-time finalization has an explicit frame-progress guarantee and does not depend silently on current pacing;
- final Host teardown/source retirement is explicitly outside the trace boundary;
- file output does not silently overwrite evidence;
- NativeWindow provenance is not confused with native replay fidelity;
- automated and manual/hardware evidence claims remain separated;
- A8 V1 remains immutable while the required absolute-pointer extension is versioned as V2;
- V2 absolute replay is justified only by the explicit fresh recorded/replay source assumption;
- version-aware persisted replay keeps A9/A12/A15 able to consume V1 while adding V2;
- exactly one first successor is selected: A17 absolute-pointer replay/persistence compatibility;
- native recording implementation remains sequenced after accepted A17 rather than bundled into it;
- exact-head canonical CI and Documentation Build pass;
- the diff remains design-only.

After accepted-main verification, proceed automatically to A17 under standing owner authorization.
