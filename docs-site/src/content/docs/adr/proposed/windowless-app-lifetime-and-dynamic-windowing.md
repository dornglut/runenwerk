---
title: Windowless App Lifetime and Dynamic Windowing
description: Proposed decision for one open-ended Runenwerk App with explicit optional native-window capability, independent work advancement, typed runtime requests, and safe shutdown.
status: draft
owner: engine
layer: architecture
canonical: true
last_reviewed: 2026-10-30
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

Evidence baseline: protected Runenwerk
`main@505581b30537901de9964badbf99cd083237f359`, 2026-10-30. This is
a static-source review. No Rust compilation, `cargo validate`, OS window
execution, GPU proof or independent consumer qualification is claimed.

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

### 2. Open-ended App lifetime is independent of frames

**Proposed new meaning:** `App::run(self)` starts the accepted App exactly once
and normally remains running until an admitted application shutdown occurs or
the runtime encounters a terminal failure. It does not secretly mean
`run_for_frames(1)` and must not force `Update` or a visual frame during an
otherwise idle period.

A deliberately empty `App::new().run()` is a valid **passive, idle process**:
the runtime waits without spinning until it receives a supported shutdown,
wake or other admitted event. It does **not** imply an interactive terminal,
stdin parser, OS terminal window, new network port, worker pool or timer.
This behavior is intentionally different from a finite batch job.

A real external or application-owned shutdown path must exist for the
blocking run API; the final interface shall expose a practical wakeable
shutdown request that survives the consuming `run(self)` call. Graceful OS
signal integration is platform-specific and cannot be claimed solely from
the presence of an OS process. The exact `AppControl`/handle method names
remain subject to the Rust API proof.

Accepted bounded operations such as `run_for_frames(n)` and the
fixed-cadence-owner `run_for_fixed_steps(n)` remain explicit **nonterminating
advancement** on a reusable App instance. Future finite work completion
policy must not redefine the meaning of open-ended `run()`.

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

An idle/wake path must support bounded drain, reliable notification, deadlines,
fairness and no lost wakeups around the transition to sleep. A running
simulation or selected continuous render policy may intentionally execute
repeatedly; a windowless service with no due work may sleep.

The pending **OnDemand GPU progress / exact terminal delivery** contract is
separately owned by #1392/#1276. This ADR must not authorize fake GPU progress
via repeated Update/FixedUpdate/RenderPrepare/RenderSubmit/FrameEnd or
unnecessary Present. It also does not create a competing scheduler or a
parallel RunenECS World.

### 4. Windows are runtime instances, not application topology

Windowing is an optionally selected pre-run capability. Individual native
windows can be **requested and retired while running** through the existing
native event-loop owner. The public interface uses a stable logical
`WindowKey`; a realized window receives a distinct internal generation,
`NativeWindowId`, platform Winit ID and, when rendering is selected, a
Render-owned surface association.

A request accepted into the command queue is **not** a physically created
window. Its owner must publish a later terminal outcome
`Created(key, generation, ...)` or `Failed(key, reason)`, correlated to
that exact request. Reopening the same human-readable key after retirement
must not allow stale events, inputs, or GPU/UI publications to target its
replacement. Duplicate live keys reject.

Windows must be created and destroyed on the event-loop thread required by
the selected platform. Cross-thread callers send bounded typed requests and
wake the owner; they never hold or mutate Winit windows directly.

Windows must **not** be mandatory Render/GPU surfaces. A native window
without a selected rendering capability remains legal. Headless UI and
offscreen GPU operations remain legal without native windows.

### 5. Startup, last-window closure, and shutdown

Startup is a one-shot App lifecycle occurrence, **not** a callback whose
existence depends on a primary physical window. The native event loop must
be prepared before Startup when native capability is selected; with zero
declared windows Startup still executes exactly once. OS suspend/resume or
later first-window creation cannot rerun Startup.

