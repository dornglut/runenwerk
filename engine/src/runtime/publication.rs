use anyhow::{Result, anyhow};
use runen_ecs::{Resource, World, WorldMut};

/// Provenance for one explicit Runenwerk product-publication phase entry.
///
/// `sequence` is local to one runtime and this publication class. It is not
/// semantic precedence, persistence identity, network identity, product
/// identity, cache identity, an ECS frontier, a physical execution group, or
/// a worker identity.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ProductPublicationOccurrence {
    sequence: u64,
    schedule_label: &'static str,
}

impl ProductPublicationOccurrence {
    pub const fn new(sequence: u64, schedule_label: &'static str) -> Self {
        Self {
            sequence,
            schedule_label,
        }
    }

    pub const fn sequence(self) -> u64 {
        self.sequence
    }

    pub const fn schedule_label(self) -> &'static str {
        self.schedule_label
    }
}

/// Provenance for one explicit Runenwerk query-snapshot-publication phase entry.
///
/// `sequence` is local to one runtime and this publication class. It is not
/// semantic precedence, persistence identity, network identity, product
/// identity, cache identity, an ECS frontier, a physical execution group, or
/// a worker identity. Query sequences are independent from product sequences
/// and have no pairing or equality meaning across publication classes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct QuerySnapshotPublicationOccurrence {
    sequence: u64,
    schedule_label: &'static str,
}

impl QuerySnapshotPublicationOccurrence {
    pub const fn new(sequence: u64, schedule_label: &'static str) -> Self {
        Self {
            sequence,
            schedule_label,
        }
    }

    pub const fn sequence(self) -> u64 {
        self.sequence
    }

    pub const fn schedule_label(self) -> &'static str {
        self.schedule_label
    }
}

type ProductPublicationHandler =
    Box<dyn Fn(&ProductPublicationOccurrence, &mut World) -> Result<()>>;
type QuerySnapshotPublicationHandler =
    Box<dyn Fn(&QuerySnapshotPublicationOccurrence, &mut World) -> Result<()>>;

#[derive(Default)]
pub(crate) struct PublicationHandlers {
    product: Vec<ProductPublicationHandler>,
    query_snapshot: Vec<QuerySnapshotPublicationHandler>,
    next_product_sequence: u64,
    next_query_snapshot_sequence: u64,
}

impl Resource for PublicationHandlers {}

impl PublicationHandlers {
    pub(crate) fn add_product<F>(&mut self, handler: F)
    where
        F: Fn(&ProductPublicationOccurrence, &mut World) -> Result<()> + 'static,
    {
        self.product.push(Box::new(handler));
    }

    pub(crate) fn add_query_snapshot<F>(&mut self, handler: F)
    where
        F: Fn(&QuerySnapshotPublicationOccurrence, &mut World) -> Result<()> + 'static,
    {
        self.query_snapshot.push(Box::new(handler));
    }

    fn begin_product(
        &mut self,
        schedule_label: &'static str,
    ) -> Result<ProductPublicationOccurrence> {
        let sequence = self.next_product_sequence;
        self.next_product_sequence = self
            .next_product_sequence
            .checked_add(1)
            .ok_or_else(|| anyhow!("product publication sequence exhausted"))?;
        Ok(ProductPublicationOccurrence::new(sequence, schedule_label))
    }

    fn begin_query_snapshot(
        &mut self,
        schedule_label: &'static str,
    ) -> Result<QuerySnapshotPublicationOccurrence> {
        let sequence = self.next_query_snapshot_sequence;
        self.next_query_snapshot_sequence = self
            .next_query_snapshot_sequence
            .checked_add(1)
            .ok_or_else(|| anyhow!("query snapshot publication sequence exhausted"))?;
        Ok(QuerySnapshotPublicationOccurrence::new(
            sequence,
            schedule_label,
        ))
    }
}

struct PublicationHandlersLease<'world> {
    world: &'world mut World,
    handlers: Option<PublicationHandlers>,
}

impl<'world> PublicationHandlersLease<'world> {
    fn acquire(world: &'world mut World) -> Self {
        Self {
            handlers: Some(
                world
                    .remove_resource::<PublicationHandlers>()
                    .unwrap_or_default(),
            ),
            world,
        }
    }
}

