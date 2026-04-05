// Owner: ecs World Resource - Resource Access APIs
use crate::component::Resource;
use crate::errors::ResourceError;
use crate::world::world::World;
use std::any::{type_name, Any, TypeId};

impl World {
	pub fn insert_resource<R: Resource>(&mut self, resource: R) {
		let type_id = TypeId::of::<R>();
		let kind = if self.resources.contains_key(&type_id) {
			crate::world::change_tracking::ResourceChangeKind::Modified
		} else {
			crate::world::change_tracking::ResourceChangeKind::Inserted
		};
		self.resources.insert(type_id, Box::new(resource));
		self.record_resource_change(type_id, type_name::<R>(), kind);
	}

	pub fn has_resource<R: Resource>(&self) -> bool {
		self.resources.contains_key(&TypeId::of::<R>())
	}

	pub fn resource<R: Resource>(&self) -> Result<&R, ResourceError> {
		self.resources
			.get(&TypeId::of::<R>())
			.and_then(|res| res.downcast_ref::<R>())
			.ok_or(ResourceError::Missing {
				resource: type_name::<R>(),
			})
	}

	pub fn resource_by_type_id(&self, type_id: TypeId) -> Option<&dyn Any> {
		self.resources.get(&type_id).map(|resource| resource.as_ref())
	}

	pub fn resource_mut<R: Resource>(&mut self) -> Result<&mut R, ResourceError> {
		let type_id = TypeId::of::<R>();
		if !self.resources.contains_key(&type_id) {
			return Err(ResourceError::Missing {
				resource: type_name::<R>(),
			});
		}

		self.record_resource_change(
			type_id,
			type_name::<R>(),
			crate::world::change_tracking::ResourceChangeKind::Modified,
		);

		let value = self
			.resources
			.get_mut(&type_id)
			.and_then(|res| res.downcast_mut::<R>())
			.ok_or(ResourceError::Missing {
				resource: type_name::<R>(),
			})?;

		Ok(value)
	}

	pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
		let type_id = TypeId::of::<R>();
		let removed = self
			.resources
			.remove(&type_id)
			.and_then(|res| res.downcast::<R>().ok().map(|boxed| *boxed));

		if removed.is_some() {
			self.record_resource_change(
				type_id,
				type_name::<R>(),
				crate::world::change_tracking::ResourceChangeKind::Removed,
			);
		}

		removed
	}

	pub fn resource_changed_since<R: Resource>(&self, tick: u64) -> bool {
		self.resource_change_ticks
			.get(&TypeId::of::<R>())
			.is_some_and(|changed| *changed > tick)
	}

	pub fn resource_changes_since(
		&self,
		tick: u64,
	) -> Vec<crate::world::change_tracking::ResourceChangeRecord> {
		self.resource_change_log
			.iter()
			.filter(|change| change.tick > tick)
			.cloned()
			.collect()
	}

	pub(crate) fn record_resource_change(
		&mut self,
		resource_type: TypeId,
		resource_name: &'static str,
		kind: crate::world::change_tracking::ResourceChangeKind,
	) {
		self.change_tick = self.change_tick.saturating_add(1);
		self.resource_change_ticks
			.insert(resource_type, self.change_tick);
		self.resource_change_log
			.push(crate::world::change_tracking::ResourceChangeRecord {
				tick: self.change_tick,
				resource_type,
				resource_name,
				kind,
			});
	}
}