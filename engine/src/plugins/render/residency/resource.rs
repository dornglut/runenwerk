use std::collections::{BTreeMap, BTreeSet};

use product::{
    FieldProductDiagnostic, FieldProductDiagnosticCode, FieldProductDiagnosticSeverity,
    ProductAuthorityClass, ProductFreshness, ProductIdentity, ProductResidency,
    RenderProductSelection, RenderResidencyRequest, RenderSelectedProduct,
};

use crate::plugins::render::{PreparedRenderProductSelectionResource, RenderGpuCacheHandle};
use crate::runtime::{Res, ResMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderGpuResidencyStatus {
    Resident,
    Preserved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderGpuResidencyJournalAction {
    Allocated,
    Preserved,
    Invalidated,
    Evicted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderGpuResidencyEntry {
    pub product_id: ProductIdentity,
    pub generation: u64,
    pub requested_residency: ProductResidency,
    pub priority: i32,
    pub hard_pin: bool,
    pub status: RenderGpuResidencyStatus,
    pub cache_handle: RenderGpuCacheHandle,
    pub diagnostics: Vec<FieldProductDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderGpuResidencyJournalEntry {
    pub action: RenderGpuResidencyJournalAction,
    pub product_id: ProductIdentity,
    pub generation: u64,
    pub requested_residency: ProductResidency,
    pub priority: i32,
    pub hard_pin: bool,
    pub cache_handle: Option<RenderGpuCacheHandle>,
    pub diagnostics: Vec<FieldProductDiagnostic>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RenderGpuResidencySummary {
    pub resident_count: usize,
    pub allocated_count: usize,
    pub preserved_count: usize,
    pub invalidated_count: usize,
    pub evicted_count: usize,
    pub rejected_count: usize,
    pub diagnostic_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ecs::Component, ecs::Resource)]
pub struct RenderGpuResidencyBudgetResource {
    pub max_resident_entries: usize,
}

impl Default for RenderGpuResidencyBudgetResource {
    fn default() -> Self {
        Self {
            max_resident_entries: 64,
        }
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct RenderGpuResidencyResource {
    entries: BTreeMap<ProductIdentity, RenderGpuResidencyEntry>,
    journal: Vec<RenderGpuResidencyJournalEntry>,
    diagnostics: Vec<FieldProductDiagnostic>,
    next_handle: u64,
    last_summary: RenderGpuResidencySummary,
}

impl Default for RenderGpuResidencyResource {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            journal: Vec::new(),
            diagnostics: Vec::new(),
            next_handle: 1,
            last_summary: RenderGpuResidencySummary::default(),
        }
    }
}

impl RenderGpuResidencyResource {
    pub fn entries(&self) -> &BTreeMap<ProductIdentity, RenderGpuResidencyEntry> {
        &self.entries
    }

    pub fn entry(&self, product_id: ProductIdentity) -> Option<&RenderGpuResidencyEntry> {
        self.entries.get(&product_id)
    }

    pub fn journal(&self) -> &[RenderGpuResidencyJournalEntry] {
        &self.journal
    }

    pub fn diagnostics(&self) -> &[FieldProductDiagnostic] {
        &self.diagnostics
    }

    pub fn last_summary(&self) -> RenderGpuResidencySummary {
        self.last_summary
    }

    pub fn derive_from_selections(
        &mut self,
        selections: &[RenderProductSelection],
        budget: &RenderGpuResidencyBudgetResource,
    ) -> RenderGpuResidencySummary {
        self.diagnostics.clear();

        let residency_plan = build_residency_plan(selections);
        let mut summary = RenderGpuResidencySummary::default();

        for (product_id, diagnostics) in residency_plan.rejected {
            summary.rejected_count = summary.rejected_count.saturating_add(1);
            self.push_rejected(product_id, diagnostics.clone());
            if let Some(entry) = self.entries.remove(&product_id) {
                summary.evicted_count = summary.evicted_count.saturating_add(1);
                self.push_journal(
                    RenderGpuResidencyJournalAction::Evicted,
                    entry.product_id,
                    entry.generation,
                    entry.requested_residency,
                    entry.priority,
                    entry.hard_pin,
                    Some(entry.cache_handle),
                    Vec::new(),
                );
            }
        }

        let requested_products = residency_plan
            .accepted
            .keys()
            .copied()
            .collect::<BTreeSet<_>>();
        let unrequested_products = self
            .entries
            .keys()
            .copied()
            .filter(|product_id| !requested_products.contains(product_id))
            .collect::<Vec<_>>();
        for product_id in unrequested_products {
            if let Some(entry) = self.entries.remove(&product_id) {
                summary.evicted_count = summary.evicted_count.saturating_add(1);
                self.push_journal(
                    RenderGpuResidencyJournalAction::Evicted,
                    entry.product_id,
                    entry.generation,
                    entry.requested_residency,
                    entry.priority,
                    entry.hard_pin,
                    Some(entry.cache_handle),
                    Vec::new(),
                );
            }
        }

        for item in residency_plan.accepted.into_values() {
            if let Some(previous) = self.entries.get(&item.product_id).cloned() {
                if previous.generation == item.generation {
                    let entry = RenderGpuResidencyEntry {
                        product_id: item.product_id,
                        generation: item.generation,
                        requested_residency: item.requested_residency,
                        priority: item.priority,
                        hard_pin: item.hard_pin,
                        status: RenderGpuResidencyStatus::Preserved,
                        cache_handle: previous.cache_handle,
                        diagnostics: Vec::new(),
                    };
                    self.entries.insert(item.product_id, entry.clone());
                    summary.preserved_count = summary.preserved_count.saturating_add(1);
                    self.push_journal(
                        RenderGpuResidencyJournalAction::Preserved,
                        entry.product_id,
                        entry.generation,
                        entry.requested_residency,
                        entry.priority,
                        entry.hard_pin,
                        Some(entry.cache_handle),
                        Vec::new(),
                    );
                } else {
                    let diagnostic = generation_invalidated_diagnostic(&previous, item.generation);
                    self.entries.remove(&item.product_id);
                    summary.invalidated_count = summary.invalidated_count.saturating_add(1);
                    self.push_journal(
                        RenderGpuResidencyJournalAction::Invalidated,
                        previous.product_id,
                        previous.generation,
                        previous.requested_residency,
                        previous.priority,
                        previous.hard_pin,
                        Some(previous.cache_handle),
                        vec![diagnostic],
                    );
                    self.allocate_entry(item, &mut summary);
                }
            } else {
                self.allocate_entry(item, &mut summary);
            }
        }

        self.evict_to_budget(budget, &mut summary);
        summary.resident_count = self.entries.len();
        summary.diagnostic_count = self.diagnostics.len();
        self.last_summary = summary;
        summary
    }

    fn allocate_entry(&mut self, item: ResidencyPlanItem, summary: &mut RenderGpuResidencySummary) {
        let cache_handle = self.allocate_handle();
        let entry = RenderGpuResidencyEntry {
            product_id: item.product_id,
            generation: item.generation,
            requested_residency: item.requested_residency,
            priority: item.priority,
            hard_pin: item.hard_pin,
            status: RenderGpuResidencyStatus::Resident,
            cache_handle,
            diagnostics: Vec::new(),
        };
        self.entries.insert(item.product_id, entry.clone());
        summary.allocated_count = summary.allocated_count.saturating_add(1);
        self.push_journal(
            RenderGpuResidencyJournalAction::Allocated,
            entry.product_id,
            entry.generation,
            entry.requested_residency,
            entry.priority,
            entry.hard_pin,
            Some(entry.cache_handle),
            Vec::new(),
        );
    }

    fn allocate_handle(&mut self) -> RenderGpuCacheHandle {
        let handle = RenderGpuCacheHandle::new(self.next_handle);
        self.next_handle = self.next_handle.saturating_add(1).max(1);
        handle
    }

    fn evict_to_budget(
        &mut self,
        budget: &RenderGpuResidencyBudgetResource,
        summary: &mut RenderGpuResidencySummary,
    ) {
        while self.entries.len() > budget.max_resident_entries {
            let Some(product_id) = self
                .entries
                .values()
                .filter(|entry| !entry.hard_pin)
                .min_by_key(|entry| (entry.priority, entry.product_id.raw(), entry.generation))
                .map(|entry| entry.product_id)
            else {
                self.diagnostics.push(pinned_budget_exceeded_diagnostic(
                    self.entries.len(),
                    budget.max_resident_entries,
                ));
                return;
            };
            if let Some(entry) = self.entries.remove(&product_id) {
                summary.evicted_count = summary.evicted_count.saturating_add(1);
                self.push_journal(
                    RenderGpuResidencyJournalAction::Evicted,
                    entry.product_id,
                    entry.generation,
                    entry.requested_residency,
                    entry.priority,
                    entry.hard_pin,
                    Some(entry.cache_handle),
                    Vec::new(),
                );
            }
        }
    }

    fn push_rejected(
        &mut self,
        product_id: ProductIdentity,
        diagnostics: Vec<FieldProductDiagnostic>,
    ) {
        self.push_journal(
            RenderGpuResidencyJournalAction::Rejected,
            product_id,
            0,
            ProductResidency::Missing,
            0,
            false,
            None,
            diagnostics,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn push_journal(
        &mut self,
        action: RenderGpuResidencyJournalAction,
        product_id: ProductIdentity,
        generation: u64,
        requested_residency: ProductResidency,
        priority: i32,
        hard_pin: bool,
        cache_handle: Option<RenderGpuCacheHandle>,
        diagnostics: Vec<FieldProductDiagnostic>,
    ) {
        self.diagnostics.extend(diagnostics.iter().cloned());
        self.journal.push(RenderGpuResidencyJournalEntry {
            action,
            product_id,
            generation,
            requested_residency,
            priority,
            hard_pin,
            cache_handle,
            diagnostics,
        });
    }
}

#[derive(Debug, Clone)]
struct ResidencyPlan {
    accepted: BTreeMap<ProductIdentity, ResidencyPlanItem>,
    rejected: BTreeMap<ProductIdentity, Vec<FieldProductDiagnostic>>,
}

#[derive(Debug, Clone)]
struct ResidencyPlanItem {
    product_id: ProductIdentity,
    generation: u64,
    requested_residency: ProductResidency,
    priority: i32,
    hard_pin: bool,
}

fn build_residency_plan(selections: &[RenderProductSelection]) -> ResidencyPlan {
    let mut selected = BTreeMap::<ProductIdentity, RenderSelectedProduct>::new();
    let mut rejected = BTreeMap::<ProductIdentity, Vec<FieldProductDiagnostic>>::new();

    let mut sorted_selections = selections.iter().collect::<Vec<_>>();
    sorted_selections.sort_by(|left, right| left.view_id.cmp(&right.view_id));

    for selection in &sorted_selections {
        let mut selected_products = selection.selected_products.iter().collect::<Vec<_>>();
        selected_products.sort_by_key(|product| product.product_id);
        for product in selected_products {
            if let Some(previous) = selected.get(&product.product_id) {
                if selected_state_conflicts(previous, product) {
                    rejected
                        .entry(product.product_id)
                        .or_default()
                        .push(selected_state_conflict_diagnostic(previous, product));
                }
            } else {
                selected.insert(product.product_id, product.clone());
            }
        }
    }

    let mut accepted = BTreeMap::<ProductIdentity, ResidencyPlanItem>::new();
    for selection in sorted_selections {
        let mut requests = selection.residency_requests.iter().collect::<Vec<_>>();
        requests.sort_by_key(|request| (request.product_id, request.priority));
        for request in requests {
            if rejected.contains_key(&request.product_id) {
                continue;
            }
            let Some(product) = selected.get(&request.product_id) else {
                rejected
                    .entry(request.product_id)
                    .or_default()
                    .push(missing_selected_product_diagnostic(request));
                continue;
            };

            let diagnostics = residency_request_diagnostics(product, request);
            if !diagnostics.is_empty() {
                rejected
                    .entry(request.product_id)
                    .or_default()
                    .extend(diagnostics);
                continue;
            }

            let entry = accepted
                .entry(request.product_id)
                .or_insert_with(|| ResidencyPlanItem {
                    product_id: request.product_id,
                    generation: product.generation,
                    requested_residency: request.residency,
                    priority: request.priority,
                    hard_pin: request.hard_pin,
                });
            entry.priority = entry.priority.max(request.priority);
            entry.hard_pin |= request.hard_pin;
        }
    }

    for product_id in rejected.keys() {
        accepted.remove(product_id);
    }

    ResidencyPlan { accepted, rejected }
}

fn selected_state_conflicts(left: &RenderSelectedProduct, right: &RenderSelectedProduct) -> bool {
    left.generation != right.generation
        || left.freshness != right.freshness
        || left.residency != right.residency
        || left.authority_class != right.authority_class
        || left.query_policy != right.query_policy
}

fn residency_request_diagnostics(
    selected: &RenderSelectedProduct,
    request: &RenderResidencyRequest,
) -> Vec<FieldProductDiagnostic> {
    let mut diagnostics = Vec::new();
    if request.residency != ProductResidency::Resident {
        diagnostics.push(
            FieldProductDiagnostic::blocking(
                FieldProductDiagnosticCode::UnsupportedConsumerRequest,
                "renderer gpu residency only materializes resident product requests",
            )
            .for_product(request.product_id),
        );
    }
    if !selected.query_policy.allows(
        selected.freshness,
        selected.residency,
        selected.authority_class,
    ) {
        diagnostics.push(
            FieldProductDiagnostic::blocking(
                rejection_code_for_selected_state(selected),
                "renderer gpu residency rejected selected product state for requested query policy",
            )
            .for_product(selected.product_id),
        );
    }
    diagnostics
}

fn rejection_code_for_selected_state(
    selected: &RenderSelectedProduct,
) -> FieldProductDiagnosticCode {
    if !matches!(selected.freshness, ProductFreshness::Current) {
        return FieldProductDiagnosticCode::StaleProduct;
    }
    if !matches!(
        selected.residency,
        ProductResidency::Resident | ProductResidency::NotApplicable
    ) {
        return FieldProductDiagnosticCode::NonResident;
    }
    if !matches!(
        selected.authority_class,
        ProductAuthorityClass::Authoritative
            | ProductAuthorityClass::ServerValidated
            | ProductAuthorityClass::DeterministicDerived
    ) {
        return FieldProductDiagnosticCode::VisualOnlyUsedForStrictQuery;
    }
    FieldProductDiagnosticCode::UnsupportedConsumerRequest
}

fn selected_state_conflict_diagnostic(
    previous: &RenderSelectedProduct,
    next: &RenderSelectedProduct,
) -> FieldProductDiagnostic {
    FieldProductDiagnostic::blocking(
        FieldProductDiagnosticCode::GenerationMismatch,
        format!(
            "render gpu residency received conflicting selected product state for product {} (generation {} vs {})",
            previous.product_id.raw(),
            previous.generation,
            next.generation
        ),
    )
    .for_product(previous.product_id)
}

fn missing_selected_product_diagnostic(request: &RenderResidencyRequest) -> FieldProductDiagnostic {
    FieldProductDiagnostic::blocking(
        FieldProductDiagnosticCode::MissingProduct,
        "render gpu residency request references a product that was not selected",
    )
    .for_product(request.product_id)
}

fn generation_invalidated_diagnostic(
    previous: &RenderGpuResidencyEntry,
    next_generation: u64,
) -> FieldProductDiagnostic {
    FieldProductDiagnostic::new(
        FieldProductDiagnosticCode::GenerationMismatch,
        FieldProductDiagnosticSeverity::Warning,
        format!(
            "renderer gpu cache generation changed from {} to {}",
            previous.generation, next_generation
        ),
    )
    .for_product(previous.product_id)
}

fn pinned_budget_exceeded_diagnostic(
    resident_count: usize,
    budget: usize,
) -> FieldProductDiagnostic {
    FieldProductDiagnostic::new(
        FieldProductDiagnosticCode::RebuildBudgetExhausted,
        FieldProductDiagnosticSeverity::Warning,
        format!(
            "renderer gpu residency has {resident_count} hard-pinned entries with budget {budget}"
        ),
    )
}

pub fn derive_render_gpu_residency_system(
    selections: Res<PreparedRenderProductSelectionResource>,
    mut residency: ResMut<RenderGpuResidencyResource>,
    budget: Res<RenderGpuResidencyBudgetResource>,
) {
    let snapshot = selections.snapshot();
    residency.derive_from_selections(&snapshot, &budget);
}

#[cfg(test)]
mod tests {
    use super::*;
    use product::{
        ProductAuthorityClass, ProductFreshness, ProductQueryPolicy, ProductScaleBand,
        RenderProductSelection, RenderResidencyRequest,
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

    #[test]
    fn render_gpu_residency_allocates_from_prepared_selection() {
        let mut residency = RenderGpuResidencyResource::default();
        let budget = RenderGpuResidencyBudgetResource::default();

        let summary =
            residency.derive_from_selections(&[selection("main", 7, 1, 100, true)], &budget);

        assert_eq!(summary.allocated_count, 1);
        assert_eq!(summary.resident_count, 1);
        let entry = residency.entry(ProductIdentity::new(7)).unwrap();
        assert_eq!(entry.generation, 1);
        assert_eq!(entry.cache_handle, RenderGpuCacheHandle::new(1));
        assert!(entry.hard_pin);
    }

    #[test]
    fn render_gpu_residency_preserves_same_generation() {
        let mut residency = RenderGpuResidencyResource::default();
        let budget = RenderGpuResidencyBudgetResource::default();
        residency.derive_from_selections(&[selection("main", 7, 1, 100, true)], &budget);

        let summary =
            residency.derive_from_selections(&[selection("main", 7, 1, 50, false)], &budget);

        assert_eq!(summary.preserved_count, 1);
        assert_eq!(summary.allocated_count, 0);
        let entry = residency.entry(ProductIdentity::new(7)).unwrap();
        assert_eq!(entry.cache_handle, RenderGpuCacheHandle::new(1));
        assert_eq!(entry.priority, 50);
        assert!(!entry.hard_pin);
    }

    #[test]
    fn render_gpu_residency_invalidates_generation_changes() {
        let mut residency = RenderGpuResidencyResource::default();
        let budget = RenderGpuResidencyBudgetResource::default();
        residency.derive_from_selections(&[selection("main", 7, 1, 100, true)], &budget);

        let summary =
            residency.derive_from_selections(&[selection("main", 7, 2, 100, true)], &budget);

        assert_eq!(summary.invalidated_count, 1);
        assert_eq!(summary.allocated_count, 1);
        let entry = residency.entry(ProductIdentity::new(7)).unwrap();
        assert_eq!(entry.generation, 2);
        assert_eq!(entry.cache_handle, RenderGpuCacheHandle::new(2));
        assert!(residency.journal().iter().any(|entry| {
            entry.action == RenderGpuResidencyJournalAction::Invalidated
                && entry.product_id == ProductIdentity::new(7)
        }));
    }

    #[test]
    fn render_gpu_residency_rejects_invalid_strict_selected_state() {
        let mut invalid = selection("main", 7, 1, 100, true);
        invalid.selected_products[0].freshness = ProductFreshness::Stale;
        let mut residency = RenderGpuResidencyResource::default();
        let budget = RenderGpuResidencyBudgetResource::default();

        let summary = residency.derive_from_selections(&[invalid], &budget);

        assert_eq!(summary.rejected_count, 1);
        assert_eq!(summary.resident_count, 0);
        assert_eq!(summary.diagnostic_count, 1);
        assert!(residency.entry(ProductIdentity::new(7)).is_none());
    }

    #[test]
    fn render_gpu_residency_evicts_low_priority_non_pinned_entries_deterministically() {
        let mut residency = RenderGpuResidencyResource::default();
        let budget = RenderGpuResidencyBudgetResource {
            max_resident_entries: 2,
        };
        let selections = [
            selection("a", 1, 1, 10, false),
            selection("b", 2, 1, 50, false),
            selection("c", 3, 1, 100, true),
        ];

        let summary = residency.derive_from_selections(&selections, &budget);

        assert_eq!(summary.allocated_count, 3);
        assert_eq!(summary.evicted_count, 1);
        assert!(residency.entry(ProductIdentity::new(1)).is_none());
        assert!(residency.entry(ProductIdentity::new(2)).is_some());
        assert!(residency.entry(ProductIdentity::new(3)).is_some());
    }

    #[test]
    fn render_gpu_residency_reports_pinned_budget_exhaustion_without_evicting_pins() {
        let mut residency = RenderGpuResidencyResource::default();
        let budget = RenderGpuResidencyBudgetResource {
            max_resident_entries: 1,
        };
        let selections = [
            selection("a", 1, 1, 10, true),
            selection("b", 2, 1, 20, true),
        ];

        let summary = residency.derive_from_selections(&selections, &budget);

        assert_eq!(summary.resident_count, 2);
        assert_eq!(summary.evicted_count, 0);
        assert_eq!(summary.diagnostic_count, 1);
    }
}
