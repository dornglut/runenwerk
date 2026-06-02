//! File: domain/ui/ui_controls/src/package.rs
//! Crate: ui_controls

use std::fmt;

use serde::{Deserialize, Serialize};
use ui_program::RouteCapability;
use ui_schema::UiSchemaRef;

use crate::diagnostics::{ControlDiagnosticDescriptor, ControlDiagnosticId};
use crate::kernel::{ControlKernelDescriptor, ControlKernelId, ControlKernelSet};
use crate::migration::{ControlMigrationHook, ControlMigrationId};
use crate::schema::{ControlSchemaDescriptor, ControlSchemaRole};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlPackageId(String);

impl ControlPackageId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("control package IDs must be namespaced")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, ControlPackageContractError> {
        let value = value.into();
        validate_control_id(&value, "package")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlKindId(String);

impl ControlKindId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("control kind IDs must be namespaced")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, ControlPackageContractError> {
        let value = value.into();
        validate_control_id(&value, "kind")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlFixtureId(String);

impl ControlFixtureId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("control fixture IDs must be namespaced")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, ControlPackageContractError> {
        let value = value.into();
        validate_control_id(&value, "fixture")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ControlPackageVersion(u32);

impl ControlPackageVersion {
    pub const fn new(value: u32) -> Self {
        assert!(value > 0);
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlKindDescriptor {
    pub control_kind_id: ControlKindId,
    pub display_name: String,
    pub property_schema: UiSchemaRef,
    pub state_schema: UiSchemaRef,
    pub event_payload_schema: UiSchemaRef,
    pub kernels: ControlKernelSet,
    #[serde(default)]
    pub required_capabilities: Vec<RouteCapability>,
    #[serde(default)]
    pub fixture_ids: Vec<ControlFixtureId>,
    #[serde(default)]
    pub diagnostic_ids: Vec<ControlDiagnosticId>,
    #[serde(default)]
    pub migration_ids: Vec<ControlMigrationId>,
}

impl ControlKindDescriptor {
    pub fn new(
        control_kind_id: ControlKindId,
        display_name: impl Into<String>,
        property_schema: UiSchemaRef,
        state_schema: UiSchemaRef,
        event_payload_schema: UiSchemaRef,
        kernels: ControlKernelSet,
    ) -> Self {
        Self {
            control_kind_id,
            display_name: display_name.into(),
            property_schema,
            state_schema,
            event_payload_schema,
            kernels,
            required_capabilities: Vec::new(),
            fixture_ids: Vec::new(),
            diagnostic_ids: Vec::new(),
            migration_ids: Vec::new(),
        }
    }

    pub fn with_required_capability(mut self, capability: RouteCapability) -> Self {
        self.required_capabilities.push(capability);
        self
    }

    pub fn with_fixture(mut self, fixture_id: ControlFixtureId) -> Self {
        self.fixture_ids.push(fixture_id);
        self
    }

    pub fn with_diagnostic(mut self, diagnostic_id: ControlDiagnosticId) -> Self {
        self.diagnostic_ids.push(diagnostic_id);
        self
    }

    pub fn with_migration(mut self, migration_id: ControlMigrationId) -> Self {
        self.migration_ids.push(migration_id);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlModuleDescriptor {
    pub kind: ControlKindDescriptor,
    #[serde(default)]
    pub schemas: Vec<ControlSchemaDescriptor>,
    #[serde(default)]
    pub kernels: Vec<ControlKernelDescriptor>,
    #[serde(default)]
    pub diagnostics: Vec<ControlDiagnosticDescriptor>,
    #[serde(default)]
    pub migrations: Vec<ControlMigrationHook>,
}

impl ControlModuleDescriptor {
    pub fn new(kind: ControlKindDescriptor) -> Self {
        Self {
            kind,
            schemas: Vec::new(),
            kernels: Vec::new(),
            diagnostics: Vec::new(),
            migrations: Vec::new(),
        }
    }

    pub fn with_schema(mut self, schema: ControlSchemaDescriptor) -> Self {
        self.schemas.push(schema);
        self
    }

    pub fn with_kernel(mut self, kernel: ControlKernelDescriptor) -> Self {
        self.kernels.push(kernel);
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: ControlDiagnosticDescriptor) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }

    pub fn with_migration(mut self, migration: ControlMigrationHook) -> Self {
        self.migrations.push(migration);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlPackageDescriptor {
    pub package_id: ControlPackageId,
    pub version: ControlPackageVersion,
    #[serde(default)]
    pub control_kinds: Vec<ControlKindDescriptor>,
    #[serde(default)]
    pub property_schemas: Vec<ControlSchemaDescriptor>,
    #[serde(default)]
    pub state_schemas: Vec<ControlSchemaDescriptor>,
    #[serde(default)]
    pub event_payload_schemas: Vec<ControlSchemaDescriptor>,
    #[serde(default)]
    pub required_capabilities: Vec<RouteCapability>,
    #[serde(default)]
    pub kernels: Vec<ControlKernelDescriptor>,
    #[serde(default)]
    pub kernel_ids: Vec<ControlKernelId>,
    #[serde(default)]
    pub fixture_ids: Vec<ControlFixtureId>,
    #[serde(default)]
    pub diagnostics: Vec<ControlDiagnosticDescriptor>,
    #[serde(default)]
    pub diagnostic_ids: Vec<ControlDiagnosticId>,
    #[serde(default)]
    pub migrations: Vec<ControlMigrationHook>,
    #[serde(default)]
    pub migration_ids: Vec<ControlMigrationId>,
}

impl ControlPackageDescriptor {
    pub fn new(package_id: ControlPackageId, version: ControlPackageVersion) -> Self {
        Self {
            package_id,
            version,
            control_kinds: Vec::new(),
            property_schemas: Vec::new(),
            state_schemas: Vec::new(),
            event_payload_schemas: Vec::new(),
            required_capabilities: Vec::new(),
            kernels: Vec::new(),
            kernel_ids: Vec::new(),
            fixture_ids: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_ids: Vec::new(),
            migrations: Vec::new(),
            migration_ids: Vec::new(),
        }
    }

    pub fn from_modules(
        package_id: ControlPackageId,
        version: ControlPackageVersion,
        modules: impl IntoIterator<Item = ControlModuleDescriptor>,
    ) -> Self {
        let mut package = Self::new(package_id, version);
        for module in modules {
            package = package.with_module(module);
        }
        package
    }

    pub fn with_module(mut self, module: ControlModuleDescriptor) -> Self {
        let ControlModuleDescriptor {
            kind,
            schemas,
            kernels,
            diagnostics,
            migrations,
        } = module;

        self.required_capabilities
            .extend(kind.required_capabilities.iter().cloned());
        self.fixture_ids.extend(kind.fixture_ids.iter().cloned());
        self.diagnostic_ids
            .extend(kind.diagnostic_ids.iter().cloned());
        self.migration_ids
            .extend(kind.migration_ids.iter().cloned());

        for kernel_id in kind.kernels.ids() {
            self.kernel_ids.push(kernel_id.clone());
        }

        for schema in schemas {
            match schema.role {
                ControlSchemaRole::Properties => self.property_schemas.push(schema),
                ControlSchemaRole::State => self.state_schemas.push(schema),
                ControlSchemaRole::EventPayload => self.event_payload_schemas.push(schema),
            }
        }

        self.kernels.extend(kernels);
        self.diagnostics.extend(diagnostics);
        self.migrations.extend(migrations);
        self.control_kinds.push(kind);
        self
    }

    pub fn control_kind(&self, id: &ControlKindId) -> Option<&ControlKindDescriptor> {
        self.control_kinds
            .iter()
            .find(|kind| &kind.control_kind_id == id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlPackageContractError {
    EmptyId { kind: &'static str },
    UnnamespacedId { kind: &'static str, value: String },
    InvalidIdCharacter { kind: &'static str, value: String },
}

impl fmt::Display for ControlPackageContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId { kind } => write!(formatter, "empty control {kind} id"),
            Self::UnnamespacedId { kind, value } => {
                write!(formatter, "control {kind} id {value} is not namespaced")
            }
            Self::InvalidIdCharacter { kind, value } => write!(
                formatter,
                "control {kind} id {value} contains an invalid character"
            ),
        }
    }
}

impl std::error::Error for ControlPackageContractError {}

fn validate_control_id(value: &str, kind: &'static str) -> Result<(), ControlPackageContractError> {
    if value.is_empty() {
        return Err(ControlPackageContractError::EmptyId { kind });
    }
    if !value.contains('.') {
        return Err(ControlPackageContractError::UnnamespacedId {
            kind,
            value: value.to_owned(),
        });
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
        return Err(ControlPackageContractError::InvalidIdCharacter {
            kind,
            value: value.to_owned(),
        });
    }
    Ok(())
}
