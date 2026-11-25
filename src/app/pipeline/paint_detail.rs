//! A multiscale sphere's detail tile, drawn in its layer's mesh pass with
//! the tile cut out of the coarse mesh.

use super::paint_layer::{channel_draw, probe_start};
use crate::app::OctantApp;
use crate::app::detail::{center_of_edge, edge_of_center};
use crate::app::layers::Layer;
use crate::plots::{Mesh3DUniformParams, MeshDetailDraw};

impl OctantApp {
    /// `layer`'s detail sphere draw call, cutting its tile out of the coarse
    /// mesh (`params.hole_bounds`), when a detail tile is shown on the sphere.
    pub(super) fn sphere_detail_draw(
        &self,
        layer: &Layer,
        params: &mut Mesh3DUniformParams,
    ) -> Option<MeshDetailDraw> {
        if !self.sphere_tiles_drawn() {
            return None;
        }
        let tile = layer.data.detail.tile.as_ref()?;
        let renderer = layer.renderers.detail_sphere.as_ref()?;
        let mut own = Mesh3DUniformParams {
            tile_bounds: tile.uv,
            hole_bounds: [0.0; 4],
            ..*params
        };
        params.hole_bounds = tile.uv;
        if params.coord_mode == 1 {
            let full = layer.data.matrix.as_ref()?;
            let coarse = layer.renderers.sphere.as_ref()?;
            let drawn = (coarse.width, coarse.height);
            regional_tile(params, &mut own, (full.width, full.height), drawn);
        }
        let base = &self.layers.base;
        let heights = (layer.id() != base.id())
            .then(|| base.renderers.detail_sphere.clone())
            .flatten();
        let channels = channel_draw(
            layer,
            &layer.renderers.detail_channels,
            probe_start(tile.probe.as_ref()),
            None,
        );
        Some(MeshDetailDraw {
            renderer: renderer.clone(),
            params: own,
            channels,
            thinning: tile.thinning,
            heights,
        })
    }

    /// Whether the sphere draws its layers' detail tiles. With relief, only
    /// when every drawn overlay has its own tile over the base's (draped on
    /// the base tile's heights): an overlay left coarse would part from the
    /// base's finer relief, so every layer stays coarse until they all have.
    pub(crate) fn sphere_tiles_drawn(&self) -> bool {
        let (mode, disp, _) = self.plot_configs.mesh.heights(true);
        let Some(base) = self.layers.base.data.detail.tile.as_ref() else {
            return true;
        };
        if mode == 0 || disp == 0.0 {
            return true;
        }
        let mut drawn = self.layers.overlays().iter().filter(|o| o.is_drawn());
        drawn.all(|o| {
            o.renderers.detail_sphere.is_some()
                && o.data.detail.tile.as_ref().is_some_and(|t| t.uv == base.uv)
        })
    }
}

/// A regional tile's own edges (`own.lon_bounds`/`lat_bounds`, where
/// sphere.wgsl draws it) and its hole in the drawn (thinned) coarse mesh's
/// fractions (`params.hole_bounds`), from `own.tile_bounds` on the plotted
/// level's `w` x `h` quad: regional cell centers sit on the bounds
/// (`get_lon_lat`).
fn regional_tile(
    params: &mut Mesh3DUniformParams,
    own: &mut Mesh3DUniformParams,
    (w, h): (usize, usize),
    (cw, ch): (usize, usize),
) {
    let [u0, v0, u1, v1] = own.tile_bounds;
    let ([lon0, lon1], [lat0, lat1]) = (params.lon_bounds, params.lat_bounds);
    let lon = |p| lon0 + center_of_edge(p, w) * (lon1 - lon0);
    let lat = |p| lat1 - center_of_edge(p, h) * (lat1 - lat0);
    own.lon_bounds = [lon(u0), lon(u1)];
    own.lat_bounds = [lat(v1), lat(v0)];
    let drawn = |p, n, m| edge_of_center(center_of_edge(p, n), m);
    params.hole_bounds = [
        drawn(u0, w, cw),
        drawn(v0, h, ch),
        drawn(u1, w, cw),
        drawn(v1, h, ch),
    ];
}
