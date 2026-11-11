# Testing and Validation

Tests belong with the owner of the invariant they prove. Prefer behavior tests, typed APIs, dependency checks, and conformance cases over source-string guards.

## Focused checks

Use the smallest checks that exercise the changed package while implementing, for example:

```text
cargo test -p <package>
cargo clippy -p <package> --all-targets --locked -- -D warnings
python tools/docs/validate_docs.py
```

Focused checks support iteration. They do not replace the merge baseline.

## Integration-test boundaries

An integration-test executable represents an actual isolation or external-boundary requirement, not an individual issue, milestone, phase, or small contract. Related contract tests should normally be modules inside a boundary-owned integration target.

Keep separate executables where platform behavior, process or environment isolation, compile-fail behavior, crash or panic isolation, native GPU conformance, or explicit backend/runtime evidence requires them. Target consolidation must not introduce shared mutable runtime state or weaken test isolation.

## Canonical validation test debug information

The product-workspace test stage inside `cargo validate` uses Cargo's `line-tables-only` debug information policy. This retains filename/line information for panic backtraces while reducing canonical validation test-build debug payload and link cost.

Direct developer `cargo test` commands keep Cargo's normal full-debug test profile. The reduced-debug policy is repository-owned by the validator and is applied identically when `cargo validate` runs locally or in CI.

## Required baseline

Before merge:

```text
cargo validate
git diff --check
```

`cargo validate` leaves source and lockfiles unchanged. It validates the repository tooling, formats the workspace, runs locked workspace tests, removes the default root `target/` build output after the tests to bound peak disk use, runs strict Clippy, validates documentation, and checks durable repository invariants. Cleanup is skipped when a Cargo target-directory override is set; it never removes a target outside the repository or one containing the running validator.

Rust CI resolves and validates the reviewed feature head for pull requests, the exact `github.sha` integration revision for `merge_group`, and `github.sha` for push and dispatch. Each selected revision is explicitly checked out and proved before validation.

Feature-head, merge-group integration, and accepted-main validation are separate evidence stages. When the repository merge queue is enabled, required merge-group checks validate the queue's exact latest-base integration revision before GitHub may merge it. Do not assume SHA equality or inequality between that merge-group revision and the eventual accepted-main commit; accepted-main push evidence independently proves the default-branch state that GitHub accepted.

## Change-aware pull-request validation

A feature-head `pull_request` touching only `docs-site/**`, `README.md`, or
`ARCHITECTURE.md` is eligible for the documentation-only path. CI classifies the
exact pull-request base/head Git history with a merge-base diff and renames disabled,
so a move **from** a Rust/source path into documentation cannot hide a code change.
Empty, mixed, invalid, and unclassifiable diffs are not documentation-only.
Classifier unit tests and explicit exact-revision checks must succeed before a
documentation-only decision is accepted.

For an eligible documentation-only PR head, the Rust canonical baseline and native
Vulkan execution suites are **skipped**, not reported as successful test executions.
The independent `Documentation site validation` job still builds the complete
Astro/Starlight site, proves its publication projection, and remains required.
The always-running `Validation scope gate` explicitly proves the selected path,
the classifier outcome, and the required jobs' actual results. It does not relabel
skipped Rust/GPU work as a successful test run.

Every source, dependency, workflow, validation-tooling, mixed or uncertain PR keeps
the full canonical and native Vulkan evidence on its feature head. Every
`merge_group`, accepted-`main` `push`, and `workflow_dispatch` also runs the
full canonical and Vulkan suites; no documentation-only exemption carries across
the merge-queue boundary. Thus the canonical `cargo validate` and supplemental
native evidence remain mandatory before protected merge, including for docs-only
changes.

Protected-branch required checks for this policy are `Validation scope gate` and
`Documentation site validation`. The individual canonical/Vulkan check contexts
must not remain independently required for documentation-only feature heads; the
scope gate requires their actual success whenever full validation is selected.
The protected ruleset and workflow must be migrated together; do not activate a
partial policy transition.

## Documentation build

Pull requests targeting `main` and merge-group integration revisions always run `Documentation site validation`, which builds the Astro/Starlight production site and validates its publication projection. Main pushes remain path-scoped to changes under `docs-site/**` or `.github/workflows/docs-validation.yml`; root-only documentation changes outside that path scope do not trigger the production build on push. The workflow independently selects and proves the pull-request feature head, merge-group `github.sha`, or push/dispatch `github.sha`; its workflow-definition ref may be a synthetic merge ref, distinct from the checked-out contents.

