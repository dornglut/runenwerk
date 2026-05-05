---
title: Diagnostics Current State
description: Current public API and boundaries for the foundation diagnostics crate.
status: active
owner: foundation
layer: foundation
canonical: true
last_reviewed: 2026-05-05
related:
  - ./README.md
  - ./implementation-roadmap.md
  - ../../design/accepted/foundation-diagnostics-design.md
---

# Diagnostics Current State

`foundation/diagnostics` owns structured diagnostic reporting vocabulary.

Diagnostics are observations. They explain issues and preserve structured
context, but they do not decide domain validity, command acceptance, command
execution, rollback, visibility, editor history, or reconciliation.

## Public API

Core entry points from `diagnostics::`:

- `Severity`
- `DiagnosticCode` and `DiagnosticDomain`
- `DiagnosticSubject`, `DiagnosticSubjectKind`, and `DiagnosticSubjectId`
- `DiagnosticLocation`, `DiagnosticTextPosition`, and `DiagnosticTextRange`
- `DiagnosticMessage` and `DiagnosticNote`
- `DiagnosticMetadata`, `DiagnosticMetadataEntry`, and
  `DiagnosticMetadataValue`
- `DiagnosticRelated`
- `Diagnostic`
- `DiagnosticReport` and `DiagnosticSeverityCounts`
- `DiagnosticSink`

## Invariants

- Diagnostic codes are stable machine-readable IDs with at least one separator.
- Diagnostic domains, subject names, and location paths must be non-empty.
- Text positions are one-based.
- Text ranges must move forward.
- Reports preserve diagnostic emission order.
- Severity aggregation is deterministic and does not imply acceptance policy.

## Boundary

Domain crates own their diagnostic code families and validity rules. This crate
does not provide a global diagnostic registry and does not depend on domain,
engine, editor, app, adapter, backend, or AI crates.
