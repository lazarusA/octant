pub mod backends;
pub mod block_orientation;
#[cfg(test)]
mod block_orientation_tests;
pub mod blocks;
pub mod calibration;
pub mod codecs;
pub mod coordinates;
pub mod dataset;
pub mod metadata;
pub mod octant_block;
pub mod procedural;
pub mod render;
pub mod slice_request;
pub mod slicing;

pub use blocks::{
    BlockBatchOutcome, BlockCache, BlockCacheKey, BlockLoadOutcome, BlockLoader, BlockPrefetcher,
    BlockRequest, BlockRequestBatch, BlockResult, BlockStore, BlockStoreError, PrefetchResult,
    ProgressCallback, VariableCacheSummary,
};
pub use calibration::DataCalibration;
pub use coordinates::{
    CartesianTopology, CoordinateGrid, CurvilinearTopology, DggsEllipsoid, DggsMetadata,
    GridTopology, HealpixTopology, Irregular1DTopology,
};
pub use dataset::{
    DataSource, DataSourceKind, Dataset, DatasetManager, SourceFactory, StoreHandle,
};
pub use metadata::{
    CoordValue, CoordValues, DatasetMetadata, SpacingCheck, VariableInfo, VariableTreeGroup,
};
pub use octant_block::OctantBlock;
pub use procedural::{
    KnownTruth4DParams, eval_known_truth_4d, generate_clenshaw_curtis_2d,
    generate_clenshaw_curtis_coords, generate_gaussian_coords, generate_gaussian_grid_2d,
    generate_known_truth_4d_block, generate_procedural_matrix, generate_procedural_volume_3d,
    generate_procedural_volume_4d, generate_stepped_resolution_2d,
    generate_stepped_resolution_coords, generate_stretched_regional_2d,
    generate_stretched_regional_coords, get_known_truth_4d_center,
};
pub use render::{
    AggregationOp, MatrixData, MatrixPyramid, PyramidLevel, RenderData, ResampledTile,
    SpatialLayout, ViewportRequest, ViewportResampler, VolumeData,
};
pub use slice_request::{DimensionSelection, SliceRequest};
