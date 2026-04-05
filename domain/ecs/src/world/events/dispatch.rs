// Owner: ecs World Events - Event Channel APIs
use super::types::{
	EventChannelConfig, EventChannelStats, EventChannelStorage, EventLifetime, EventObserver,
	EventObserverNotification, ObserverTrigger, OverflowPolicy,
};
use crate::world::world::World;
use std::any::TypeId;

impl World {
	pub fn has_event_channel<T: 'static>(&self) -> bool {
		self.event_channels.contains_key(&TypeId::of::<T>())
	}

	pub fn ensure_event_channel<T: 'static>(&mut self) -> bool {
		let type_id = TypeId::of::<T>();
		if self.event_channels.contains_key(&type_id) {
			return false;
		}
		self.event_channels
			.insert(type_id, EventChannelStorage::new::<T>());
		true
	}

	pub fn configure_event_channel<T: 'static>(&mut self, config: EventChannelConfig) {
		let type_id = TypeId::of::<T>();
		let channel = self
			.event_channels
			.entry(type_id)
			.or_insert_with(EventChannelStorage::new::<T>);
		channel.config = config;
	}

	pub fn emit_event<T: 'static>(&mut self, event: T) {
		let type_id = TypeId::of::<T>();
		let (event_type_name, emitted_count) = {
			let channel = self
				.event_channels
				.entry(type_id)
				.or_insert_with(EventChannelStorage::new::<T>);
			let config = channel.config;
			let event_type_name = channel.event_type_name;
			let mut dropped = false;
			let mut accepted = false;
			let mut removed_from_front = 0usize;

			{
				let events = channel.events_mut::<T>();
				let before = events.len();
				match config.capacity {
					None => {
						events.push(event);
						accepted = true;
					}
					Some(capacity) => {
						if capacity == 0 {
							dropped = true;
							if matches!(config.overflow, OverflowPolicy::Panic) {
								panic!(
									"event channel overflow for {event_type_name} with capacity=0"
								);
							}
						} else if before < capacity {
							events.push(event);
							accepted = true;
						} else {
							match config.overflow {
								OverflowPolicy::DropOldest => {
									events.remove(0);
									removed_from_front = 1;
									events.push(event);
									dropped = true;
									accepted = true;
								}
								OverflowPolicy::DropNewest => {
									dropped = true;
								}
								OverflowPolicy::Panic => {
									panic!(
										"event channel overflow for {event_type_name} at capacity={capacity}"
									);
								}
							}
						}
					}
				}
			}

			if removed_from_front > 0 {
				channel.advance_sequence_for_removed(removed_from_front);
			}

			channel.emitted = channel.emitted.saturating_add(1);
			if dropped {
				channel.dropped = channel.dropped.saturating_add(1);
			}
			if accepted {
				channel.next_sequence = channel.next_sequence.saturating_add(1);
			}

			(event_type_name, usize::from(accepted))
		};

		if emitted_count > 0 {
			self.trigger_observers(
				type_id,
				event_type_name,
				ObserverTrigger::OnEmit,
				emitted_count,
			);
		}
	}

	pub fn read_events<T: 'static>(&self) -> &[T] {
		self.event_channels
			.get(&TypeId::of::<T>())
			.map(|channel| channel.events_ref::<T>())
			.unwrap_or(&[])
	}

	pub(crate) fn read_events_since<T: 'static>(&self, sequence: u64) -> (&[T], u64) {
		let Some(channel) = self.event_channels.get(&TypeId::of::<T>()) else {
			return (&[], 0);
		};
		(
			channel.events_ref_since::<T>(sequence),
			channel.next_sequence,
		)
	}

	pub fn drain_events<T: 'static>(&mut self) -> Vec<T> {
		let type_id = TypeId::of::<T>();
		let (drained, event_type_name, drained_count) = {
			let Some(channel) = self.event_channels.get_mut(&type_id) else {
				return Vec::new();
			};
			let event_type_name = channel.event_type_name;
			let drained = std::mem::take(channel.events_mut::<T>());
			let drained_count = drained.len();
			if drained_count > 0 {
				channel.advance_sequence_for_removed(drained_count);
				channel.drained = channel.drained.saturating_add(drained_count as u64);
			}
			(drained, event_type_name, drained_count)
		};

		if drained_count > 0 {
			self.trigger_observers(
				type_id,
				event_type_name,
				ObserverTrigger::OnDrain,
				drained_count,
			);
		}

		drained
	}

	pub fn clear_events<T: 'static>(&mut self) -> usize {
		let Some(channel) = self.event_channels.get_mut(&TypeId::of::<T>()) else {
			return 0;
		};
		let removed = channel.clear_any();
		channel.drained = channel.drained.saturating_add(removed as u64);
		removed
	}

	pub fn event_count<T: 'static>(&self) -> usize {
		self.event_channels
			.get(&TypeId::of::<T>())
			.map(|channel| channel.events_ref::<T>().len())
			.unwrap_or(0)
	}

	pub fn event_channel_stats<T: 'static>(&self) -> Option<EventChannelStats> {
		self.event_channels
			.get(&TypeId::of::<T>())
			.map(|channel| EventChannelStats {
				emitted: channel.emitted,
				drained: channel.drained,
				dropped: channel.dropped,
				pending: channel.events_ref::<T>().len(),
			})
	}

	pub fn observe_events<T: 'static>(
		&mut self,
		observer_id: impl Into<String>,
		trigger: ObserverTrigger,
	) -> bool {
		let observer_id = observer_id.into();
		let created = !self.event_observers.contains_key(&observer_id);
		self.event_observers.insert(
			observer_id.clone(),
			EventObserver {
				observer_id,
				event_type: TypeId::of::<T>(),
				trigger,
				invocations: 0,
			},
		);
		created
	}

	pub fn remove_event_observer(&mut self, observer_id: &str) -> bool {
		self.event_observers.remove(observer_id).is_some()
	}

	pub fn event_observer_invocations(&self, observer_id: &str) -> Option<u64> {
		self.event_observers
			.get(observer_id)
			.map(|observer| observer.invocations)
	}

	pub fn drain_event_observer_notifications(&mut self) -> Vec<EventObserverNotification> {
		std::mem::take(&mut self.event_observer_notifications)
	}

	pub fn drain_events_map<T: 'static, U, F>(&mut self, map: F) -> Vec<U>
	where
		F: FnMut(T) -> U,
	{
		self.drain_events::<T>().into_iter().map(map).collect()
	}

	pub fn drain_events_filter<T: 'static, F>(&mut self, mut predicate: F) -> Vec<T>
	where
		F: FnMut(&T) -> bool,
	{
		self.drain_events::<T>()
			.into_iter()
			.filter(|event| predicate(event))
			.collect()
	}

	pub fn finish_event_frame(&mut self) {
		let mut end_of_frame_triggers: Vec<(TypeId, &'static str, usize)> = Vec::new();

		for (type_id, channel) in &mut self.event_channels {
			let pending = channel.events_len_any();
			if pending > 0 {
				end_of_frame_triggers.push((*type_id, channel.event_type_name, pending));
			}
			if matches!(channel.config.lifetime, EventLifetime::FrameTransient) && pending > 0 {
				let removed = channel.clear_any();
				channel.drained = channel.drained.saturating_add(removed as u64);
			}
		}

		for (event_type, event_type_name, pending) in end_of_frame_triggers {
			self.trigger_observers(
				event_type,
				event_type_name,
				ObserverTrigger::EndOfFrame,
				pending,
			);
		}
	}

	pub(crate) fn trigger_observers(
		&mut self,
		event_type: TypeId,
		event_type_name: &'static str,
		trigger: ObserverTrigger,
		event_count: usize,
	) {
		for observer in self.event_observers.values_mut() {
			if observer.event_type != event_type || observer.trigger != trigger {
				continue;
			}

			observer.invocations = observer.invocations.saturating_add(1);
			self.event_observer_notifications
				.push(EventObserverNotification {
					observer_id: observer.observer_id.clone(),
					trigger: trigger.clone(),
					event_type: event_type_name,
					event_count,
				});
		}
	}
}