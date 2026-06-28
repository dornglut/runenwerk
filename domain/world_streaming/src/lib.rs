use chunking::{ChunkStreamer, ChunkStreamingConfig, StreamingFocus};
use spatial::{ChunkCoord3, GridPartitionConfig, WorldId};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamRequestId(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StreamRequestKind {
    Load,
    Unload,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkPriority(pub i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StreamRequest {
    pub request_id: StreamRequestId,
    pub world_id: WorldId,
    pub chunk: ChunkCoord3,
    pub kind: StreamRequestKind,
    pub priority: ChunkPriority,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProviderEventKind {
    Loaded,
    LoadFailed,
    Unloaded,
    UnloadFailed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ProviderEvent {
    pub request_id: StreamRequestId,
    pub chunk: ChunkCoord3,
    pub kind: ProviderEventKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WorldStreamingEventKind {
    LoadRequested,
    UnloadRequested,
    Loaded,
    LoadFailed,
    Unloaded,
    UnloadFailed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WorldStreamingEvent {
    pub world_id: WorldId,
    pub chunk: ChunkCoord3,
    pub kind: WorldStreamingEventKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChunkLifecycleState {
    Unloaded,
    Loading,
    Resident,
    Unloading,
    Failed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChunkRuntimeRecord {
    pub chunk: ChunkCoord3,
    pub lifecycle: ChunkLifecycleState,
    pub last_request_id: Option<StreamRequestId>,
}

impl ChunkRuntimeRecord {
    pub fn new(chunk: ChunkCoord3) -> Self {
        Self {
            chunk,
            lifecycle: ChunkLifecycleState::Unloaded,
            last_request_id: None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StreamingBudgets {
    pub max_load_requests_per_tick: usize,
    pub max_unload_requests_per_tick: usize,
}

impl Default for StreamingBudgets {
    fn default() -> Self {
        Self {
            max_load_requests_per_tick: usize::MAX,
            max_unload_requests_per_tick: usize::MAX,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldStreamingConfig {
    pub world_id: WorldId,
    pub partition: GridPartitionConfig,
    pub chunking: ChunkStreamingConfig,
    pub budgets: StreamingBudgets,
}

impl WorldStreamingConfig {
    pub fn new(
        world_id: WorldId,
        partition: GridPartitionConfig,
        chunking: ChunkStreamingConfig,
    ) -> Self {
        Self {
            world_id,
            partition,
            chunking,
            budgets: StreamingBudgets::default(),
        }
    }

    pub fn with_budgets(mut self, budgets: StreamingBudgets) -> Self {
        self.budgets = budgets;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamingTick {
    pub focus: Option<StreamingFocus>,
    pub provider_events: Vec<ProviderEvent>,
}

impl StreamingTick {
    pub fn from_focus(focus: StreamingFocus) -> Self {
        Self {
            focus: Some(focus),
            provider_events: Vec::new(),
        }
    }

    pub fn from_provider_events(provider_events: Vec<ProviderEvent>) -> Self {
        Self {
            focus: None,
            provider_events,
        }
    }

    pub fn with_provider_events(mut self, provider_events: Vec<ProviderEvent>) -> Self {
        self.provider_events = provider_events;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StreamingTickOutput {
    pub requests: Vec<StreamRequest>,
    pub events: Vec<WorldStreamingEvent>,
}

pub struct WorldStreamingController {
    config: WorldStreamingConfig,
    streamer: ChunkStreamer,
    records: BTreeMap<ChunkCoord3, ChunkRuntimeRecord>,
    next_request_id: u64,
}

impl WorldStreamingController {
    pub fn new(config: WorldStreamingConfig) -> Self {
        let streamer = ChunkStreamer::new(config.partition.clone(), config.chunking);

        Self {
            config,
            streamer,
            records: BTreeMap::new(),
            next_request_id: 1,
        }
    }

    pub fn world_id(&self) -> WorldId {
        self.config.world_id
    }

    pub fn config(&self) -> &WorldStreamingConfig {
        &self.config
    }

    pub fn record(&self, chunk: ChunkCoord3) -> Option<&ChunkRuntimeRecord> {
        self.records.get(&chunk)
    }

    pub fn records(&self) -> impl Iterator<Item = &ChunkRuntimeRecord> {
        self.records.values()
    }

    pub fn tick(&mut self, tick: StreamingTick) -> StreamingTickOutput {
        let mut output = StreamingTickOutput::default();

        for event in tick.provider_events {
            self.apply_provider_event(event, &mut output);
        }

        if let Some(focus) = tick.focus {
            let diff = self.streamer.update_focus(focus);

            for chunk in diff
                .entered
                .into_iter()
                .take(self.config.budgets.max_load_requests_per_tick)
            {
                let request = self.request(chunk, StreamRequestKind::Load);
                self.record_mut(chunk).lifecycle = ChunkLifecycleState::Loading;
                self.record_mut(chunk).last_request_id = Some(request.request_id);
                output
                    .events
                    .push(self.event(chunk, WorldStreamingEventKind::LoadRequested));
                output.requests.push(request);
            }

            for chunk in diff
                .exited
                .into_iter()
                .take(self.config.budgets.max_unload_requests_per_tick)
            {
                let request = self.request(chunk, StreamRequestKind::Unload);
                self.record_mut(chunk).lifecycle = ChunkLifecycleState::Unloading;
                self.record_mut(chunk).last_request_id = Some(request.request_id);
                output
                    .events
                    .push(self.event(chunk, WorldStreamingEventKind::UnloadRequested));
                output.requests.push(request);
            }
        }

        output
    }

    fn request(&mut self, chunk: ChunkCoord3, kind: StreamRequestKind) -> StreamRequest {
        let request_id = StreamRequestId(self.next_request_id);
        self.next_request_id += 1;

        StreamRequest {
            request_id,
            world_id: self.config.world_id,
            chunk,
            kind,
            priority: ChunkPriority(0),
        }
    }

    fn record_mut(&mut self, chunk: ChunkCoord3) -> &mut ChunkRuntimeRecord {
        self.records
            .entry(chunk)
            .or_insert_with(|| ChunkRuntimeRecord::new(chunk))
    }

    fn apply_provider_event(&mut self, event: ProviderEvent, output: &mut StreamingTickOutput) {
        let lifecycle = match event.kind {
            ProviderEventKind::Loaded => ChunkLifecycleState::Resident,
            ProviderEventKind::LoadFailed | ProviderEventKind::UnloadFailed => {
                ChunkLifecycleState::Failed
            }
            ProviderEventKind::Unloaded => ChunkLifecycleState::Unloaded,
        };

        self.record_mut(event.chunk).lifecycle = lifecycle;
        self.record_mut(event.chunk).last_request_id = Some(event.request_id);

        let kind = match event.kind {
            ProviderEventKind::Loaded => WorldStreamingEventKind::Loaded,
            ProviderEventKind::LoadFailed => WorldStreamingEventKind::LoadFailed,
            ProviderEventKind::Unloaded => WorldStreamingEventKind::Unloaded,
            ProviderEventKind::UnloadFailed => WorldStreamingEventKind::UnloadFailed,
        };

        output.events.push(self.event(event.chunk, kind));
    }

    fn event(&self, chunk: ChunkCoord3, kind: WorldStreamingEventKind) -> WorldStreamingEvent {
        WorldStreamingEvent {
            world_id: self.config.world_id,
            chunk,
            kind,
        }
    }
}