impl Drop for PublicationHandlersLease<'_> {
    fn drop(&mut self) {
        if let Some(handlers) = self.handlers.take() {
            self.world.insert_resource(handlers);
        }
    }
}

pub fn dispatch_product_publication(world: &mut World, schedule_label: &'static str) -> Result<()> {
    let mut lease = PublicationHandlersLease::acquire(world);
    let occurrence = lease
        .handlers
        .as_mut()
        .expect("publication handler lease should own handlers")
        .begin_product(schedule_label)?;
    let handlers = &lease
        .handlers
        .as_ref()
        .expect("publication handler lease should own handlers")
        .product;
    for handler in handlers {
        handler(&occurrence, lease.world)?;
    }
    Ok(())
}

pub fn dispatch_query_snapshot_publication(
    world: &mut World,
    schedule_label: &'static str,
) -> Result<()> {
    let mut lease = PublicationHandlersLease::acquire(world);
    let occurrence = lease
        .handlers
        .as_mut()
        .expect("publication handler lease should own handlers")
        .begin_query_snapshot(schedule_label)?;
    let handlers = &lease
        .handlers
        .as_ref()
        .expect("publication handler lease should own handlers")
        .query_snapshot;
    for handler in handlers {
        handler(&occurrence, lease.world)?;
    }
    Ok(())
}

pub fn dispatch_product_publication_system(mut world: WorldMut<'_>) -> Result<()> {
    dispatch_product_publication(&mut world, "Update")
}

