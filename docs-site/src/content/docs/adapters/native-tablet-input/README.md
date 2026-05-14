---
title: Native Tablet Input
description: Current documentation for the native_tablet_input adapter crate.
status: active
owner: adapter
layer: adapter/tool
canonical: true
last_reviewed: 2026-05-14
---

# Native Tablet Input

`native_tablet_input` is the native tablet packet normalization boundary for
Runenwerk drawing workflows.

It maps Windows Pointer/Ink packets, optional Wintab DTOs, and macOS NSEvent
DTOs into platform-neutral `domain/ui/ui_input` pointer events. winit
mouse/touch remains the app/runtime fallback path through the same `ui_input`
vocabulary. The crate is intentionally a normalization boundary only.

## Ownership

This crate belongs to the adapter/tool layer.

It may translate native packet shape, detected capabilities, and packet samples
into `ui_input` contracts, but it must not own drawing semantics, stroke
ratification, brush smoothing, canvas state, package IO, or render/tile
formation.

## Current Contract

The current adapter contract supports:

- Windows Pointer/Ink DTO mapping, including chronological coalesced samples;
- optional Wintab DTO mapping and backend health/probe reporting;
- macOS NSEvent tablet point and proximity DTO mapping;
- tablet platform, vendor, backend, and device tags;
- tool kind mapping for pen, brush, marker, airbrush, eraser, and unknown tools;
- device id preservation;
- pressure, tilt, twist, tangential pressure, hover, eraser, barrel-button,
  calibration, raw timestamp, coalesced sample, and predicted sample capability
  flags;
- missing-capability diagnostics;
- low-latency preview packet classification.

## Known Gaps

- `AutoOsFirst` still needs explicit backend arbitration before production
  hardware use; it should choose one active native stream instead of publishing
  duplicate packets from every available backend.
- Wintab and macOS DTOs must represent contact separately from proximity. Being
  in range or hovering may drive cursor/diagnostic state, but it must not be
  normalized as stroke contact.
- Real hardware acceptance is still required for Windows Ink, Wacom Wintab, and
  macOS tablet devices before the adapter should be treated as production-ready.

## Adapter Boundary

In scope:

- native tablet packet DTOs;
- mapping packet facts into `PointerEvent` and `PointerPacket`;
- preserving supported stylus facts without inventing unsupported values;
- reporting missing tablet capabilities through adapter diagnostics;
- backend health, capability, calibration, and fallback metadata.

Out of scope:

- drawing stroke creation;
- brush dynamics and smoothing policy;
- app shell routing and profile/input-policy UX;
- renderer or GPU resources;
- package persistence.

## Validation

Run:

```text
cargo test -p native_tablet_input
cargo test -p ui_input
```
