# Project Guidelines

This file is the top-level guidance for all new proposals and implementation plans.

## Design Direction

1. ECS-first by default for runtime state and feature composition.
2. Keep responsibilities clearly separated across plugins/domains.
3. Keep shared/core layers generic and orchestration-focused.
4. Optimize for maintainability and extension without core rewrites.

## System Rules

1. Runtime state should live in ECS resources/components unless there is a strong reason otherwise.
2. Prefer ECS systems + explicit schedule stages/edges over ad-hoc callback chains.
3. Producers write ECS data; consumers read ECS data through typed contracts.
4. Domain-specific data/types belong to the owning plugin/domain, not shared core.
5. Avoid hidden side effects and keep failure handling predictable and observable.
6. Design changes to be incremental and reversible when possible.

## Documentation Rules

1. Each plugin crate or feature area must include a `README.md`.
2. Each `README.md` must include:
   - purpose/scope
   - usage guide
   - key concepts and ownership boundaries
3. Each plugin crate or feature area must include a `requests.md` for proposal-driven feature requests.
4. New proposal gaps must be added to the nearest owning plugin/feature `requests.md`.
5. Completed requests should be marked completed or removed from the open section.
6. Root `requests.md` must include a project-wide index of ongoing requests (with links to owning files).
7. Root `requests.md` should also track cross-cutting requests that span multiple plugins/features.

## Minimal Templates

`README.md` template:

- Purpose
- Usage
- Ownership boundaries
- Extension points

`requests.md` template:

- Status
- Requested date
- Problem
- Proposal
- Acceptance criteria

## Proposal Checklist

- Is ownership clear?
- Is data flow explicit?
- Does this follow ECS-first ownership and data flow?
- Is the design extensible without editing unrelated core modules?
- Are failure paths diagnosable and safe in production?
- Are docs and tests updated with behavior changes?

## Change Process

1. Describe the problem and constraints.
2. Propose the smallest viable change.
3. Define acceptance criteria before implementation.
4. Validate with tests and/or runtime checks.
5. Update documentation to match delivered behavior.
