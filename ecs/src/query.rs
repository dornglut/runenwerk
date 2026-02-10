use std::collections::HashSet;
use crate::{World, Entity, ComponentRow};

/// Query two component types
pub fn query2<T1: 'static, T2: 'static>(world: &World) -> Vec<(Entity, (&T1, &T2))> {
	let t1 = match world.get_table::<T1>() { Some(t) => t, None => return vec![] };
	let t2 = match world.get_table::<T2>() { Some(t) => t, None => return vec![] };

	// Find common FKs (entities that have both components)
	let common_fks: HashSet<_> = t1.fk_index.keys()
		.filter(|fk| t2.fk_index.contains_key(fk))
		.cloned()
		.collect();

	common_fks.into_iter().map(|fk| {
		let row1 = &t1.get_by_fk(fk).unwrap()[0];
		let row2 = &t2.get_by_fk(fk).unwrap()[0];
		(Entity { id: fk }, (&row1.data, &row2.data))
	}).collect()
}

/// Query three component types
pub fn query3<T1: 'static, T2: 'static, T3: 'static>(world: &World) -> Vec<(Entity, (&T1, &T2, &T3))> {
	let t1 = match world.get_table::<T1>() { Some(t) => t, None => return vec![] };
	let t2 = match world.get_table::<T2>() { Some(t) => t, None => return vec![] };
	let t3 = match world.get_table::<T3>() { Some(t) => t, None => return vec![] };

	// Find common FKs (entities that have all three components)
	let common_fks: HashSet<_> = t1.fk_index.keys()
		.filter(|fk| t2.fk_index.contains_key(fk) && t3.fk_index.contains_key(fk))
		.cloned()
		.collect();

	common_fks.into_iter().map(|fk| {
		let row1 = &t1.get_by_fk(fk).unwrap()[0];
		let row2 = &t2.get_by_fk(fk).unwrap()[0];
		let row3 = &t3.get_by_fk(fk).unwrap()[0];
		(Entity { id: fk }, (&row1.data, &row2.data, &row3.data))
	}).collect()
}

