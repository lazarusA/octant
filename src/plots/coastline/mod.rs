//! Coastline overlays for 2D flatmaps and 3D globes/surfaces.

pub mod expansion;
pub mod loader;
pub mod renderer_2d;
pub mod renderer_3d;
pub mod types;

pub use expansion::expand_coastline_line_list;
pub use loader::{coastline_110m_static, fetch_coastline_async, load_coastline_sync};
pub use renderer_2d::{CoastlineCallback, CoastlineRenderer};
pub use renderer_3d::{Coastline3DCallback, Coastline3DRenderer};
pub use types::{
    Coastline3DParams, Coastline3DUniforms, CoastlineBuffer, CoastlineFetchResult, CoastlineLod,
    CoastlineReceiver, CoastlineSender, CoastlineUniforms, dataset_geo_bounds,
};