## R8 ordinary public consumer proof

The Vulkan lane also executes the downstream-shaped R8 ordinary public consumer integration test:

```bash
RUNENRENDER_R8_REQUIRE_GPU=1 cargo +stable test -p engine --test runenrender_ordinary_public_api --locked -- --nocapture --test-threads=1
```

The execution case authors its own renderer scene, request, semantic surface input, availability fact, and output destination using only the candidate transferable RunenRender surface plus public RunenGPU. It requests its own headless RunenGPU context without Runenwerk GPU-context policy, submits through `submit_render`, and requires the returned public RunenGPU submission to complete. The ordinary path authors no CPU readback and does not depend on App/ECS/Winit/World/Render-Lab/private deterministic modules.

## Stateful RunenGPU G5 conformance

The hosted native Vulkan lane must execute actual G5 work, not silently skip unavailable GPU
adapters:

```bash
RUNENWERK_G5_REQUIRE_GPU=1 cargo +stable test -p engine --test render_gpu_retained_cross_frame --locked -- --nocapture --test-threads=1
RUNENWERK_G5_REQUIRE_GPU=1 cargo +stable test -p engine --test render_gpu_fixed_step_continuity --locked -- --nocapture --test-threads=1
```

| Fixture | Required evidence |
| --- | --- |
| `render_gpu_retained_cross_frame` | Independent public G5 fragments sharing a real retained buffer: read/read participation, typed RAW/WAW/WAR conflicts, terminal release, and observed GPU bytes after fresh admission |
| `render_gpu_fixed_step_continuity` | Immutable historical steps, accepted/verified tick separation, same-submission readback identity, capacity, refused semantic commit, verified checkpoint reimport, and safe replay after reconstruction |

These tests exercise public RunenGPU G5 participation, **not** arbitrary application-level
aliasing of separately authored `RenderFlow` buffers. Checkpoint reimport requires
producer-supplied verified bytes; it does not prove old Boids state survives physical device
loss or partial GPU writes.

## Native historical RenderFlow conformance

The native Host GPU hook is exercised under Mesa/Vulkan with an actual attached native surface:

```bash
RUNENWERK_NATIVE_HISTORICAL_GPU_SMOKE=1 cargo +stable test -p engine --test native_host_smoke --locked -- --nocapture --test-threads=1
```

The fixture requires exact source-owned fixed-step dispatch and uniform inputs,
source-correlated G5 receipts, terminal shader output (the accumulated counter changes from
**6** to **15** across two accepted intervals), and no duplicate submission. It preserves an
unverified accepted interval while capturing later immutable steps and rejects an over-capacity
sixth input. A **genuinely attached second window** requesting the same flow must defer until
verification; an unrelated flow must progress independently. It also exercises refused semantic
verification and subsequent commit using exact GPU bytes, not the lossy diagnostic ECS image
cache.

This is an executable native software-Vulkan contract test, not proof of physical driver-loss
restoration, arbitrary cross-flow shared resources, or monitor scanout.

## Boids example and native consumer qualification

`cargo validate` runs workspace tests but does not run unit tests embedded in the Boids example
target. The native Vulkan CI job additionally requires:

```bash
cargo +stable test -p engine --example boids_render_flow --locked
```

CI exercises original and `--species` Boids with a real window and GPU, requiring 120 submitted
Presents, correlated terminal GPU receipts and verified source ticks. Both visual modes must
produce moving image evidence; the no-full-image-readback modes must still verify the mandatory
compact GPU-written final-stage witness. The separate injected **source fault after tick 12**
must fail stop and allow a fresh-process epoch in both laws, without replaying prior accepted
state.

