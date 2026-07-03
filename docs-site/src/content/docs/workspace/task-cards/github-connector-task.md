---
title: GitHub Connector Task
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../start-here.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../evidence-quality-taxonomy.md
  - ../complete-merge-readiness-gate.md
---

# GitHub Connector Task

Use this card when work happens through GitHub file access.

Repository: `Crystonix/Runenwerk`

Routine: start at `docs-site/src/content/docs/workspace/start-here.md` and follow the selected routine.

Rules:

- Do not assume a full local checkout.
- Do not assume command execution.
- Inspect files by exact path.
- Patch only files required by scope.
- Verify complete investigation gate evidence when required.
- Verify complete design gate evidence when required.
- Classify connector inspection as connector/file evidence, not command validation.
- Check merge readiness before recommending merge.
- Do not claim command validation unless actual output is available.

Final report:

- files changed;
- exact sections, modules, or functions changed;
- authority files inspected;
- evidence classes used;
- complete investigation gate status;
- complete design gate status;
- merge readiness status when relevant;
- manual validation performed;
- command validation unavailable or actual output;
- remaining risks.
