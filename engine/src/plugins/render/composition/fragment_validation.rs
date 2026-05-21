use super::{
    RenderFragmentDescriptor, RenderFragmentDiagnostic, RenderFragmentDiagnosticKind,
    RenderFragmentDiagnosticReport, RenderFragmentLabelRef, RenderFragmentPackageDescriptor,
    RenderFragmentPassDescriptor, RenderFragmentPassKind, RenderFragmentResourceDescriptor,
    RenderFragmentResourceKind, SUPPORTED_RENDER_FRAGMENT_SCHEMA_VERSION,
};
use std::collections::BTreeSet;

pub fn validate_fragment_package(
    package: &RenderFragmentPackageDescriptor,
) -> RenderFragmentDiagnosticReport {
    let mut diagnostics = Vec::<RenderFragmentDiagnostic>::new();

    if package.package_id.as_str().trim().is_empty() {
        diagnostics.push(
            RenderFragmentDiagnostic::error(
                RenderFragmentDiagnosticKind::EmptyPackageId,
                "render fragment package id must not be empty",
            )
            .with_package(package),
        );
    }
    if package.namespace.as_str().trim().is_empty() {
        diagnostics.push(
            RenderFragmentDiagnostic::error(
                RenderFragmentDiagnosticKind::EmptyNamespace,
                "render fragment package namespace must not be empty",
            )
            .with_package(package),
        );
    }
    if package.schema_version != SUPPORTED_RENDER_FRAGMENT_SCHEMA_VERSION {
        diagnostics.push(
            RenderFragmentDiagnostic::error(
                RenderFragmentDiagnosticKind::UnsupportedSchemaVersion,
                format!(
                    "render fragment package schema version {} is not supported; expected {}",
                    package.schema_version, SUPPORTED_RENDER_FRAGMENT_SCHEMA_VERSION
                ),
            )
            .with_package(package),
        );
    }

    let mut fragment_ids = BTreeSet::<String>::new();
    for fragment in &package.fragments {
        if !fragment_ids.insert(fragment.id.as_str().to_string()) {
            diagnostics.push(
                RenderFragmentDiagnostic::error(
                    RenderFragmentDiagnosticKind::DuplicateFragmentId,
                    format!("duplicate fragment id '{}'", fragment.id),
                )
                .with_package(package)
                .with_fragment(fragment),
            );
        }
        validate_fragment(package, fragment, &mut diagnostics);
    }

    RenderFragmentDiagnosticReport::new(diagnostics)
}

fn validate_fragment(
    package: &RenderFragmentPackageDescriptor,
    fragment: &RenderFragmentDescriptor,
    diagnostics: &mut Vec<RenderFragmentDiagnostic>,
) {
    if fragment.id.as_str().trim().is_empty() {
        diagnostics.push(
            RenderFragmentDiagnostic::error(
                RenderFragmentDiagnosticKind::EmptyFragmentId,
                "render fragment id must not be empty",
            )
            .with_package(package)
            .with_fragment(fragment),
        );
    }
    if fragment.namespace.as_str().trim().is_empty() {
        diagnostics.push(
            RenderFragmentDiagnostic::error(
                RenderFragmentDiagnosticKind::EmptyNamespace,
                "render fragment namespace must not be empty",
            )
            .with_package(package)
            .with_fragment(fragment),
        );
    }
    if fragment.namespace != package.namespace {
        diagnostics.push(
            RenderFragmentDiagnostic::error(
                RenderFragmentDiagnosticKind::FragmentNamespaceMismatch,
                format!(
                    "fragment '{}' uses namespace '{}' but package '{}' declares '{}'",
                    fragment.id, fragment.namespace, package.package_id, package.namespace
                ),
            )
            .with_package(package)
            .with_fragment(fragment),
        );
    }

    let resource_labels = validate_resource_labels(package, fragment, diagnostics);
    let pass_labels = validate_pass_labels(package, fragment, diagnostics);
    for pass in &fragment.passes {
        validate_pass_shape(package, fragment, pass, diagnostics);
        validate_pass_references(
            package,
            fragment,
            pass,
            &resource_labels,
            &pass_labels,
            diagnostics,
        );
    }
}

