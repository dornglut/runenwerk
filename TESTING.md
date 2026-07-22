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

GitHub Actions runs the same command at the reviewed commit. That exact-head result is the merge authority for the baseline.

## Documentation build

Documentation changes also run the Astro/Starlight production build through the path-scoped documentation workflow.

## Evidence

Report focused checks, `cargo validate`, exact-head CI, and anything not run. Do not convert source inspection or user-reported output into a stronger validation claim.

Long-form workflow authority lives in [`docs-site/src/content/docs/workspace/engineering-workflow.md`](docs-site/src/content/docs/workspace/engineering-workflow.md).
