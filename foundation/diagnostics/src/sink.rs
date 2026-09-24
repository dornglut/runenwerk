use crate::Diagnostic;

/// Minimal diagnostic sink interface.
///
/// A sink receives diagnostics in emission order. It must not decide domain
/// acceptance policy.
pub trait DiagnosticSink {
    fn emit(&mut self, diagnostic: Diagnostic);
}
