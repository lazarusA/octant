//! Overlay tiles following the base layer's: `sst` over `t2m` in the
//! in-memory GeoZarr pyramid (both levels of the same grid, holding the
//! same values).

use super::tests::{plotted, rest_at, tile_level};
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::app::test_support::poll_until;
use crate::ui::hover::card::{HoverValue, LayerValue};
use crate::ui::hover::detail::detail_cell;
use crate::ui::hover::overlays::{HoverAt, overlay_values};

/// `t2m` plotted at level 2 with `sst` overlaid.
fn overlaid() -> (OctantApp, LayerId) {
    let mut app = plotted();
    let meta = app.plotted().metadata.clone().expect("metadata");
    let sst = meta
        .variables
        .iter()
        .position(|v| v.name.trim_matches('/') == "0/sst")
        .expect("sst");
    let id = app.add_overlay(sst).expect("overlay");
    poll_until(&mut app, |app| {
        app.layers.get(id).is_some_and(|l| l.data.matrix.is_some())
    });
    (app, id)
}

/// Rests at `zoom` and waits for the overlay's tile too.
fn rest_both(app: &mut OctantApp, id: LayerId, zoom: f32) {
    rest_at(app, zoom);
    let waiting = |app: &OctantApp| {
        app.layers
            .get(id)
            .is_some_and(|l| l.data.detail.pending.is_some())
    };
    poll_until(app, |app| !waiting(app));
}

fn overlay_tile(app: &OctantApp, id: LayerId) -> Option<super::DetailTile> {
    app.layers.get(id)?.data.detail.tile.clone()
}

#[test]
fn an_overlay_reads_its_level_over_the_base_tile_cells() {
    let (mut app, id) = overlaid();
    rest_both(&mut app, id, 4.0);
    assert_eq!(tile_level(&app), Some(0));
    let base = app.layers.base.data.detail.tile.clone().expect("base tile");
    let tile = overlay_tile(&app, id).expect("overlay tile");
    assert_eq!(tile.level, 0);
    assert_eq!(
        (tile.origin, tile.end, tile.uv),
        (base.origin, base.end, base.uv)
    );
    let meta = app.plotted().metadata.clone().expect("metadata");
    assert_eq!(meta.variables[tile.var_idx].name.trim_matches('/'), "0/sst");

    rest_both(&mut app, id, 2.0);
    assert_eq!(overlay_tile(&app, id).map(|t| t.level), Some(1));

    rest_both(&mut app, id, 1.0);
    assert!(overlay_tile(&app, id).is_none(), "zoomed out: no base tile");
}

#[test]
fn a_hidden_overlay_has_no_tile() {
    let (mut app, id) = overlaid();
    app.set_layer_visible(id, false);
    rest_both(&mut app, id, 4.0);
    assert_eq!(tile_level(&app), Some(0));
    assert!(overlay_tile(&app, id).is_none());
}

#[test]
fn the_hover_reads_the_overlay_tile_under_the_pointer() {
    let (mut app, id) = overlaid();
    rest_both(&mut app, id, 4.0);
    let norm = (0.47, 0.53);
    let base = detail_cell(&app, norm.0, norm.1).expect("base tile cell");
    let mut rows = [LayerValue::EMPTY; crate::app::overlays::MAX_OVERLAYS];
    let at = HoverAt { cell: (0, 0), norm };
    assert_eq!(overlay_values(&app, at, &mut rows), 1);
    // The same values: the overlay's tile reads what the base tile does.
    assert_eq!(rows[0].value, HoverValue::Scalar(base.value));
}
