---
title: UI Accessibility Internationalization And Text Conformance Design
description: Long-term accessibility, internationalization, text shaping, bidirectional text, localization, platform accessibility, and conformance requirements for Runenwerk UI.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-source-projection-and-program-lowering-design.md
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-data-binding-forms-and-effects-design.md
  - ./ui-program-architecture-owner-map.md
---

# UI Accessibility Internationalization And Text Conformance Design

## Status

Active long-term UI design direction. This document defines accessibility,
internationalization, text shaping, localization, bidirectional text, platform
accessibility, and conformance requirements. It does not authorize implementation
by itself.

## Decision

Accessibility, internationalization, and text are framework-level contracts, not
optional rendering polish.

UI source, programs, artifacts, evaluator output, and host integration must carry
semantic text/accessibility facts, not only pixels or draw packets.

## Accessibility Baseline

Runenwerk should use WCAG and ARIA/APG as reference models where applicable, while
mapping to native/game/editor/world-space host constraints.

Required semantic facts:

```text
role
accessible name
accessible description
value
state
property
focus order
navigation hints
keyboard activation
selection semantics
live region semantics
modal semantics
relationship semantics
error/validation semantics
```

Every control package must define accessibility behavior or explicitly report why
it is unsupported in a given host profile.

## Host Accessibility Profiles

Accessibility support varies by host. Each host must report capabilities:

```text
DesktopAccessibilityHost
EditorAccessibilityHost
GameAccessibilityHost
WorldSpaceAccessibilityHost
HeadlessAccessibilityHost
RemotePreviewAccessibilityHost
```

Capability examples:

```text
screen reader bridge
semantic tree export
keyboard navigation
controller navigation
high contrast
reduced motion
text scaling
focus visualization
caption/subtitle support
color-blind safe palette hints
```

Unsupported accessibility features must produce compatibility diagnostics.

## Semantic Tree

The framework must produce an accessibility semantic tree or equivalent facts:

```text
SemanticNodeId
role
name
description
bounds
focusable
focused
selected
expanded/collapsed
checked/unchecked/mixed
value
range metadata
parent/child relationships
labelled-by/described-by relationships
source-map provenance
```

The semantic tree is derived output. It must not become source truth.

## Keyboard And Controller Accessibility

Required behaviors:

```text
keyboard focus traversal
activation by keyboard/controller
focus visible state
modal focus trap
escape/back dismissal
skip/landmark navigation where applicable
active-descendant behavior
roving focus for composite widgets
controller equivalent for pointer-only affordances
```

Controls without non-pointer access must emit diagnostics in accessible host
profiles.

## Localization Model

Text source should contain:

```text
TextKey
FallbackText
Locale
FormatArgs
PluralRules
Gender/selector rules where applicable
Number formatting
Date/time formatting
Unit formatting
SourceMapRef
MissingLocalizationPolicy
```

Localization must be snapshot-based and reportable. Missing keys, missing format
args, invalid plural forms, and fallback usage must be diagnostics.

## Text Shaping And Layout

Text support must cover:

```text
font fallback
script/language tagging
glyph shaping requests
line breaking
hyphenation policy
wrapping
truncation
ellipsis policy
rich text spans
inline icons/glyphs
emoji presentation policy
selection geometry
cursor positioning
text measurement cache keys
```

Text shaping output is derived artifact/evaluator output, not source truth.

## Bidirectional Text

The text system must support bidirectional text and mixed-direction content.

Required concepts:

```text
paragraph direction
inline direction isolation
logical text order
visual order mapping
bidi cursor movement
selection across bidi runs
mirrored punctuation/glyph handling
source diagnostics for unsafe directional controls
```

Bidirectional handling must be defined in `ui_text` and surfaced through reports;
it must not be left as renderer behavior.

## Editable Text And IME

Editable text must support:

```text
text cursor
selection
composition range
IME preedit text
candidate interaction facts where host exposes them
copy/cut/paste
undo/redo
input filters
secure text entry
multiline editing
screen-reader announcements for edits
```

Text editing state belongs to retained UI runtime state, not app model truth,
unless committed through an explicit typed action or binding proposal.

## Color, Motion, And Sensory Accessibility

Theme/style systems must support:

```text
contrast diagnostics
high-contrast variants
reduced-motion policy
focus visible policy
non-color-only state indication
text scaling
minimum target size policy
animation disable/replace policy
flashing/seizure-safe diagnostics
```

## Accessibility Reports

Required reports:

```text
UiAccessibilityValidationReport
UiSemanticTreeReport
UiFocusTraversalReport
UiKeyboardAccessReport
UiControllerAccessReport
UiLocalizationReport
UiTextShapingReport
UiBidiReport
UiContrastReport
UiReducedMotionReport
UiHostAccessibilityCompatibilityReport
```

## Conformance Proofs

Required proof classes:

```text
AccessibleNameProof
KeyboardNavigationProof
ControllerNavigationProof
FocusVisibleProof
ModalFocusTrapProof
ValidationErrorAnnouncementProof
LocalizationFallbackProof
BidiTextProof
TextScalingProof
ReducedMotionProof
ContrastDiagnosticProof
```

## Rejected Shapes

Reject:

```text
accessibility as post-render annotation only
text as raw strings without keys/args/fallback policy
screen-reader semantics only in product code
keyboard/controller support as optional per-button hack
bidi handling delegated entirely to renderer
localized strings without source-map provenance
validation errors only as red text
```
