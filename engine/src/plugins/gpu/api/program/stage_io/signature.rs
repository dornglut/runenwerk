use super::super::contract_diagnostics::{GpuProgramContractCause, GpuProgramContractError};
use super::super::entry_point::GpuEntryPointName;
use super::GpuShaderIoLocation;
use super::builtin::{
    GpuFragmentOutputBuiltin, GpuVertexInputBuiltin, normalize_fragment_output_builtins,
    normalize_vertex_input_builtins,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct GpuShaderIoSignature {
    locations: Vec<GpuShaderIoLocation>,
}

impl GpuShaderIoSignature {
    fn new(
        role: &'static str,
        locations: impl IntoIterator<Item = GpuShaderIoLocation>,
    ) -> Result<Self, GpuProgramContractError> {
        let mut locations = locations.into_iter().collect::<Vec<_>>();
        locations.sort_by_key(|location| location.location());
        if let Some(duplicate) = locations
            .windows(2)
            .find(|pair| pair[0].location() == pair[1].location())
            .map(|pair| pair[0].location())
        {
            return Err(GpuProgramContractError::invalid(
                "construct GPU shader-stage IO signature",
                format!("{role} location={duplicate}"),
                GpuProgramContractCause::StageIoSignatureInvalid,
                "provide each shader location exactly once",
            ));
        }
        Ok(Self { locations })
    }

    fn locations(&self) -> impl ExactSizeIterator<Item = &GpuShaderIoLocation> {
        self.locations.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuExpectedVertexInputSignature {
    entry_point: GpuEntryPointName,
    signature: GpuShaderIoSignature,
}

impl GpuExpectedVertexInputSignature {
    pub fn new(
        entry_point: GpuEntryPointName,
        locations: impl IntoIterator<Item = GpuShaderIoLocation>,
    ) -> Result<Self, GpuProgramContractError> {
        Ok(Self {
            entry_point,
            signature: GpuShaderIoSignature::new("expected vertex input", locations)?,
        })
    }

    pub fn entry_point(&self) -> &GpuEntryPointName {
        &self.entry_point
    }

    pub fn locations(&self) -> impl ExactSizeIterator<Item = &GpuShaderIoLocation> {
        self.signature.locations()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuObservedVertexInputSignature {
    entry_point: GpuEntryPointName,
    signature: GpuShaderIoSignature,
    builtins: Vec<GpuVertexInputBuiltin>,
}

impl GpuObservedVertexInputSignature {
    pub fn new(
        entry_point: GpuEntryPointName,
        locations: impl IntoIterator<Item = GpuShaderIoLocation>,
        builtins: impl IntoIterator<Item = GpuVertexInputBuiltin>,
    ) -> Result<Self, GpuProgramContractError> {
        Ok(Self {
            entry_point,
            signature: GpuShaderIoSignature::new("observed vertex input", locations)?,
            builtins: normalize_vertex_input_builtins(builtins)?,
        })
    }

    pub fn entry_point(&self) -> &GpuEntryPointName {
        &self.entry_point
    }

    pub fn locations(&self) -> impl ExactSizeIterator<Item = &GpuShaderIoLocation> {
        self.signature.locations()
    }

    pub fn builtins(&self) -> impl ExactSizeIterator<Item = GpuVertexInputBuiltin> + '_ {
        self.builtins.iter().copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuExpectedFragmentOutputSignature {
    entry_point: GpuEntryPointName,
    signature: GpuShaderIoSignature,
}

impl GpuExpectedFragmentOutputSignature {
    pub fn new(
        entry_point: GpuEntryPointName,
        locations: impl IntoIterator<Item = GpuShaderIoLocation>,
    ) -> Result<Self, GpuProgramContractError> {
        Ok(Self {
            entry_point,
            signature: GpuShaderIoSignature::new("expected fragment output", locations)?,
        })
    }

    pub fn entry_point(&self) -> &GpuEntryPointName {
        &self.entry_point
    }

    pub fn locations(&self) -> impl ExactSizeIterator<Item = &GpuShaderIoLocation> {
        self.signature.locations()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuObservedFragmentOutputSignature {
    entry_point: GpuEntryPointName,
    signature: GpuShaderIoSignature,
    builtins: Vec<GpuFragmentOutputBuiltin>,
}

impl GpuObservedFragmentOutputSignature {
    pub fn new(
        entry_point: GpuEntryPointName,
        locations: impl IntoIterator<Item = GpuShaderIoLocation>,
        builtins: impl IntoIterator<Item = GpuFragmentOutputBuiltin>,
    ) -> Result<Self, GpuProgramContractError> {
        Ok(Self {
            entry_point,
            signature: GpuShaderIoSignature::new("observed fragment output", locations)?,
            builtins: normalize_fragment_output_builtins(builtins)?,
        })
    }

    pub fn entry_point(&self) -> &GpuEntryPointName {
        &self.entry_point
    }

    pub fn locations(&self) -> impl ExactSizeIterator<Item = &GpuShaderIoLocation> {
        self.signature.locations()
    }

    pub fn builtins(&self) -> impl ExactSizeIterator<Item = GpuFragmentOutputBuiltin> + '_ {
        self.builtins.iter().copied()
    }
}
