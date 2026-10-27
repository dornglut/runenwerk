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

The native transport acknowledgement race/closure proof also requires a separate
process, with `RUNENWERK_NATIVE_WAKE_ACK_TEST=1`:

```text
cargo test -p engine --lib runtime::native_host_wake::tests::native_wake_acknowledgement_preserves_racing_and_closed_intent --locked -- --exact --nocapture --test-threads=1
```

It uses a real Winit proxy to test coalescing, duplicate acknowledgement, producer
wakes before/after and racing the settlement checkpoint, terminal closure, a
fresh transport, and failed delivery after event-loop destruction. The timer-free
worker test above separately proves runtime delivery on the native invoker.

## Native OnDemand GPU progress

The `native_ui_input_qualification` example has two separate bounded OnDemand
controls. The existing lifetime modes below remain ContinuousCapped(30).

```text
cargo test -p engine --example native_ui_input_qualification --locked
cargo build -p engine --example native_ui_input_qualification --locked
cargo run -p engine --example native_ui_input_qualification --locked -- --ondemand-changing --frames 16
cargo run -p engine --example native_ui_input_qualification --locked -- --ondemand-static --frames 1
```

Run without cursor motion, input injection or a periodic application timer.
Changing requires 16 exact text-generation completions, each following one
explicit `RW1392_CONTINUATION` and one next-frame `RW1392_ACTION`. The initial
publication plus those actions require 17 ordinary frames/submitted Presents;
any additional frame needs a separately identified genuine native cause.
`RW1392_TERMINAL` retains exact submission/publication identity and host-observed
terminal age. `RW1392_DRAINED` requires submissions and independent readbacks
to drain. Changing then approves close without another visual frame.
For a separate dynamic final-idle control, set
`RUNENWERK_ONDEMAND_HOLD_AFTER_DRAIN=1`. This retains the same actions and drain
checks but leaves the completed application in native Wait. End that observation
with an external watchdog and retain all frames, native wakes and CPU samples;
watchdog termination does not count as normal workload exit.

Static has no application continuation. After one frame and exact terminal UI
delivery it deliberately remains in native Wait. Use an external watchdog for
the idle observation; its termination is expected control termination, not a
normal exit or GPU failure. Retain the entire idle CPU/wake series. Enable
`RUST_LOG=engine.runtime.gpu_progress=debug,engine.runtime.native_wake=trace`
for native wake causes, ordinary request causes/callers, proxy acknowledgement,
selected waits, progress deadlines and lateness. A callback with already-covered
intent must not request another ordinary frame.
Retain exact clean source SHA, executable SHA256, build/toolchain/OS/driver,
stdout, stderr, process CPU samples, exit code and watchdog outcome. Repeat in
a fresh process. These controls establish neither hardware input nor scanout.

The Windows prepared-only/readback/frame-affinity proof uses its own process:

```powershell
$env:RUNENWERK_NATIVE_GPU_PROGRESS_TEST = '1'
cargo test -p engine --lib plugins::render::renderer::tests::native_progress_clears_frame_affinity_and_ignores_prepared_only_work --locked -- --exact --nocapture --test-threads=1
Remove-Item Env:RUNENWERK_NATIVE_GPU_PROGRESS_TEST
```

Run the native readiness modes, timer-free worker wake, ordinary Engine tests,
continuous lifetime controls and canonical baseline as separate regressions.
Device-generation replacement is a controlled software lifecycle test; it
does not prove arbitrary physical device-loss recovery.

The GPU progress opt-in also submits a second readback before graceful context
shutdown and requires actual completion, mapped bytes, closed lifecycle and
retired progress interest. To test the qualification consumer's identity guard,
run its example test in a separate Windows process:

```powershell
$env:RUNENWERK_NATIVE_CONSUMER_WAIT_TEST = '1'
cargo test -p engine --example native_ui_input_qualification --locked native_consumer_wait_rejects_retired_and_foreign_identity -- --nocapture --test-threads=1
Remove-Item Env:RUNENWERK_NATIVE_CONSUMER_WAIT_TEST
```

It uses real GPU contexts and mounted UI scopes with controlled lifecycle inputs
to reject foreign context/slot/scope, stale binding epoch, close intent, retired
recipient and removed target. It does not inject physical device or display loss.

