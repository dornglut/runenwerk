---
title: WR-171 Design Coverage And Semantic Truth Closure Plan
status: active
wr_id: WR-171
milestone_id: PM-UI-PROGRAM-ARCH-013
---

# WR-171 Design Coverage And Semantic Truth Closure Plan

WR-171 is the final UiProgram architecture truth-closure gate after PM-012. It supersedes durable completion claims until accepted design requirements, current code, semantic verifiers, resolver-backed evidence, truth certificates, contract packs, run ledgers, and repo checkpoint state agree with zero findings, gaps, risks, or drift.

## Scope

Allowed scope is limited to the structured truth/conformance workflow, the PT-UI-PROGRAM-ARCHITECTURE manifest and planning metadata, the design coverage matrix, verifier registry, current truth certificates, and generated production/roadmap reports from validation render commands.

This plan does not authorize MaterialProgram implementation, `foundation/meta` extraction, placeholder folders, broad retained UI rewrites, engine/app/adapter work, or unrelated UI feature behavior.

## Required Outcomes

- `pt-ui-program-architecture.requirements.yaml` maps accepted design requirements to owners, code subjects, tests, evidence kinds, semantic verifier IDs, and deferral authority when applicable.
- `ui_program_architecture_conformance` enforces design coverage and semantic graph/control/package truth, including all nine graph families and ControlPackage obligations.
- Strong truth certificates fail when registry-derived source subjects are omitted from certificate digests.
- `truth:audit` reruns satisfied strong verifiers instead of trusting old certificate files.
- `task track` reports `complete_uncheckpointed` while repo-visible authority, code, evidence, certificate, ledger, or generated-report files are dirty or untracked.

## Validation

- `task workflow:test`
- `cargo test -p ui_program architecture_contract`
- `cargo test -p ui_controls control_package`
- `cargo test -p ui_artifacts artifact_contract`
- `cargo test -p ui_compiler compiler_contract`
- `cargo test -p ui_evaluator evaluator_contract`
- `cargo test -p ui_testing architecture_fixtures`
- `task truth:verify -- --track PT-UI-PROGRAM-ARCHITECTURE --claim ui-program-perfectionist-conformance`
- `task truth:audit -- --track PT-UI-PROGRAM-ARCHITECTURE`
- `task truth:post-completion-audit -- --track PT-UI-PROGRAM-ARCHITECTURE`
- `task production:validate`
- `task roadmap:validate`
- `task docs:validate`
- `task planning:validate`
- `git diff --check`

## Stop Conditions

- Stop if any accepted design requirement lacks code/test/evidence/verifier coverage.
- Stop if any semantic verifier remains only a broad text-shape check where a concrete code/test/evidence binding is required.
- Stop if any strong certificate is stale, omits a registry-derived subject, or has findings, known gaps, risks, or drift.
- Stop if durable completion would be reported before a validated checkpoint state exists.
- Stop if completion requires MaterialProgram, `foundation/meta`, crate creation outside existing authority, or broad product behavior.
