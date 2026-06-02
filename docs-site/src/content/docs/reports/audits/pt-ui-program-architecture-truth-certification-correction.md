---
title: PT-UI-PROGRAM-ARCHITECTURE Truth Certification Correction
description: Supersedes the overbroad UiProgram architecture completion claim with code-truth certification requirements.
status: completed
owner: ui
last_reviewed: 2026-06-01
---

# PT-UI-PROGRAM-ARCHITECTURE Truth Certification Correction

`PT-UI-PROGRAM-ARCHITECTURE` is no longer treated as completed architecture truth merely because production milestones, closeouts, evidence records, modules, and symbols exist.

The historical closeouts remain historical records. They are superseded by the current truth-certification requirement:

- `ui-program-architecture-implementation` is blocked until `ui_program_architecture_conformance` passes with zero findings.
- `ui-program-perfectionist-conformance` is blocked until the same verifier reports zero findings, zero known gaps, zero known risks, and zero drift.
- MaterialProgram proof planning and `foundation/meta` extraction remain blocked.

## Current Code Truth

The current codebase exposes early UiProgram owner boundaries, but the implementation is not the final accepted architecture:

- graph families are still placeholder/string-bag contracts rather than typed semantic graphs;
- compiler lowering is still scaffold/pass-through behavior;
- evaluator behavior is still scaffold/pass-through behavior;
- final owner-map closure is incomplete for `ui_controls`, `ui_accessibility`, and `ui_geometry`;
- existing evidence records do not independently prove final architecture semantics.

## Correct Completion Rule

The architecture track is complete only when:

- concrete code satisfies the accepted design;
- semantic tests and fixtures prove compiler, artifact, evaluator, host, diagnostics, source-map, migration, and visual/render behavior;
- evidence records point to real code, tests, fixtures, diagnostics, source maps, artifacts, and migration outputs;
- generated reports and closeouts match code truth;
- the independent truth certificate has no findings, gaps, risks, or drift.

Until then, completed milestone metadata is bounded historical evidence, not final architecture truth.
