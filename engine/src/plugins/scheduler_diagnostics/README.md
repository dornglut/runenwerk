# Scheduler Diagnostics Plugin

## Purpose

Emits periodic scheduler/runtime diagnostics logs.

## Usage

- Plugin: `SchedulerDiagnosticsPlugin`
- Typed schedule: `RenderSubmit`
- Legacy scheduler node: `scheduler_diagnostics`
- Runs after: render submit on both runtimes

By default logs every `120` frames.

## Ownership Boundaries

- Owns periodic diagnostics emission policy.
- Consumes runtime state for logging only.
- Does not own scheduler execution or render submission.

The typed runtime variant currently logs `Time` and `WindowState` only. Scene-specific diagnostics remain on the legacy path until scene migration happens.

## Extension Points

- Add additional structured fields to diagnostics logs.
- Make logging interval configurable via resource/config.
