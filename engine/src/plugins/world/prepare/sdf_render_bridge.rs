use std::collections::{BTreeMap, BTreeSet};

use product::{ProductIdentity, RenderProductSelection};
use world_sdf::SdfChunkPayload;

use super::super::adapters::resources::SdfChunkStoreResource;
use super::super::build::integration::WorldRuntimeSdfProductCatalogResource;
use crate::plugins::render::features::world::{
    RenderSdfRaymarchAccelerationConfig, RenderSdfRaymarchAccelerationResource,
    RenderSdfResidencyBudgetResource, RenderSdfResidencyResource, RenderSdfResidencySourceResource,
};
use crate::plugins::render::frame::PreparedRenderProductSelectionResource;
use crate::runtime::WorldMut;

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct WorldSdfRenderBridgeStateResource {
    published_product_ids: BTreeSet<ProductIdentity>,
}

impl WorldSdfRenderBridgeStateResource {
    pub fn published_product_ids(&self) -> &BTreeSet<ProductIdentity> {
        &self.published_product_ids
    }
}

pub fn prepare_world_sdf_render_bridge_system(mut world: WorldMut) {
    let catalog = match world.resource::<WorldRuntimeSdfProductCatalogResource>() {
        Ok(catalog) => catalog.clone(),
        Err(_) => return,
    };
    let store = match world.resource::<SdfChunkStoreResource>() {
        Ok(store) => store.clone(),
        Err(_) => return,
    };
    let selections = match world.resource::<PreparedRenderProductSelectionResource>() {
        Ok(selections) => selections.snapshot(),
        Err(_) => return,
    };
    let budget = match world.resource::<RenderSdfResidencyBudgetResource>() {
        Ok(budget) => *budget,
        Err(_) => return,
    };
    if world.resource::<RenderSdfResidencySourceResource>().is_err()
        || world.resource::<RenderSdfResidencyResource>().is_err()
        || world
            .resource::<RenderSdfRaymarchAccelerationResource>()
            .is_err()
    {
        return;
    }

    let previously_published = world
        .resource::<WorldSdfRenderBridgeStateResource>()
        .map(|state| state.published_product_ids.clone())
        .unwrap_or_default();

    let mut valid_sources = BTreeMap::<ProductIdentity, (u64, SdfChunkPayload)>::new();
    let catalog_ids = catalog.products().keys().copied().collect::<BTreeSet<_>>();

    for (product_id, descriptor) in catalog.products() {
        let Some(payload_ref) = descriptor.payload_refs.first() else {
            continue;
        };
        let Some(payload) = store.chunks.get(&payload_ref.chunk_id) else {
            continue;
        };
        if payload.chunk_id != payload_ref.chunk_id
            || payload.chunk_revision != payload_ref.chunk_revision
            || payload.checksum != payload_ref.checksum
        {
            continue;
        }
        let generation = descriptor.product_core().lineage.generation;
        valid_sources.insert(*product_id, (generation, payload.clone()));
    }

    let currently_published = valid_sources.keys().copied().collect::<BTreeSet<_>>();
    if let Ok(sources) = world.resource_mut::<RenderSdfResidencySourceResource>() {
        for product_id in previously_published.difference(&currently_published) {
            sources.remove_product(*product_id);
        }
        for (product_id, (generation, payload)) in &valid_sources {
            sources.upsert_payload(*product_id, *generation, payload.clone());
        }
    } else {
        return;
    }

    if let Ok(state) = world.resource_mut::<WorldSdfRenderBridgeStateResource>() {
        state.published_product_ids = currently_published;
    }

    let filtered_selections = selections
        .into_iter()
        .filter_map(|mut selection| {
            filter_selection_to_world_sdf_products(&mut selection, &catalog_ids);
            if selection.selected_products.is_empty() && selection.residency_requests.is_empty() {
                None
            } else {
                Some(selection)
            }
        })
        .collect::<Vec<_>>();

    let sources = match world.resource::<RenderSdfResidencySourceResource>() {
        Ok(sources) => sources.clone(),
        Err(_) => return,
    };
    if let Ok(residency) = world.resource_mut::<RenderSdfResidencyResource>() {
        residency.derive_from_sources(&filtered_selections, &sources, &budget);
    } else {
        return;
    }

    let residency = match world.resource::<RenderSdfResidencyResource>() {
        Ok(residency) => residency.clone(),
        Err(_) => return,
    };
    if let Ok(acceleration) = world.resource_mut::<RenderSdfRaymarchAccelerationResource>() {
        acceleration.derive_from_residency(
            &residency,
            RenderSdfRaymarchAccelerationConfig::default(),
        );
    }
}

fn filter_selection_to_world_sdf_products(
    selection: &mut RenderProductSelection,
    product_ids: &BTreeSet<ProductIdentity>,
) {
    selection
        .selected_products
        .retain(|selected| product_ids.contains(&selected.product_id));
    selection
        .residency_requests
        .retain(|request| product_ids.contains(&request.product_id));
}
