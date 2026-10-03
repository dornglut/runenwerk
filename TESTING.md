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

## Test debug information

Repository test builds use Cargo's `line-tables-only` debug information policy. This keeps filename/line information for panic backtraces while reducing test-build debug payload and link cost.

For an interactive debugging session that needs full local test debuginfo, override the repository default explicitly:

```text
CARGO_PROFILE_TEST_DEBUG=full cargo test -p <package>
```

This override is for local debugging; merge-readiness evidence uses the repository default.

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

## Evidence

Report focused checks, `cargo validate`, exact-head CI, and anything not run. Do not convert source inspection or user-reported output into a stronger validation claim.

The accepted base, reviewed feature head, merge-group integration revision, queue/squash transition, and accepted-main push result are separate evidence stages or records. Separate stages may reference the same commit object, so each transition is verified explicitly rather than inferred from SHA equality or inequality.

This file owns Runenwerk-local validation semantics. Organization-wide GitHub, review, and validation-evidence rules are owned by [`dornglut/engineering`](https://github.com/dornglut/engineering).