Window-close intent may be accepted, vetoed or deferred by its owner.
After a window actually closes, the **application's selected lifetime
policy** determines whether the App quits. Do not terminate merely because
a distinguished legacy primary NativeWindowId closed.

Proposed ordinary defaults:

- An App declared with initial desktop windows and no explicit keep-alive
  intent requests **quit after the last eligible window has retired**, once
  the owners permit clean shutdown.
- An explicitly `enable_windowing()` App can remain alive with zero windows
  so a terminal/IPC/automation caller may reopen one later. This explicit
  persistent intent is order-independent with `.window(...)`.
- A windowless App remains active until explicit App shutdown, even if
  it has no active simulation sessions or pending windows.
- An expert product may override the last-window/keep-alive policy and may
  veto or postpone close (unsaved editor documents, long-running work).

Do not equate: window close, document close, simulation termination, command
source EOF/disconnect, background job completion, App shutdown, or OS forced
process termination.

The shutdown protocol must stop admission of new work, quiesce/cancel selected
owners, settle pending typed UI/Render/GPU/Input obligations, and return a
truthful success or failure. Previously committed Startup side effects
are not generically reversible; existing fail-stop semantics remain.
An exact shutdown lifecycle shape and timeout policy need executable owner
proof before acceptance.

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

## Illustrative public surface (not final Rust API)

```rust
// Open-ended, windowless, event-driven runtime; no terminal is implied.
App::new().run()?;

// Declared native window; no explicit public Host selection.
App::new()
    .window(Window::new("editor").title("Editor"))
    .run()?;

// Native event loop exists before any physical window; later runtime commands
// can ask its owner to open the first window.
App::new()
    .enable_windowing()
    .run()?;

// Optional local command adapter; does not open an OS terminal window.
App::new()
    .enable_windowing()
    .console(Console::stdin())
    .run()?;

// Explicit bounded headless advancement for tests and simulations.
let app = App::new().run_for_frames(100)?;
```

These snippets specify desired ergonomics, **not compile-proven signatures**.
Exact `Window` construction, typed key representation, explicit UI state,
runtime command receivers, return/ownership types, clock admission,
generic method resolution, error types and `.ui::<A>()` naming require
an isolated Rust compile probe and an independently maintained consumer.

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

## Migration and acceptance gates

**Intentional public breaking changes**, if accepted:
`App::new()` no longer immediately selects native; headless
`App::run()` no longer returns after one frame; maintained native
products must explicitly declare/select windowing; headless one-frame
tools must choose bounded execution. Existing `set_title`, native
size/close policies, `AppUiExt::ui()`, and default-plugin ordering must
be reconciled without indefinite compatibility aliases.

Before marking the decision accepted or publishing implementation:

1. **Rust API:** Compile-test the fluent by-value/error contract and the
   `AppUiExt::ui()` vs `.ui::<A>()` collision; verify typed default
   UI state, explicit target, and registration order independence.
2. **Idle headless:** Prove Startup once, no idle Update/frame spin,
   reliable wake, correct shutdown request while the consuming
   `run(self)` blocks, no implicit console/port/GPU/window.
3. **Headless bounded:** Prove zero/positive frame counts, repeatability
   and fixed-step cadence errors/progress remain well-defined.
4. **Native zero-window:** Prove Winit readiness and Startup **without a
   primary window**, late first-window physical creation/failure, suspended
   /resumed Startup identity and correct close policy.
5. **Multiple native windows:** Prove independent close/reopen, logical
   key generations, DPI/resize/focus/input correlation and Render
   surface ownership after failures and retirement.
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
    automation/terminal designs, and competing U9-C/Render PRs.
    Require repository `cargo validate`, exact-head hosted evidence,
    independent critical review and protected merge.

**Gate verdict at proposal:** architecture direction is supported by
source and independent use cases; final Rust signatures, sustained
headless driver, zero-window Startup and lifecycle/shutdown/owner-retirement
remain **HOLD**. A proposed ADR is not permission to bypass these gates.
