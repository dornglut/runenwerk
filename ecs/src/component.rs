// src/component.rs
use std::collections::HashMap;
use crate::{Fk, Pk};

#[derive(Debug, Clone)]
pub struct ComponentRow<T> {
	pk: Pk,
	fk: Fk,
	pub(crate) data: T,
}

use std::any::Any;

pub trait ComponentTableTrait {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct ComponentTable<T> {
	pub(crate) rows: Vec<ComponentRow<T>>,
	pub(crate) fk_index: HashMap<Fk, Vec<usize>>,  // FK → row indices
	pk_index: HashMap<Pk, usize>,       // PK → row index
	next_pk: Pk,
}

impl<T: 'static> ComponentTableTrait for ComponentTable<T> {
	fn as_any(&self) -> &dyn Any { self }
	fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl<T> ComponentTable<T> {
	pub(crate) fn new() -> Self {
		Self {
			rows: Vec::new(),
			fk_index: HashMap::new(),
			pk_index: HashMap::new(),
			next_pk: 0,
		}
	}

	pub(crate) fn add(&mut self, fk: Fk, data: T) -> Pk {
		let pk = self.next_pk;
		self.next_pk += 1;

		let row = ComponentRow { pk, fk, data };
		let idx = self.rows.len();
		self.rows.push(row);

		self.pk_index.insert(pk, idx);
		self.fk_index.entry(fk).or_default().push(idx);

		pk
	}

	fn remove_by_pk(&mut self, pk: Pk) -> Option<ComponentRow<T>> {
		if let Some(&idx) = self.pk_index.get(&pk) {
			let row = self.rows.swap_remove(idx);

			// Update pk_index
			self.pk_index.remove(&pk);
			if let Some(last) = self.rows.get(idx) {
				self.pk_index.insert(last.pk, idx);
			}

			// Update fk_index
			if let Some(vec) = self.fk_index.get_mut(&row.fk) {
				vec.retain(|&i| i != idx);
				if vec.is_empty() {
					self.fk_index.remove(&row.fk);
				}
			}

			Some(row)
		} else {
			None
		}
	}

	pub fn get_by_fk(&self, fk: Fk) -> Option<Vec<&ComponentRow<T>>> {
		self.fk_index.get(&fk).map(|indices| indices.iter().map(|&i| &self.rows[i]).collect())
	}
}
