use crate::components::ansi_grid::AnsiGrid;



pub struct Textgrid<'a> {
    grid: &'a AnsiGrid,
}

impl<'a> Textgrid<'a> {
    pub fn new(grid: &'a AnsiGrid) -> Self {
        Self { grid }
    }
}