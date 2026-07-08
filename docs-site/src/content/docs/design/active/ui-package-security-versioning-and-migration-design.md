---
title: UI Package Security Versioning And Migration Design
description: Long-term package trust, capability security, schema/version compatibility, migration, sandboxing, provenance, and governance requirements for Runenwerk UI.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-source-projection-and-program-lowering-design.md
  - ./domain-authoring-source-and-program-pattern.md
  - ./ui-testing-conformance-and-proof-matrix-design.md
  - ./ui-program-architecture-owner-map.md
---

# UI Package Security Versioning And Migration Design

## Status

Active long-term UI design direction. This document defines package trust,
capability security, schema/version compatibility, migration, sandboxing,
provenance, and governance requirements. It does not authorize implementation by
itself.

## Decision

UI package growth must be explicit, versioned, capability-checked, migratable,
and inspectable.

Packages are how UI grows without central enum bottlenecks. They must not become
hidden global mutable registries or arbitrary code execution channels.

## Package Identity

Every package must declare:

```text
package id
package version
package kind
owner/domain
provided controls/components/kits
required dependencies
required capabilities
schema versions
migration hooks
fixture set
checksum/provenance
compatibility metadata
```

## Package Kinds

Supported package kinds:

```text
ControlPackage
ComponentPackage
CompositionKitPackage
ThemePackage
IconPackage
FontPackage
LocalizationPackage
AccessibilityPackage
HostAdapterPackage
PreviewFixturePackage
MigrationPackage
```

Package kinds may share manifest structure, but package meaning stays UI-owned.

## Capability Security

Capabilities must be explicit and fail closed.

Capability categories:

```text
ui.input.activate
ui.input.text
ui.focus.write
ui.clipboard.read
ui.clipboard.write
ui.dragdrop.read
ui.dragdrop.write
ui.asset.read
ui.asset.preview
ui.host.command.propose
ui.accessibility.emit
ui.debug.inspect
ui.remote.preview
```

A package must not perform host IO, app mutation, file IO, network IO, clipboard
access, asset writes, or remote preview behavior without declared capability and
host acceptance.

## Trust Levels

Package trust levels:

```text
BuiltinTrusted
WorkspaceTrusted
ProjectTrusted
ThirdPartyReviewed
ThirdPartyUntrusted
GeneratedUntrusted
```

Untrusted packages must be restricted to declarative descriptors unless an
explicit sandbox/execution design authorizes more.

## Sandbox Boundary

Control/package behavior should prefer declarative descriptors and deterministic
kernels.

If executable package kernels are allowed, they must declare:

```text
execution phase
allowed inputs
allowed outputs
capabilities
resource budget
determinism policy
panic/error policy
sandbox policy
host compatibility
proof fixtures
```

Executable kernels must not silently mutate host/app/game state.

## Versioning

Versioned entities:

```text
UiSourceVersion
UiProgramVersion
UiRuntimeArtifactVersion
ControlPackageVersion
ControlDescriptorVersion
ComponentVersion
SchemaVersion
ActionSchemaVersion
ThemeVersion
LocalizationBundleVersion
HostProfileVersion
```

Compatibility decisions must be explicit:

```text
Compatible
CompatibleWithMigration
CompatibleWithWarnings
IncompatibleSchema
IncompatibleCapability
IncompatibleHost
IncompatiblePackageVersion
Unknown
```

## Migration

Migration is required for durable source and package evolution.

Migration kinds:

```text
source migration
control descriptor migration
component/template migration
schema migration
theme token migration
localization key migration
runtime state migration
artifact invalidation migration
host profile migration
```

Migration output:

```text
MigrationReport
changed source ids
changed schema ids
changed package refs
automatic edits
manual repair requirements
source-map preservation
compatibility decision
```

## Provenance

Every source/program/artifact/report should preserve provenance:

```text
authored file
Rust projection source
visual designer source id
generated source id
package id/version
host profile id
tool version
commit/build id where available
```

Provenance is required for debugging, AI-agent review, migration, and trust.

## Registry Governance

Registries must be explicit snapshots:

```text
UiPackageRegistrySnapshot
UiControlCatalogSnapshot
UiComponentCatalogSnapshot
UiThemeCatalogSnapshot
UiHostCatalogSnapshot
```

No hidden global mutable registry may affect compilation/evaluation without being
visible in an assembly or package-resolution report.

## Reports

Required reports:

```text
UiPackageResolutionReport
UiCapabilityDecisionReport
UiPackageTrustReport
UiCompatibilityReport
UiMigrationReport
UiRegistrySnapshotReport
UiProvenanceReport
UiSandboxDecisionReport
```

## Rejected Shapes

Reject:

```text
implicit package discovery in hot paths
unversioned control descriptors
capability checks only in product code
executable third-party package kernels without sandbox/trust policy
hidden global mutable package registry
migration only by manual search/replace
source generated without provenance
artifact compatibility assumed from file names
```
