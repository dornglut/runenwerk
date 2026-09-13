---
title: UI Component Platform Control Kernel Design
description: Implemented Runenwerk-local ControlPackage, ControlKind, schema, kernel, evidence, compatibility, and mount-eligibility contract.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ./ui-component-platform-authoring-kit-design.md
  - ./ui-component-platform-story-proof-envelope-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Component Platform Control Kernel Design

## Status

Implemented. This document describes the current reusable control-package kernel in Runenwerk rather than the former activation branch.

## Canonical implemented vocabulary

The current package model is centered on:

- `ControlPackageDescriptor` and package identity/version metadata;
- `ControlKindDescriptor` and control kind identity;
- `ControlModuleDescriptor`;
- property, state, and event-payload schemas;
- `ControlKernelSet` with layout, interaction, visual, accessibility, and inspection kernel roles;
- route/capability requirements;
- fixture, diagnostic, migration, and story ids/descriptors;
- binding, theme, accessibility, render-evidence, and budget-evidence requirements;
- explicit compatibility and mount-eligibility state;
- package extensions for implemented interaction, overlay, editable-text, generic-text, and Surface2D declarations.

Current source authority is the `ui_controls` package/kernel/schema/validation implementation, especially `domain/ui/ui_controls/src/package/descriptor.rs` and adjacent modules.

## Ownership

`ui_controls` owns reusable control package semantics and validation. Other UI crates retain generic owner vocabulary and execution where separately established. Hosts own product mutation and route decisions; runtime owns live formation; renderer layers own backend execution.

## Implemented law

Every reusable control family is represented through stable package/control identity, typed schemas, the five kernel roles, explicit route/capability and evidence requirements, and conservative mount eligibility. Package presence is not proof and does not authorize runtime behavior by itself.

The package contract is deliberately extensible through owner-aligned descriptors rather than one universal widget/runtime object.

## Intentional landed differences

The proposal-era vocabulary named future surface specializations (`SpatialCanvas`, `NodeCanvas`, `PortGraphCanvas`, `ProgressionTreeView`, `TrackSurface`) alongside kernel concepts. Those future targets are not part of this implemented classification and remain separately lifecycle-owned. Their names do not become implemented merely because the kernel exists.

Later implementation also added concrete descriptor families required by proven slices without changing the kernel's ownership law.

## Boundary rules

- Package/kernel authority does not own OS input, product mutation, renderer backends, or ECS domain semantics.
- Story/evidence requirements are not equivalent to proof outcomes.
- Mount eligibility remains explicit and fail-closed.
- Generic future framework extraction is not implied by this Runenwerk-local implementation.

## Evidence

Current package descriptors, validation, registry/authoring tests, base-control package tests, and later capability tests collectively exercise the implemented kernel contract.
