#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SchemaCompatibility {
    Compatible,
    BackwardCompatible,
    ForwardCompatible,
    Breaking,
    Deprecated,
    #[default]
    Unknown,
}
