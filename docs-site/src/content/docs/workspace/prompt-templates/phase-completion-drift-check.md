---
title: Phase Completion Drift Check Prompt
description: Reusable prompt for end-of-phase completeness, drift, roadmap, and validation checks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../routines/phase-completion-drift-check-routine.md
  - ./README.md
---

# Phase Completion Drift Check Prompt

Use this prompt after a phased implementation completes and before starting the next phase.

```text
You are working in the Runenwerk repository.

Task:
Perform an end-of-phase completeness and drift check for:

Completed phase:
- <phase name and owning design/roadmap>

Current implemented state:
- <brief factual state from code/tests>

Before editing:
- Read AGENTS.md.
- Read AI_GUIDE.md, ARCHITECTURE.md, CRATES.md, DEPENDENCY_RULES.md, DOMAIN_MAP.md, GLOSSARY.md, TESTING.md.
- Read the owning accepted design or roadmap fully.
- Inspect the actual implementation for the completed phase.
- Inspect workspace manifests and relevant feature declarations.
- Inspect docs-site routine and prompt-template indexes.
- Treat current code as source of truth.
- Do not guess. If docs and code disagree, preserve current working code and call out the mismatch.

Strict scope:
- Update documentation, roadmap/status text, routines, and prompt templates only as needed.
- Do not start the next phase.
- Do not implement deferred consumers, integrations, descriptors, commands, registries, reflection, runtime behavior, or unrelated work.
- Do not rewrite unrelated docs opportunistically.

Checks:
1. Phase completeness:
   - Did the completed phase implement exactly what the accepted design promised?
   - Were expected files, modules, features, APIs, tests, and validation commands handled?
2. Phase drift:
   - Did implementation differ from the phase design, scope, names, features, APIs, or validation plan?
   - Preserve working implementation and update stale prose rather than forcing code to match stale docs.
3. Overall roadmap drift:
   - Are design docs, root docs, crate inventory, roadmap status, prompt templates, routine indexes, and next-step recommendations aligned with current repo truth?
4. Deferred-work integrity:
   - Did later-phase work accidentally start early?
   - Do docs imply later-phase work is complete when it is not?
   - Explicitly name the next phase and what must not be done yet.
5. Validation completeness:
   - Were focused tests, feature-matrix tests, docs validation, and the quiet full gate run?
   - If skipped, give the concrete reason.

Required updates:
- Update phase status in the owning design or roadmap doc.
- Update current-state docs if implemented repo state changed.
- Update routine/prompt docs if the workflow learned a reusable rule.
- Explicitly name the next phase.
- Explicitly name the not-yet list.
- Preserve deferred-work boundaries.

Validation:
- Run task docs:validate.
- Run ./quiet_full_gate.sh when code or workspace behavior changed.

Output:
- Findings first, grouped by severity.
- Exact files and sections changed.
- Exact stale statements corrected.
- Validation commands run and results.
- Explicitly state that the next phase and deferred work were not implemented.
```
