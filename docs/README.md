# Project Documentation

## Purpose

Contains project-level documentation and documentation indexes.

## Usage

- Start from `docs/index.md` for the active document map.
- Use `docs/current-state.md` for the current runtime/networking status before making architecture assumptions.
- Keep docs aligned with current implemented behavior.
- Use `docs/authoring-layer.md` when changing data-driven asset/config workflows.

## Ownership Boundaries

- Owns project documentation structure/content.
- Does not own implementation behavior itself.

## Extension Points

- Add new docs under `docs/` and link them from `docs/index.md`.
- Keep proposal and architecture docs synchronized with delivered code.
