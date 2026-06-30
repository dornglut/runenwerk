use ui_controls::BaseControlsPlugin;
use ui_input::{FocusInputFact, KeyboardInputFact, NormalizedInputFact, NormalizedInputSample, PointerInputFact, SemanticInputFact, TextIntentFact, UiInputEvent};
use ui_runtime::{BASE_CONTROLS_EXECUTABLE_INTERACTION_STORY_ID, InteractionBoundaryAssertions, InteractionFormationReport, InteractionProofRenderFrame, InteractionReplayLiveParityReport, InteractionStoryExecutionMode, InteractionStoryRunReport, InteractionStorySession, InteractionStoryStepEvidence, InteractionVisualProof, base_controls_executable_interaction_story_session};
use ui_static_mount::UiStaticMountReport;

#[derive(Debug, Clone, PartialEq)]
pub struct BaseControlsInteractionProofHost {
    session: InteractionStorySession,
}

impl BaseControlsInteractionProofHost {
    pub fn new() -> Self {
        let compiled = BaseControlsPlugin::new().compile();
        Self { session: base_controls_executable_interaction_story_session(&compiled, InteractionStoryExecutionMode::Live) }
    }

    pub fn apply_sample(&mut self, sample: NormalizedInputSample) -> InteractionStoryStepEvidence {
        self.session.apply_sample(sample)
    }

    pub fn apply_input_event(&mut self, step_id: impl Into<String>, event: UiInputEvent) -> InteractionStoryStepEvidence {
        self.apply_sample(input_event_sample(step_id, event))
    }

    pub fn apply_focus(&mut self, step_id: impl Into<String>, focus: FocusInputFact) -> InteractionStoryStepEvidence {
        self.apply_sample(NormalizedInputSample::new(step_id.into()).with_fact(NormalizedInputFact::Focus(focus)))
    }

    pub fn report(&self) -> &InteractionFormationReport { self.session.report() }
    pub fn current_proof(&self) -> InteractionVisualProof { self.session.current_proof() }
    pub fn current_frame(&self) -> InteractionProofRenderFrame { self.session.current_frame() }
    pub fn boundary_assertions(&self) -> InteractionBoundaryAssertions { self.session.report().boundary_assertions }
    pub fn static_mount_report(&self) -> UiStaticMountReport { UiStaticMountReport::from_frame(self.current_frame().frame) }
    pub fn run_report(&self) -> InteractionStoryRunReport { self.session.run_report() }
    pub fn replay_live_parity_report(&self) -> InteractionReplayLiveParityReport { self.session.replay_live_parity_report() }
    pub const fn story_id(&self) -> &'static str { BASE_CONTROLS_EXECUTABLE_INTERACTION_STORY_ID }
}

impl Default for BaseControlsInteractionProofHost {
    fn default() -> Self { Self::new() }
}

fn input_event_sample(step_id: impl Into<String>, event: UiInputEvent) -> NormalizedInputSample {
    let fact = match event {
        UiInputEvent::Pointer(pointer) => NormalizedInputFact::Pointer(PointerInputFact { kind: pointer.kind, position: pointer.position, delta: pointer.delta, button: pointer.button, modifiers: pointer.modifiers, click_count: pointer.click_count, packet: pointer.packet }),
        UiInputEvent::Keyboard(keyboard) => NormalizedInputFact::Keyboard(KeyboardInputFact { key: keyboard.key, state: keyboard.state, modifiers: keyboard.modifiers }),
        UiInputEvent::Semantic(semantic) => NormalizedInputFact::Semantic(SemanticInputFact::new(semantic)),
        UiInputEvent::Text(text) => NormalizedInputFact::TextIntent(TextIntentFact::insert_text(text.text)),
    };
    NormalizedInputSample::new(step_id.into()).with_fact(fact)
}
