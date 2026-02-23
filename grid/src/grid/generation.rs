use crate::*;
use fastnoise_lite::*;

/// For grids where L: Occupancy + Default
impl Grid2d<CellType> {
    /// Fill the grid with random noise-based walls
    /// threshold: 0.0..1.0, higher = more walls
    pub fn fill_with_noise(&mut self, scale: f32, threshold: f32, seed: u32) {
        let mut noise = FastNoiseLite::new();
        noise.set_seed(Some(seed as i32));
        noise.set_noise_type(Some(NoiseType::OpenSimplex2));
        noise.set_fractal_type(Some(FractalType::FBm));
        noise.set_fractal_octaves(Option::from(4));
        noise.set_frequency(Option::from(scale));

        for y in 0..self.height {
            for x in 0..self.width {
                // Domain warp for more organic shapes
                let (wx, wy) = noise.domain_warp_2d(x as f32, y as f32);
                let mut n = (noise.get_noise_2d(wx, wy) + 1.0) / 2.0;

                // Optional: exaggerate walls
                n = (n * 1.3).clamp(0.0, 1.0);

                self.cells[y * self.width + x] = if n > threshold {
                    CellType::Wall
                } else {
                    CellType::Empty
                };
            }
        }
    }
}
