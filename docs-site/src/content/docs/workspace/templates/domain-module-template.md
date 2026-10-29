---
title: Domain Module Template
description: Template for engine-agnostic domain module documentation.
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

# [Module name] — Domain module

> **Using this template:** Replace this title and all copied frontmatter with the
> owning module's real metadata. Template-only `draft`, `pagefind`, and
> `sidebar.hidden` flags must not accidentally hide the published module page.
> Remove this instruction after copying. Follow the actual domain owner's
> accepted API and dependency boundaries.

## Purpose

What domain concepts and user-visible behavior does the module represent?
Why is a local engine-agnostic domain owner appropriate?

## Concepts and rules

Define the core terms, valid state, and invariants. Distinguish authored
facts from projections, caches, and derived values where relevant.

## Public domain API

Link the real types, functions, inputs and outputs. Explain guaranteed
behavior and the limits of the module, without claiming unaccepted stability.

## Dependencies and non-ownership

What other domain contracts does it consume? What stays with the Engine,
adapters, applications, or standalone framework repositories? Keep runtime
and platform policy in their accepted owners.

## Error behavior and edge cases

Explain invalid inputs, unsupported states, failure reporting and the
important negative or boundary cases.

## Usage example

Provide one small **realistic, domain-level** example with expected behavior.
Omit further examples unless they add distinct value.

## Validation

Link the relevant tests, invariants and real consumers. Identify any required
conformance or integration proof and currently unverified claims.

## Integration and evolution (when relevant)

Describe the actual adapter boundary, current limitations, and justified
compatibility/migration requirements. Avoid speculative feature inventories.

## Related authority

Link the owning ADR, design, source documentation or public framework contract,
rather than restating another repository's semantics.
