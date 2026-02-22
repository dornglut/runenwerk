# Execution Plan

## Objective
Track the active implementation state for the retained ECS console UI foundation and define immediate next work.

## Completed
- Retained ECS UI pipeline for console flow.
- Console panel with scrollback, input field, and confirm button.
- Unified submit bridge: `UiSubmitEvent -> GameCommandEvent -> command execution`.
- Basic command set and command registry extensions (`help`, `clear`, `echo`, `history`, `count`, aliases/usages).
- Text rendering integration with fallback behavior.
- DPI-aware scaling for high-res displays.
- Input clipping and scrollback viewport slicing.
- Editor-buffer-based input model.
- Caret alignment fixes via glyph metrics.
- Hot-reloadable `.ron` template at `assets/ui/console.ron`.
- True renderer-side text clipping/scissor.
- Tiny-window layout hardening for footer controls.
- Component-tree templates with stable node IDs and keyed patch/update.
- Multiline editor mode: wrapping, up/down navigation, viewport behavior.
- Interactive UI editor mode (M1): `F1` toggle, click-to-select node, mouse drag move, arrow-key nudging, save-to-template.

## Next (Active)
- Selection/copy-paste and richer editing behavior.

## Recommended Breakdown For Next
1. Add explicit text selection model (`anchor`, `caret`, selection range helpers).
2. Add click-to-caret and Shift+Arrow selection expansion for editor text field.
3. Add clipboard integration (copy/cut/paste).
4. Add tests for selection normalization, replacement semantics, and clipboard operations.

## Later
- Template diagnostics and reload feedback tooling.
- UI diff/rebuild and batching performance pass.

## Definition Of Done For Current Phase
- Console path remains deterministic and test-covered.
- Input, rendering, and template hot-reload paths remain stable under resize and DPI changes.
- Regressions in caret, clipping, and submit path are covered by tests.
