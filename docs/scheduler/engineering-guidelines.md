# Scheduler Engineering Guidelines

## Scope
Applies to code under `scheduler/`.

## Engineering Rules
- Preserve deterministic ordering behavior.
- Validate graph edits strictly.
- Keep setup/build APIs side-effect free.
- Keep scheduler generic and runtime-safe.

## Error Handling
- Use typed/fallible errors for expected misuse.
- Error messages should identify node/edge names or IDs involved.

## Testing Rules
- Every scheduler behavior change requires tests in `scheduler/tests`.
- New validation path requires a failure test.
- Ordering changes require deterministic ordering tests.

## Performance Rules
- Keep `run()` overhead minimal.
- Avoid unnecessary allocations in per-frame execution.
- Separate graph construction costs from frame execution costs.

## Documentation Rules
- Public API behavior and invariants must stay aligned with `docs/scheduler/design-goals.md`.
