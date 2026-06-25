# Testing Strategy

Tests are executable architecture documentation. They protect long-term refactors and assisted changes.

## Principles

- KISS: validate the simplest behavior path that proves the invariant.
- DRY: avoid duplicated test setup and assertion meaning.
- YAGNI: do not add broad harnesses before a real boundary needs them.
- SOLID: test each owner through its public contract.
- Separation of Concerns: keep test tiers distinct.
- Avoid Premature Optimization: do not optimize test infrastructure before it is a bottleneck.
- Law of Demeter: test through direct APIs and contracts.

## Test naming

Prefer behavior names and avoid vague names like `works`, `test_1`, or `surface_test`.

## Test tiers

- Unit tests
- Domain invariant tests
- Ratification tests
- Command behavior tests
- Projection golden tests
- Schema compatibility tests
- Migration tests
- Smoke tests

## Manual validation

When commands cannot be run, validate by inspection and report the limitation.

Manual validation should name authority files read, owning crate/domain/subsystem, dependency direction, public API impact, docs impact, changed files, local tests to run, and unverified risks.

## Optional local commands

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 tools/docs/validate_docs.py
```

These helpers do not define workflow authority.

## Minimum rule

Every important invariant should have at least one test, ratification case, or documented manual validation path.
