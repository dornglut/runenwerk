//! File: domain/ui/ui_controls/src/registry.rs
//! Crate: ui_controls

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::package::{
    ControlKindDescriptor, ControlKindId, ControlPackageDescriptor, ControlPackageId,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ControlPackageRegistry {
    packages: BTreeMap<String, ControlPackageDescriptor>,
}

impl ControlPackageRegistry {
    pub fn new() -> Self {
        Self {
            packages: BTreeMap::new(),
        }
    }

    pub fn with_package(
        mut self,
        package: ControlPackageDescriptor,
    ) -> Result<Self, ControlPackageRegistryError> {
        self.register(package)?;
        Ok(self)
    }

    pub fn register(
        &mut self,
        package: ControlPackageDescriptor,
    ) -> Result<(), ControlPackageRegistryError> {
        let package_key = package.package_id.as_str().to_owned();
        if self.packages.contains_key(&package_key) {
            return Err(ControlPackageRegistryError::DuplicatePackage {
                package_id: package_key,
            });
        }

        for kind in &package.control_kinds {
            if self.contains_kind(&kind.control_kind_id) {
                return Err(ControlPackageRegistryError::DuplicateControlKind {
                    control_kind_id: kind.control_kind_id.as_str().to_owned(),
                });
            }
        }

        self.packages.insert(package_key, package);
        Ok(())
    }

    pub fn package(&self, package_id: &ControlPackageId) -> Option<&ControlPackageDescriptor> {
        self.packages.get(package_id.as_str())
    }

    pub fn contains_kind(&self, control_kind_id: &ControlKindId) -> bool {
        self.packages.values().any(|package| {
            package
                .control_kinds
                .iter()
                .any(|kind| &kind.control_kind_id == control_kind_id)
        })
    }

    pub fn snapshot(&self) -> ControlPackageRegistrySnapshot {
        let packages: Vec<_> = self.packages.values().cloned().collect();
        let control_kinds = packages
            .iter()
            .flat_map(|package| package.control_kinds.iter().cloned())
            .collect();
        ControlPackageRegistrySnapshot {
            packages,
            control_kinds,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlPackageRegistrySnapshot {
    pub packages: Vec<ControlPackageDescriptor>,
    pub control_kinds: Vec<ControlKindDescriptor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlPackageRegistryError {
    DuplicatePackage { package_id: String },
    DuplicateControlKind { control_kind_id: String },
}

impl fmt::Display for ControlPackageRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicatePackage { package_id } => {
                write!(
                    formatter,
                    "control package {package_id} is already registered"
                )
            }
            Self::DuplicateControlKind { control_kind_id } => write!(
                formatter,
                "control kind {control_kind_id} is already registered"
            ),
        }
    }
}

impl std::error::Error for ControlPackageRegistryError {}
