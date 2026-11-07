---
title: Windowless App Lifetime and Dynamic Windowing
description: Proposed decision for one explicit-intent, windowless-capable Runenwerk App with bounded owner-controlled progression, typed window requests, and safe shutdown.
status: draft
owner: engine
layer: architecture
canonical: true
last_reviewed: 2026-11-07
publication: reference
pagefind: false
related_adrs:
  - ../accepted/0013-app-neutral-ui-composition-clean-cutover.md
  - ../accepted/0019-batteries-included-application-composition.md
  - ../accepted/0023-normalize-app-runtime-host-lifecycle-and-capability-ownership.md
related_designs:
  - ../../design/accepted/runenwerk-app-runtime-and-composition-semantic-model.md
  - ../../design/accepted/runenwerk-application-automation-session-semantic-model.md
  - ../../design/accepted/runenwerk-terminal-automation-orchestration-and-authored-scenario-design.md
---

# Proposed ADR: Windowless App Lifetime and Dynamic Windowing

## Status and authority

**Draft / proposed. Not accepted and not current behavior.**

Owning investigation: [Runenwerk #1423](https://github.com/dornglut/runenwerk/issues/1423).
Typed UI-facing integration: [Runenwerk #1405](https://github.com/dornglut/runenwerk/issues/1405).

This proposal records the intended *behavioral* contract and approval gates, not
compiled Rust signatures, an authorized production implementation or a claim
that platform behavior has been tested. ADR 0019, ADR 0023 and their accepted
semantic model remain authoritative until an independently reviewed successor
decision is accepted. No new generic AppRunner, universal Host trait or mutable
command registry is accepted by this draft.

Evidence: original protected Runenwerk
`main@505581b30537901de9964badbf99cd083237f359` (2026-10-30);
source findings rechecked against protected
`main@b4683b99939464c3bc79e85f04c752687f01bc86` (2026-11-07),
including accepted native UI/input and non-frame GPU progress. This ADR
remains a source-backed proposal, not an implementation or native OS proof.
The separate bounded Rust test-local typed composition candidate in
[#1426 / draft PR #1441](https://github.com/dornglut/runenwerk/pull/1441)
passed exact-head CI, but it does not compile-prove these App lifetime,
window-command or error-recovery contracts.

## Context and verified gaps

A single `App` must serve an ordinary desktop Todo, Editor/Draw with
multiple native windows, a custom-render game, a headless server, an offline
finite job, typed headless UI, a windowless offscreen-GPU worker, and a
console-first engine which opens its first window after startup.

Current accepted source does not satisfy this entry/lifetime contract:

- `engine/src/app/domain/app.rs`: `App::new()` immediately selects
  `AppHostSelection::NativeWindow`; `App::headless()` selects Headless.
- `engine/src/app/runtime/lifecycle.rs` and
  `engine/src/app/platform/headless.rs`: headless `run()` advances one
  frame then returns; bounded advancement requires headless selection.
- `engine/src/runtime/winit_runner.rs::resumed`: physical primary-window
  realization occurs *before* Startup. There is no accepted zero-initial-window
  native path; suppressing window creation alone would suppress Startup.
- The same native runner exits its Winit event loop when the **primary**
  window's close is approved. Other windows and background work may still exist.
- `engine/src/runtime/window.rs`: requested secondary windows already have
  `Requested`, `Created` and `CreationFailed` states and native identities.
  Editor currently reserves corresponding Render surfaces and coordinates
  failed creation; the generic desired public window API is absent.
- `engine/src/runtime/native_host_wake.rs` carries a specific coalesced
  RunenUI wake, not a general owner-typed command inbox.
- `apps/runenwerk_runtime_preview/src/main.rs` owns an external Tokio
  command loop; its headless path runs that loop before a later single Engine
  frame. The Engine does not currently integrate a sustained headless driver.
- `engine/src/plugins/ui/app_ext.rs` already exposes
  `AppUiExt::ui(&mut self) -> UiAppMounting`; a fluent `.ui::<A>()`
  spelling is not a drop-in compatible overload.
- `engine/src/main.rs` currently consists of `App::new().run()`, which
  starts a native window today; interpreting that spelling as a passive,
  windowless or invalid App is an observable supported-consumer break.
- Current `winit_runner` has bounded Render-owned native non-frame GPU
  progress and accepted U9-C input ownership from closed #1392/#1276.
  Those specialized paths do not provide a general headless service driver,
  a NativeWindow request handle or a universal non-frame command inbox.

A new `App::new()` default would deliberately supersede the earlier
behavior-preserving #884 and #875 decisions, rather than reinterpret those
decisions as having accepted the new behavior.

## Proposed decision

### 1. One App, initially unselected, with a stable admitted Host

`App::new()` constructs **one** Runenwerk application composition root and
initially selects **no physical windowing environment**. It does not initially
create a window, OS event loop, UI mount, GPU, renderer or network listener.
During Configuring, declarative capabilities are collected and validated. At
admission/startup, the execution environment is resolved **once**:

- Without native windowing selection, a windowless/headless environment is used.
- Declaring an initial `window(...)` selects the concrete native-window path.
- Explicit `enable_windowing()` selects native support with **zero** initial
  windows, allowing an OS window to be requested after startup.
- Once prepared, a Headless-running App may not silently turn into a Native
  App. Window count is *not* Host identity.

Host selection remains a concrete internal Runenwerk/OS integration concern.
Ordinary application authors do not need `.host(Host::native())` or an
overloaded public Host hierarchy. Unsupported platform combinations reject
before entering a running state wherever admission can detect them.
**No valid execution intent** also rejects admission for an open-ended run;
merely constructing an App does not count as intentional service execution.
No RunenUI slot may first receive a headless clock and then silently switch
to the Native clock at Host selection.

### 2. Open-ended App lifetime requires explicit execution intent

**Proposed new meaning:** `App::run(self)` starts an admitted App exactly
once and normally remains running until selected application shutdown or
terminal failure. It does not secretly mean `run_for_frames(1)` and does
not force Update or a visual frame during idle periods.

**Fail-first ordinary default (owner-selected design policy):** a bare
`App::new().run()?` with no declared execution intent is **an actionable
admission error**, not an application that silently waits forever and not an
implicit native window. Its error identifies the absent execution intent
and suggests a supported window, service, bounded-advancement or deliberate
keep-alive choice. Validation occurs before Startup or owner effects; the
consuming API must either preflight safely or expose a recoverable result
when admission can fail. Exact Rust error/return types remain a compile gate.

Execution intent must be *real and explicit*, not inferred from incidental
resources, builtin plugins, a `Time` resource or the existence of ECS
systems. Qualifying declared paths include:

- One or more initial `window(...)` declarations: ordinary native desktop
  execution, with a documented last-window lifetime policy.
- An explicit `enable_windowing()`: intentional native event-loop support
  with **zero** initially realized windows, able to receive typed requests
  after readiness.
- An owner-backed long-running service/job/command ingress with documented
  wake, progress, failure and shutdown integration. The exact capability
  admission contract must be proved; arbitrary `add_plugin(...)` calls do
  **not** automatically establish that a live service exists.
- Explicit `keep_alive` policy (spelling provisional): deliberately allow
  a passive App even without other work, with documented idle diagnostics and
  a real pre-run shutdown path. Opt-in is intentional; it is not silently
  selected by `App::new()`.

A bounded explicit `run_for_frames(n)` or applicable fixed-step operation
is a valid *finite advancement request*, even if no open-ended service
intent exists. Zero-frame behavior, repeatability and returning App identity
remain subject to the existing advancement-owner contract.

A real external or application-owned shutdown path must exist for a
blocking `run(self)`. Before consuming App, a caller must be able to
obtain a cheap, clonable **App-owned shutdown handle** that survives the
move and can wake either the selected headless or native execution. That
handle scopes to App lifetime, **not** a generic command bus:

- Graceful shutdown requests are nonblocking, thread-safe and valid while
  idle; distinguish `Accepted`, `AlreadyStopping` and `Closed`
  (illustrative categories). Acceptance is **not** completed cleanup.
- App transitions toward stopping once; future work admission rejects.
  Existing owner operations must complete or receive terminal cancellation,
  including window-request tickets and typed UI/Render/GPU obligations.
  Cleanup failures remain observable.
- A handle acquired before `run(self)` may be held by an explicitly
  configured controller or signal adapter. No implicit terminal reader,
  network listener or OS signal handler is installed.
- Wake races with admission, blocking waits or shutdown must not lose
  accepted work; a closed or failed transport cannot claim success.

An intentionally passive `keep_alive` App without an external controller
can still remain blocked by choice; this is explicit opt-in and should
be inspectable as `Idle / no active ingress`, not misrepresented as useful
work. An ordinary configured server may sleep without frames while waiting
for its actual admitted ingress. If this liveness policy proves impossible
to implement safely for a selected Host, that configuration must reject
rather than busy-spin.

The spelling `app.shutdown_handle()` **before** `app.run()?`, and the
future `keep_alive` selector, are illustrative. Their ownership,
`Send`/`Sync` bounds, result types and native/headless wake integration
require executable and platform proof. Direct OS forced termination is not
graceful App shutdown. Successful bounded advancement does not terminate
the App; finite work completion remains distinct from `run()`.

### 3. A wake is not automatically an application frame

Distinguish work classes and their owner-controlled progression:

| Work | Typical trigger | Requires visual Render/Present? |
| --- | --- | --- |
| shutdown, lifecycle and typed command admission | event/notification | No |
| job completion and publication | worker completion | No |
| network or terminal ingress | transport message | No |
| fixed simulation step | due simulation deadline / explicit advance | No |
| UI interaction | admitted event and UI work | Only if presentation requests it |
| GPU submission/readback completion | pending completion obligation | No new frame solely for observation |
| scene or custom RenderFlow | actual render request / product policy | When selected |

An idle/wake path must support bounded drain, reliable notification,
deadlines, fairness and no lost wakeups around the transition to sleep.
A running simulation or selected continuous render policy may intentionally
execute repeatedly; a windowless service with no due work may sleep.

**Progress delivery is an owner integration question, not a new universal
schedule.** Current `engine/src/runtime/frame_lifecycle.rs::run_frame`
invokes the conventional PreUpdate, FixedUpdate (where selected), Update,
RenderPrepare, RenderSubmit and FrameEnd schedules. This proposal does **not**
make those schedule-based consumers runnable without a frame by assertion.
For a non-frame completion, the owning capability must expose or reuse an
explicit, bounded completion/observation path, invoked by the concrete
headless/native driver at a permitted lifecycle point and proved not to
execute the regular frame schedules. If an existing consumer requires
`Update` to make application-level progress, that is a **legitimate
scheduled App update when explicitly requested by its owner**, not an
implicit fake frame from a generic wake. Admission must make the consumer's
execution needs inspectable; unsupported no-frame delivery is not
silently claimed. Fixed-step simulation deadlines are advancement requests
and remain distinct from completion-only turns.

**Current accepted source correction:** #1392's native OnDemand GPU-only
completion and #1276's U9-C native input integration are merged into
`main@b4683b99939464c3bc79e85f04c752687f01bc86`. Reuse their
bounded source-owner completion and input-settlement paths; do not create
another progress loop or claim they provide arbitrary network/headless
command delivery. No fake Update/FixedUpdate/RenderPrepare/RenderSubmit/
FrameEnd or unnecessary Present is authorized. This proposal creates
neither a competing scheduler nor a parallel RunenECS World.

### 4. Windows are runtime instances, not application topology

Windowing is an explicitly selected capability, while physical native
windows are independently requested, realized and retired by the existing
native event-loop owner.

**Typed ingress after consuming `run(self)`:** after selecting windowing
during configuration, the caller can obtain a cheap clonable *window-owner*
request handle **before** passing the App to `run` (illustrative
`app.window_requests()?`). The handle owns no Winit window, ECS World,
Render surface, general command bus or second application runtime. The
native owner provides its actual wake transport when ready. The contract
must expose readiness/closed state; a request submitted before readiness
must either be explicitly and boundedly buffered or return `NotReady`.
The precise choice and API require compile and event-loop proof. External
callers never create/destroy Winit windows on the wrong thread.

A request has a stable correlation ticket and a logical `WindowKey`;
**queued/accepted** is distinct from a terminal result:
`Created(key, incarnation, ...)`, `CreationFailed(reason)` or
`Cancelled/RejectedOnShutdown(reason)` (illustrative categories).
Queue capacity, duplicate active keys, wrong Host, closed ingress and
unsupported placement must reject truthfully. Every **accepted ticket**
must eventually have one observable terminal disposition, including when
the event loop exits, startup fails or shutdown races a request. No
accepted request is silently dropped on queue drain or owner cleanup.
Physical window creation and required owner attachment must complete
before claiming `Created`; first Present/input focus may still be later.

**Identity and primary roles:** `WindowKey` is a logical, app-facing
name/reference, not `NativeWindowId`, Winit `WindowId` or an ECS
`Entity`. Each successful incarnation is correlated with a fresh
generation and owner-controlled physical IDs. Reopening a retired key
cannot revive stale input, paint, UI bindings or outstanding receipts.

The **primary presentation/input convenience role** is likewise a
logical, explicitly assigned owner role. A zero-window native App begins
with *no* primary physical window. Its first late-created window must
not silently become primary just because it is first; a request may
explicitly claim the vacant primary role if that operation is supported
and admitted, otherwise primary-only UI/Render helpers return a precise
unbound/unsupported-target result. Current `NativeWindowId::primary()`
and `RenderSurfaceId::primary()` assumptions require an owner-qualified
migration. A native window without Render/GPU remains valid; headless UI
and offscreen GPU do not imply native window creation.

The physical realization and disposal of windows stay on the selected
event-loop thread. No universal entity registry, mutable service locator,
new `Host` hierarchy or global command layer follows from this model.

### 5. Startup, last-window closure, and shutdown

**Startup occurs once per admitted App**, not once per realized native
window. Native capability/event-loop readiness and the current
Host-specific resources precede Startup, but the first native window
need not exist.

The initial attachment frontier is deliberate:

1. Validate configuration and execution intent. Select the Host and
   clock once; establish native event-loop readiness if selected.
2. For **mandatory initially declared windows**, realize physical windows
   and validate the *already-declared mandatory* owner attachments
   that can actually be checked before Startup. A failed mandatory
   realization rejects Startup with truthful error and owner cleanup.
   Preflight is not proof of later Present, input display, or rollback
   of already committed external effects.
3. When **no initial native windows** are declared, run Startup once
   after selected native readiness without inventing a primary window,
   Render surface or UI input target.
4. Operations **created during Startup** (new UI mounts, RenderFlow,
   window requests or dynamically selected targets where supported)
   are later owner operations with independent outcomes; they are
   not retroactively made required pre-Startup attachments.
5. After Startup, window requests yield correlated `Created`,
   `CreationFailed` or terminal cancellation without rerunning
   Startup. OS suspend/resume and close/reopen do not replay Startup.

Current `winit_runner::resumed` unconditionally realizes a primary
window before Startup, and `realize_declared_primary_input()` currently
occurs **after** Startup. Do not move that input binding earlier by
assertion: decide which selected pre-Startup recipient invariants actually
need preflight, then qualify both ordinary desktop and Startup-authored UI.
A post-mount `PlacementRetained` or pending HostProtocol terminal work
requires owner-specific cleanup and error recovery; there is no universal
transactional rollback of Window, UI, Input and Render effects.

**Two independent policies must not be conflated:**

- **Window-close authorization:** native/product policy approves, vetoes
  or defers one close intent; current primary default is
  `NativePrimaryClosePolicy::Explicit`, while
  `ApproveUnhandled` is opt-in. The ordinary future
  `.window(...)` convenience must explicitly select its documented
  unhandled-close behavior without removing the expert veto path.
- **App lifetime after safe retirement:** App-specific policy separately
  decides whether the application keeps running. The old special
  `NativeWindowId::primary()` must no longer cause unconditional exit.

**Proposed ordinary lifetime defaults** (not final Rust spellings):

| Intent | Default lifetime |
| --- | --- |
| No execution intent, `App::new().run()` | **Reject at admission**, actionable `NoExecutionIntent` (illustrative error) |
| One or more initially declared windows | Request shutdown only after the last window **safely retires and accepted pending window requests are settled** |
| Explicit `enable_windowing()` without initial windows | Remain alive and accept typed window requests after readiness |
| Owner-backed persistent service/ingress | Remain alive until admitted shutdown or terminal failure |
| Explicit `keep_alive` | Remain alive even with zero windows and no other work |
| Explicit `quit_on_last_window_close` | Request shutdown after final safe retirement and prior accepted request disposition |

`enable_windowing()` selects capability, **not** a conflicting
implicit keep-alive override. `.window(...).enable_windowing()`
and `.enable_windowing().window(...)` have the same inferred
ordinary desktop lifetime, unless an explicit lifetime choice overrides
it. Duplicate contradictory explicit policies reject.

**Last-window/request race rule:** only the Native owner can linearize
request admission, window retirement and the transition to stopping.
An accepted request pending when the last window retires prevents a
premature “zero-window -> quit” conclusion: drain/settle that request,
allow a successfully realized replacement to keep the App running,
or close admission and issue terminal cancellation/failure for every
unrealized accepted request before shutdown. A request arriving after
the stopping gate must reject, not be reported accepted and lost.
A vetoed close or pending UI/Input/Render retirement never counts as
safe final retirement. Explicit App shutdown may cancel queued work,
but must publish correlated terminal dispositions. These rules require
concurrent/racing native proof; they are **not** implemented today.

Do not equate window close, document close, simulation termination,
terminal EOF/disconnect, network command completion, App shutdown or OS
forced termination. Previously committed Startup effects are not
generically reversible. Stopping must reject new work, quiesce or cancel
owner work, settle UI/Render/GPU/Input and request tickets, and return
truthful failure when cleanup cannot complete. Exact shutdown sequencing,
boundedness and failure recovery remain executable owner gates.

### 6. Console and external control remain optional callers

Starting an executable from a terminal does not grant an interactive
command language or mean the application should open an OS terminal window.
An explicit stdin console/REPL or other product-owned terminal integration
may accept commands, while native GUI, headless server, CI runner and
background service apps may run with no stdin at all.

`WindowCommands`, simulation-owner commands, job-owner APIs and App-owned
shutdown requests must remain typed and semantically owned by their existing
domains. A console, in-app palette, automation, or authorized IPC/RPC endpoint
may lower parsed input into **those owner contracts**; it may not create a
global untyped command enum, generic mutable service locator, second
execution runtime or arbitrary OS shell authority.

Strict executable launch argument parsing remains product-owned (#1311).
Terminal developer workflow remains #1393. Existing Runenwerk automation
session and terminal orchestration accepted designs govern automation; this
draft introduces no automatic attach, public IPC port, remote shell, CLI
schema, persistent script language or broad new engine registry.

To support a Native window-first or zero-window process, stdin/IPC reads
must not block the native event-loop thread. A closed/failed request channel
cannot be reported as successful command delivery. The same semantics
apply to other wake sources and to shutdown races.

### 7. UI and Render remain owner-controlled

The UI-facing convenience design belongs to #1405. A typed RunenUI app
may be mounted without a physical window. Placement to a realized native
target, offscreen target or custom flow is **separate** from typed UI
runtime identity. Windowless native startup may have a mounted UI with
no immediate Present target.

For an ordinary unambiguous single-window UI-only composition, a selected
Render-owned clear + UI composite + Present recipe is allowed. The
renderer owns clear color, GPU work, and terminal Present; a custom game
or Editor RenderFlow retains those responsibilities. Never fabricate
a primary surface, duplicate Present, or promote input eligibility merely
because a request was enqueued or submitted.

Failure after a typed UI mount, window attachment or terminal publication
must retain the owner-specific pending-work/retirement contract. The
existing `UiAppPrimaryMountError::PlacementRetained` is evidence that a
consuming `run(self) -> Result<()>` cannot naively discard partially
mounted UI on failure. Typed preflight and cleanup or an actionable
recovery outcome require direct proof.

A future **App Info** inspection surface may expose *read-only,
owner-derived* Configuring/Admitted/Requested/Realized/Stopping/Retired
status, selected intent, shutdown policy, window tickets and physical
bindings, typed UI instances, installed ECS resources, and errors with
source/owner links. It must not create a parallel writable App registry
or transfer semantic authority from Host, ECS, RunenUI or Render.

## Illustrative public surface (not final Rust API)

```rust
// Accidental empty open-ended application: clear admission error.
App::new().run()?; // NoExecutionIntent; no native window or silent idle hang.

// Deliberately passive, open-ended application: explicit intent.
// The caller should acquire a shutdown handle before consuming App.
let app = App::new().keep_alive();
let shutdown = app.shutdown_handle();
app.run()?;

// Ordinary native window: no public Host enum necessary.
App::new()
    .window(Window::new("editor").title("Editor"))
    .run()?;

// Native Host with zero initial windows: obtain its typed request handle
// before moving App into run; the owner publishes readiness and receipts.
let app = App::new().enable_windowing();
let windows = app.window_requests()?;
app.run()?; // Windows can be requested by a separately admitted controller.

// Native typed UI convenience, separate #1405 API decision.
App::new()
    .window(Window::new("todo").title("Todo"))
    .ui(TodoUi::new(TodoUiState::load()?))
    .run()?;

// Explicit finite headless advancement remains a separate operation.
let app = App::new().run_for_frames(100)?;
```

All snippets above specify **intent and preferred ergonomics**, not
existing compile-proven `App` signatures. In particular the constructor
shape `TodoUi::new(state)` was tested only through #1426's **test-local**
blanket extension trait/intent facade; public prelude visibility, inherent
`new` conflicts and the accepted `AppUiExt::ui()` method-name collision
remain #1405 decisions. `Window`, `window_requests`, `keep_alive`,
shutdown handle, ticket/result and `NoExecutionIntent` types are
conceptual. A real application must move a request handle into an
actual controller rather than rely on an unused local binding.
Owner-specific admission, failure and state-retention types require
separate Rust compile and platform proofs.

## Rejected alternatives

- Keep `App::new()` intrinsically native-windowed and add a separate
  permanent `HeadlessApp`: makes application semantics depend on the
  desktop-centric constructor and conflicts with one composition root.
- Construct a running Headless Host and silently upgrade it to Native
  when the first window opens: violates stable Host identity and Winit
  event-loop/thread admission.
- Make every App initialize Winit, a GPU, a network server or a terminal:
  imposes unrelated cost/permissions and fails true server/headless cases.
- Make all headless apps run Update continuously while waiting for commands:
  wastes CPU, hides event delivery and breaks fixed simulation/render meaning.
- Require every App to configure a public Host enum: leaks common internal
  environment selection into ordinary application code.
- Introduce a universal AppRunner/command bus/scheduler or Tokio runtime:
  duplicates existing concrete ownership and contradicts retired #875
  and accepted automation designs.
- Report accepted window requests as fully created, or use one unversioned
  window string as a physical handle: permits stale events and incorrect
  render/input attachment.
- Treat an accidentally empty open-ended `App::new().run()` as implicit
  keep-alive: conceals missing execution intent and can hang without any
  useful admission or actionable diagnostics. Explicit keep-alive remains
  supported.

## Migration and acceptance gates

**Intentional public breaking changes**, if accepted:
`App::new()` no longer immediately selects native; bare
`App::new().run()` becomes a diagnostic admission failure; previously
headless `App::run()` no longer returns after one frame; maintained native
products must explicitly declare/select windowing, and headless one-frame
tools must choose bounded execution. **`engine/src/main.rs` must migrate**
from its current bare `App::new().run()` to a declared native window or
another real explicit intent. Audit maintained Arena, Editor, Draw and
Runtime Preview, plus `App::headless`, `set_title`, native size/close
policies, `AppUiExt::ui()`, default-plugin ordering, and direct-world
window/RunenUI mutations before replacing supported call sites.
No indefinite compatibility aliases or silent behavior-preserving
fallbacks are presumed.

**Two distinct approval steps are required.** This *proposed* document may
be critically reviewed and its **semantic policy** accepted only once
its behavioral choices and ownership boundaries are coherent with the
existing accepted ADRs. That does **not** ratify uncompiled Rust
method names, certify unimplemented runtime capabilities, or authorize
a source migration. Any public implementation/delivery issue must
separately demonstrate executable proof and exact-head acceptance;
the initial API-signature selection under #1423/#1405 still needs a
Rust compile gate before being treated as fixed.

Before accepting the final public API or publishing implementation:

1. **Rust API:** Compile-test the fluent by-value/error contract and the
   `AppUiExt::ui()` vs `.ui::<A>()` collision; verify typed default
   UI state, explicit target, and registration order independence.
2. **Execution intent / idle headless:** Prove bare empty `App::new().run()`
   rejects **before Startup** with actionable error; incidental resource,
   ECS system and generic plugin presence do not count as a live service.
   Explicit keep-alive and real owner-backed service remain idle without
   Update/frame spin, have reliable wake and external shutdown after
   `run(self)` consumes App, and install no implicit console/port/GPU/window.
3. **Headless bounded:** Prove zero/positive frame counts, repeatability
   and fixed-step cadence errors/progress remain well-defined.
4. **Native zero-window / typed request:** Prove pre-run acquisition of a
   window-owner request handle, readiness or NotReady behavior, reliable
   no-frame command wake, correlated accepted/Created/Failed/Cancelled
   ticket disposition, Winit Startup **without a primary window**, late
   first-window physical creation/failure, suspended/resumed Startup
   identity and correct close policy.
5. **Multiple native windows:** Prove safe close/reopen, logical
   key + incarnation, explicit primary role versus first-created named
   window, DPI/resize/focus/input correlation, Render surface ownership,
   and last-close versus queued-request/shutdown races, including
   terminal settlement of every previously accepted request.
6. **Integrated UI/game:** Prove native Todo UI, game HUD with custom
   RenderFlow, no duplicate Present and terminal UI/GPU/input settlement.
7. **Mixed work:** Prove concurrent network/worker completion/fixed
   simulation and idle UI window work make progress without starvation,
   lost wake, unbounded backpressure or false visual frames.
8. **Service/automation:** Prove Runtime Preview's Tokio command owner
   can coordinate with App execution without blocking or duplication;
   reject unauthorized/late/stale/unsupported commands.
9. **Portability:** Verify actual supported Windows/Linux/macOS and
   explicitly scope other OS/embedded/web behavior; no universal
   guarantee of a blocking main-thread `run()` on every platform.
10. **Governance:** Reconcile accepted ADR 0019/0023, #884/#875,
    source-owner split #1423/#1405, strict launch ownership #1311,
    automation/terminal designs and **already merged** U9-C (#1276)
    and non-frame GPU progress (#1392). Verify exact-current protected
    main and maintained consumers. Require repository `cargo validate`,
    exact-head hosted evidence, critical review and protected merge.

**Gate verdict at proposal:** the owner-selected **explicit-intent,
fail-first empty App** policy and one-App/windowless direction are now
written down for critical review. This remains a **DRAFT / HOLD**, not
an accepted ADR, public Rust API, sustained headless driver, zero-window
native implementation or native UI proof. Window ticket/wake/shutdown,
primary role, Startup attachment, retained RunenUI/Render work and
maintained-consumer migration require separately qualified execution.
Do not merge or repin Todo from draft heads.
