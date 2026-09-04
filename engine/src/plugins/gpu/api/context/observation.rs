use super::descriptor::{
    GpuAdapterClass, GpuAlignmentKind, GpuBackendFamily, GpuContextDescriptor, GpuFormatRole,
    GpuLimitConstraint, GpuLimitKind,
};
use crate::plugins::gpu::GpuTextureFormat;

impl GpuContextDescriptor {
    /// Iterates the normalized backend allowlist in canonical backend-family order.
    ///
    /// An empty iterator means the request does not constrain backend families.
    pub fn backend_allowlist(&self) -> impl ExactSizeIterator<Item = GpuBackendFamily> + '_ {
        self.allowed_backends().iter().copied()
    }

    /// Iterates normalized backend-preference priorities in canonical backend-family order.
    ///
    /// Lower priority values are preferred. Backends omitted from this iterator have no explicit
    /// preference and therefore retain the existing lowest-priority fallback semantics.
    pub fn backend_preference_priorities(
        &self,
    ) -> impl ExactSizeIterator<Item = (GpuBackendFamily, u8)> + '_ {
        self.backend_preference()
            .iter()
            .map(|(&backend, &priority)| (backend, priority))
    }

    /// Iterates the normalized adapter-class allowlist in canonical adapter-class order.
    ///
    /// An empty iterator means the request does not constrain adapter classes.
    pub fn adapter_class_allowlist(&self) -> impl ExactSizeIterator<Item = GpuAdapterClass> + '_ {
        self.allowed_adapter_classes().iter().copied()
    }

    /// Iterates normalized limit constraints in canonical limit-kind order.
    pub fn limit_constraints(
        &self,
    ) -> impl ExactSizeIterator<Item = (GpuLimitKind, GpuLimitConstraint)> + '_ {
        self.limits
            .iter()
            .map(|(&kind, &constraint)| (kind, constraint))
    }

    /// Iterates required texture-format roles in canonical `(format, role)` order.
    pub fn format_role_requirements(
        &self,
    ) -> impl ExactSizeIterator<Item = (GpuTextureFormat, GpuFormatRole)> + '_ {
        self.format_roles.iter().copied()
    }

    /// Iterates normalized maximum alignment requirements in canonical alignment-kind order.
    pub fn alignment_requirements(
        &self,
    ) -> impl ExactSizeIterator<Item = (GpuAlignmentKind, u64)> + '_ {
        self.alignments
            .iter()
            .map(|(&kind, &maximum)| (kind, maximum))
    }
}
