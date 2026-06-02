---
title: WR-170 Perfectionist Conformance Hardening Closure Plan
status: active
owner: workflow
---

# WR-170 Perfectionist Conformance Hardening Closure Plan

## Scope

WR-170 reopens the final UiProgram architecture completion claim only to harden truth certification and remove ownership drift. It does not implement MaterialProgram, extract `foundation/meta`, create placeholder folders, or expand UI feature behavior.

## Authority

Executable authority lives in `plan.contract.yaml`. This prose plan is explanatory only.

## Work

- Supersede the PM-011 perfectionist claim until PM-012 closes with a fresh zero-finding certificate.
- Replace stale harness validation authority with `task workflow:test`.
- Require structured validation provenance for truth evidence.
- Dispatch semantic UI conformance checks through registered verifier functions.
- Split `ui_artifacts` catch-all helper ownership into responsibility modules.
- Guard workflow tests against catch-all responsibility drift.
- Add run-ledger retention policy so failed stale ledgers do not drive current track state.

## Non-Goals

- No MaterialProgram planning or implementation.
- No `foundation/meta` extraction.
- No placeholder future folders.
- No unrelated UI feature expansion.
- No broad rewrite outside the WR-170 sidecar scope.

## Validation

- `cargo test -p ui_artifacts artifact_contract`
- `cargo test -p ui_program architecture_contract`
- `cargo test -p ui_compiler compiler_contract`
- `cargo test -p ui_evaluator evaluator_contract`
- `cargo test -p ui_testing architecture_fixtures`
- `task workflow:test`
- `task truth:verify -- --track PT-UI-PROGRAM-ARCHITECTURE --claim ui-program-perfectionist-conformance`
- `task truth:audit -- --track PT-UI-PROGRAM-ARCHITECTURE`
- `task truth:post-completion-audit -- --track PT-UI-PROGRAM-ARCHITECTURE`
- `task production:validate`
- `task roadmap:validate`
- `task docs:validate`
- `task planning:validate`
