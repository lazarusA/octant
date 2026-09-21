pub mod backends;
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

// Direct re-exports for backwards compatibility
pub mod data_source {
    pub use crate::data::dataset::source::*;
}
pub mod dataset_manager {
    pub use crate::data::dataset::manager::*;
}
pub mod matrix_data {
    pub use crate::data::render::matrix::*;
}
pub mod pyramid {
    pub use crate::data::render::pyramid::*;
}
pub mod render_data {
    pub use crate::data::render::payload::*;
}
pub mod resampler {
    pub use crate::data::render::resampler::*;
}
pub mod source_factory {
    pub use crate::data::dataset::factory::*;
}
pub mod store_handle {
    pub use crate::data::dataset::handle::*;
}
pub mod volume_data {
    pub use crate::data::render::volume::*;
}

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
pub use metadata::{DatasetMetadata, VariableInfo, VariableTreeGroup};
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
