---
title: Projections Are Derived State
description: Decision to treat projections as rebuildable derived state unless explicitly documented otherwise.
status: accepted
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-27
---

# ADR: Projections Are Derived State

## Status

Accepted

## Context

Editor and UI architecture uses projections from authoritative state into shell artifacts, mount plans, routes, and DTOs.

## Decision

Treat projections as rebuildable derived state unless explicitly documented otherwise.

## Rejected Alternatives

Letting projections accumulate independent authority; mutating projection output as the source of truth.

## Consequences

Projection bugs are easier to test with golden snapshots. State ownership remains clear.
