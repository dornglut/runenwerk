---
title: Keep AI Out of Foundation
description: Decision to keep LLM clients, prompt logic, agents, vendor SDKs, and AI-specific policy out of foundation crates.
status: accepted
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-27
---

# ADR: Keep AI Out of Foundation

## Status

Accepted

## Context

AI-assisted tooling is useful, but foundation crates define engine vocabulary used by all layers.

## Decision

Do not put LLM clients, prompt logic, AI agents, vendor SDKs, or AI-specific policy into foundation crates.

## Rejected Alternatives

A foundation/ai crate; prompt templates in domain crates; LLM calls from core engine code.

## Consequences

The engine remains portable and vendor-independent. AI tooling must use normal contracts.
