---
title: Roadmap Intake WR-052
description: Roadmap intake proposal generated from a new idea.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-22
---

# Roadmap Intake WR-052

Idea: PM-UI-DESIGN-008 first generic live preview fixture, scenario, target matrix, and evidence descriptor contract slice for domain/ui/ui_definition, consuming PM008 accepted design without app-hosted Preview Lab UI, screenshot capture, renderer golden comparison, provider sessions, runtime replay, persistence activation, or production-readiness hardening.
Suggested title: PM-UI-DESIGN-008 Preview Fixture Scenario And Target Matrix Contracts
Initial planning state: `ready_next`

## Governance Notes

- Architecture governance review was run before intake. The accepted PM-008 design keeps generic fixture, scenario, matrix, and evidence descriptor contracts in domain/ui/ui_definition, target-specific adapters in editor or future game UI domains, and preview orchestration/captures in app/runtime/renderer consumer layers.
- No new ADR is required for the first bounded generic contract slice because ADR-0004, ADR-0005, ADR-0006, and ADR-0012 already govern description-versus-execution, derived projections, provider proposal boundaries, and capability policy.

## Open Questions


## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
