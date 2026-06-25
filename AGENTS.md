# AGENTS.md

Repository instructions for AI coding agents working in Runenwerk.

These rules define how agents should read, modify, and explain repository changes from any interface: local checkout, GitHub connector, ChatGPT context tooling, Codex, or manual file browsing.

## Core Operating Rule

Runenwerk workflows are **repository-readable first**.

Do not assume that the agent can export the full repository, execute scripts, run Taskfile tasks, or regenerate derived files. Scripts and commands may be useful local helpers, but they are never the default workflow authority.

For any non-trivial work, start here:

```text
docs-site/src/content/docs/workspace/start-here.md
```

Use the matching routine under:

```text
docs-site/src/content/docs/workspace/routines/
```

Use reusable task cards under:

```text
docs-site/src/content/docs/workspace/task-cards/
```

## Repository Conventions

### Architecture

- Code is organized by domain, not by technical layer.
- Each domain owns its models, services, rules, and vocabulary.
- Changes must respect domain boundaries and avoid leaking logic across modules.
- Prefer long-term, architecture-correct solutions over local patches.
- Prefer explicit types, clear interfaces, and discoverable public APIs.
- Avoid global mutable state unless it is already an intentional repository pattern.
- Follow nearby module and crate conventions before introducing new abstractions.

### Module structure

Follow `docs-site/src/content/docs/guidelines/module-structure-guidelines.md`.

Key expectations:

- Organize modules by subdomain responsibility, not technical layers.
- Prefer subdomain folders with `mod.rs` boundaries for larger subsystems.
- Use explicit module names that describe responsibility.
- Avoid `include!` module composition.
- Avoid `_internal` module suffixes.
- Avoid catch-all files such as `utils.rs`, `helpers.rs`, or `misc.rs`.

When adding new code:

1. Choose the owning domain: `foundation`, `domain`, `engine`, `net`, `apps`, or `adapters`.
2. Choose the owning crate.
3. Choose the owning subsystem inside that crate.
4. Add the file or module there.

Foundation is for cross-domain primitives. Domain is for engine-agnostic reusable logic. Engine is glue that composes runtime behavior around domain and foundation contracts.

## Code Discovery

Before editing:

- Search or inspect for existing implementations, helpers, and patterns.
- Prefer reuse over duplication.
- Follow patterns from nearby modules.
- If a helper already solves the problem, use it instead of introducing a new abstraction.
- For public APIs, inspect `lib.rs`, `prelude.rs`, README files, examples, tests, and docs together.

## Documentation Structure

Documentation lives under:

```text
docs-site/src/content/docs
```

For placement, lifecycle states, frontmatter, naming rules, and archival policy, follow:

```text
docs-site/src/content/docs/workspace/documentation-structure.md
```

Root Markdown files are short entrypoints only. They must not become full workflow manuals, generated status views, design documents, roadmaps, or historical reports.

## Scriptless Workflow

Default workflow shape:

```text
Read authority files -> inspect working files -> patch exact files -> manually validate evidence -> report changed files and unresolved gaps
```

Do not make command output the only way to understand a task. If a script is unavailable, continue by reading the authority files listed by the relevant routine and report that validation was manual or not run.

Optional local commands may be mentioned only as helpers. They do not replace repository inspection, architecture doctrine, dependency rules, accepted designs, ADRs, tests, manual review, or closeout evidence.

## Planning and Authority

Use these workspace docs:

- `docs-site/src/content/docs/workspace/start-here.md` for task routing.
- `docs-site/src/content/docs/workspace/operating-model.md` for the scriptless workflow model.
- `docs-site/src/content/docs/workspace/authority-model.md` for authority conflicts.
- `docs-site/src/content/docs/workspace/planning/README.md` for planning records.
- `docs-site/src/content/docs/workspace/routines/README.md` for executable human/agent routines.
- `docs-site/src/content/docs/workspace/task-cards/README.md` for copy-paste task cards.

## Documentation Changes

When creating or editing docs:

- Keep docs-site filenames in kebab-case.
- Use `README.md` for docs-site section landing pages.
- Do not introduce new docs-site `readme.md` files.
- Update internal links when files move or are renamed.
- Prefer moving obsolete process material to archive or migration reports over keeping active duplicate authority.
- Do not require generated files or scripts to understand the current workflow.

## Public API, Usage Ergonomics, and Examples

Ease of use is part of implementation quality.

For public-facing crates and modules:

- Prefer APIs that are easy to discover, combine, and use correctly on the first read.
- Optimize for the common happy path.
- Keep advanced or domain-specific APIs in owning modules unless they are commonly needed.
- Treat awkward discoverability as a real defect.
- Keep examples aligned with the preferred public API.

## Benchmark and Artifact Conventions

When working on benchmarks, profiles, and reports:

- Keep executable benchmark runners in conventional code locations such as `benches/` or `examples/`.
- Keep raw outputs in dedicated artifact folders.
- Keep human-readable reports in docs benchmark or report folders.
- Do not mix raw benchmark output and prose reports casually.

## General Behavior

- Act as a senior software engineer working directly inside the codebase.
- Work autonomously: inspect context, infer conventions, implement, verify, and explain outcomes without unnecessary back-and-forth.
- Default to delivering working changes rather than only analysis or plans.
- Make reasonable assumptions when details are missing unless a real blocker exists.
- Be concise but technically precise.
- Treat usability, discoverability, and documentation quality as part of implementation quality.

Agents must always include:

- the file path;
- the exact function, method, module, routine, or document section changed or recommended;
- validation performed, or a clear statement that command validation was unavailable.

## Implementation Standards

- Address root causes rather than surface symptoms.
- Avoid speculative refactors unless they are required for correctness.
- Avoid duplicated logic.
- Do not introduce silent failures or success-shaped error handling.
- Surface errors clearly in a way consistent with existing patterns.
- Add concise comments only where logic would otherwise be difficult to understand.
- Preserve intended behavior unless the task explicitly requires changing it.

<!-- BEGIN RUNENWERK:UI_COMPONENT_PLATFORM:root-note -->
## UI Component Platform note

The current UI Component Platform activation is `PT-UI-COMPONENT-PLATFORM`, starting after `PM-UI-STORY-004`. It defines reusable, story-proven `ControlPackage` and surface kernels before product-specific Gallery, Workbench, Designer, game HUD, or world-space UI behavior. See `docs-site/src/content/docs/design/active/runenwerk-ui-platform-capability-roadmap.md` and the `ui-component-platform-*-design.md` active design docs.
<!-- END RUNENWERK:UI_COMPONENT_PLATFORM:root-note -->
