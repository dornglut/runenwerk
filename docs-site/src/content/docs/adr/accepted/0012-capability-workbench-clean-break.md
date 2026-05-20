---
title: Capability Workbench Clean Break
description: Decision to remove legacy Workbench tool-surface compatibility and use typed suite, surface, profile, provider, and capability declarations as the durable authority.
status: accepted
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-05-20
related_designs:
  - ../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
  - ../../design/active/runenwerk-capability-workbench-target-architecture.md
  - ../../design/active/material-lab-and-material-preview-design.md
related_adrs:
  - 0001-use-domain-owned-commands.md
  - 0004-separate-description-from-execution.md
  - 0005-projections-are-derived-state.md
  - 0006-editor-surface-provider-plugin-seam.md
  - 0010-graph-substrate-canvas-boundary.md
preserves_context_from:
  - ../proposed/editor-tool-suite-registry-and-provider-owned-routing.md
---

# ADR: Capability Workbench Clean Break

## Status

Accepted.

This ADR accepts the breaking Workbench identity rule needed by the
Capability Workbench Platform track.

## Context

The editor Workbench started with enum-backed tool-surface identity and then
added stable tool-suite keys. That transition left parallel authorities:
`ToolSurfaceKind`, legacy persisted surface kinds, reverse stable-key mapping
helpers, and profile/provider fallback metadata.

Those compatibility paths make normal Workbench usage harder to reason about.
They also block multi-host composition because full editor, standalone
Material Lab, headless validation, and constrained hosts need the same typed
suite/profile/provider model.

## Decision

Workbench identity must be cleanly registry-owned.

The durable authorities are:

- typed suite declarations;
- stable surface keys;
- typed profile declarations;
- provider family and provider bundle declarations;
- host capability policy declarations.

`ToolSurfaceKind` is not a Workbench identity, persistence, provider request,
profile construction, or Material Lab routing authority.

New workspace persistence is stable-key-only. Old persisted workspace layouts
that depend on legacy surface-kind fields are unsupported and must fail with a
clear unsupported-schema diagnostic. They are not migrated.

External dynamic components remain blocked until a separate sandbox and
security decision is accepted.

## Rejected Alternatives

Keep a compatibility enum. Rejected because it preserves two authorities and
keeps old profile/persistence behavior alive.

Auto-migrate old Workbench layouts. Rejected because the persisted legacy
surface-kind data is exactly the authority being removed.

Delay the break until external plugins exist. Rejected because external
components need a cleaner host policy and composition model, not the old
fallback path.

## Consequences

Old workspace/profile persistence that depends on `ToolSurfaceKind`, legacy
material surface kinds, or V5 legacy fallback metadata will no longer load.

Tests that preserve legacy compatibility should be removed or replaced with
tests proving stable-key-only behavior.

Material Lab must mount in the full editor and standalone Workbench through
typed suite handles, provider bundles, and registry-backed profiles.

Host capability policy becomes the gate for command, product, and resource
capabilities before providers can mutate app or domain state.
