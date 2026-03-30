---
title: Overview
description: Overview of the engine-agnostic domain layer and its core concepts.
---

# Domain Layer Overview

## Purpose
- The domain layer represents the core logic and concepts of the system.
- It is engine-agnostic and reusable across different applications.
- Defines the rules, abstractions, and behaviors that are central to your system.
- Provides a clear separation between pure domain logic and engine/runtime concerns.

## Key Concepts (Domain Model)
- Entities, components, and systems (for ECS modules)
- Scheduler concepts and deterministic execution patterns
- Spatial math, geometry, and SDF operations
- Invariants and rules that define valid states
- Interactions between domain modules without reference to engine specifics

## Implementation / API (Domain-Level)
- Domain modules map concepts to types, interfaces, or functions
- Must maintain constraints and domain-level guarantees
- Implementation should not rely on rendering, networking, or engine-specific APIs

## Invariants & Rules
- Domain-level rules that must always hold
- Relationships between entities, components, and systems
- Guarantees that enable engine-independent reasoning

## Usage Examples (Domain-Level)
### Example 1: ECS interaction
Describe how entities, components, and systems interact in a pure domain context.
Explain expected outcomes of operations, such as queries or system updates.

### Example 2: Scheduler tick
Illustrate a deterministic scheduling tick with dependency resolution.
Explain how execution order respects domain invariants.

## Design Guidelines
- Follow naming conventions and module boundaries consistently
- Avoid engine, rendering, or networking dependencies
- Keep modules cohesive and focused on a single domain concern

## Integration Notes
- Engine adapters consume domain modules via well-defined interfaces
- Domain modules remain reusable across different engines or runtime contexts
- Links to engine-level docs can be added when available

## Future Considerations
- Potential enhancements to domain abstractions
- Optimizations or extensions to ECS, Scheduler, SDF, or Geometry modules
- Known limitations or constraints for future developers

## References & Links
- Related domain modules: ECS, Scheduler, SDF, Geometry
- External references for DDD, ECS patterns, or spatial math