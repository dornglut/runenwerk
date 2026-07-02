use std::collections::BTreeSet;

use crate::{
    ControlPackageDescriptor, ControlPackageValidationDiagnostic, ControlPackageValidationReason,
    ControlPackageValidationReport,
};

pub trait ControlOverlayPackageValidationExt {
    fn validate_overlay_descriptors(&self) -> ControlPackageValidationReport;
}

impl ControlOverlayPackageValidationExt for ControlPackageDescriptor {
    fn validate_overlay_descriptors(&self) -> ControlPackageValidationReport {
        let mut report = ControlPackageValidationReport::new();
        let kind_ids = self
            .control_kinds
            .iter()
            .map(|kind| kind.control_kind_id.as_str().to_owned())
            .collect::<BTreeSet<_>>();
        let mut seen = BTreeSet::new();
        for descriptor in &self.overlay_descriptors {
            let id = descriptor.control_kind_id.as_str().to_owned();
            if !kind_ids.contains(&id) {
                report.push(ControlPackageValidationDiagnostic::kind(
                    descriptor.control_kind_id.clone(),
                    ControlPackageValidationReason::UnresolvedReference,
                    "overlay descriptor references no control kind in the package",
                ));
            }
            if !seen.insert(id.clone()) {
                report.push(ControlPackageValidationDiagnostic::package(
                    self.package_id.clone(),
                    ControlPackageValidationReason::UnresolvedReference,
                    format!("duplicate overlay descriptor for control kind {id}"),
                ));
            }
            if descriptor.requirements.is_empty() {
                report.push(ControlPackageValidationDiagnostic::kind(
                    descriptor.control_kind_id.clone(),
                    ControlPackageValidationReason::UnresolvedReference,
                    "overlay descriptor must contain at least one requirement",
                ));
            }
        }
        report
    }
}
