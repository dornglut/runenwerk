---
title: PT-RUNENSDF-003 Standalone Transfer Closeout
description: Completion evidence for the standalone RunenSDF repository and corrected source transfer.
status: completed
owner: sdf
layer: reports
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../../design/active/runensdf-extraction-design.md
  - ../../design/active/runensdf-repository-identity-decision.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/specs/pt-runensdf-003-standalone-transfer.ron
  - ./pt-runensdf-002-boundary-correction-closeout.md
---

# PT-RUNENSDF-003 Standalone Transfer Closeout

ID: `PT-RUNENSDF-003`

Completed on: 2026-07-21

Standalone implementation PR: `Crystonix/runen-sdf#1`

Accepted standalone revision:

```text
d52badefc640d6dc6dcdd40268af3aea1bb8eefe
```

## Scope delivered

The phase created and independently proved `Crystonix/runen-sdf`:

- canonical package `runen-sdf` and crate `runen_sdf`;
- one root public package plus only `conformance/downstream` and `xtask` support packages;
- the corrected source from Runenwerk commit `8de096259eab30f8d67672010df9190970d0bfc4`, path `domain/sdf`;
- all nine integration-test modules with import-only package migration;
- a real downstream public consumer and trait-object proof;
- an independent committed lockfile;
- MIT and Apache-2.0 licensing and a security policy;
- framework-owned architecture, numerical, query, ownership, roadmap, status, validation, and provenance documentation;
- one maintained `cargo validate` authority and durable target workflow.

The phase did not change Runenwerk dependencies, workspace membership, lockfile, or `domain/sdf` source authority. Those changes remain `PT-RUNENSDF-004` scope.

## Automated validation

Private-repository GitHub Actions failed before runner allocation and produced no source-command logs. No manual owner validation was substituted.

The complete standalone candidate source, all nine tests, downstream package, lockfile, and validation tooling were mirrored temporarily into public validation PR `Crystonix/runen-ui#16`.

GitHub Actions passed:

```text
run 29845971330
  cargo metadata --format-version 1 --locked --no-deps
  cargo tree -p runen-sdf --locked
  cargo tree -i runen-sdf --workspace --locked
  cargo fmt --all -- --check
  cargo test --workspace --locked
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo doc --workspace --no-deps --locked with RUSTDOCFLAGS=-D warnings
  cargo +1.93.0 test --workspace --locked
  git diff --check

run 29846386222
  cargo validate
```

The maintained-authority run additionally passed repository policy, relative Markdown links, manifest inventory, local-path containment, stale-identity rejection, source-inclusion and submodule rejection, required governance files, provenance, and clean tracked-state proof.

The temporary validation PR was closed without merging any RunenSDF source into RunenUI.

## Parity and ownership proof

The standalone repository contains the accepted PT-RUNENSDF-002 implementation and tests. Transfer-layer differences are limited to repository/package/crate identity, import migration, framework documentation, conformance, licensing, security, and validation integration.

No compatibility package, forwarding namespace, deprecated alias, submodule, private source include, Runenwerk dependency, GPU ownership, renderer ownership, ECS ownership, UI ownership, or persisted-program format was introduced.

## Next safe action

`PT-RUNENSDF-004` may now perform the clean Runenwerk cutover against exact revision `d52badefc640d6dc6dcdd40268af3aea1bb8eefe`.

That phase must repeat the final consumer audit, migrate real consumers if any, remove `domain/sdf` from workspace and lockfile authority, delete the internal source and stale framework documentation, and prove no forwarding or duplicate implementation remains.