The runner-level readback-only test drives controlled owner snapshots through
the actual native progress-turn scheduler: submission terminality leaves one
pending readback, unrelated events retain its due deadline, terminal delivery
occurs once, and drain retires interest without requesting a frame:

```text
cargo test -p engine --lib readback_only_interest_survives_runner_turns_and_retires_after_delivery --locked -- --nocapture
```

Those snapshots simulate execution timing; they do not establish physical
submission/readback overlap. The affinity comes from a real context; absence of
an adapter is reported as unavailable, not native qualification.

Run the attached-native lifecycle test in a separate Windows process for each
`RUNENWERK_NATIVE_PENDING_TEARDOWN` value, `close` and `context-drop`:

```text
cargo test -p engine --lib attached_on_demand_close_preserves_exact_pending_terminal_outcomes --locked -- --nocapture --test-threads=1
```

It uses the actual OnDemand Winit runner, an attached GPU/UI surface, an exact
accepted submission and retained public readback handles. Controlled native close
precedes final observation; a separate mode destroys the context before close.
Both require no stale displayed input, one ordinary frame, no continuation,
retired deadline, closed wake transport, and exact `ContextDropped` outcomes for
pending retained handles after owner destruction. Use an external watchdog and
preserve failures. Neither mode proves driver-level physical device loss or
successful recovery after such loss. The separately owned `native_host_smoke`
source is not modified by these tests.

## Native RunenUI resource lifetime

The existing `native_ui_input_qualification` example retains its owner-operated
`--interactive` mode with on-demand pacing. Its explicit lifetime modes use the
same native Host, selected UI recipient and RenderFlow at 30 FPS:

```text
cargo test -p engine --example native_ui_input_qualification --locked
cargo build -p engine --example native_ui_input_qualification --locked
cargo run -p engine --example native_ui_input_qualification --locked -- --lifetime-static --frames 4096
cargo run -p engine --example native_ui_input_qualification --locked -- --lifetime-changing --frames 4096
cargo run -p engine --example native_ui_input_qualification --locked -- --lifetime-control --frames 4096
```

`--frames` requires at least that many terminal GPU frame completions with the
selected UI publication confirmed by the existing completed-Present association,
not host callbacks. Changing mode additionally requires that many applied text
generations to reach newer matching paint and completed publications; it waits
for each acknowledgement before requesting the next change. Missing UI or a
stalled change fails after 15 seconds. The control does not require UI evidence.
Short runs qualify fixture plumbing only. The 4,096-frame operator runs exceed
the original 148/594-frame failures; they do not establish a universal duration
or memory threshold. The control keeps the same mounted UI and Host while
omitting its GPU UI composite pass; it is a non-UI GPU baseline, not a different
UI framework or a direct standalone text-rendering proof.

Changing mode submits explicit application state changes through the existing
World-owned native UI transaction and typed action ingress. It does not inject keyboard, pointer or IME events.
During the UI runs, physically click the button, use W/Space/Tab, change focus,
and move/resize the window. Preserve the original `U9C_OS`, `U9C_FRAME` and
activation observations. Absence of actual input cannot qualify input behavior.
Set `RUST_LOG=engine.runtime.input=trace` to retain the native input owner's
withholding and failure diagnostics on stderr. The example enables tracing only
when this variable is set; diagnostics do not grant input permission.

A native close intent cancels the current UI gesture before product close policy
approves or vetoes it. Input received while that decision is pending remains
neutral physical evidence without UI, Editor or game permission. A veto does
not restore a cancelled gesture or held gameplay grant; recovery requires a
genuine release and new press. Focus, modifiers, resize/DPI and repeated close
callbacks must preserve this boundary. Focused Engine tests cover these pending
and veto cases; they do not replace original OS interaction evidence.

After the requested completion count, the example removes its flow, requires
all observed accepted frame submissions to drain within 15 seconds, observes
three seconds of idle without new GPU submissions, recreates the flow, requires
120 recovery completions, drains again and explicitly approves native close.
Closing early, a failed/invalidated submission, a terminal delivery gap, a
changing-text action failure or incomplete drain is failure. A successful run
prints both `U9C_LIFETIME_COMPLETE` and `U9C_EXIT result=ok` and exits zero.
Restart the executable for a second fresh-process run of each case; flow
recreation is not device-loss reconstruction or process restart.