fn validate_resource_labels(
    package: &RenderFragmentPackageDescriptor,
    fragment: &RenderFragmentDescriptor,
    diagnostics: &mut Vec<RenderFragmentDiagnostic>,
) -> BTreeSet<String> {
    let mut labels = BTreeSet::<String>::new();
    for resource in &fragment.resources {
        validate_resource_label(package, fragment, resource, diagnostics);
        if is_local_resource(resource) && !labels.insert(resource.label.clone()) {
            diagnostics.push(
                RenderFragmentDiagnostic::error(
                    RenderFragmentDiagnosticKind::DuplicateResourceLabel,
                    format!(
                        "fragment '{}' declares duplicate resource label '{}'",
                        fragment.id, resource.label
                    ),
                )
                .with_package(package)
                .with_fragment(fragment)
                .with_label(resource.label.clone()),
            );
        }
    }
    labels
}

fn validate_resource_label(
    package: &RenderFragmentPackageDescriptor,
    fragment: &RenderFragmentDescriptor,
    resource: &RenderFragmentResourceDescriptor,
    diagnostics: &mut Vec<RenderFragmentDiagnostic>,
) {
    if resource.label.trim().is_empty() {
        diagnostics.push(
            RenderFragmentDiagnostic::error(
                RenderFragmentDiagnosticKind::EmptyLabel,
                format!(
                    "fragment '{}' declares an empty resource label",
                    fragment.id
                ),
            )
            .with_package(package)
            .with_fragment(fragment),
        );
    }
}

fn is_local_resource(resource: &RenderFragmentResourceDescriptor) -> bool {
    !matches!(
        resource.kind,
        RenderFragmentResourceKind::SurfaceColor | RenderFragmentResourceKind::SurfaceDepth
    )
}

fn validate_pass_labels(
    package: &RenderFragmentPackageDescriptor,
    fragment: &RenderFragmentDescriptor,
    diagnostics: &mut Vec<RenderFragmentDiagnostic>,
) -> BTreeSet<String> {
    let mut labels = BTreeSet::<String>::new();
    for pass in &fragment.passes {
        if pass.label.trim().is_empty() {
            diagnostics.push(
                RenderFragmentDiagnostic::error(
                    RenderFragmentDiagnosticKind::EmptyLabel,
                    format!("fragment '{}' declares an empty pass label", fragment.id),
                )
                .with_package(package)
                .with_fragment(fragment),
            );
        }
        if !labels.insert(pass.label.clone()) {
            diagnostics.push(
                RenderFragmentDiagnostic::error(
                    RenderFragmentDiagnosticKind::DuplicatePassLabel,
                    format!(
                        "fragment '{}' declares duplicate pass label '{}'",
                        fragment.id, pass.label
                    ),
                )
                .with_package(package)
                .with_fragment(fragment)
                .with_label(pass.label.clone()),
            );
        }
    }
    labels
}

