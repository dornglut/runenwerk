---
title: Production Implementation Contract
description: Reusable prompt template for planning a durable implementation contract from a production milestone and WR roadmap item.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-16
related:
  - ../production-track-planning-model.md
  - ../production-tracks.yaml
  - ../roadmap-items.yaml
---

# Production Implementation Contract

Use this template when a production milestone and WR roadmap item need a
durable work package before implementation.

Start with:

```text
task production:plan -- --milestone <PM-ID> --roadmap <WR-ID>
```

Then use the generated prompt to create or update the contract at the reported
path.

## Prompt Shape

```text
Prepare the next production-slice implementation contract.

Production milestone: <PM-ID>
Candidate WR item: <WR-ID>

Use the current production tracks, roadmap items, design docs, and task workflow.
First verify whether the WR item is actually ready to promote or implement.
If it is not ready, write the design/planning work needed instead.

Create or update the durable implementation contract under:
<reported contract path>

The contract must be decision-complete:
- goal and production outcome;
- source-of-truth docs and roadmap rows;
- readiness, gates, blockers, and dependencies;
- owning domains, crates, modules, and expected files;
- explicit non-goals;
- implementation steps;
- public API, data-flow, diagnostics, persistence, or migration impact;
- tests and validation commands;
- stop conditions;
- closeout requirements and roadmap/production evidence updates.

No product code changes.
```

## Contract Sections

Every substantial production implementation contract should include:

- `Goal`
- `Source Of Truth`
- `Readiness`
- `Implementation Scope`
- `Acceptance Criteria`
- `Stop Conditions`
- `Closeout Requirements`

The contract is not a substitute for accepted designs, ADRs, roadmap gates, or
validation. It records exactly how one accepted or promotable WR slice will be
implemented without losing the production milestone context.