The opt-in smoke settings, release performance measurements, and interactive close behavior are
described in the [Boids example
guide](https://github.com/dornglut/runenwerk/blob/main/docs-site/src/content/docs/engine/examples/boids-render-flow/README.md).
Native software-Vulkan success does not substitute for a real-hardware performance measurement.
A controlled application source failure does not prove arbitrary physical GPU-device loss,
partially executed writes, or in-place reconstruction.

## Supplemental RunenRender GPU execution proof

Current Runenwerk CI also runs `RunenRender Vulkan execution proofs` for pull-request feature heads, merge-group integration revisions, accepted-main pushes, and manual dispatch. It resolves and proves the selected repository revision, restricts Vulkan loading to the installed Mesa software Vulkan implementation, and runs maintained native integration, direct standalone public-consumer, founding-direct, and Render Lab temporal-quality evidence.

This is Runenwerk-owned consumer and render-integration evidence. It does not make Runenwerk the owner of RunenGPU implementation or standalone framework conformance. RunenGPU implementation and conformance are owned by the external `dornglut/runen-gpu` repository and its repository-owned validation/conformance workflows.

Runenwerk validates its exact-revision RunenGPU dependency, public-API consumption, and product/runtime integration through its own locked baseline and focused integration proofs.

## Render Lab camera-motion product evidence

The native Render Lab camera scenario retains eight stationary temporal submissions,
first motion at ordinal 9, continued motion at 10, and holds the moved pose from
ordinal 11. Ordinals 11–14 form four stationary reconvergence samples; 15–20
exercise visible stability across subsequent phases. Capture ordinals 8–20
separately for the settled, motion, stop, reconvergence, and stable-cycle images.
The product proof uses standalone public `RenderTemporalExecutionEvidence` and
Runenwerk-owned capture/stability evidence. Reusable renderer conformance belongs
to standalone `dornglut/runen-render`.

For a clean committed source revision:

```text
RUNENWERK_SOURCE_REVISION=<full-clean-source-SHA> cargo +stable run -p runenwerk_render_lab --locked -- --rl2-camera-quality render-lab/camera-9 --window-size-px 1920x1080 --submitted-frames 9
```

The source SHA in an artifact records provenance; verify the checkout and clean tree
before claiming exact-revision evidence.

## Native host readiness

Native input lifecycle regressions exercise the actual focus-loss/cancellation
handlers and the mounted RunenUI M13 queue with prolonged legal application work:

```text
cargo test -p engine --lib runtime::winit_runner::retirement_tests --locked
```

They distinguish cancellation admission from the later exact terminal receipt,
verify held Space and touch cleanup across bounded pumps, and keep a retired
recipient quarantined until its original scope's cancellation evidence arrives.
Missing scopes cannot certify retirement; World-only retirement still requires
actual framework shutdown. These deterministic integration proofs supplement
the real native OS input qualification; they are not hardware touch evidence.

The existing process-isolated `native_host_smoke` executable exercises the native
host with ordinary App-configured, thread-affine RunenUI applications. Select one
`RUNENWERK_NATIVE_UI_READINESS_SMOKE` mode per process:

| Mode | Owning proof |
|---|---|
| `hidden` | Hidden primary, secondary-window callbacks, cross-thread effect wake, timer cancellation and earlier replacement under OnDemand. |
| `visible` | The same readiness and retirement path with visible windows. |
| `crossed-deadline` | Replacement deadline crossing during Engine Update retains another native execution opportunity. |
| `continuous` | Readiness respects the 30 FPS continuous cap and invokes the canonical frame once. |
| `failed-frame` | Update failure is returned after native slot retirement and late-wake closure. |
| `closed-proxy` | Destroying the real typed event loop makes initial and repeated notification fail. |
| `late-manual-registry` | A directly mounted manual registry installed during Startup fails native qualification and receives observed retirement. |

For example, in PowerShell:

```powershell
$env:RUNENWERK_NATIVE_UI_READINESS_SMOKE = 'hidden'
cargo test -p engine --test native_host_smoke --locked
Remove-Item Env:RUNENWERK_NATIVE_UI_READINESS_SMOKE
```

In a POSIX shell, set the same variable on the Cargo invocation. Linux native
execution requires an available display; record X11/XWayland or Wayland explicitly.
These host proofs supplement the existing native no-Render, Render/UI terminal
Present, and Editor surface-identity smokes in the Vulkan lane. They do not establish
pen/touch hardware, IME hardware, macOS/iOS or AccessKit action-routing evidence.
On an interactive desktop, the broader readiness fixture credits at most one
frame for new raw mouse motion since the preceding canonical Update. Coalesced
surplus never credits later frames; the bound on unexplained frames and the
wall-time, frame-hook and receipt checks remain active.

The owning Engine unit tests exercise initial typed/program clock qualification,
deterministic headless time, real framework motion/type-ahead deadlines, bounded
journal admission and FrameEnd retirement. The public typed observation contract
also has compile-fail doctests protecting native clock/wake ownership:

```text
cargo test -p engine --lib --locked
cargo test -p engine --doc --locked
```

The timer-free native wake owner probe runs in a separate process with
`RUNENWERK_NATIVE_IDLE_WAKE_SMOKE=1`:

```text
cargo test -p engine --lib runtime::native_host_wake::tests::native_idle_wake_reaches_real_thread_affine_runtime --locked -- --exact --nocapture --test-threads=1
```

It releases a worker only after the concrete native gate is idle and the real UI
registry has no deadline or continuation. Acceptance observes the actual worker
waker call, native proxy acknowledgement, invoker-thread application update and
slot retirement. Enforce an external process timeout when running it; a missing
wake must fail rather than be helped by an application timer.
This autonomous probe rejects external raw mouse motion as contaminated evidence;
run it on a quiet native display.

## Native OnDemand GPU progress

The source-neutral `native_gpu_progress` example uses the ordinary App/Render/UI
path with OnDemand pacing. Argument `0` is passive static UI; `16` requests one
application action after each exact completed publication. Application actions
run in the ordinary native frame hook before native slot settlement; GPU-only
callbacks observe terminal facts and request a continuation, without acting.

```text
cargo build -p engine --example native_gpu_progress --locked
cargo run -p engine --example native_gpu_progress --locked -- 0
cargo run -p engine --example native_gpu_progress --locked -- 16
cargo run -p engine --example native_gpu_progress --locked -- 0 on-demand capture
cargo run -p engine --example native_gpu_progress --locked -- 0 continuous-30
```

Use an external process watchdog and a quiet native display. Startup OS resize
events can create legitimate initial frames; retain their causes separately.
Require `qualification_complete`, exact accepted/completed identities, matching
final typed application state/paint publication, and zero assisting input. After
`idle_start`, observe a stationary frame/Present count and process CPU before
closing the test window. A watchdog termination is not graceful shutdown proof.
`RUST_LOG=warn,engine.runtime.gpu_progress=trace` exposes GPU-only turns and deadline
retirement. Record the source, executable, actual adapter, wake causes and idle
measurement conditions; close competing games/GPU workloads for latency evidence.

Two opt-in owner-local native tests run in separate filtered processes:

```text
cargo test -p engine --lib plugins::render::renderer::gpu_progress_tests::native_prepared_readback_affinity_and_shutdown --locked -- --exact --nocapture --test-threads=1
cargo test -p engine --lib runtime::winit_runner::gpu_progress_tests::native_deadline_retirement_and_context_replacement --locked -- --exact --nocapture --test-threads=1
```

Set `RUNENWERK_NATIVE_GPU_PROGRESS_SMOKE=1` for the first test and
`RUNENWERK_NATIVE_GPU_DEADLINE_SMOKE=1` for the second. These exercise real native
contexts, prepared-only idle, independent readback terminality, stale early-frame
affinity, controlled shutdown, and deterministic deadline retirement/replacement.
Controlled shutdown/context replacement is not physical driver-loss evidence;
a deterministic readback-interest test is not a forced physical readback-only
interval. Retain the separate physical qualification requirements.

The existing timer-free idle-wake probe also protects late producer delivery.
Set `RUNENWERK_NATIVE_WAKE_ACK_TEST=1` in a separate filtered process for
`runtime::native_host_wake::tests::native_wake_acknowledgement_preserves_racing_and_closed_intent`
to exercise the real proxy's stale-callback, racing-producer and closed-gate cases.

## Evidence

Report focused checks, `cargo validate`, exact-head CI, and anything not run. Do not convert source inspection or user-reported output into a stronger validation claim.

The accepted base, reviewed feature head, merge-group integration revision, queue/squash transition, and accepted-main push result are separate evidence stages or records. Separate stages may reference the same commit object, so each transition is verified explicitly rather than inferred from SHA equality or inequality.

This file owns Runenwerk-local validation semantics. Organization-wide GitHub, review, and validation-evidence rules are owned by [`dornglut/engineering`](https://github.com/dornglut/engineering).
