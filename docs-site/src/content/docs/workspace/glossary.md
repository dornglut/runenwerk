---
title: Glossary
description: Shared Runenwerk architecture and authoring vocabulary.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-22
related_docs:
  - ./start-here.md
  - ../architecture/repository-family-architecture.md
---

# Glossary

## Foundation

Low-level reusable vocabulary and contracts. Foundation crates do not depend on domain, engine, app, adapter, or workflow code.

## Domain

A crate or subsystem that owns specific concepts and their invariants.

## Runtime

The executable composition layer that schedules, executes, and adapts domain descriptions into running behavior.

## Adapter

A boundary that translates between owners, frameworks, hosts, or backends without duplicating their algorithms or authority.

## Surface definition

A registered kind of editor or UI surface that can be mounted.

## Surface instance

A concrete mounted occurrence of a surface definition.

## Surface host

A structural location capable of holding a surface instance.

## Widget

A UI-level element produced by projection or rendering. A widget is not automatically a domain surface.

## Projection

A deterministic transformation from authoritative state into a read model, composition artifact, mount plan, route map, or UI-facing structure.

## Ratification

A domain-owned check that generated, imported, projected, migrated, or externally modified state satisfies the domain's invariants.

## Command

An explicit request to mutate domain state through a validated boundary.

## Transaction

A grouped set of related commands treated as one authored operation.

## Diagnostic code

A stable machine-readable identifier such as `ui.surface.mount.unknown_host`.

## Typed ID

A domain-specific wrapper around an identity value.

## Derived state

State produced from authoritative state. It should be rebuildable and must not silently become a second source of truth.
