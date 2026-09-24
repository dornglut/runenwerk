use crate::RatificationReport;

/// Domain-owned candidate ratifier contract.
///
/// Implementors own the candidate type, context access, issue codes, subjects,
/// and validity rules. The trait only standardizes the returned report shape.
pub trait Ratifier<Candidate> {
    type Code;
    type Subject;

    fn ratify(&self, candidate: &Candidate) -> RatificationReport<Self::Code, Self::Subject>;
}
