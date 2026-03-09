// src/grid/macros.rs
#[macro_export]
macro_rules! grid {
    ( $( [ $( $cell:ident ),* $(,)? ] ),* $(,)? ) => {{
        use crate::grid2d::{Grid2d, CellType};

        let mut cells = Vec::new();
        let mut row_lengths = Vec::new();

        $(
            let mut row_count = 0;
            $(
                let cell_value = match $cell {
                    W => CellType::Wall,
                    E => CellType::Empty,
                    _ => panic!("Unknown cell shorthand `{}`", stringify!($cell)),
                };
                cells.push(cell_value);
                row_count += 1;
            )*
            row_lengths.push(row_count);
        )*

        // Validate row lengths
        let width = if let Some(&first) = row_lengths.first() {
            for &l in &row_lengths {
                if l != first {
                    panic!("All rows in grid! must have the same length");
                }
            }
            first
        } else { 0 };

        let height = row_lengths.len();

        Grid2d::<CellType> { width, height, cells }
    }};
}
