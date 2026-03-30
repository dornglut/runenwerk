---
title: "Time Plugin Advanced Guide"
description: "Documentation for Time Plugin Advanced Guide."
---

# Time Plugin Advanced Guide

## Extension Pattern

- Extend behavior in the owning module: engine/src/plugins/time/mod.rs.
- Keep composition changes in plugin build surfaces and avoid moving ownership across domains.
- Keep schedule assumptions aligned with: PreUpdate (CoreSet::Time).

## Integration Notes

- Reuse existing helpers and resources before introducing new abstractions.
- Prefer typed schedule ordering (CoreSet and schedule markers) when adding systems.
- Preserve semantic contracts documented in local README and crate architecture docs.

## Validation Focus

- Verify startup and resource installation behavior in headless tests.
- Verify schedule ordering when adding or reordering systems.
- Verify cross-plugin integration behavior through existing engine integration tests.
