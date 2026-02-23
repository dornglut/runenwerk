use std::fmt;
// src/grid/grid2d.rs
use crate::{MeshId, Tile};
use glam::IVec2;
use std::fmt::Debug;

/// Trait for logic cells: must define occupancy
pub trait Occupancy: Copy + Default {
    fn is_wall(&self) -> bool;
}

/// Generic 2D grid
#[derive(Debug)]
pub struct Grid2d<L: Copy + Default> {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<L>,
}

impl<L: Copy + Default> Grid2d<L> {
    /// Create a new grid filled with default logic cells
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![L::default(); width * height],
        }
    }

    /// Check if a coordinate is inside the grid
    pub fn contains(&self, coord: IVec2) -> bool {
        coord.x >= 0
            && coord.y >= 0
            && (coord.x as usize) < self.width
            && (coord.y as usize) < self.height
    }

    fn index_of(&self, coord: IVec2) -> Option<usize> {
        if self.contains(coord) {
            Some(coord.y as usize * self.width + coord.x as usize)
        } else {
            None
        }
    }

    pub fn get(&self, coord: IVec2) -> Option<&L> {
        self.index_of(coord).map(|i| &self.cells[i])
    }

    pub fn get_mut(&mut self, coord: IVec2) -> Option<&mut L> {
        self.index_of(coord).map(|i| &mut self.cells[i])
    }

    /// Iterate over each logic cell
    pub fn for_each_cell<F>(&self, mut f: F)
    where
        F: FnMut(IVec2, &L),
    {
        for y in 0..self.height {
            for x in 0..self.width {
                let coord = IVec2::new(x as i32, y as i32);
                f(coord, &self.cells[y * self.width + x]);
            }
        }
    }

    /// Iterate over each mutable logic cell
    pub fn for_each_cell_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(IVec2, &mut L),
    {
        for y in 0..self.height {
            for x in 0..self.width {
                let coord = IVec2::new(x as i32, y as i32);
                f(coord, &mut self.cells[y * self.width + x]);
            }
        }
    }

    /// Map logic grid to a visual grid
    pub fn map_to_visual<V, F>(&self, mut f: F) -> Vec<Vec<V>>
    where
        F: FnMut(IVec2, &L) -> V,
        V: Copy + Default,
    {
        let mut grid = vec![vec![V::default(); self.width]; self.height];
        for y in 0..self.height {
            for x in 0..self.width {
                let coord = IVec2::new(x as i32, y as i32);
                grid[y][x] = f(coord, &self.cells[y * self.width + x]);
            }
        }
        grid
    }

    /// Generic print method for logic grid
    pub fn print_logic<F>(&self, mut f: F)
    where
        F: FnMut(&L) -> char,
    {
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &self.cells[y * self.width + x];
                print!("{} ", f(cell));
            }
            println!();
        }
    }

    /// Generic print method for visual grid
    pub fn print_visual<V: std::fmt::Display>(&self, visual_grid: &[Vec<V>]) {
        for row in visual_grid {
            for v in row {
                print!("{} ", v); // no closure needed
            }
            println!();
        }
    }
}

/// Generic corner tile
#[derive(Debug, Copy, Clone)]
pub struct CornerTile<V> {
    pub mask: u8,
    pub logic: bool,
    pub visual: V,
}

/// Corner logic for any L: Occupancy
impl<L: Occupancy> Grid2d<L> {
    /// 4-bit mask for top-left corner at `coord`
    /// Bit order: 0=top-left,1=top,2=left,3=current
    pub fn corner_mask(&self, coord: IVec2) -> u8 {
        let offsets = [
            IVec2::new(-1, -1),
            IVec2::new(0, -1),
            IVec2::new(-1, 0),
            IVec2::new(0, 0),
        ];

        offsets.iter().enumerate().fold(0u8, |mask, (i, offset)| {
            let neighbor_coord = coord + *offset;
            if self.get(neighbor_coord).map_or(false, |c| c.is_wall()) {
                mask | (1 << i)
            } else {
                mask
            }
        })
    }
}

impl<L: Occupancy> Grid2d<L> {
    /// Generate corner tiles from logic grid
    pub fn corner_tiles<V, F>(&self, mut f: F) -> Vec<Vec<CornerTile<V>>>
    where
        F: FnMut(u8, bool) -> V,
        V: Copy + Default,
    {
        // corner grid is 1 larger than logic grid
        let mut grid = vec![
            vec![
                CornerTile {
                    mask: 0,
                    logic: false,
                    visual: V::default(),
                };
                self.width + 1
            ];
            self.height + 1
        ];

        for y in 0..=self.height {
            for x in 0..=self.width {
                // ✅ use exact corner coordinates without -1
                let coord = IVec2::new(x as i32, y as i32);
                let mask = self.corner_mask(coord);
                let logic = mask != 0;
                let visual = f(mask, logic);

                grid[y][x] = CornerTile {
                    mask,
                    logic,
                    visual,
                };
            }
        }

        grid
    }

    pub fn mesh_tiles(&self) -> Vec<Vec<Tile<MeshId>>> {
        let mut grid = vec![
            vec![
                Tile {
                    mask: 0,
                    logic: false,
                    visual: MeshId::default(),
                };
                self.width + 1
            ];
            self.height + 1
        ];

        for y in 0..=self.height {
            for x in 0..=self.width {
                let coord = IVec2::new(x as i32, y as i32);
                let mask = self.corner_mask(coord);
                let logic = mask != 0;
                let visual = MeshId::from_mask(mask);

                grid[y][x] = Tile {
                    mask,
                    logic,
                    visual,
                };
            }
        }

        grid
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum CellType {
    #[default]
    Empty,
    Wall,
}

impl Occupancy for CellType {
    fn is_wall(&self) -> bool {
        *self == CellType::Wall
    }
}
