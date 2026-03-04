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

The runtime diagnostics currently log `Time` and `WindowState`. Scene-specific diagnostics can be extended from the scene plugin state as needed.

## Extension Points

- Add additional structured fields to diagnostics logs.
- Make logging interval configurable via resource/config.