pub fn dispatch_query_snapshot_publication_system(mut world: WorldMut<'_>) -> Result<()> {
    dispatch_query_snapshot_publication(&mut world, "Update")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn product_and_query_sequences_are_independent() {
        let mut world = World::new();
        world.insert_resource(PublicationHandlers::default());

        let product = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let query = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let product_observations = product.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_product(move |occurrence, _| {
                product_observations
                    .borrow_mut()
                    .push(occurrence.sequence());
                Ok(())
            });
        let query_observations = query.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_query_snapshot(move |occurrence, _| {
                query_observations.borrow_mut().push(occurrence.sequence());
                Ok(())
            });

        dispatch_product_publication(&mut world, "Update").unwrap();
        dispatch_product_publication(&mut world, "Update").unwrap();
        dispatch_query_snapshot_publication(&mut world, "Update").unwrap();
        dispatch_product_publication(&mut world, "Update").unwrap();
        dispatch_query_snapshot_publication(&mut world, "Update").unwrap();

        assert_eq!(&*product.borrow(), &vec![0, 1, 2]);
        assert_eq!(&*query.borrow(), &vec![0, 1]);
    }

    #[test]
    fn empty_dispatch_consumes_each_class_local_sequence() {
        let mut world = World::new();
        world.insert_resource(PublicationHandlers::default());
        dispatch_product_publication(&mut world, "Update").unwrap();
        dispatch_query_snapshot_publication(&mut world, "Update").unwrap();

        let product_observations = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let observed_products = product_observations.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_product(move |occurrence, _| {
                observed_products.borrow_mut().push(occurrence.sequence());
                Ok(())
            });
        let query_observations = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let observed_queries = query_observations.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_query_snapshot(move |occurrence, _| {
                observed_queries.borrow_mut().push(occurrence.sequence());
                Ok(())
            });
        dispatch_product_publication(&mut world, "Update").unwrap();
        dispatch_query_snapshot_publication(&mut world, "Update").unwrap();

        assert_eq!(&*product_observations.borrow(), &vec![1]);
        assert_eq!(&*query_observations.borrow(), &vec![1]);
    }

    #[test]
    fn product_error_preserves_prior_effect_and_registry_for_next_occurrence() {
        let mut world = World::new();
        world.insert_resource(PublicationHandlers::default());

        let first = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let first_observations = first.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_product(move |occurrence, _| {
                first_observations.borrow_mut().push(occurrence.sequence());
                Ok(())
            });
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_product(|occurrence, _| {
                if occurrence.sequence() == 0 {
                    Err(anyhow!("product publication failure"))
                } else {
                    Ok(())
                }
            });
        let third = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let third_observations = third.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_product(move |occurrence, _| {
                third_observations.borrow_mut().push(occurrence.sequence());
                Ok(())
            });

        let error = dispatch_product_publication(&mut world, "Update")
            .expect_err("the second product handler should fail");
        assert!(format!("{error:#}").contains("product publication failure"));
        assert_eq!(&*first.borrow(), &vec![0]);
        assert!(third.borrow().is_empty());
        assert!(world.has_resource::<PublicationHandlers>());

        dispatch_product_publication(&mut world, "Update").unwrap();
        assert_eq!(&*first.borrow(), &vec![0, 1]);
        assert_eq!(&*third.borrow(), &vec![1]);
    }

    #[test]
    fn query_error_preserves_prior_effect_and_registry_for_next_occurrence() {
        let mut world = World::new();
        world.insert_resource(PublicationHandlers::default());

        let first = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let first_observations = first.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_query_snapshot(move |occurrence, _| {
                first_observations.borrow_mut().push(occurrence.sequence());
                Ok(())
            });
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_query_snapshot(|occurrence, _| {
                if occurrence.sequence() == 0 {
                    Err(anyhow!("query publication failure"))
                } else {
                    Ok(())
                }
            });
        let third = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let third_observations = third.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_query_snapshot(move |occurrence, _| {
                third_observations.borrow_mut().push(occurrence.sequence());
                Ok(())
            });

        let error = dispatch_query_snapshot_publication(&mut world, "Update")
            .expect_err("the second query handler should fail");
        assert!(format!("{error:#}").contains("query publication failure"));
        assert_eq!(&*first.borrow(), &vec![0]);
        assert!(third.borrow().is_empty());
        assert!(world.has_resource::<PublicationHandlers>());

        dispatch_query_snapshot_publication(&mut world, "Update").unwrap();
        assert_eq!(&*first.borrow(), &vec![0, 1]);
        assert_eq!(&*third.borrow(), &vec![1]);
    }

    #[test]
    fn product_panic_restores_registry_and_preserves_sequence() {
        let mut world = World::new();
        world.insert_resource(PublicationHandlers::default());

        let first = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let first_observations = first.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_product(move |occurrence, _| {
                first_observations.borrow_mut().push(occurrence.sequence());
                Ok(())
            });
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_product(|occurrence, _| {
                if occurrence.sequence() == 0 {
                    panic!("product publication panic");
                }
                Ok(())
            });
        let third = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let third_observations = third.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_product(move |occurrence, _| {
                third_observations.borrow_mut().push(occurrence.sequence());
                Ok(())
            });

        let caught = catch_unwind(AssertUnwindSafe(|| {
            dispatch_product_publication(&mut world, "Update").unwrap();
        }));
        assert!(caught.is_err());
        assert_eq!(&*first.borrow(), &vec![0]);
        assert!(third.borrow().is_empty());
        assert!(world.has_resource::<PublicationHandlers>());

        dispatch_product_publication(&mut world, "Update").unwrap();
        assert_eq!(&*first.borrow(), &vec![0, 1]);
        assert_eq!(&*third.borrow(), &vec![1]);
    }

    #[test]
    fn query_panic_restores_registry_and_preserves_sequence() {
        let mut world = World::new();
        world.insert_resource(PublicationHandlers::default());

        let first = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let first_observations = first.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_query_snapshot(move |occurrence, _| {
                first_observations.borrow_mut().push(occurrence.sequence());
                Ok(())
            });
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_query_snapshot(|occurrence, _| {
                if occurrence.sequence() == 0 {
                    panic!("query publication panic");
                }
                Ok(())
            });
        let third = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let third_observations = third.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_query_snapshot(move |occurrence, _| {
                third_observations.borrow_mut().push(occurrence.sequence());
                Ok(())
            });

        let caught = catch_unwind(AssertUnwindSafe(|| {
            dispatch_query_snapshot_publication(&mut world, "Update").unwrap();
        }));
        assert!(caught.is_err());
        assert_eq!(&*first.borrow(), &vec![0]);
        assert!(third.borrow().is_empty());
        assert!(world.has_resource::<PublicationHandlers>());

        dispatch_query_snapshot_publication(&mut world, "Update").unwrap();
        assert_eq!(&*first.borrow(), &vec![0, 1]);
        assert_eq!(&*third.borrow(), &vec![1]);
    }
}
