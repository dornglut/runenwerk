use crate::{Archetype, Column};
use std::any::TypeId;

/// Trait to fetch components from a row in an archetype.
pub trait ComponentTuple<'a> {
    type Output;
    fn fetch(archetype: &'a Archetype, row: usize, type_ids: &[TypeId]) -> Option<Self::Output>;
}

// QueryBuilder chains `.with<A>().with<B>()` as type `(B, (A, ()))`.
// These impls map that nested chain into flat output tuples in insertion order.
impl<'a, A: 'static> ComponentTuple<'a> for (A, ()) {
    type Output = (&'a A,);

    fn fetch(archetype: &'a Archetype, row: usize, type_ids: &[TypeId]) -> Option<Self::Output> {
        let a = archetype
            .column_storage(*type_ids.first()?)?
            .as_any()
            .downcast_ref::<Column<A>>()?
            .get(row)?;
        Some((a,))
    }
}

impl<'a, A: 'static, B: 'static> ComponentTuple<'a> for (B, (A, ())) {
    type Output = (&'a A, &'a B);

    fn fetch(archetype: &'a Archetype, row: usize, type_ids: &[TypeId]) -> Option<Self::Output> {
        let a = archetype
            .column_storage(*type_ids.first()?)?
            .as_any()
            .downcast_ref::<Column<A>>()?
            .get(row)?;
        let b = archetype
            .column_storage(*type_ids.get(1)?)?
            .as_any()
            .downcast_ref::<Column<B>>()?
            .get(row)?;
        Some((a, b))
    }
}

impl<'a, A: 'static, B: 'static, C: 'static> ComponentTuple<'a> for (C, (B, (A, ()))) {
    type Output = (&'a A, &'a B, &'a C);

    fn fetch(archetype: &'a Archetype, row: usize, type_ids: &[TypeId]) -> Option<Self::Output> {
        let a = archetype
            .column_storage(*type_ids.first()?)?
            .as_any()
            .downcast_ref::<Column<A>>()?
            .get(row)?;
        let b = archetype
            .column_storage(*type_ids.get(1)?)?
            .as_any()
            .downcast_ref::<Column<B>>()?
            .get(row)?;
        let c = archetype
            .column_storage(*type_ids.get(2)?)?
            .as_any()
            .downcast_ref::<Column<C>>()?
            .get(row)?;
        Some((a, b, c))
    }
}

impl<'a, A: 'static, B: 'static, C: 'static, D: 'static> ComponentTuple<'a> for (D, (C, (B, (A, ())))) {
    type Output = (&'a A, &'a B, &'a C, &'a D);

    fn fetch(archetype: &'a Archetype, row: usize, type_ids: &[TypeId]) -> Option<Self::Output> {
        let a = archetype
            .column_storage(*type_ids.first()?)?
            .as_any()
            .downcast_ref::<Column<A>>()?
            .get(row)?;
        let b = archetype
            .column_storage(*type_ids.get(1)?)?
            .as_any()
            .downcast_ref::<Column<B>>()?
            .get(row)?;
        let c = archetype
            .column_storage(*type_ids.get(2)?)?
            .as_any()
            .downcast_ref::<Column<C>>()?
            .get(row)?;
        let d = archetype
            .column_storage(*type_ids.get(3)?)?
            .as_any()
            .downcast_ref::<Column<D>>()?
            .get(row)?;
        Some((a, b, c, d))
    }
}
