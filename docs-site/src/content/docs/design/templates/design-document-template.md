---
title: Design Document Template
description: Optional general-purpose feature and subsystem architecture design outline.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-10-29
publication: repository-current
draft: true
pagefind: false
sidebar:
  hidden: true
---

# [Feature or subsystem] — Design

> **Using this template:** This is a flexible outline for a real feature,
> subsystem, or integration design. Replace copied frontmatter and remove
> template-only hidden/draft flags where publication requires it. Replace
> this title and remove this note. Follow the [design lifecycle](../README.md).

## Purpose and current problem

What real requirement, failure, or consumer pressure motivates a design?
Link the owning issue and source/contract evidence.

## Scope, non-goals and owners

What changes, what stays unchanged, and which repository/component is the
accepted semantic owner? Identify downstream consumers and related authority.
This design does not replace an ADR, roadmap, or implementation issue.

## Proposed behavior and architecture

Describe the meaningful responsibilities, data/state flow, public seams,
dependency direction and lifecycle. Use a diagram only if it clarifies the
actual design.

## Invariants and failure behavior

What must always hold? What happens with invalid input, cancellation, loss,
timing/ordering, exhaustion, or recovery where relevant? Name ownership and
negative requirements, not just happy paths.

## Alternatives and trade-offs

Compare realistic approaches, including keeping existing behavior. Explain
why the selected choice is worth its coupling, complexity, compatibility and
maintenance cost. Note unsupported assumptions.

## Integration, compatibility and migration (when relevant)

Which real consumers, data formats, external frameworks or platforms are
affected? Document sequencing, accepted revision pins, rollback, and
conformance obligations when the design needs them.

## Validation and acceptance evidence

What source checks, positive/negative tests, consumer examples, physical
qualification or measurements could falsify the design? State the actual
proof owner and distinguish observed evidence from future requirements.

## Open decisions and limitations

What remains unknown, what would stop implementation, and which owning
issue/ADR must settle any new contract or authority? Do not present an
unaccepted design as implementation authorization.
