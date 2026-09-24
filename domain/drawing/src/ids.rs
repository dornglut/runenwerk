//! File: domain/drawing/src/ids.rs
//! Purpose: Stable typed identifiers for drawing domain contracts.

macro_rules! drawing_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub u64);

        impl $name {
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }

            pub const fn raw(self) -> u64 {
                self.0
            }
        }
    };
}

drawing_id!(DrawingDocumentId);
drawing_id!(LayerStackEntryId);
drawing_id!(StrokeId);
drawing_id!(BrushId);
drawing_id!(PaperId);
drawing_id!(PaintSourceId);
drawing_id!(ReferenceImageId);
drawing_id!(DrawingTileProductId);
drawing_id!(DrawingOperationId);
