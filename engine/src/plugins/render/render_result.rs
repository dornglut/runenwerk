use super::admission::AdmittedRenderPlan;
use super::deterministic_verification::VerifiedDeterministicRender;
use super::method::RenderMethodId;
use super::request::RenderRequest;
use super::scene::{RenderObjectId, RenderSceneRevision, RenderSceneSnapshot};
use super::semantic_plan::{RenderApplicableRepresentationUse, RenderOutputApproximation};
use super::surface_input::RenderSurfaceSemanticInputBinding;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Renderer-owned evidence that one deterministic finite output satisfied the requested tolerance.
///
/// This witness is deliberately private to result formation. The only constructor is used while
/// consuming one execution-scoped [`VerifiedDeterministicRender`], so an output-index token cannot
/// be detached and reused across submissions or semantic-input substitutions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RenderDeterministicOutputFormationEvidence {
    output_index: usize,
}

impl RenderDeterministicOutputFormationEvidence {
    const fn requested_tolerance_satisfied(output_index: usize) -> Self {
        Self { output_index }
    }

    const fn output_index(self) -> usize {
        self.output_index
    }
}

/// Immutable renderer-semantic representation provenance retained for one completed output.
///
/// Physical bindings, GPU execution evidence, readback identities, and output bytes deliberately do
/// not participate in this semantic result evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderResultObjectRepresentation {
    object_id: RenderObjectId,
    representation: RenderApplicableRepresentationUse,
}

impl RenderResultObjectRepresentation {
    pub const fn object_id(&self) -> RenderObjectId {
        self.object_id
    }

    pub const fn representation(&self) -> RenderApplicableRepresentationUse {
        self.representation
    }
}

/// Semantic evidence for one successfully formed requested output.
///
/// Output-to-observation correlation is intentionally not duplicated here. `RenderResult` retains
/// the exact immutable `RenderRequest`, so callers derive that relation through the requested output
/// at `output_index`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderResultOutputEvidence {
    output_index: usize,
    approximation: RenderOutputApproximation,
    object_representations: Vec<RenderResultObjectRepresentation>,
}

impl RenderResultOutputEvidence {
    pub const fn output_index(&self) -> usize {
        self.output_index
    }

    pub const fn approximation(&self) -> RenderOutputApproximation {
        self.approximation
    }

    pub fn object_representations(&self) -> &[RenderResultObjectRepresentation] {
        &self.object_representations
    }
}

/// Complete RunenRender semantic outcome evidence for one admitted execution.
///
/// This is intentionally not a value container. Output values may remain in physical bindings,
/// retained renderer products, readback results, or presentation destinations. The result retains
/// only immutable semantic provenance projected from the exact admitted plan that produced the work,
/// including the exact selected request-scoped semantic surface inputs once at result level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderResult {
    scene: RenderSceneSnapshot,
    request: RenderRequest,
    method_id: RenderMethodId,
    surface_semantic_inputs: Vec<RenderSurfaceSemanticInputBinding>,
    outputs: Vec<RenderResultOutputEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RenderResultFormationError {
    OutputOutOfRange {
        output_index: usize,
        output_count: usize,
    },
    DuplicateOutput {
        output_index: usize,
    },
    MissingOutput {
        output_index: usize,
    },
}

impl fmt::Display for RenderResultFormationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutputOutOfRange {
                output_index,
                output_count,
            } => write!(
                formatter,
                "formed render output {output_index} is outside {output_count} admitted outputs"
            ),
            Self::DuplicateOutput { output_index } => {
                write!(
                    formatter,
                    "render output {output_index} was formed more than once"
                )
            }
            Self::MissingOutput { output_index } => {
                write!(formatter, "render output {output_index} was not formed")
            }
        }
    }
}

impl Error for RenderResultFormationError {}

impl RenderResult {
    /// Consume one exact execution-scoped EVAL-001 proof and form its semantic result evidence.
    ///
    /// The verified execution is taken by value so its per-output finite-evaluation evidence cannot
    /// be reused after formation. Private output witnesses are minted only inside this owner-controlled
    /// flow from the exact admitted outputs already bound to the verified submission. GPU submission,
    /// readback, decoder, and completion identities are consumed as proof context and remain outside
    /// public `RenderResult` identity.
    pub(super) fn from_verified_deterministic(
        verified: VerifiedDeterministicRender,
    ) -> Result<Self, RenderResultFormationError> {
        let admitted = verified.submitted().admitted().admitted();
        let output_evidence = admitted.outputs().iter().map(|output| {
            RenderDeterministicOutputFormationEvidence::requested_tolerance_satisfied(
                output.output_index(),
            )
        });
        Self::complete_deterministic(admitted, output_evidence)
    }

    /// Form complete semantic result evidence from one internally correlated deterministic proof.
    ///
    /// This raw constructor is private to the module. Callers cannot combine an arbitrary admitted
    /// plan with detached output-index assertions; the only owner-controlled entry above derives the
    /// witness set while consuming the exact `VerifiedDeterministicRender` that established it.
    fn complete_deterministic(
        admitted: &AdmittedRenderPlan,
        output_evidence: impl IntoIterator<Item = RenderDeterministicOutputFormationEvidence>,
    ) -> Result<Self, RenderResultFormationError> {
        let output_count = admitted.outputs().len();
        let mut formed = BTreeSet::new();
        for evidence in output_evidence {
            let output_index = evidence.output_index();
            if output_index >= output_count {
                return Err(RenderResultFormationError::OutputOutOfRange {
                    output_index,
                    output_count,
                });
            }
            if !formed.insert(output_index) {
                return Err(RenderResultFormationError::DuplicateOutput { output_index });
            }
        }

        for output in admitted.outputs() {
            let output_index = output.output_index();
            if !formed.contains(&output_index) {
                return Err(RenderResultFormationError::MissingOutput { output_index });
            }
        }

        let outputs = admitted
            .outputs()
            .iter()
            .map(|output| RenderResultOutputEvidence {
                output_index: output.output_index(),
                approximation: output.approximation(),
                object_representations: output
                    .object_representations()
                    .iter()
                    .map(|object| RenderResultObjectRepresentation {
                        object_id: object.object_id(),
                        representation: object.representation(),
                    })
                    .collect(),
            })
            .collect();

        Ok(Self {
            scene: admitted.plan().scene().clone(),
            request: admitted.plan().request().clone(),
            method_id: admitted.selected_candidate().method_id(),
            surface_semantic_inputs: admitted.surface_semantic_inputs().to_vec(),
            outputs,
        })
    }

    pub const fn scene_revision(&self) -> RenderSceneRevision {
        self.scene.revision()
    }

    pub const fn scene(&self) -> &RenderSceneSnapshot {
        &self.scene
    }

    pub const fn request(&self) -> &RenderRequest {
        &self.request
    }

    pub const fn method_id(&self) -> RenderMethodId {
        self.method_id
    }

    pub fn surface_semantic_inputs(&self) -> &[RenderSurfaceSemanticInputBinding] {
        &self.surface_semantic_inputs
    }

    pub fn outputs(&self) -> &[RenderResultOutputEvidence] {
        &self.outputs
    }
}
