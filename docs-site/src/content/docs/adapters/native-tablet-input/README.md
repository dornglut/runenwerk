---
title: Native Tablet Input
description: Current documentation for the native_tablet_input adapter crate.
status: active
owner: adapter
layer: adapter/tool
canonical: true
last_reviewed: 2026-05-10
---

# Native Tablet Input

`native_tablet_input` is the first native tablet packet adapter proof for
Runenwerk drawing workflows.

It maps macOS/Wacom-oriented packet facts into platform-neutral
`domain/ui/ui_input` pointer events. The crate is intentionally a normalization
boundary only. It does not bind to a Wacom SDK or operating-system API yet.

## Ownership

This crate belongs to the adapter/tool layer.

It may translate native packet shape, detected capabilities, and packet samples
into `ui_input` contracts, but it must not own drawing semantics, stroke
ratification, brush smoothing, canvas state, package IO, or render/tile
formation.

## Current Contract

The current adapter proof supports:

- macOS/Wacom-oriented packet metadata;
- tablet platform and vendor tags;
- tool kind mapping for pen, brush, marker, airbrush, eraser, and unknown tools;
- device id preservation;
- pressure, tilt, twist, tangential pressure, hover, eraser, barrel-button,
  calibration, raw timestamp, coalesced sample, and predicted sample capability
  flags;
- missing-capability diagnostics;
- low-latency preview packet classification.

## Adapter Boundary

In scope:

- native tablet packet DTOs;
- mapping packet facts into `PointerEvent` and `PointerPacket`;
- preserving supported stylus facts without inventing unsupported values;
- reporting missing tablet capabilities through adapter diagnostics.

Out of scope:

- real macOS event tap or Wacom SDK integration;
- drawing stroke creation;
- brush dynamics and smoothing policy;
- app shell routing;
- renderer or GPU resources;
- package persistence.

## Validation

Run:

```text
cargo test -p native_tablet_input
cargo test -p ui_input
```
