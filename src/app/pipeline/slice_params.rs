//! The slicer's cut as the surface and sphere read it.

use crate::app::OctantApp;
use crate::app::layers::Layer;
use crate::plots::slice::{SliceUniform, has_relief};
use crate::utils::height::HeightScale;

impl OctantApp {
    /// The slicer's cut for `layer` drawn as a sphere or surface in `mode`
    /// with displacement `disp`, heights flipped or not: its w range as the
    /// normalized heights it keeps (`Slicer::height_range`), open when the
    /// plot draws no relief.
    pub(super) fn mesh_slice(
        &self,
        layer: &Layer,
        sphere: bool,
        (mode, disp, flip): (u32, f32, bool),
    ) -> SliceUniform {
        let scale = HeightScale {
            cmin: layer.color.range_min,
            cmax: layer.color.range_max,
            composite: layer.composite.enabled,
            flip,
        };
        let slicer = &self.plot_configs.slice;
        let heights = slicer.height_range(&scale, has_relief(&scale, sphere, mode, disp));
        slicer.uniform_in([false; 3], Some(heights))
    }
}
