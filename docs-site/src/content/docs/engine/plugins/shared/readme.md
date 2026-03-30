---
title: "Shared Plugin Utilities"
description: "Documentation for Shared Plugin Utilities."
---

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

## Guides

- Usage: [../../../docs/reference/plugins/shared/usage-guide.md](../../reference/plugins/shared/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/shared/advanced-guide.md](../../reference/plugins/shared/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/shared/architecture.md](../../reference/plugins/shared/architecture.md)


