// src/grid/tile.rs

use std::fmt;

/// Tile types for corners/edges
#[derive(Debug, Copy, Clone, PartialEq, Default, Eq)]
pub enum MeshType {
    #[default]
    Empty,    // 0000
    Corner,   // 0001 | 0010 | 0100 | 1000
    Edge,     // 1100 | 0101 | 0011 | 1010
    T,        // 1110 | 0111 | 1011 | 1101
    Diagonal, // 1001 | 0110
    Full,     // 1111
    Debug,
}

impl fmt::Display for MeshType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            MeshType::Empty => '.',
            MeshType::Corner => 'C',
            MeshType::Edge => '─',
            MeshType::Diagonal => 'D',
            MeshType::T => 'T',
            MeshType::Full => 'F',
            MeshType::Debug => '?',
        };
        write!(f, "{}", c)
    }
}

/// Generic corner tile
#[derive(Debug, Copy, Clone)]
pub struct Tile<V> {
    pub mask: u8,    // 4-bit occupancy mask
    pub logic: bool, // is any neighbor filled
    pub visual: V,   // visual representation (TileType or TileId)
}

impl fmt::Display for Tile<MeshType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.visual)
    }
}

impl fmt::Display for Tile<MeshId> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.visual)
    }
}

/// A TileId is a fully self-contained tile descriptor
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MeshId {
    pub mask: u8,       // 4-bit mask
    pub kind: MeshType, // Tile type
    pub rotation: u16,  // rotation in degrees: 0, 90, 180, 270
}

impl MeshId {
    /// Create a TileId from a 4-bit mask
    pub fn from_mask(mask: u8) -> Self {
        let kind = match mask {
            0b0000 => MeshType::Empty,
            0b0001 | 0b0010 | 0b0100 | 0b1000 => MeshType::Corner,
            0b0011 | 0b0101 | 0b1010 | 0b1100 => MeshType::Edge,
            0b0110 | 0b1001 => MeshType::Diagonal,
            0b0111 | 0b1011 | 0b1101 | 0b1110 => MeshType::T,
            0b1111 => MeshType::Full,
            _ => MeshType::Debug,
        };

        // Determine rotation based on mask
        let rotation = match kind {
            MeshType::Corner => match mask {
                0b0001 => 0,
                0b0010 => 90,
                0b0100 => 180,
                0b1000 => 270,
                _ => 0,
            },
            MeshType::Edge => match mask {
                0b0011 => 0,
                0b0101 => 90,
                0b1010 => 270,
                0b1100 => 180,
                _ => 0,
            },
            MeshType::Diagonal => match mask {
                0b1001 => 0,
                0b0110 => 90,
                _ => 0,
            },
            MeshType::T => match mask {
                0b1110 => 0,
                0b0111 => 90,
                0b1011 => 180,
                0b1101 => 270,
                _ => 0,
            },
            _ => 0,
        };

        MeshId { mask, kind, rotation }
    }

    /// Rotate tile 90° clockwise
    pub fn rotate90(&self) -> Self {
        let new_mask = ((self.mask << 1) | (self.mask >> 3)) & 0b1111; // rotate 4-bit mask
        MeshId::from_mask(new_mask)
    }

    /// Rotate tile by arbitrary multiples of 90°
    pub fn rotate(&self, degrees: u16) -> Self {
        let mut result = *self;
        let steps = (degrees / 90) % 4;
        for _ in 0..steps {
            result = result.rotate90();
        }
        result
    }
}

impl Default for MeshId {
    fn default() -> Self {
        MeshId {
            mask: 0,
            kind: MeshType::Empty,
            rotation: 0,
        }
    }
}

impl fmt::Display for MeshId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Only print the symbol, ignore rotation for grid view
        write!(f, "{}", self.kind)
    }
}

