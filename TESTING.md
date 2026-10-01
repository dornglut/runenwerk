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

## Required baseline

Before merge:

```text
cargo validate
git diff --check
```

`cargo validate` is read-only and lockfile-safe. It validates the repository tooling, formats the workspace, runs locked workspace tests, runs strict Clippy, validates documentation, and checks durable repository invariants.

Rust CI resolves and validates the reviewed feature head for pull requests, the exact `github.sha` integration revision for `merge_group`, and `github.sha` for push and dispatch. Each selected revision is explicitly checked out and proved before validation.

Feature-head, merge-group integration, and accepted-main validation are separate evidence stages. When the repository merge queue is enabled, required merge-group checks validate the queue's exact latest-base integration revision before GitHub may merge it. Do not assume SHA equality or inequality between that merge-group revision and the eventual accepted-main commit; accepted-main push evidence independently proves the default-branch state that GitHub accepted.

## Documentation build

Pull requests targeting `main` and merge-group integration revisions always run `Documentation site validation`, which builds the Astro/Starlight production site and validates its publication projection. Main pushes remain path-scoped to changes under `docs-site/**` or `.github/workflows/docs-validation.yml`; root-only documentation changes outside that path scope do not trigger the production build on push. The workflow independently selects and proves the pull-request feature head, merge-group `github.sha`, or push/dispatch `github.sha`; its workflow-definition ref may be a synthetic merge ref, distinct from the checked-out contents.

## Supplemental RunenRender GPU execution proof

Current Runenwerk CI also runs `RunenRender Vulkan execution proofs` for pull-request feature heads, merge-group integration revisions, accepted-main pushes, and manual dispatch. It resolves and proves the selected repository revision, restricts Vulkan loading to the installed Mesa software Vulkan implementation, and runs the maintained native, R6, R7, founding-direct, and static-footprint temporal-quality evidence.

This is Runenwerk-owned consumer and render-integration evidence. It does not make Runenwerk the owner of RunenGPU implementation or standalone framework conformance. RunenGPU implementation and conformance are owned by the external `dornglut/runen-gpu` repository and its repository-owned validation/conformance workflows.

Runenwerk validates its exact-revision RunenGPU dependency, public-API consumption, and product/runtime integration through its own locked baseline and focused integration proofs.

## P100 camera-history correspondence qualification

The private RunenRender GPU behavior tests run the maintained evaluator and camera
reconstruction shaders together. Require an available adapter instead of accepting
the no-adapter skip when collecting GPU evidence:

```text
RUNENRENDER_R7_REQUIRE_GPU=1 cargo +stable test -p engine camera_history_proof --locked -- --nocapture --test-threads=1
```

The native Render Lab camera scenario retains eight stationary temporal submissions,
first motion at ordinal 9, continued motion at 10, and holds the moved pose from
ordinal 11. Ordinals 11–14 form the four stationary reconvergence samples; 15–20
exercise stable output across subsequent phases. Capture ordinals 8–20 separately
for the settled, motion, stop, reconvergence and stable-cycle images. Raw window
startup observations are not temporal submissions.

For a clean, committed source revision, set `RUNENWERK_SOURCE_REVISION` to its full
SHA and `RUNENWERK_CAMERA_HISTORY_DIAGNOSTICS_DIR` to the run's artifact directory:

```text
RUNENWERK_SOURCE_REVISION=<full-clean-source-SHA> RUNENWERK_CAMERA_HISTORY_DIAGNOSTICS_DIR=render-lab/camera-9 cargo +stable run -p runenwerk_render_lab --locked -- --rl2-camera-quality render-lab/camera-9 --window-size-px 1920x1080 --submitted-frames 9
```

`camera-motion-evidence.json` correlates `camera-motion-cells.json` to the exact
first-motion producer frame, phase 0, history age 8 and prior completed same-pose
state 4. The fixed 32-cell sample set records canonical floor lookup and a
phase-aware nearest control under the same correspondence rule. Unknown or
unevaluated semantic values are null; `raw_words` preserves the actual shader
record. The first five probes use the investigated 640×480 cells, scaled to the
requested extent; changing aspect ratio does not preserve their world location.

A separate run with `RUNENWERK_CAMERA_HISTORY_CURRENT_ONLY_FIRST_MOTION=1` supplies
the current-only oracle at that exact temporal ordinal. It requires the explicit
diagnostic request and does not change earlier stationary samples. Compare both
640×480 and 1920×1080 controls. Native artifacts identify the observed backend and
adapter facts; unavailable software/fallback classification remains unknown.

Diagnostic shader work and its bounded follow-up GPU readback add cost. These
qualification runs must not be used as performance measurements for #1107. A SHA
written into an artifact is source provenance, not an independent clean-tree proof;
verify the checked-out revision and clean tree before running qualification.

## Requested-lattice current coverage qualification

The renderer-private coverage proof is included in the existing hosted
`deterministic_execution_r7_proof` filter. The proof explicitly opts the same
sub-native producer into requested-lattice coverage and adds bounded carrier
observations to that GPU fragment and submission. Until an accepted reconstruction
consumer exists, ordinary Render Flow preparation does not request coverage and
therefore authors neither coverage dispatch/resources nor coverage readback.

For clean, committed qualification:

```text
RUNENWERK_SOURCE_REVISION=<full-clean-source-SHA> RUNENRENDER_R7_REQUIRE_GPU=1 cargo +stable test -p engine --lib deterministic_execution_r7_proof_coverage --locked -- --nocapture --test-threads=1
```

The log identifies the observed backend/adapter and records each terminal
requested cell's state and depth bits, requested extent, radiance evaluation
extent, coverage extent, current phase and source generation. Hosted R7 receives
the already proved checkout SHA through `RUNENWERK_SOURCE_REVISION`. The supplied
SHA alone does not prove a clean checkout; verify it before native qualification.

The proofs compare current coverage with the actual maintained P100 primary
query for all four phases, exercise camera/source changes, sampled-field payload
packing and generations, invalid normals and query-budget exhaustion, distinct
producer resources and resize. State values are private `Invalid=0`,
`KnownBackground=1`, and `Hit=2`; depth is finite only for Hit. Row padding is not
requested coverage. The typed runtime record describes prepared work; completion
of its correlated producer submission is required before claiming execution.
This prerequisite preserves sub-native camera-pose history reset and supplies no
radiance fallback or sub-native camera reconstruction policy.

## Evidence

Report focused checks, `cargo validate`, exact-head CI, and anything not run. Do not convert source inspection or user-reported output into a stronger validation claim.

The accepted base, reviewed feature head, merge-group integration revision, queue/squash transition, and accepted-main push result are separate evidence stages or records. Separate stages may reference the same commit object, so each transition is verified explicitly rather than inferred from SHA equality or inequality.

This file owns Runenwerk-local validation semantics. Organization-wide GitHub, review, and validation-evidence rules are owned by [`dornglut/engineering`](https://github.com/dornglut/engineering).
