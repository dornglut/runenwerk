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

## Downstream stateful RunenGPU G5 conformance

The native Vulkan consumer lane also exercises the public RunenGPU retained-work
integration boundary used by Runenwerk. These tests request a headless GPU
context and MUST fail, rather than silently skip, when the required adapter is
unavailable in the hosted lane:

```bash
RUNENWERK_G5_REQUIRE_GPU=1 cargo +stable test -p engine --test render_gpu_retained_cross_frame --locked -- --nocapture --test-threads=1
RUNENWERK_G5_REQUIRE_GPU=1 cargo +stable test -p engine --test render_gpu_fixed_step_continuity --locked -- --nocapture --test-threads=1
```

The first proves conservative read-only retained-resource exclusion, independent
retained-resource admission, terminal readback, and fresh post-terminal
submission. The native historical RenderFlow hook also asserts that no two
still-pending G5 frame receipts can select its registered stateful flow,
including after its producer has committed all steps and subsequent frames
merely read the retained counter for presentation.
The second exercises the `RenderGpuStepDemand<T>` immutable-input transport with a test-owned producer: bounded historical input capture across
deferred host frames, exact GPU results, transient G5 conflict followed by
fresh preparation, and explicitly verified GPU reset followed by requeue of the
*same* historical inputs. The recovery fixture reconstructs a semantically
unverified **terminal-success** batch; it does not emulate an unknown
partially executed write or device loss. These public G3/G5 tests do not
implement production RenderFlow scheduling or prove native windowed boids
`--species` continuity, post-failure GPU reconstruction, or visible Present.
An exact-head green Vulkan lane is therefore a *scoped* integration result,
not completion of [Runenwerk #1291](https://github.com/dornglut/runenwerk/issues/1291).

## Native historical RenderFlow execution proof

The hosted Vulkan native-window lane runs the opt-in
`RUNENWERK_NATIVE_HISTORICAL_GPU_SMOKE=1` mode of
`engine/tests/native_host_smoke.rs` on Mesa Vulkan under Xvfb. It constructs a
real App/RenderPlugin and an ordinary composed RenderFlow with a G3-owned
initially zeroed retained counter, independently captured heterogeneous
producer uniforms and fixed-time controls, and a real WGSL compute mutation.
The test requires exact G5 source receipt identity, terminal physical
success and the absence of duplicated admission. Since a fast GPU may retire
a renderer's pending-only receipt before the native hook runs, the native
proof also corroborates successful primary and secondary submissions through
surface-keyed submitted-frame history, never by mistaking mere preparation
or native-window attachment for successful GPU admission. A second actual
fullscreen GPU pass reads the retained counter and colors the captured surface
pixel green iff the original heterogeneous step values sum to exactly 6; the
test requires that exact green RGBA pixel and only then commits the
producer's semantic tick 3. **While the first three-step G5 batch is
accepted but semantically unverified**, the producer separately captures
historical ticks 4 and 5 on two different host redraws. The bounded demand
holds three accepted plus two pending inputs; a sixth input must return typed
`CapacityExceeded` and its exact unchanged payload. The test deliberately retains the *physically completed* predecessor
for at least four further native redraws after the first verified result.
An independently resourced and explicitly surface-scoped second flow must
submit a real native secondary-surface frame while the primary is deferred;
there must be no extra primary submission before semantic verification.
This proves independence from actual submitted frame history, not merely
window attachment. The first terminal green pixel is
verified, but semantic verification is deliberately refused once and must
leave the accepted batch and both pending inputs unchanged. After an explicit
successful commit to tick 3, the original tick-4 and tick-5 inputs form a
fresh two-step G5 batch. Its fullscreen shader must expose exactly 15
(1+2+3+4+5) as a blue RGBA pixel before semantic tick 5 is committed. Required source results are associated with the exact accepted G5
submission in the producer's `RenderGpuFlowStepStreams`. The native producer
tests that G5 receipt and its readback bytes with optional diagnostic captures
disabled; derived `RenderCapturedTextureState` is not continuity authority.
Separately, ordinary progressed GPU diagnostic publication can survive a
primary frame's failed surface acquisition, preparation or G5 admission,
without turning the failed frame into a submitted frame or Present. Secondary-surface success/error delivery and multi-surface capture
isolation remain unqualified. Captures include an originating render surface
in their identity, serialized manifest entries and deterministic image filenames.
Artifact export also requires a matching `(frame, surface)` for every capture
before writing; each frame manifest's name and payload contain its source
surface. A two-surface regression proves collision-free same-ordinal manifests,
independent images, both run-manifest entries, and fail-closed foreign-source
rejection. This qualifies export source isolation only, not ECS evidence delivery.
Pending diagnostic transactions now distinguish (frame, surface) so simultaneous
window frame ordinals and selector indices cannot route terminal captures to a
foreign transaction; primary report partitioning also treats a same-frame capture
from another surface as foreign terminal input. A same-frame/two-surface
regression covers delayed completion.
Secondary-surface diagnostics now begin on successful secondary submissions
when normal full-report policy is enabled, with their own capture plan,
source scope and simulation-tick snapshot. Completed reports and canonical
diagnostics entries now carry exact (frame, surface) identity. Every terminal
capture selector result, including unmatched/disabled outcomes without GPU
readback identity, preserves its originating frame and surface. Routing and
transaction admission reject foreign-source results rather than assigning
them to the currently submitting surface. Source-keyed bounded ECS delivery
and accepted G5 readback association have separate proofs; full native concurrent/deferred-primary stress and lossless producer
result delivery remain unqualified.
It rejects shader fallback rather than permitting a no-op simulation.

This is a **hosted software Vulkan execution proof**, not a native end-user
hardware/driver qualification, a raw storage-buffer readback, a proof of
reconstruction after partial GPU failure, or original and
`--species` boids at 120+ visible frames. Those remain #1291 acceptance work.

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

## Evidence

Report focused checks, `cargo validate`, exact-head CI, and anything not run. Do not convert source inspection or user-reported output into a stronger validation claim.

The accepted base, reviewed feature head, merge-group integration revision, queue/squash transition, and accepted-main push result are separate evidence stages or records. Separate stages may reference the same commit object, so each transition is verified explicitly rather than inferred from SHA equality or inequality.

This file owns Runenwerk-local validation semantics. Organization-wide GitHub, review, and validation-evidence rules are owned by [`dornglut/engineering`](https://github.com/dornglut/engineering).

## RW-1291-SURF07 — derived capture delivery (draft)

Terminal capture/selector observations now drain from renderer lifecycle exactly
once after primary success, deferral or failure and after secondary success,
deferral or failure. Observations for earlier source `(frame_index,surface_id)`
are routed to **existing** diagnostic transactions, without fabricating
a transaction for a deferred/unsubmitted frame. The derived ECS capture view
retains up to 64 records and 64 MiB across source identities. Overflow (old-record eviction or oversized incoming capture rejection)
increments `dropped_observations` and retains `last_dropped_identity`;
unscoped matching captures on multiple surfaces are refused, while
`find_from_source` requires exact origin. Unit tests check independent
same-ordinal surfaces and observable bounded eviction.

This is **not** producer-semantic proof or lossless delivery: losing a capture
cannot advance a producer verification cursor. Complete new-secondary
diagnostic initiation and accepted G5 capture receipt association are now
implemented in later increments; end-to-end native sustained two-surface
deferral/capture stress, partially executed GPU state restoration,
unrelated same-surface progress and real original/species boids
120+ visible frames still require their own qualification.

## RW-1291-G5CAP08 — exact accepted readback/producer receipt correlation (draft)

Renderer terminal capture observations originating from an accepted public G5
submission carry `RenderCaptureSubmissionReceipt`, with **that submission's
ID, context/device-generation affinity, and exact GpuReadbackId**. A producer
holding its own immutable accepted `GpuSubmission` must verify
`RenderCapturedTexture::matches_completed_submission(submission)` before
considering captured bytes: the method requires a completed capture, present
RGBA payload, matching submission ID and affinity, that submission's physical
`Completed` status, and the exact readback's `Ready` status.
Preaccept/unsupported capture failures cannot synthesize a receipt.

The native historical fixed-step smoke tracks observation loss separately
for each accepted G5 producer interval. Eviction while that exact interval
awaits semantic verification invalidates its diagnostic evidence; once the
producer commits the interval, later permitted diagnostic history eviction
does not retroactively invalidate the committed state. Real green and blue
pixel verification still requires the exact accepted producer submission's
readback receipt. Pixel comparison and producer-owned `commit_verified`
remain **separate** operations. No field or method here asserts Present,
validated pixel meaning, zero-gap delivery, or successful retained-state
reconstruction. This is a source/provenance API and one native smoke assertion,
not a general lossless producer verification queue or end-to-end
cross-surface/recovery qualification.

## RW-1291-SURF10 — full source-keyed secondary diagnostic reports (draft)

A successfully submitted secondary surface now begins the normal full
diagnostic transaction from its own submission-time resolved capture plan,
provenance, original simulation tick, selector results and readback observations,
using `(frame_index, surface_id)` as the transaction identity. Progressed
terminal observations for earlier sources remain routed to their already
pending source transactions. A deferred or rejected *new* surface never
fabricates a successfully submitted diagnostic snapshot.

Completed `RenderDebugFrameReport` includes `surface_id`. Bounded report
history is keyed by `(frame, surface)`, with exact-source methods and
frame-only legacy convenience lookups that refuse an ambiguous collision.
Canonical render diagnostics payload schema version 3 adds `surface_id`;
entry and artifact attachment IDs also include source surface to prevent
different windows at the same frame ordinal from sharing identifiers.
Regression tests cover same-frame two-surface completed report isolation,
unique canonical diagnostic/attachment IDs, and retained original tick
and surface in delayed transactions.

This is a Runenwerk diagnostics delivery and source-provenance correction,
not RunenGPU lifecycle authority. These bounded diagnostic surfaces are not
a lossless producer-verification transport. Validation of genuinely
simultaneous native windows, deferred primary with secondary readback,
resource-sharing across surfaces, partial-failure restoration, and original
boids / `--species` at 120+ visible frames remains separately required.

## RW-1291-PROD11 — retained G5 capture evidence, without diagnostic-image delivery

In the ordinary historical fixed-step RenderFlow, each accepted producer interval
holds the exact public `GpuSubmission`. Immediately after real G5 acceptance,
the renderer associates **only canonical capture readback IDs actually present
in that same submission** with the registered producer's flow and exact
`(frame, surface, pass, capture point)`. The `RenderGpuFlowStepStreams`
sidecar retains up to 64 lightweight readback descriptors per active accepted
interval; exceeding the bound is an explicit `CapacityExceeded`, never a
truncated apparently-valid list. Descriptors are invalidated by new accepted
interval association and cannot be returned after the producer no longer owns
the matching accepted G5 submission.

The producer's `accepted_capture_readbacks(flow)` obtains these descriptors
without reading the derived `RenderCapturedTextureState` ECS cache. Its
`sample_center_rgba8(submission)` only provides pixels for exact submission
ID/affinity + physically completed G5 work + `Ready` canonical readback,
matching format, dimensions and byte length, and correctly maps BGRA to RGBA.
Missing/pending/failed/out-of-generation evidence returns **no pixel**; it
does not advance the producer. The source then independently checks expected
pixels, immutable step lineage, and commits with `commit_verified`. The
public RunenGPU submission, not a Runenwerk event queue, retains the
readback bytes until the producer releases that submission.

The native historical smoke has been changed to verify its green/blue
semantic counter pixels through this producer-held G5 readback, rather
than looking up potentially evicted diagnostic ECS images. It still
checks two successive accepted intervals, 1+2+3=6 and 6+4=10, bounded
historical input capture, explicit producer verification refusal,
non-duplicate admission, and correct source/surface identity.

This is a **bounded source-flow receipt association**, not an assertion that
all RenderFlow capture selectors belong to a producer, that G5 guarantees
pixel meaning or Present, that unsupported/missing readbacks can prove
simulation progress, or that a production boids owner has adopted the
verification policy. Arbitrary cross-flow evidence composition, GPU
partial-write restoration, native two-surface stress, and 120+ presented
original/species boids frames remain separately unqualified.

## RW-1291-PROD12 — required producer result readback authoring

A registered fixed-step producer can now `require_capture(flow, selector)`
using an exact flow, pass, Before/After capture stage and resource. This
requirement belongs to the producer's source contract rather than optional
`RenderDebugConfigResource` selectors or
`RenderDebugControlResource.{capture_enabled,readback_enabled}`. The producer
may not mutate requirements while its prior accepted G5 interval is
unverified. Duplicate, wildcard, unregistered and descriptor-capacity requests
are rejected at registration; unresolvable, unsupported, multiply matched,
aggregate-readback-budget overflow (>64 MiB RGBA8-equivalent per submission),
or failed readback preparation is rejected before submission, without losing
source-owned pending steps. Ordinary optional diagnostics retain their
existing behavior and reporting indices; producer-required captures do not
populate diagnostic image history.

The native historical green/blue fixed-step test now disables **both** debug
capture and readback flags, registers the source-required display-pass texture
capture, and still verifies its exact G5 readback pixels and source interval
before `commit_verified`. Focused `FrameCaptureRuntime` tests exercise
source requirement vs disabled diagnostic controls, omission from diagnostic
reporting, terminal-failure rejection, and exact/unique registration policy.

This does not generalize automatic readback creation to storage buffers or
other GPU result classes, recover partially executed retained state, guarantee
independent rendering of conflicting resources, or qualify real owner-native
boids/three-species 120+ visible frames. Those remain separate acceptance
requirements.

## RW-1291-REC13 — actual G5 checkpoint/reconstruction/replay qualification

The focused `gpu_step_demand` recovery test uses the pinned public RunenGPU
buffer upload, readback, submitted work and context lifecycle, **not**
fabricated submission statuses or CPU-only verification. It establishes an
initial source-owned GPU checkpoint from real terminal readback bytes, accepts
an unverified retained-buffer overwrite with exact historical input ticks,
queues another immutable input, then drops the original GPU context before
the overwritten contents can be trusted. The accepted submission becomes
Failed, cannot commit, and all captured inputs remain owned.

A different GPU context first restores **wrong** producer-checkpoint bytes:
even though that upload and its readback complete, the producer verifier
rejects the physical result and preserves the accepted/queued inputs. A later
distinct source-backed reimport of the producer's original verified checkpoint
must itself become terminal and return the exact expected bytes before
`requeue_verified_after_reconstruction` can release the original accepted
prefix. The test rejects replay while the old write remains pending, while
reimport is pending, and on wrong bytes. It then resubmits precisely the
original immutable historical prefix, verifies real GPU readback and commits
only those steps; the later queued successor remains pending.

This qualifies an executable **owner-supplied recovery contract** with
real G5 bytes and generation change. It does not claim that ordinary RenderFlow
automatically persists producer checkpoints, rebuilds the right retained
buffer graph, repairs all shared graphics resources after a device failure,
or proves original/three-species boids recovery on owner hardware. The
source/application must provide and verify checkpoint data; zero-filled
new GPU resources do not count as restoration.

## RW-1291-PROD14 — fail-closed producer readback cardinality

The producer-facing `accepted_capture_readbacks(flow)` now checks that every
registered required selector has **exactly one** associated capture in the
currently accepted G5 batch. No required result, a duplicate matching result,
or a descriptor pointing at a different submission/affinity or missing G5
readback causes a typed error rather than an empty or apparently complete
success. An empty result remains valid only for a stream with no required
captures. These checks do not replace the producer's semantic pixel/content
verification or create another GPU completion authority. Pure regression
cases cover missing, exact and duplicate capture-point identities.

## RW-1291-DISP23 — per-tick compute dispatch source integrity (draft)

Ordinary historical RenderFlow input snapshots include `dispatch_workgroups` keyed
by every state-driven compute pass in the fixed-step region, in addition to
source uniform bytes and iteration control. Frame preparation rejects absent,
extra or mismatched dispatch identities before G5, never allowing the latest
ECS-projected workgroup dimensions to replace a delayed tick's source value.
Fixed authored dispatches remain invariant. The frame-preparation regression
uses three distinct captured workgroup counts while latest ECS projects a
fourth value; the canonical pass projection binds each source count to the
corresponding actual compute occurrence rather than applying one latest
value across historical ticks. This does not change source clock, physical
RunenGPU admission or semantic completion ownership. This test is not native
Boids consumer acceptance; #1290 still must capture all eight state-driven
compute dispatch values at each producer tick.
