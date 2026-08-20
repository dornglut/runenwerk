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

## Required baseline

Before merge:

```text
cargo validate
git diff --check
```

`cargo validate` is read-only and lockfile-safe. It validates the repository tooling, formats the workspace, runs locked workspace tests, runs strict Clippy, validates documentation, and checks durable repository invariants.

Rust CI resolves, explicitly checks out, and validates the reviewed feature head for pull requests; push and dispatch use `github.sha`. That exact-head result is the merge authority for the baseline.

## Documentation build

Documentation changes also run the Astro/Starlight production build through the path-scoped documentation workflow. It independently selects and proves the same event-derived repository revision; its workflow-definition ref may be a synthetic merge ref, distinct from the checked-out contents.

## RunenGPU native conformance

RunenGPU implementation and conformance changes additionally run the path-scoped `RunenGPU Native Conformance` workflow. It independently selects and proves the reviewed repository revision, restricts Vulkan loading to the installed Mesa software Vulkan implementation, and executes the ignored public `gpu_g5b_native_runtime` integration proof.

This is supplemental runtime evidence required by the affected RunenGPU slice, not a second Rust baseline and not general hardware certification. Its claims are limited to the exact test and Vulkan software adapter actually executed; `cargo validate` remains the repository-owned merge baseline.

## Evidence

Report focused checks, `cargo validate`, exact-head CI, and anything not run. Do not convert source inspection or user-reported output into a stronger validation claim.

The accepted base, reviewed feature head, synthetic merge result, squash commit, and accepted-main push result are separate evidence objects.

Long-form workflow authority lives in [`docs-site/src/content/docs/workspace/engineering-workflow.md`](docs-site/src/content/docs/workspace/engineering-workflow.md).
