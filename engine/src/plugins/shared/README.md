# Shared Plugin Utilities

## Purpose

Contains reusable shared plugin helpers used across multiple plugins.

## Usage

Current exported module:

- `reload` utilities for file-watch/reload status checks and formatting.

## Ownership Boundaries

- Owns cross-plugin helper functions/utilities.
- Should avoid feature-specific business logic.

## Extension Points

- Add helper modules that are consumed by multiple plugins.
- Keep utility APIs stable and data-oriented for reuse.
