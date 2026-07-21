# Testing and Validation

Tests are executable architecture evidence. They should prove behavior and ownership through the smallest appropriate public contract.

## Test principles

- Name tests after behavior or invariants, not implementation steps.
- Keep unit, domain, integration, conformance, migration, and smoke tests distinct.
- Reuse setup without hiding assertion meaning.
- Put tests with the owner of the invariant.
- Do not enforce repository workflow policy in application or domain tests.
- Do not create broad harnesses before a real boundary needs them.

## Validation profiles

Runenwerk uses three profiles:

```text
focused
baseline
extended
```

It does not use quick, full, or quiet gates. Output verbosity does not change validation semantics.

### Focused

Use the smallest checks that exercise the changed owner while implementing.

Examples:

```text
cargo test -p <package>
cargo test -p <package> <behavior>
cargo clippy -p <package> --all-targets -- -D warnings
python tools/docs/validate_docs.py
```

Focused checks are iteration tools. They do not replace the baseline before merge.

### Baseline

Run before merge:

```text
cargo validate
git diff --check
```

`cargo validate` is the canonical, read-only, lockfile-safe implementation shared by local development and GitHub Actions. It runs:

```text
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python tools/docs/validate_docs.py
deterministic repository audit
```

The baseline must not modify source, manifests, lockfiles, generated documents, or formatting.

GitHub Actions must run exactly `cargo validate`. CI at the reviewed commit is the merge authority for the baseline.

### Extended

Use for releases, scheduled validation, dependency-policy work, or changes that explicitly require broader evidence:

```text
cargo xtask validate --extended
```

The extended profile may require:

```text
cargo-deny
cargo-machete
lychee
ast-grep
pnpm and the docs-site toolchain
```

Extended checks are not ordinary PR blockers unless an accepted policy promotes a specific check into the baseline.

## Test tiers

- **Unit tests:** local algorithms and value contracts.
- **Domain invariant tests:** rules owned by one domain.
- **Integration tests:** contracts between named owners.
- **Conformance tests:** reusable or extracted implementations against a stable contract.
- **Migration tests:** compatibility and persisted-data transitions.
- **Smoke tests:** bounded startup, host, rendering, or product paths.
- **Benchmarks:** performance evidence; never a substitute for correctness tests.

## Architecture guards

Prefer typed APIs, dependency checks, and behavior tests over source-string scanning.

A repository audit may enforce durable repository invariants such as required authority files, dependency direction, or a canonical CI command. Product tests must not own branch policy, workflow names, prompt formats, roadmap states, or merge procedure.

When a source guard is temporarily necessary, it must name:

```text
owning subsystem
forbidden bypass
reason a typed or behavioral proof is not yet available
removal or replacement condition
```

## Evidence reporting

Report validation precisely:

```text
focused commands run and results
cargo validate result
CI result and reviewed commit
manual inspection performed
checks unavailable or not run
remaining risk
```

User-reported command output is valid evidence when identified as user-reported. It is not equivalent to connector-observed CI.

## Command-unavailable work

When commands cannot be run:

- inspect the owning code, tests, public exports, dependency direction, and documentation;
- report command validation as unavailable;
- do not claim the branch is merge-ready when the baseline or required CI remains unknown.

## Minimum rule

Every important invariant requires at least one executable test, conformance case, deterministic repository check, or explicitly documented manual validation path.