Each `U9C_RESOURCE` record contains G4C1 resource kinds/count/limit, G4C2
binding counts/limit/reservations, instantaneous G5 in-flight count and separate
renderer-accepted/terminal observations. UI frame credit additionally requires
the existing terminal outcome to carry the matching publication: the renderer
verified its exact contribution token and Present against that submission and
context affinity. This optional metadata uses the existing bounded delivery
queue; missing metadata grants no credit. It does not replace native input
permission. These are point-in-time diagnostic
snapshots, not atomic cross-owner statistics. Registry-only entries can remain
until pressure; idle does not promise zero registry counts. Inspect the full
post-warm-up series for repeated pressure recovery, bounded working-set shape,
and exceptions rather than treating a final low count as a memory proof.
Set `GROTTO_RENDER_REALIZATION_LOG=1` to retain the existing binding log and
the exact successfully projected UI-publication count at the preparation
boundary, labeled `prepared_not_submitted`. Neither preparation nor a paint
intake ID establishes accepted work, completion or displayed input permission.

For exact-source Windows qualification, use a clean committed checkout, retain
`git rev-parse HEAD`, `git status --porcelain`, `rustc -Vv` and the build log,
and set `RUNENWERK_SOURCE_REVISION` to that verified SHA. The example prints this
as an operator-provided provenance claim, not self-verifying source evidence.
Retain stdout/stderr separately, actual adapter facts, exit status and memory
samples. For example, after building in PowerShell (adjust the executable path
when using `CARGO_TARGET_DIR`):

```powershell
$env:RUNENWERK_SOURCE_REVISION = (git rev-parse HEAD).Trim()
if (git status --porcelain) { throw 'Qualification requires a clean source' }
$env:GROTTO_RENDER_REALIZATION_LOG = '1'
$env:RUST_LOG = 'engine.runtime.input=trace'
$exe = (Resolve-Path 'target/debug/examples/native_ui_input_qualification.exe').Path
$case = 'static-1'
$evidence = Join-Path $env:TEMP ('runenwerk-u9c-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $evidence -ErrorAction Stop | Out-Null
$case = Join-Path $evidence $case
$proc = Start-Process -FilePath $exe -ArgumentList '--lifetime-static','--frames','4096' -PassThru -RedirectStandardOutput "$case.stdout.log" -RedirectStandardError "$case.stderr.log"
$deadline = (Get-Date).AddMinutes(15)
while (-not $proc.HasExited) {
    $proc.Refresh()
    if (-not $proc.HasExited) {
        [pscustomobject]@{Time=(Get-Date).ToUniversalTime().ToString('o');Pid=$proc.Id;WorkingSet=$proc.WorkingSet64;PrivateBytes=$proc.PrivateMemorySize64} | Export-Csv "$case.memory.csv" -Append -NoTypeInformation
    }
    if ((Get-Date) -gt $deadline) { Stop-Process -Id $proc.Id; throw 'Qualification timed out' }
    Start-Sleep -Seconds 1
}
$proc.WaitForExit()
$proc.Refresh()
if ($null -eq $proc.ExitCode) { throw 'Exit status was not observed' }
$proc.ExitCode | Set-Content "$case.exit.txt"
if ($proc.ExitCode -ne 0) { throw 'Qualification failed; retain stderr' }
```

Repeat with changing/control flags and fresh case names, twice per case. Process
working set/private bytes are CPU/process observations, not GPU driver memory.
Compare like-for-like adapter, extent, pacing and input conditions; retain any
failures. Independent review must inspect source provenance, complete logs,
resource/completion evolution, idle/recovery and shutdown before claiming native
qualification. Software-Vulkan execution requires its actual native display and
adapter; local Metal evidence cannot substitute for the original Windows GPU.
No arbitrary driver reset, readback-map fault or monitor scanout is established
by this fixture.

## Evidence

Report focused checks, `cargo validate`, exact-head CI, and anything not run. Do not convert source inspection or user-reported output into a stronger validation claim.

The accepted base, reviewed feature head, merge-group integration revision, queue/squash transition, and accepted-main push result are separate evidence stages or records. Separate stages may reference the same commit object, so each transition is verified explicitly rather than inferred from SHA equality or inequality.

This file owns Runenwerk-local validation semantics. Organization-wide GitHub, review, and validation-evidence rules are owned by [`dornglut/engineering`](https://github.com/dornglut/engineering).
