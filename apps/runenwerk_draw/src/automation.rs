//! Product-owned automation observations for Runenwerk Draw.

use drawing::{DrawingDocument, DrawingDocumentId};
use engine::automation::AutomationOwnerAdapter;

use crate::runtime::resources::DrawingHostResource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawingAutomationTarget {
    pub document_id: DrawingDocumentId,
}

#[derive(Debug)]
pub enum DrawingAutomationCommand {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingAutomationQuery {
    Document,
}

pub struct DrawingAutomationAdapter<'a> {
    host: &'a mut DrawingHostResource,
}

impl<'a> DrawingAutomationAdapter<'a> {
    pub fn new(host: &'a mut DrawingHostResource) -> Self {
        Self { host }
    }

    fn resolve_document(
        &self,
        target: &DrawingAutomationTarget,
    ) -> Result<&DrawingDocument, &'static str> {
        let document = self
            .host
            .app
            .document()
            .ok_or("Draw automation document is unavailable")?;
        if document.document_id != target.document_id {
            return Err("Draw automation document target is missing or stale");
        }
        Ok(document)
    }
}

impl AutomationOwnerAdapter for DrawingAutomationAdapter<'_> {
    type Target = DrawingAutomationTarget;
    type Command = DrawingAutomationCommand;
    type Query = DrawingAutomationQuery;
    type Observation = DrawingDocument;
    type Error = &'static str;

    fn dispatch(
        &mut self,
        _target: &Self::Target,
        command: Self::Command,
    ) -> Result<(), Self::Error> {
        match command {}
    }

    fn query(
        &mut self,
        target: &Self::Target,
        query: Self::Query,
    ) -> Result<Self::Observation, Self::Error> {
        match query {
            DrawingAutomationQuery::Document => Ok(self.resolve_document(target)?.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use drawing::CanvasCoordinate;
    use engine::automation::{
        AppAutomationInputReplayExt, AppAutomationInputTraceExt, AutomationInputReplayOutcome,
        AutomationInputReplaySourceMap, AutomationInputReplayStateAssumption,
        AutomationInputTracePlugin, AutomationInputTraceRecordingWitness, AutomationSession,
        AutomationSessionId, AutomationStepResult, InputSourceId, export_automation_input_trace_v1,
        import_automation_input_trace_v1,
    };
    use engine::prelude::App;
    use native_tablet_input::{
        NativeTabletDelta, NativeTabletEventKind, NativeTabletFrameResource, NativeTabletPacket,
        NativeTabletPosition, NativeTabletSample,
    };

    fn current_target(app: &App) -> DrawingAutomationTarget {
        let host = app
            .world()
            .resource::<DrawingHostResource>()
            .expect("headless Draw should install DrawingHostResource");
        let document = host
            .app
            .document()
            .expect("headless Draw should own its default document");
        DrawingAutomationTarget {
            document_id: document.document_id,
        }
    }

    fn query_document(app: &mut App, target: DrawingAutomationTarget) -> DrawingDocument {
        let host = app
            .world_mut()
            .resource_mut::<DrawingHostResource>()
            .expect("headless Draw should install DrawingHostResource");
        let mut adapter = DrawingAutomationAdapter::new(host);
        let mut session =
            AutomationSession::new(AutomationSessionId::new(72), InputSourceId::new(72_001));
        match session.query_owner(&mut adapter, &target, DrawingAutomationQuery::Document) {
            AutomationStepResult::EffectConfirmed(document) => document,
            other => panic!("Draw document query should confirm product state, got {other:?}"),
        }
    }

    fn screen_position(app: &App, x: f64, y: f64) -> NativeTabletPosition {
        let host = app
            .world()
            .resource::<DrawingHostResource>()
            .expect("headless Draw should install DrawingHostResource");
        let point = host
            .app
            .composition_projection()
            .canvas_view
            .canvas_to_screen(CanvasCoordinate::new(x, y))
            .expect("test canvas point should be visible");
        NativeTabletPosition::new(point.x as f32, point.y as f32)
    }

    fn delta(from: NativeTabletPosition, to: NativeTabletPosition) -> NativeTabletDelta {
        NativeTabletDelta::new(to.x - from.x, to.y - from.y)
    }

    fn push_packet(app: &mut App, packet: NativeTabletPacket) {
        app.world_mut()
            .resource_mut::<NativeTabletFrameResource>()
            .expect("headless Draw should install NativeTabletFrameResource")
            .push_packet(packet);
    }

    #[test]
    fn document_target_fails_closed_when_identity_is_stale() {
        let mut app = crate::runtime::build_headless_app()
            .expect("headless Draw automation fixture should build");
        let stale = DrawingAutomationTarget {
            document_id: DrawingDocumentId::new(u64::MAX),
        };
        let host = app
            .world_mut()
            .resource_mut::<DrawingHostResource>()
            .expect("headless Draw should install DrawingHostResource");
        let mut adapter = DrawingAutomationAdapter::new(host);

        assert!(
            adapter
                .query(&stale, DrawingAutomationQuery::Document)
                .is_err()
        );
    }

    #[test]
    fn persisted_atomic_tablet_trace_replays_to_the_same_drawing_document() {
        let mut recording = crate::runtime::build_headless_app()
            .expect("headless Draw automation fixture should build");
        recording.add_plugin(AutomationInputTracePlugin);
        recording
            .start_automation_input_trace()
            .expect("Draw automation trace should start");

        let target = current_target(&recording);
        let begin_position = screen_position(&recording, 96.0, 96.0);
        let historical_position = screen_position(&recording, 128.0, 112.0);
        let current_position = screen_position(&recording, 176.0, 136.0);
        let end_position = screen_position(&recording, 224.0, 160.0);

        push_packet(
            &mut recording,
            NativeTabletPacket::windows_pointer(
                501,
                NativeTabletEventKind::Down,
                begin_position,
                NativeTabletDelta::ZERO,
            )
            .with_timestamp_micros(1_000)
            .with_pressure(0.40),
        );
        recording = recording
            .run_for_frames(1)
            .expect("tablet begin frame should run");

        push_packet(
            &mut recording,
            NativeTabletPacket::windows_pointer(
                501,
                NativeTabletEventKind::Move,
                current_position,
                delta(historical_position, current_position),
            )
            .with_timestamp_micros(1_200)
            .with_pressure(0.70)
            .with_coalesced_samples([NativeTabletSample::new(
                historical_position,
                delta(begin_position, historical_position),
            )
            .with_timestamp_micros(1_100)
            .with_pressure(0.55)]),
        );
        recording = recording
            .run_for_frames(1)
            .expect("tablet update frame should run");

        push_packet(
            &mut recording,
            NativeTabletPacket::windows_pointer(
                501,
                NativeTabletEventKind::Up,
                end_position,
                delta(current_position, end_position),
            )
            .with_timestamp_micros(1_300)
            .with_pressure(0.30),
        );
        recording = recording
            .run_for_frames(1)
            .expect("tablet end frame should run");

        let recorded_document = query_document(&mut recording, target);
        let trace = recording
            .stop_automation_input_trace()
            .expect("Draw automation trace should stop");

        assert_eq!(trace.frames().len(), 3);
        assert!(trace.trailing_groups().is_empty());
        assert_eq!(trace.frames()[0].groups().len(), 1);
        assert_eq!(trace.frames()[1].groups().len(), 1);
        assert_eq!(trace.frames()[2].groups().len(), 1);
        assert_eq!(
            trace.frames()[1].groups()[0].observations.len(),
            2,
            "the update must remain one atomic tablet group containing history plus current input"
        );

        assert_eq!(recorded_document.strokes.len(), 1);
        let recorded_stroke = &recorded_document.strokes[0];
        assert_eq!(recorded_stroke.samples.len(), 4);
        assert_eq!(
            recorded_stroke
                .samples
                .iter()
                .map(|sample| sample.timestamp_micros)
                .collect::<Vec<_>>(),
            vec![Some(1_000), Some(1_100), Some(1_200), Some(1_300)]
        );
        assert_eq!(
            recorded_stroke
                .samples
                .iter()
                .map(|sample| sample.pressure)
                .collect::<Vec<_>>(),
            vec![Some(0.40), Some(0.55), Some(0.70), Some(0.30)]
        );

        let encoded = export_automation_input_trace_v1(
            &trace,
            AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart,
            None,
        )
        .expect("Draw tablet trace should persist as V1");
        let imported = import_automation_input_trace_v1(encoded.as_bytes())
            .expect("persisted Draw tablet trace should import");
        assert_eq!(
            imported.recording_witness(),
            AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart
        );

        let recorded_source = imported.trace().frames()[0].groups()[0].context.source;
        let replay_source = InputSourceId::new(72_100);
        assert_ne!(recorded_source, replay_source);
        let source_map = AutomationInputReplaySourceMap::new([(recorded_source, replay_source)]);

        let mut replay = crate::runtime::build_headless_app()
            .expect("fresh headless Draw replay target should build");
        let replay_target = current_target(&replay);
        assert_eq!(
            replay_target, target,
            "fresh Draw fixture should resolve the same product-owned document identity"
        );

        let report = replay.replay_automation_input_trace(
            imported.trace(),
            &source_map,
            AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
        );
        assert_eq!(report.outcome(), AutomationInputReplayOutcome::Completed);
        assert_eq!(report.completed_frames(), 3);

        let replayed_document = query_document(&mut replay, replay_target);
        assert_eq!(
            replayed_document, recorded_document,
            "normalized replay should reproduce Draw-owned committed document state"
        );
        assert_eq!(replayed_document.strokes[0].samples.len(), 4);

        replay
            .teardown_automation_input_replay()
            .expect("Draw replay teardown should release only replay-owned input state");
    }
}
