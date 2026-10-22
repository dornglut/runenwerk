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

`--frames` counts observed terminal GPU frame completions, not host callbacks.
Short runs qualify fixture plumbing only. The 4,096-frame operator runs exceed
the original 148/594-frame failures; they do not establish a universal duration
or memory threshold. The control keeps the same mounted UI and Host while
omitting its GPU UI composite pass; it is a non-UI GPU baseline, not a different
UI framework or a direct standalone text-rendering proof.

Changing mode submits explicit application state changes through the existing
typed action ingress. It does not inject keyboard, pointer or IME events.
During the UI runs, physically click the button, use W/Space/Tab, change focus,
and move/resize the window. Preserve the original `U9C_OS`, `U9C_FRAME` and
activation observations. Absence of actual input cannot qualify input behavior.

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
renderer-accepted/terminal observations. These are point-in-time diagnostic
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
$exe = (Resolve-Path 'target/debug/examples/native_ui_input_qualification.exe').Path
$case = 'static-1'
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
