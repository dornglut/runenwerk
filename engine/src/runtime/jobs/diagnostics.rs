use product::{
    FieldProductDiagnostic, FieldProductDiagnosticCode, ProductIdentity, ProductJobDescriptor,
};

use crate::runtime::jobs::types::{RuntimeJobGeneration, RuntimeJobHandle};

pub fn runtime_job_failure_diagnostic(message: impl Into<String>) -> FieldProductDiagnostic {
    FieldProductDiagnostic::blocking(FieldProductDiagnosticCode::FormationFailure, message)
}

pub fn runtime_job_panic_diagnostic() -> FieldProductDiagnostic {
    runtime_job_failure_diagnostic("runtime product job panicked during execution")
}

pub fn runtime_job_backpressure_diagnostic(capacity: usize) -> FieldProductDiagnostic {
    FieldProductDiagnostic::blocking(
        FieldProductDiagnosticCode::RebuildBudgetExhausted,
        format!("runtime product job queue backpressure at capacity {capacity}"),
    )
}

pub fn runtime_job_disconnected_diagnostic() -> FieldProductDiagnostic {
    runtime_job_failure_diagnostic("runtime product job worker queue is disconnected")
}

pub fn runtime_job_stale_diagnostic(
    handle: RuntimeJobHandle,
    latest: RuntimeJobGeneration,
) -> FieldProductDiagnostic {
    FieldProductDiagnostic::new(
        FieldProductDiagnosticCode::PotentiallyStale,
        product::FieldProductDiagnosticSeverity::Warning,
        format!(
            "runtime product job generation {} is stale; latest generation is {}",
            handle.generation.raw(),
            latest.raw()
        ),
    )
    .for_product(ProductIdentity::new(handle.product_job_id.raw()))
}

pub fn runtime_job_error_for_product(
    diagnostic: FieldProductDiagnostic,
    product_job: &ProductJobDescriptor,
) -> FieldProductDiagnostic {
    if diagnostic.product_id.is_some() {
        return diagnostic;
    }
    if let Some(product_id) = product_job.output_products.first().copied() {
        diagnostic.for_product(product_id)
    } else {
        diagnostic
    }
}
