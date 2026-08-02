/// 3D Block-like / Voxel Grid Rendering Plot.
///
/// Renders 3D scalar data or 2D matrix grids as extruded 3D blocks/voxels.
pub struct BlockPlot {
    pub block_size: f32,
}

impl BlockPlot {
    pub fn new(block_size: f32) -> Self {
        Self { block_size }
    }
}
