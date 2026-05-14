---
title: Native Tablet Input and Latency Contract
description: Active design for OS-native tablet input, optional Wacom backends, diagnostics, and fast-stroke sample density in runenwerk_draw.
status: active
owner: workspace
layer: cross-domain
canonical: true
last_reviewed: 2026-05-13
related_designs:
  - ./drawing-authoring-and-comic-layout-platform-design.md
---

# Native Tablet Input and Latency Contract

## Status

Active implementation contract.

The first implementation slice adds `engine/src/runtime/native_window_hooks.rs`
as a generic window hook registry, expands `adapters/native_tablet_input` into
neutral DTOs plus Windows Pointer, Wintab, and macOS NSEvent adapter modules, and
routes `runenwerk_draw` through native tablet packets before winit mouse/touch
fallback. Hardware validation still needs real Windows Ink, Wacom Wintab, and
macOS Wacom devices.

## Goal

Fast strokes must receive dense, ordered source samples before brush logic sees
them. Brush smoothing is not part of this contract. If a device or backend
supplies sparse samples, the app must report that as diagnostics instead of
hiding it in brush code.

## Ownership

- `engine/src/runtime/native_window_hooks.rs`: generic live-window attachment,
  event observation, per-frame drain, and detach hooks.
- `adapters/native_tablet_input`: native backend DTOs, capability reporting,
  backend health, calibration controls, and mapping into `ui_input`.
- `apps/runenwerk_draw`: native-first input routing, fallback suppression during
  active native stylus contact, and visible device/backend diagnostics.
- `domain/drawing`: consumes only platform-neutral `ui_input::PointerPacket`
  facts and never depends on operating-system or Wacom APIs.

## Backend Policy

Default mode is `AutoOsFirst`:

- Windows uses Win32 Pointer/Ink first because it covers Wacom, Surface, and
  generic pen devices through one OS contract.
- Wintab is optional and dynamically detected. It exists for Wacom-specific data,
  older Wacom setups, or explicit user selection. CI must not require a Wacom
  driver or proprietary SDK install.
- macOS v1 uses public AppKit/NSEvent tablet data. Wacom-specific macOS
  enrichment is limited to documented public mechanisms.
- winit mouse/touch remains the fallback for unsupported devices and normal
  mouse/trackpad input.

## Contract Gaps Before Hardware Acceptance

`AutoOsFirst` is the desired default policy, but implementation still needs
explicit backend arbitration. It must choose the active OS/native stream and
suppress duplicate backend streams; it must not mean that every backend capable
of producing packets is active at once.

Wintab and macOS DTOs must preserve contact separately from proximity.
Proximity or in-range state can drive hover and diagnostics, but contact is the
fact that determines whether a drawing stroke should append samples.

Hardware acceptance remains required before treating the adapter path as
production-ready. The manual passes below are part of the contract, not
optional polish.

## Sample Contract

Backends must preserve:

- device id, backend kind, platform, and vendor;
- source kind and tool kind;
- pressure, tilt, twist, tangential pressure, eraser, barrel buttons, hover, and
  proximity when present;
- timestamps;
- coalesced samples in chronological order before the current sample;
- calibration metadata;
- missing-capability diagnostics when a backend cannot provide a field.

Windows Pointer history APIs return history newest-first. The adapter must
reverse that data before publishing coalesced samples so drawing strokes append
in source order.

## Diagnostics Contract

`runenwerk_draw` must project:

- active backend and active device;
- sample rate and max segment gap;
- pressure and tilt availability;
- dropped and duplicate sample counts;
- unavailable backend warnings;
- backend mode, pressure scale/bias, cursor offset, and calibration reset state.

This panel is runtime state, not drawing domain state.

## Test Coverage

Required automated coverage:

- adapter DTO mapping for Windows Pointer, Wintab, and macOS NSEvent fields;
- Windows Pointer history order: newest-first native rows become chronological
  coalesced samples;
- native packets route before winit fallback in `runenwerk_draw`;
- active native contact suppresses duplicate fallback input;
- native coalesced samples append as ordered stroke samples;
- `AutoOsFirst` backend arbitration chooses one active native stream instead of
  publishing duplicate streams;
- Wintab/macOS proximity and contact facts map separately so hover packets do
  not draw;
- diagnostics project into the app workspace panel.

Required manual hardware coverage:

- one Windows Ink pen device;
- one Windows Wacom tablet with Wintab available;
- one macOS Wacom tablet.

Each manual pass records sample rate, max segment gap, pressure/tilt presence,
warnings, and long fast stroke behavior.

## References

- Microsoft Win32 pointer history APIs:
  `GetPointerFrameInfoHistory`, `GetPointerFramePenInfoHistory`, and related
  pointer history functions.
- Wacom Device Kit, Wintab, and Windows Ink documentation.
- Apple AppKit `NSEvent` tablet point and tablet proximity events.
