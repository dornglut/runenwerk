use super::broadcast::{BroadcastLifetime, BroadcastObserverTrigger};
use crate::world::world::World;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct MessagingFinalizationCounters {
    pub frame_boundaries: u64,
    pub tick_boundaries: u64,
}

impl World {
    pub fn finalize_frame_boundary(&mut self) {
        let mut end_of_frame_triggers: Vec<(std::any::TypeId, &'static str, usize)> = Vec::new();

        for (type_id, stream) in &mut self.broadcast_streams {
            let pending = stream.messages_len_any();
            if pending > 0 {
                end_of_frame_triggers.push((*type_id, stream.stream_type_name, pending));
            }

            if matches!(stream.config.lifetime, BroadcastLifetime::FrameTransient) && pending > 0 {
                let removed = stream.clear_any();
                stream.drained = stream.drained.saturating_add(removed as u64);
            }
        }

        for (stream_type, stream_type_name, pending) in end_of_frame_triggers {
            self.trigger_broadcast_observers(
                stream_type,
                stream_type_name,
                BroadcastObserverTrigger::EndOfFrame,
                pending,
            );
        }

        self.messaging_finalization_counters.frame_boundaries = self
            .messaging_finalization_counters
            .frame_boundaries
            .saturating_add(1);
        self.current_frame_index = self.current_frame_index.saturating_add(1);
    }

    pub fn finalize_tick_boundary(&mut self, finalized_tick: u64) {
        self.current_buffer_tick = finalized_tick;

        for stream in self.tick_buffers.values_mut() {
            if stream.config.retain_finalized_ticks {
                continue;
            }

            let removed = stream.finalize_up_to_any(finalized_tick);
            if removed > 0 {
                stream.pending_messages = stream.pending_messages.saturating_sub(removed);
                stream.drained = stream.drained.saturating_add(removed as u64);
            }
        }

        self.messaging_finalization_counters.tick_boundaries = self
            .messaging_finalization_counters
            .tick_boundaries
            .saturating_add(1);
    }

    pub fn messaging_finalization_counters(&self) -> MessagingFinalizationCounters {
        self.messaging_finalization_counters
    }
}
