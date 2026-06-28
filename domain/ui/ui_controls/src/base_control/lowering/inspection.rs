//! File: domain/ui/ui_controls/src/base_control/lowering/inspection.rs
//! Crate: ui_controls

use crate::catalog::{ControlLayoutInspectionExt, ControlRenderInspectionExt};
use crate::{ControlInspectionDescriptor, ControlPackageDescriptor};

use super::super::compiler::LoweredControl;

pub(crate) fn lower_inspection(
    package: &ControlPackageDescriptor,
    control: &LoweredControl,
) -> ControlInspectionDescriptor {
    ControlInspectionDescriptor::from_control_kind(package, &control.module.kind)
        .with_input_summary(&control.input.summary())
        .with_state_summary(&control.state.summary())
        .with_theme_summary(&control.theme.summary())
        .with_accessibility_summary(&control.accessibility.summary())
        .with_control_layout_summary(&control.layout.summary())
        .with_control_render_summary(&control.render.summary())
}
