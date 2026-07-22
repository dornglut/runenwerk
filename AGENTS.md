# AGENTS.md

Runenwerk uses repository evidence, ordinary GitHub issues and pull requests, accepted architecture documents, and permanent CI. Do not create generated prompts, execution locks, truth certificates, temporary authoring workflows, or parallel workflow state.

## Before editing

1. Identify the owner of the behavior or invariant.
2. Inspect the current code, tests, and relevant accepted ADR or design.
3. Check the active issue when the work is already planned.
4. Keep the change to one coherent boundary.

For repository-wide architecture and extraction work, start with:

- [`ARCHITECTURE.md`](ARCHITECTURE.md)
- [`docs-site/src/content/docs/workspace/engineering-workflow.md`](docs-site/src/content/docs/workspace/engineering-workflow.md)
- the relevant accepted ADR or design under `docs-site/src/content/docs`.

## Validation

Use focused package checks while editing. Before merge, run:

```text
cargo validate
git diff --check
```

GitHub Actions runs the same `cargo validate` baseline at the reviewed commit.

## Rules

- Preserve one-way dependency direction.
- Put reusable framework semantics in the owning framework boundary and product integration in Runenwerk.
- Do not add compatibility aliases, forwarding modules, or duplicate source without a real consumer and a removal condition.
- Do not use GitHub Actions to author feature-branch commits.
- Do not claim tests, CI, runtime behavior, or cleanup that was not actually observed.

Report what changed, the owning boundary, validation actually run, and any remaining blocker.