fn validate_pass_shape(
    package: &RenderFragmentPackageDescriptor,
    fragment: &RenderFragmentDescriptor,
    pass: &RenderFragmentPassDescriptor,
    diagnostics: &mut Vec<RenderFragmentDiagnostic>,
) {
    match pass.kind {
        RenderFragmentPassKind::Compute => {
            if pass.compute_dispatch.is_none() {
                diagnostics.push(pass_shape_error(
                    package,
                    fragment,
                    pass,
                    "compute pass must declare dispatch",
                ));
            }
            if !pass.sample_textures.is_empty() {
                diagnostics.push(pass_shape_error(
                    package,
                    fragment,
                    pass,
                    "compute pass does not support sampled textures; use a fullscreen or graphics pass until compute texture sampling has a dedicated API",
                ));
            }
        }
        RenderFragmentPassKind::Fullscreen => {
            let color_outputs = pass.color_outputs.len() + usize::from(pass.write_surface_color);
            if color_outputs != 1 {
                diagnostics.push(pass_shape_error(
                    package,
                    fragment,
                    pass,
                    format!(
                        "fullscreen pass must declare exactly one color output; found {}",
                        color_outputs
                    ),
                ));
            }
        }
        RenderFragmentPassKind::Graphics => {
            let color_outputs = pass.color_outputs.len() + usize::from(pass.write_surface_color);
            if color_outputs != 1 {
                diagnostics.push(pass_shape_error(
                    package,
                    fragment,
                    pass,
                    format!(
                        "graphics pass must declare exactly one color output; found {}",
                        color_outputs
                    ),
                ));
            }
            if pass.draw.is_none() {
                diagnostics.push(pass_shape_error(
                    package,
                    fragment,
                    pass,
                    "graphics pass must declare draw",
                ));
            }
        }
        RenderFragmentPassKind::Copy => {
            if pass.copy_source.is_none() || pass.copy_destination.is_none() {
                diagnostics.push(pass_shape_error(
                    package,
                    fragment,
                    pass,
                    "copy pass must declare source and destination",
                ));
            }
        }
        RenderFragmentPassKind::Present => {
            if pass.present_source.is_none() && !pass.write_surface_color {
                diagnostics.push(pass_shape_error(
                    package,
                    fragment,
                    pass,
                    "present pass must declare a source or surface color",
                ));
            }
        }
        RenderFragmentPassKind::BuiltinUiComposite => {}
    }
}

fn pass_shape_error(
    package: &RenderFragmentPackageDescriptor,
    fragment: &RenderFragmentDescriptor,
    pass: &RenderFragmentPassDescriptor,
    message: impl Into<String>,
) -> RenderFragmentDiagnostic {
    RenderFragmentDiagnostic::error(RenderFragmentDiagnosticKind::InvalidPassShape, message)
        .with_package(package)
        .with_fragment(fragment)
        .with_label(pass.label.clone())
}

fn validate_pass_references(
    package: &RenderFragmentPackageDescriptor,
    fragment: &RenderFragmentDescriptor,
    pass: &RenderFragmentPassDescriptor,
    resource_labels: &BTreeSet<String>,
    pass_labels: &BTreeSet<String>,
    diagnostics: &mut Vec<RenderFragmentDiagnostic>,
) {
    for reference in pass
        .sample_textures
        .iter()
        .chain(pass.write_textures.iter())
        .chain(pass.color_outputs.iter())
        .chain(pass.depth_target.iter())
        .chain(pass.copy_source.iter())
        .chain(pass.copy_destination.iter())
        .chain(pass.present_source.iter())
    {
        validate_resource_ref(
            package,
            fragment,
            pass,
            reference,
            resource_labels,
            diagnostics,
        );
    }

    for dependency in &pass.dependencies {
        if dependency.is_local() && !pass_labels.contains(dependency.raw_label()) {
            diagnostics.push(
                RenderFragmentDiagnostic::error(
                    RenderFragmentDiagnosticKind::MissingPassReference,
                    format!(
                        "pass '{}' depends on missing local pass '{}'",
                        pass.label,
                        dependency.raw_label()
                    ),
                )
                .with_package(package)
                .with_fragment(fragment)
                .with_label(pass.label.clone()),
            );
        }
    }
}

fn validate_resource_ref(
    package: &RenderFragmentPackageDescriptor,
    fragment: &RenderFragmentDescriptor,
    pass: &RenderFragmentPassDescriptor,
    reference: &RenderFragmentLabelRef,
    resource_labels: &BTreeSet<String>,
    diagnostics: &mut Vec<RenderFragmentDiagnostic>,
) {
    if reference.is_local() && !resource_labels.contains(reference.raw_label()) {
        diagnostics.push(
            RenderFragmentDiagnostic::error(
                RenderFragmentDiagnosticKind::MissingResourceReference,
                format!(
                    "pass '{}' references missing local resource '{}'",
                    pass.label,
                    reference.raw_label()
                ),
            )
            .with_package(package)
            .with_fragment(fragment)
            .with_label(pass.label.clone()),
        );
    }
}
