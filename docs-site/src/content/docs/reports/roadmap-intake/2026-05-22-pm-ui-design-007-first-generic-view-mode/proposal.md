---
title: Roadmap Intake WR-051
description: Roadmap intake proposal generated from a new idea.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-22
---

# Roadmap Intake WR-051

Idea: PM-UI-DESIGN-007 first generic view-model capability and intent binding contract slice for domain/ui/ui_definition, consuming PM007 accepted design without app-hosted Designer UI, concrete command execution, game-runtime loading, preview matrices, persistence activation, or production-readiness hardening.
Suggested title: PM-UI-DESIGN-007 View-Model Capability And Intent Binding Contracts
Initial planning state: `ready_next`

## Governance Notes

- Architecture governance review was run before intake. The accepted PM-007 design keeps generic binding and intent contracts in domain/ui/ui_definition, concrete editor/workbench adapters in domain/editor/editor_definition, future game adapters in game-owned UI domains, and app/runtime layers as consumers.
- No new ADR is required for the first bounded generic contract slice because ADR-0001, ADR-0004, ADR-0005, ADR-0006, and ADR-0012 already govern domain-owned commands, description-versus-execution, derived projections, provider proposals, and capability policy.

## Open Questions


## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
