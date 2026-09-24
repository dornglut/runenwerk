use engine::plugins::render::inspect::inspect_render_gpu_residency;
use engine::plugins::render::{
    RenderGpuResidencyBudgetResource, RenderGpuResidencyBudgetStatus, RenderGpuResidencyResource,
};
use product::{
    FieldProductDiagnosticCode, ProductAuthorityClass, ProductFreshness, ProductIdentity,
    ProductQueryPolicy, ProductResidency, ProductScaleBand, RenderProductSelection,
    RenderResidencyRequest, RenderSelectedProduct,
};

fn selected(product_id: u64, generation: u64) -> RenderSelectedProduct {
    RenderSelectedProduct {
        product_id: ProductIdentity::new(product_id),
        scale_band: ProductScaleBand::Preview,
        generation,
        freshness: ProductFreshness::Current,
        residency: ProductResidency::Resident,
        authority_class: ProductAuthorityClass::DeterministicDerived,
        query_policy: ProductQueryPolicy::StrictCurrentOnly,
    }
}

fn selection(
    view_id: &str,
    product_id: u64,
    generation: u64,
    priority: i32,
    hard_pin: bool,
) -> RenderProductSelection {
    RenderProductSelection::new(view_id)
        .with_selected_product(selected(product_id, generation))
        .with_residency_request(RenderResidencyRequest::new(
            ProductIdentity::new(product_id),
            ProductResidency::Resident,
            priority,
            hard_pin,
        ))
}

fn scale_budget() -> RenderGpuResidencyBudgetResource {
    RenderGpuResidencyBudgetResource {
        max_resident_entries: 4,
        max_resident_bytes: 1024,
        max_upload_bytes_per_frame: 512,
        resident_bytes_per_entry: 128,
        upload_bytes_per_allocation: 64,
    }
}

#[test]
fn render_scale_working_set_reports_selected_resident_and_budget_evidence() {
    let mut residency = RenderGpuResidencyResource::default();
    let budget = scale_budget();

    let summary = residency.derive_from_selections(
        &[
            selection("main", 1, 10, 100, false),
            selection("main", 2, 11, 90, false),
        ],
        &budget,
    );

    assert_eq!(summary.addressable_count, 2);
    assert_eq!(summary.selected_count, 2);
    assert_eq!(summary.requested_count, 2);
    assert_eq!(summary.accepted_count, 2);
    assert_eq!(summary.resident_count, 2);
    assert_eq!(summary.resident_bytes, 256);
    assert_eq!(summary.upload_bytes, 128);
    assert_eq!(
        summary.resident_byte_budget_status,
        RenderGpuResidencyBudgetStatus::WithinBudget
    );
    assert_eq!(
        summary.upload_byte_budget_status,
        RenderGpuResidencyBudgetStatus::WithinBudget
    );

    let inspection = inspect_render_gpu_residency(&residency);
    assert_eq!(inspection.addressable_count, 2);
    assert_eq!(inspection.selected_count, 2);
    assert_eq!(inspection.requested_count, 2);
    assert_eq!(inspection.accepted_count, 2);
    assert_eq!(inspection.resident_bytes, 256);
    assert_eq!(inspection.upload_bytes, 128);
    assert_eq!(inspection.budget.resident_entry_status, "within_budget");
    assert_eq!(inspection.budget.resident_byte_status, "within_budget");
    assert_eq!(inspection.budget.upload_byte_status, "within_budget");
    assert_eq!(inspection.entries[0].resident_bytes, 128);
    assert_eq!(inspection.entries[0].upload_bytes, 64);
    assert!(
        inspection.entries[0]
            .cache_id
            .starts_with("render-gpu-cache:")
    );
}

#[test]
fn render_scale_budget_pressure_is_explicit_without_product_fallback() {
    let mut residency = RenderGpuResidencyResource::default();
    let budget = RenderGpuResidencyBudgetResource {
        max_resident_entries: 1,
        max_resident_bytes: 128,
        max_upload_bytes_per_frame: 64,
        resident_bytes_per_entry: 128,
        upload_bytes_per_allocation: 64,
    };

    let summary = residency.derive_from_selections(
        &[
            selection("main", 1, 10, 100, true),
            selection("main", 2, 11, 90, true),
        ],
        &budget,
    );

    assert_eq!(summary.resident_count, 2);
    assert_eq!(summary.evicted_count, 0);
    assert!(summary.hard_pinned_over_entry_budget);
    assert_eq!(
        summary.resident_entry_budget_status,
        RenderGpuResidencyBudgetStatus::OverBudget
    );
    assert_eq!(
        summary.resident_byte_budget_status,
        RenderGpuResidencyBudgetStatus::OverBudget
    );
    assert_eq!(
        summary.upload_byte_budget_status,
        RenderGpuResidencyBudgetStatus::OverBudget
    );
    assert!(residency
        .diagnostics()
        .iter()
        .all(|diagnostic| diagnostic.code == FieldProductDiagnosticCode::RebuildBudgetExhausted));

    let inspection = inspect_render_gpu_residency(&residency);
    assert_eq!(inspection.budget.resident_entry_status, "over_budget");
    assert_eq!(inspection.budget.resident_byte_status, "over_budget");
    assert_eq!(inspection.budget.upload_byte_status, "over_budget");
    assert!(inspection.budget.hard_pinned_over_entry_budget);
    assert_eq!(inspection.diagnostic_count, residency.diagnostics().len());
}

#[test]
fn render_scale_missing_selected_product_fails_closed() {
    let mut residency = RenderGpuResidencyResource::default();
    let selection =
        RenderProductSelection::new("main").with_residency_request(RenderResidencyRequest::new(
            ProductIdentity::new(99),
            ProductResidency::Resident,
            100,
            true,
        ));

    let summary = residency
        .derive_from_selections(&[selection], &RenderGpuResidencyBudgetResource::default());

    assert_eq!(summary.addressable_count, 0);
    assert_eq!(summary.requested_count, 1);
    assert_eq!(summary.accepted_count, 0);
    assert_eq!(summary.rejected_count, 1);
    assert_eq!(summary.resident_count, 0);
    assert!(residency.diagnostics().iter().any(|diagnostic| {
        diagnostic.code == FieldProductDiagnosticCode::MissingProduct
            && diagnostic.product_id == Some(ProductIdentity::new(99))
    }));

    let inspection = inspect_render_gpu_residency(&residency);
    assert_eq!(inspection.journal[0].action, "Rejected");
    assert_eq!(inspection.journal[0].product_id, 99);
    assert_eq!(inspection.journal[0].cache_id, None);
}
