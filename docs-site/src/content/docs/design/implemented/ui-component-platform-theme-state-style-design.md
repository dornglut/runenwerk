---
title: UI Component Platform Theme State And Style Design
description: Implemented Runenwerk-local per-control theme-token, visual-state, style-role, fallback, diagnostic, and inspection declaration contract.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ./ui-component-platform-state-binding-host-intent-design.md
  - ./ui-component-platform-input-gesture-device-design.md
  - ./ui-component-platform-catalog-discovery-inspection-design.md
  - ../active/runenwerk-ui-story-driven-golden-workflow-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Component Platform Theme State And Style Design

## Status

Implemented as the current Runenwerk-local control-facing theme/style declaration model.

## Implemented contract

Current `domain/ui/ui_controls/src/theme.rs` plus base-control lowering provide:

- `ControlThemeTokenKind`, token roles, and token requirements;
- `ControlVisualState` and visual-state requirements;
- `ControlStyleRole` and style requirements;
- `ControlStyleFallback`;
- style diagnostics and diagnostic kinds;
- `ControlThemeDescriptor`;
- derived capability and inspection summaries.

Implemented token kinds cover semantic color, spacing, typography, radius, border, opacity, and elevation requirements. Visual-state vocabulary covers normal, hover, pressed, focused, selected, disabled, error, warning, info, active, loading, and read-only. Style roles cover container, label, icon, value, background, foreground, border, accent, focus ring, and overlay.

## Ownership

`ui_controls` owns per-control requirements and read-only projection in the current Runenwerk implementation. Concrete product themes, brand values, user customization, persistence, runtime theme policy, and backend materialization remain outside this contract.

The active Component Platform ownership-realignment work may further normalize generic theme vocabulary ownership; this implemented lifecycle move does not claim that standalone framework extraction/cutover has occurred.

## Boundary rules

- Theme tokens are requirements, not concrete product values.
- Visual states are declarations, not renderer state machines.
- Style roles are semantic requirements, not backend materials.
- Fallbacks/diagnostics remain inspectable facts rather than product resolution policy.
- Catalog/inspection projection is read-only.
- No theme declaration grants runtime mount eligibility.

## Evidence

Current theme source, base-control lowering, `control_theme_contract.rs`, `control_theme_catalog_contract.rs`, and package/catalog integration prove the implemented contract and its no-renderer/no-product-ownership boundary.
