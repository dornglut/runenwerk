---
title: WR-031 Workbench Clean-Break Governance And ADR Closeout
description: Completed governance closeout for accepting the Capability Workbench clean break before compatibility-removal implementation.
status: completed
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-20
related_designs:
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
  - ../../../design/active/runenwerk-capability-workbench-target-architecture.md
related_adrs:
  - ../../../adr/superseded/0012-capability-workbench-clean-break.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
related_reports:
  - ../../implementation-plans/wr-031-workbench-clean-break-governance-and-adr/plan.md
  - ../../implementation-plans/wr-032-typed-suite-surface-profile-and-provider-handles/plan.md
  - ../../implementation-plans/wr-033-remove-legacy-tool-surface-identity/plan.md
  - ../../implementation-plans/wr-034-registry-backed-workspace-profiles/plan.md
  - ../../implementation-plans/wr-035-clean-persistence-format/plan.md
  - ../../implementation-plans/wr-036-material-lab-clean-migration-proof/plan.md
---

# WR-031 Workbench Clean-Break Governance And ADR Closeout

## Status

Completed as governance evidence on 2026-05-20.

WR-031 accepts the Workbench clean-break decision and records the roadmap and
production sequencing needed before compatibility-removal code starts. It does
not implement typed handles, remove `ToolSurfaceKind`, change workspace
profiles, change persistence, or prove Material Lab routing.

## Completion Evidence

- `docs-site/src/content/docs/adr/superseded/0012-capability-workbench-clean-break.md`
  is accepted and states that `ToolSurfaceKind` is not Workbench identity,
  persistence, provider request, profile construction, or Material Lab routing
  authority.
- `docs-site/src/content/docs/design/active/editor-tool-suite-registry-and-workbench-host-design.md`
  is active and binds the Workbench host/tool-suite design to ADR 0012.
- `docs-site/src/content/docs/workspace/production-tracks.yaml` records
  `PT-WB-CAP` and keeps `PM-WB-CAP-001` active for the implementation slices
  that follow WR-031.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` sequences
  `WR-032` through `WR-040` after the clean-break governance row.
- `docs-site/src/content/docs/reports/implementation-plans/wr-031-workbench-clean-break-governance-and-adr/plan.md`
  records the architecture governance review and closeout requirements.
- `docs-site/src/content/docs/reports/implementation-plans/wr-032-typed-suite-surface-profile-and-provider-handles/plan.md`,
  `docs-site/src/content/docs/reports/implementation-plans/wr-033-remove-legacy-tool-surface-identity/plan.md`,
  `docs-site/src/content/docs/reports/implementation-plans/wr-034-registry-backed-workspace-profiles/plan.md`,
  `docs-site/src/content/docs/reports/implementation-plans/wr-035-clean-persistence-format/plan.md`,
  and
  `docs-site/src/content/docs/reports/implementation-plans/wr-036-material-lab-clean-migration-proof/plan.md`
  record downstream readiness contracts and blockers.

## Architecture Governance

The bounded context owner is `domain/editor/editor_shell` for reusable
Workbench contracts, with `apps/runenwerk_editor` owning concrete suite,
provider, and host composition. Governance records belong in `docs-site`.

Dependency direction remains valid:

- domain Workbench contracts do not depend on app composition;
- app composition consumes domain contracts;
- governance docs record durable decisions without creating runtime authority.

ADR need is satisfied by ADR 0012. No additional ADR is required unless a later
slice changes ownership, restores compatibility migration, or moves
Workbench/Material Lab authority across domain/app boundaries.

The accepted tradeoff is explicit: old workspace compatibility is dropped so
the platform can converge on typed suite/profile/provider declarations and
multi-host composition. This is classified as `bounded_contract` completion:
the governance contract is complete, while product behavior remains downstream.

## Validation

Contract-writing validation completed before this closeout:

- `task docs:validate` passed.
- `task planning:validate` passed.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` passed and still
  reported PM-WB-CAP-001 as active with bounded WR work remaining.

Closeout validation required after this file and roadmap evidence are recorded:

- `task roadmap:render`
- `task planning:validate`
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred`

No cargo validation is required for WR-031 because this closeout changes only
governance documentation and roadmap evidence.

## Remaining Work

- WR-032 owns typed suite, surface, profile, and provider handles.
- WR-033 owns legacy Workbench identity removal.
- WR-034 owns registry-backed workspace profiles.
- WR-035 owns stable-key-only persistence and unsupported old-schema
  diagnostics.
- WR-036 owns the full-editor and standalone Material Lab clean migration
  proof.
- PM-WB-CAP-001 remains active until those downstream rows are implemented,
  validated, closed out, and reflected in roadmap/production evidence.

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- No Rust product code changed in WR-031.
- Legacy Workbench compatibility paths remain until WR-032 through WR-036 land
  their own implementation closeouts.
- PM-WB-CAP-001 remains incomplete after WR-031 because this row is only the
  governance gate.

Perfectionist status is intentionally not claimed. The completed scope is the
accepted governance decision and sequencing record, not the clean Workbench
runtime behavior.
