pub use crate::{SchemaCompatibility, SchemaId, SchemaVersion};

#[cfg(feature = "alloc")]
pub use crate::{
    SchemaConstraint, SchemaDescriptor, SchemaField, SchemaPath, SchemaPathSegment, SchemaShape,
    SchemaValue,
};
