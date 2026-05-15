---
title: Schema
description: Current public API and boundaries for the foundation schema crate.
status: active
owner: foundation
layer: foundation
canonical: true
last_reviewed: 2026-05-05
related:
  - ../../design/implemented/foundation-schema-design.md
---

# Schema

`foundation/schema` owns portable vocabulary for describing typed data shapes,
paths, values, fields, constraints, descriptors, compatibility metadata, and
schema-definition issues.

Schema describes. It does not validate domain data, execute commands, mutate
runtime state, reflect Rust values, register schemas globally, own concrete
domain schemas, or give AI/tools direct mutation access.

## Public API

Core entry points from `schema::`:

- `SchemaId` and `SchemaVersion`
- `SchemaPath` and `SchemaPathSegment`
- `SchemaValue`, `SchemaValueMapEntry`, and `SchemaValueObjectField`
- `SchemaShape`
- `SchemaField`
- `SchemaConstraint`
- `SchemaMetadata`, `SchemaMetadataEntry`, and `SchemaMetadataValue`
- `SchemaDescriptor`
- `SchemaCompatibility`
- `SchemaIssue`, `SchemaIssueCode`, and `SchemaIssueSubject`

With the `diagnostics` feature, constructor and schema-definition issues can be
projected into `foundation/diagnostics` reports.

## Invariants

- Schema IDs are non-empty, whitespace-free, and use stable identifier
  characters.
- Schema versions start at `1`.
- Path field/key/variant segments are non-empty.
- Floating values and numeric constraints must be finite.
- Object fields, map entries, metadata entries, enum options, and object shape
  fields reject duplicate keys where the vocabulary owns that invariant.
- Descriptor and collection helpers preserve caller-provided order.

## Boundary

Owning domains publish concrete descriptors and enforce concrete meaning.
Generic `SchemaValue`-against-`SchemaShape` validation is intentionally not part
of this crate.
