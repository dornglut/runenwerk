---
title: Separate Description From Execution
description: Decision to separate serializable, inspectable descriptions from backend-aware execution/runtime objects.
status: accepted
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-27
---

# ADR: Separate Description From Execution

## Status

Accepted

## Context

Many systems need both editable descriptions and optimized runtime execution objects.

## Decision

Keep serializable/inspectable descriptions separate from backend-aware execution/runtime objects.

## Rejected Alternatives

Editing runtime objects directly; persisting execution objects; mixing backend resources into domain descriptions.

## Consequences

Descriptions become diffable, ratifiable, testable, and AI-editable. Execution remains optimized.
