---
title: Use Domain-Owned Commands
description: Decision to keep concrete command families owned by their domains instead of centralizing mutation in one global command enum.
status: accepted
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-27
---

# ADR: Use Domain-Owned Commands

## Status

Accepted

## Context

Important mutations should be modeled as commands, but one global command enum would centralize ownership incorrectly.

## Decision

Use domain-owned command families. Foundation may define descriptor/result vocabulary, not all concrete commands.

## Rejected Alternatives

A universal command enum; direct mutation everywhere; hidden macro-generated behavior.

## Consequences

Mutation boundaries stay close to invariants. AI, scripts, editor tools, tests, and humans can use the same command contracts.
