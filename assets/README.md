# Assets

## Purpose

Stores runtime/editor/game assets used by the project.

## Usage

- Organize assets by domain-specific subfolders.
- Keep stable paths expected by loaders/configs.

## Ownership Boundaries

- Owns asset content and organization.
- Does not own runtime loading logic.

## Extension Points

- Add domain subfolders with local `README.md` where needed.
- Document asset pipeline constraints close to asset roots.
