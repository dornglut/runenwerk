---
title: Public API Review Routine
description: Bounded routine for reviewing discoverability and usability of Runenwerk public APIs.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../agents.md
---

# Public API Review Routine

## Purpose

Use this routine when reviewing whether a public crate or module is easy to discover, use, and compose correctly.

## Preconditions

Before reviewing:

1. Identify the public entry points.
2. Inspect `lib.rs`, `README.md`, usage docs, examples, and tests.
3. Identify the common happy path.
4. Identify advanced or uncommon APIs.
5. Identify whether a prelude exists or should exist.

## Routine

1. Review exports from `lib.rs`.
2. Review module names and type names.
3. Review constructors and normal workflow ergonomics.
4. Review error/diagnostic discoverability.
5. Review examples and usage guides.
6. Compare docs with tests.
7. Identify friction that would make correct usage hard on first read.
8. Recommend small API or docs improvements before larger redesigns.

## Findings Format

For each finding:

```text
Severity:
File:
Function/module:
Issue:
Why it matters:
Recommended change:
Validation:
```

## Stop Conditions

Stop and report when:

- changing the public API would require a design decision;
- compatibility impact is unclear;
- docs and implementation disagree and ownership is unclear.
