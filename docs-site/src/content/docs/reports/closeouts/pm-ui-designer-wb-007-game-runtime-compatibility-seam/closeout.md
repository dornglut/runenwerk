---
title: PM-UI-DESIGNER-WB-007 Game Runtime Compatibility Seam Closeout
description: Runtime-proven evidence-acceptance closeout for the UI Designer Workbench game.runtime compatibility seam without implementing game HUD runtime behavior.
status: completed
owner: editor
layer: domain/ui-definition / domain/editor
canonical: false
last_reviewed: 2026-05-26
related_designs:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
  - ../../../design/active/game-runtime-ui-projection-and-hud-platform-design.md
related_reports:
  - ../pm-ui-designer-wb-006-scenario-evidence-and-performance-baselines/closeout.md
  - ../pm-editor-ux-008-game-ui-readiness-seam/closeout.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-archive.yaml
---

# PM-UI-DESIGNER-WB-007 Game Runtime Compatibility Seam Closeout

## Summary

`PM-UI-DESIGNER-WB-007` completed the UI Designer Workbench game-runtime
compatibility seam by accepting and revalidating the completed `WR-118` /
`PM-EDITOR-UX-008` evidence. No new product code was required for this
milestone because the bounded compatibility seam already lives in the generic
`ui_definition` validators and the editor-owned UX Lab evidence adapter.

This closeout proves descriptor and evidence compatibility for
`game.runtime`. It does not implement game HUD runtime behavior, game-runtime
UI projection execution, SDF HUD rendering, or a native runtime screenshot.
Those remain owned by the separate game-runtime UI production track.

## Accepted Evidence

The accepted seam evidence is runtime-proven by the existing implementation:

- `domain/ui/ui_definition/src/preview_fixture/mod.rs` module
  `preview_fixture`: validates `game.runtime` preview matrix compatibility
  axes for safe area, input modality, platform prompt, localization,
  accessibility, size, performance/readability, and view-model freshness.
- `domain/ui/ui_definition/src/production_readiness/mod.rs` module
  `production_readiness`: validates readiness packets and inspection reports
  against the required `game.runtime` compatibility axes.
- `domain/ui/ui_definition/src/view_binding/mod.rs` module `view_binding`:
  rejects missing or stale runtime view-model packages and rejects editor
  command descriptors for `game.runtime`.
- `domain/ui/ui_definition/src/component_recipe/mod.rs` module
  `component_recipe`: proves recipe expansion preserves accessibility, layout,
  and focus contracts for `game.runtime`.
- `domain/ui/ui_definition/src/persistence_activation/mod.rs` module
  `persistence_activation`: proves runtime activation still requires migration
  reports, deterministic diffs, and activation diffs.
- `domain/ui/ui_definition/src/visual_layout/apply.rs` module
  `visual_layout::apply`: proves visual-layout edits are target-profile gated
  for `game.runtime`.
- `domain/editor/editor_shell/src/ux_lab/game_ui_readiness.rs` module
  `ux_lab::game_ui_readiness`: exposes editor-owned compatibility evidence
  while avoiding editor runtime vocabulary in the game-runtime contract terms.

## Governance Review

Architecture governance was rerun for this evidence-acceptance action:

```text
task ai:architecture-governance -- --task "PM-UI-DESIGNER-WB-007 game runtime compatibility seam evidence acceptance" --scope "docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md; docs-site/src/content/docs/design/active/game-runtime-ui-projection-and-hud-platform-design.md; docs-site/src/content/docs/reports/closeouts/pm-editor-ux-008-game-ui-readiness-seam/closeout.md; domain/ui/ui_definition; domain/editor/editor_shell/src/ux_lab"
```

The DDD owner for generic target-profile, descriptor, recipe, binding,
preview, readiness, persistence, and visual-layout truth remains
`domain/ui/ui_definition`. The editor owner for workbench evidence projection
is `domain/editor/editor_shell/src/ux_lab`. The translation boundary is the
read-only compatibility evidence report; `game.runtime` consumers must not
depend on editor shell providers, Workbench host policy, editor command routes,
or editor provider vocabulary.

No ADR or design update is required for this milestone because PM007 accepts
existing evidence without adding a game-runtime UI owner crate, changing
dependency direction, or implementing runtime HUD behavior. A future ADR or
accepted design update remains required before adding a concrete game-runtime
UI owner or changing the owner boundary.

## Validation Results

Focused validation run on 2026-05-26:

```text
cargo test -p ui_definition game -- --nocapture passed.
cargo test -p editor_shell game -- --nocapture passed.
```

The `ui_definition` suite executes nine focused tests for `game.runtime`
preview axis coverage, readiness evidence, editor-command rejection,
stale/missing view-model packets, persistence activation, recipe expansion,
and visual layout target-profile gating. The `editor_shell` suite executes two
focused UX Lab tests proving the compatibility report covers all required axes
and avoids editor runtime vocabulary in runtime contract terms.

Final metadata validation completed with this closeout and completed
production milestone state:

```text
task docs:validate passed.
task roadmap:render passed.
task roadmap:validate passed.
task roadmap:check passed.
task production:render passed.
task production:validate passed.
task production:check passed.
task planning:validate passed.
task puml:validate passed.
git diff --check passed.
```

`./quiet_full_gate.sh` is intentionally not part of this closeout because this
is a bounded compatibility seam evidence-acceptance slice with no product code
changes.

## Completion Quality

Completion quality is `runtime_proven` for the bounded compatibility seam. The
proof is executable and fail-closed for the PM007 risks: incomplete
compatibility axes, editor-only evidence, stale or missing view-model packets,
editor command descriptors in `game.runtime`, missing readiness reports,
recipe contract drift, persistence activation drift, and unsupported
target-profile layout edits.

Known quality gaps remain by design:

- `PM-UI-DESIGNER-WB-008` still owns final runtime-proven track closeout, usage
  docs, examples, and handoff notes.
- `PT-GAME-RUNTIME-UI` still owns concrete game HUD runtime behavior,
  game-runtime UI projection execution, and SDF HUD rendering proof.
- PM007 does not claim native runtime-window screenshot evidence, in-frame HUD
  rendering evidence, packaged release readiness, or perfectionist no-gap
  quality.

## Drift Check

The accepted evidence satisfies the PM007 acceptance criteria:

- `game.runtime` descriptor and evidence checks remain in `domain/ui` and do
  not import editor shell, Workbench host policy, editor command routes, or
  editor provider vocabulary.
- The editor UX Lab evidence adapter remains editor-owned derived evidence and
  does not become game-runtime source truth.
- The seam proves descriptor and evidence compatibility only; no game HUD
  runtime behavior is implemented or claimed.
