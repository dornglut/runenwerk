---
title: Roadmap Intake WR-050
description: Roadmap intake proposal generated from a new idea.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-22
---

# Roadmap Intake WR-050

Idea: PM-UI-DESIGN-006 Component Surface And Widget Recipe Library
Suggested title: PM-UI-DESIGN-006 Component Surface And Widget Recipe Contracts
Initial planning state: `ready_next`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.
- Architecture governance for PM-UI-DESIGN-006 keeps generic component, widget, and surface recipe contracts in `domain/ui/ui_definition`, token id consumption in `domain/ui/ui_theme`, editor/workbench adapters in `domain/editor/editor_definition`, and app-hosted recipe browser/preview surfaces in `apps/runenwerk_editor`.
- No new ADR is required for the first bounded generic recipe contract row.

## Open Questions

- None for ready-next planning; implementation still requires task production:plan, promotion preflight, focused tests, and closeout evidence before PM-UI-DESIGN-006 can be completed.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
