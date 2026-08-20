pub mod backends;
pub mod block_cache;
pub mod block_loader;
pub mod block_prefetch;
pub mod block_request;
pub mod block_store;
pub mod data_source;
pub mod dataset;
pub mod dataset_manager;
pub mod matrix_data;
pub mod metadata;
pub mod octant_block;
pub mod procedural;
pub mod pyramid;
pub mod resampler;
pub mod slice_request;
pub mod source_factory;
pub mod store_handle;
pub mod volume_data;

pub use metadata::{DatasetMetadata, VariableInfo};

pub use block_cache::{BlockCache, BlockCacheKey};
pub use block_loader::{BlockBatchOutcome, BlockLoadOutcome, BlockLoader};
pub use block_prefetch::{BlockPrefetcher, PrefetchResult};
pub use block_request::{BlockRequest, BlockRequestBatch, BlockResult};
pub use block_store::{BlockStore, BlockStoreError};
pub use data_source::{DataSource, DataSourceKind};
pub use dataset::Dataset;
pub use dataset_manager::DatasetManager;
pub use matrix_data::MatrixData;
pub use octant_block::OctantBlock;
pub use procedural::{
    KnownTruth4DParams, eval_known_truth_4d, generate_known_truth_4d_block,
    generate_procedural_matrix, generate_procedural_volume_3d, generate_procedural_volume_4d,
    get_known_truth_4d_center,
};
pub use pyramid::{AggregationOp, MatrixPyramid, PyramidLevel};
pub use resampler::{ViewportRequest, ViewportResampler};
pub use slice_request::{DimensionSelection, SliceRequest};
pub use source_factory::SourceFactory;
pub use store_handle::StoreHandle;
pub use volume_data::VolumeData;
