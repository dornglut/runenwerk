pub mod allocator;
pub mod tag;
pub mod typed_id;
pub mod typed_id_sequence;

pub use allocator::{GenerationalId, GenerationalIdAllocator, MonotonicIdAllocator};
pub use tag::IdTag;
pub use typed_id::TypedId;
#[allow(deprecated)]
pub use typed_id_sequence::TypedIdSequence;
