//! Phase 12A executable interaction proof-host core.
//!
//! This module adapts high-level editor input events into normalized input
//! samples and feeds `ui_runtime::InteractionStorySession`. It is proof-host
//! infrastructure only: it does not execute editor commands, mutate product
//! state, open overlays, or perform text editing.

use ui_controls::BaseControlsPlugin;
use ui_input::{
    FocusInputFact, KeyboardInputFact, NormalizedInputFact, NormalizedInputSample,
    PointerInputFact, SemanticInputFact, TextIntentFact, UiInputEvent,
};
use ui_runtime::{
    InteractionBoundaryAssertions, InteractionFormationReport, InteractionProofRenderFrame,
    InteractionReplayLiveParityReport, InteractionStoryExecutionMode, InteractionStoryRunReport,
    InteractionStorySession, InteractionStoryStepEvidence, InteractionVisualProof,
    PHASE12_EXECUTABLE_GENERIC_INTERACTION_STORY_ID,
    phase12_executable_generic_interaction_story_session,
};
use ui_static_mount::UiStaticMountReport;

/// Narrow editor-side proof host for the Phase 12A executable interaction story.
#[derive(Debug, Clone)]
pub struct Phase12aInteractionProofHost {
    session: InteractionStorySession,
}

impl Phase12aInteractionProofHost {
    /// Builds the canonical Phase 12A proof host from base-control descriptors.
    pub fn new() -> Self {
        let compiled = BaseControlsPlugin::new().compile();
        Self {
            session: phase12_executable_generic_interaction_story_session(
                &compiled,
                InteractionStoryExecutionMode::Live,
            ),
        }
    }

    /// Applies one already-normalized input sample.
    pub fn apply_sample(&mut self, sample: NormalizedInputSample) -> InteractionStoryStepEvidence {
        self.session.apply_sample(sample)
    }

    /// Applies one high-level UI input event through the proof-host adapter.
    pub fn apply_input_event(
        &mut self,
        step_id: impl Into<String>,
        event: UiInputEvent,
    ) -> InteractionStoryStepEvidence {
        self.apply_sample(input_event_sample(step_id, event))
    }

    /// Applies an explicit focus sample for proof cases that need focus setup.
    pub fn apply_focus(
        &mut self,
        step_id: impl Into<String>,
        focus: FocusInputFact,
    ) -> InteractionStoryStepEvidence {
        let step_id = step_id.into();
        self.apply_sample(
            NormalizedInputSample::new(step_id).with_fact(NormalizedInputFact::Focus(focus)),
        )
    }

    /// Current descriptor-backed formation report.
    pub fn report(&self) -> &InteractionFormationReport {
        self.session.report()
    }

    /// Current visible proof model.
    pub fn current_proof(&self) -> InteractionVisualProof {
        self.session.current_proof()
    }

    /// Current renderer-neutral proof frame.
    pub fn current_frame(&self) -> InteractionProofRenderFrame {
        self.session.current_frame()
    }

    /// Current no-bypass boundary counters.
    pub fn boundary_assertions(&self) -> InteractionBoundaryAssertions {
        self.session.report().boundary_assertions
    }

    /// Static mount report for the current proof frame.
    pub fn static_mount_report(&self) -> UiStaticMountReport {
        UiStaticMountReport::from_frame(self.current_frame().frame)
    }

    /// Current run report.
    pub fn run_report(&self) -> InteractionStoryRunReport {
        self.session.run_report()
    }

    /// Semantic parity report from replaying the recorded live-shaped input log.
    pub fn replay_live_parity_report(&self) -> InteractionReplayLiveParityReport {
        self.session.replay_live_parity_report()
    }

    /// Stable proof-host story id.
    pub const fn story_id(&self) -> &'static str {
        PHASE12_EXECUTABLE_GENERIC_INTERACTION_STORY_ID
    }
}

impl Default for Phase12aInteractionProofHost {
    fn default() -> Self {
        Self::new()
    }
}

fn input_event_sample(step_id: impl Into<String>, event: UiInputEvent) -> NormalizedInputSample {
    let step_id = step_id.into();
    let fact = match event {
        UiInputEvent::Pointer(pointer) => NormalizedInputFact::Pointer(PointerInputFact {
            kind: pointer.kind,
            position: pointer.position,
            delta: pointer.delta,
            button: pointer.button,
            modifiers: pointer.modifiers,
            click_count: pointer.click_count,
            packet: pointer.packet,
        }),
        UiInputEvent::Keyboard(keyboard) => NormalizedInputFact::Keyboard(KeyboardInputFact {
            key: keyboard.key,
            state: keyboard.state,
            modifiers: keyboard.modifiers,
        }),
        UiInputEvent::Semantic(semantic) => {
            NormalizedInputFact::Semantic(SemanticInputFact::new(semantic))
        }
        UiInputEvent::Text(text) => {
            NormalizedInputFact::TextIntent(TextIntentFact::insert_text(text.text))
        }
    };

    NormalizedInputSample::new(step_id).with_fact(fact)
}
