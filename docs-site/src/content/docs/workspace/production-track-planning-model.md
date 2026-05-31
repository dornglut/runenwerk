---
title: Production Track Planning Model
description: Long-term planning model that layers production outcomes above the dependency roadmap.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-16
related:
  - ./production-tracks.yaml
  - ./production-track-index.md
  - ./production-milestone-register.md
  - ./track-execution-manifest.md
  - ./roadmap-items.yaml
  - ./roadmap-decision-register.md
  - ./planning-and-implementation-workflow.md
---

# Production Track Planning Model

## Purpose

Production tracks define coherent long-term outcomes. They answer what complete
capability the project is trying to make real.

The WR roadmap remains the dependency-checked execution graph. It answers which
implementation rows are legal, ready, blocked, completed, or deferred.

This split prevents two failure modes:

- choosing isolated roadmap rows without a production outcome;
- turning a production goal into a shortcut that bypasses architecture,
  ownership, validation, or roadmap gates.

## Core Rule

Production intent never overrides execution legality.

If a capability needs architecture or design decisions, the production milestone
must remain `designing`, `blocked`, or `deferred` until the required design docs
or ADRs are accepted. A milestone may guide design work while it is designing,
but it must not authorize implementation before the WR roadmap and decision
gates allow it.

## Model

`ProductionTrack` is the strategic container:

- stable track id such as `PT-SDF-OW`;
- title, owner, state, strategic goal, and success criteria;
- ordered production milestones.

`ProductionMilestone` is a complete product outcome:

- stable milestone id such as `PM-SDF-OW-001`;
- `kind`: `design`, `implementation`, `hardening`, or `release`;
- `state`: `designing`, `ready_next`, `active`, `completed`, `blocked`, or
  `deferred`;
- dependencies on other production milestones;
- links to WR roadmap rows that govern implementation;
- design gates for required docs, ADRs, or roadmap decisions;
- evidence gates for completed milestone proof;
- acceptance criteria stated as observable production capability.

Production milestones may depend on milestones in the same or another track.
That allows future release, drawing, networking, editor, or platform tracks to
share prerequisites without adding one-off schema fields.

`TrackExecutionManifest` is the full-track execution contract:

- stable track id;
- authority level for the manifest;
- accepted design dependencies;
- milestone sequence;
- owning WR per milestone, or the explicit Track Expansion blocker that must
  create it;
- predecessor dependencies;
- exact write scopes and forbidden scopes;
- required implementation contracts;
- validation commands;
- evidence gates and expected closeout paths;
- next legal action;
- whether each milestone is docs-only, design-only, implementation, hardening,
  or closeout;
- whether each milestone may create code, create crates, or modify production
  behavior.

The manifest is planning authority only. It makes `/goal` execution explicit,
but it does not replace WR roadmap authority, production milestone gates,
implementation contracts, validation, or closeout evidence.

## States

Use these states consistently:

| State | Meaning |
|---|---|
| `designing` | Architecture, ownership, contracts, or acceptance criteria are being resolved. Implementation is not authorized. |
| `ready_next` | Required design gates are satisfied, WR links exist, and the milestone is a candidate for near-term planning. |
| `active` | The milestone is the current production focus. Work still flows through WR roadmap rows or design tasks. |
| `completed` | Evidence gates prove the production outcome, not just isolated tasks. |
| `blocked` | A concrete unresolved blocker prevents progress. |
| `deferred` | Valid long-term work, intentionally not active. |

Design milestones may be `active` while they resolve their own design gates.
Implementation, hardening, and release milestones may not be `ready_next`,
`active`, or `completed` with unmet design gates.

## Gates

Design gates reference docs, ADRs, roadmap rows, or other decision records that
must hold a required frontmatter `status` before implementation states are
legal.

Evidence gates reference closeouts, reports, or docs that prove a completed
milestone. Completed production milestones must include evidence gates so the
track cannot become prose-only status reporting.

Roadmap links connect production milestones to WR rows. They do not duplicate
roadmap dependencies. The WR roadmap remains the source of truth for dependency
order, scoring, blocker levels, current candidates, write scopes, validation,
and completion evidence.

## Workflow

Normal long-term planning follows this order:

1. Review `production-tracks.yaml` to identify the current production milestone.
2. Create or update the Track Execution Manifest for full-track work. The
   manifest records the legal sequence before `/goal` starts coordinating
   milestone execution.
3. If the manifest names missing WR rows, run Track Expansion to create or
   update deferred WR candidates and link milestones to owning WRs. Expansion
   does not authorize implementation.
4. Run a Track Readiness Audit to verify that every milestone has WR ownership
   or an explicit blocker, dependencies, evidence gates, validation commands,
   and expected closeout paths.
5. If the milestone is `designing`, plan or perform the required design work
   first.
6. If the milestone is `ready_next` or `active`, use its WR links to inspect the
   legal roadmap rows.
7. Use active `roadmap-items.yaml` rows and `task batch:kickoff -- --next` for
   execution selection; use the archive and deferred roadmap sources only to
   resolve historical links, dependencies, and evidence gates.
8. After implementation closeout, update roadmap evidence first, then update
   production milestone state only when acceptance and evidence gates are true.

Validation entrypoints:

```sh
task ai:goal -- --track PT-RENDER-PERFECTION --stack
task production:plan-track -- --track PT-SDF-OW
task production:expand-track -- --track PT-SDF-OW
task production:run-track -- --track PT-SDF-OW --allow auto_safe --max-actions 1
task production:run-track -- --track PT-SDF-OW --allow auto_safe --allow agent_design --deny product_code --max-actions 2
task production:run-track -- --track PT-SDF-OW --allow auto_safe --allow agent_design --allow agent_closeout --deny product_code --max-actions 10
task production:run-track -- --track PT-SDF-OW --allow auto_safe --allow agent_design --allow agent_closeout --allow product_code --allow product_implementation --max-actions 10
task production:next -- --track PT-SDF-OW
task production:audit-track -- --track PT-SDF-OW
task production:plan -- --milestone PM-SDF-OW-001 --roadmap WR-019
task production:validate
task production:render
task production:check
task planning:validate
```

Generated production docs are outputs, not sources of truth.

Machine-readable Track Execution Manifest sources live under:

```text
docs-site/src/content/docs/workspace/track-execution-manifests/<track-id>.yaml
```

Human-readable manifest reports live under:

```text
docs-site/src/content/docs/reports/track-execution-manifests/<track-id>/manifest.md
```

The YAML source is the execution authority. The Markdown report is not parsed as
authority.

Use `task production:plan` as the normal bridge from production milestone to WR
implementation contract. The command checks that the WR row is linked by the
milestone, reports readiness, classifies the next action, and prints the prompt
for a durable implementation contract. It is read-only unless
`--write-scaffold` is explicitly passed.

Use `task ai:goal -- --track <PT-ID> --stack` when a target production track is
an end-state audit with cross-track prerequisites. Stack mode keeps the
production tracks as the sequencing layer, resolves prerequisite track order
from milestone dependencies, and selects the first incomplete track for the
normal one-action production loop.
