---
title: Roadmap Intake WR-120
description: Roadmap intake proposal generated from a new idea.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-25
---

# Roadmap Intake WR-120

Idea: Proper UI Designer productization repair: ship a real standalone UI Designer app/workbench with component recipe library/catalog, canvas preview, hierarchy, inspector, bindings/actions, diagnostics, project-backed preview fixtures, direct manipulation workflows, and explicit editor resize/performance evidence. This follows the current code-truth gap where WR-114/PT-UI-LAB/PM-EDITOR-UX closeouts claim a standalone UI Designer, but apps/runenwerk_editor has no runenwerk_ui_designer binary, no app-hosted recipe browser/catalog, and the existing UI Designer path is still a thin self_authoring provider through the full editor shell.
Suggested title: Proper UI Designer productization repair: ship a real standalone UI Designer app/workbench with 
Initial planning state: `blocked_deferred`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Open Questions

- What accepted design, ADR, or closeout evidence justifies promotion?
- Which existing WR items does this depend on?
- Which exact write scopes and validation commands will bound implementation?

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
