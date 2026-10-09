//! The settings support table against what each renderer reads.

use super::support::{PlotState, Support};
use crate::plots::PlotType;

const ALL_TYPES: [PlotType; 6] = [
    PlotType::Heatmap,
    PlotType::Line,
    PlotType::Surface,
    PlotType::Sphere,
    PlotType::Volume,
    PlotType::PointCloud,
];

fn state(plot_type: PlotType) -> PlotState {
    PlotState {
        plot_type,
        geographic: true,
        ..Default::default()
    }
}

fn volume(algorithm: u32) -> PlotState {
    PlotState {
        volume_algorithm: algorithm,
        ..state(PlotType::Volume)
    }
}

fn overridden(support: Support) -> bool {
    matches!(support, Support::Overridden(_))
}

#[test]
fn plain_colormapped_plots_honor_every_color_setting() {
    for plot_type in ALL_TYPES {
        let s = state(plot_type).support();
        assert_eq!(s.color_range, Support::Yes, "{plot_type:?}");
        assert_eq!(s.color_mapping, Support::Yes, "{plot_type:?}");
        assert_eq!(s.opacity, Support::Yes, "{plot_type:?}");
    }
}

#[test]
fn composites_override_colors_and_opacity_but_keep_nan_color() {
    for plot_type in ALL_TYPES {
        let s = PlotState {
            composite: true,
            translucent: true,
            ..state(plot_type)
        }
        .support();
        assert!(overridden(s.color_range), "{plot_type:?}");
        assert!(overridden(s.color_mapping), "{plot_type:?}");
        assert!(overridden(s.opacity), "{plot_type:?}");
        assert_ne!(s.transparency, Support::Yes, "{plot_type:?}");
        if plot_type != PlotType::Line {
            assert_eq!(s.nan_color, Support::Yes, "{plot_type:?}");
        }
    }
}

#[test]
fn line_color_overrides_keep_the_value_axis() {
    for (custom, all_series) in [(true, false), (false, true), (true, true)] {
        let s = PlotState {
            line_custom_color: custom,
            line_all_series: all_series,
            ..state(PlotType::Line)
        }
        .support();
        assert_eq!(s.color_range, Support::Yes, "range places the line");
        assert!(overridden(s.color_mapping));
        assert!(overridden(s.opacity));
    }
    // The line settings mean nothing to other plots.
    let s = PlotState {
        line_custom_color: true,
        line_all_series: true,
        ..state(PlotType::Heatmap)
    }
    .support();
    assert_eq!(s.color_mapping, Support::Yes);
}

#[test]
fn lines_have_no_nan_color() {
    assert_eq!(state(PlotType::Line).support().nan_color, Support::No);
}

#[test]
fn only_dvr_volumes_draw_translucent_colors() {
    assert_eq!(volume(0).support().opacity, Support::Yes);
    for algorithm in 1..=7 {
        assert!(
            overridden(volume(algorithm).support().opacity),
            "{algorithm}"
        );
    }
}

#[test]
fn indexed_volumes_ignore_the_colormap_range() {
    let s = volume(7).support();
    assert!(overridden(s.color_range));
    assert!(overridden(s.color_mapping));
    assert!(overridden(s.nan_color));
    for algorithm in 0..=6 {
        assert_eq!(volume(algorithm).support().color_range, Support::Yes);
    }
}

#[test]
fn volume_transparency_follows_the_algorithm() {
    for algorithm in [0, 1, 2, 3, 5] {
        assert_eq!(
            volume(algorithm).support().volume_transparency,
            Support::Yes,
            "{algorithm}"
        );
    }
    for algorithm in [4, 6, 7] {
        assert!(
            overridden(volume(algorithm).support().volume_transparency),
            "{algorithm}"
        );
    }
    for plot_type in ALL_TYPES.into_iter().filter(|&t| t != PlotType::Volume) {
        assert_eq!(state(plot_type).support().volume_transparency, Support::No);
    }
}

#[test]
fn mesh_transparency_needs_translucent_colors() {
    for plot_type in [PlotType::Sphere, PlotType::Surface, PlotType::PointCloud] {
        assert!(overridden(state(plot_type).support().transparency));
        let translucent = PlotState {
            translucent: true,
            ..state(plot_type)
        };
        assert_eq!(translucent.support().transparency, Support::Yes);
    }
    for plot_type in [PlotType::Heatmap, PlotType::Line, PlotType::Volume] {
        assert_eq!(state(plot_type).support().transparency, Support::No);
    }
}

#[test]
fn coastlines_follow_the_paint_pass_and_geographic_data() {
    for plot_type in ALL_TYPES {
        let expected = if plot_type.draws_coastlines() {
            Support::Yes
        } else {
            Support::No
        };
        assert_eq!(state(plot_type).support().coastlines, expected);
        let microscopy = PlotState {
            geographic: false,
            ..state(plot_type)
        };
        assert_eq!(microscopy.support().coastlines, Support::No);
    }
}

#[test]
fn aggregation_is_heatmap_only() {
    for plot_type in ALL_TYPES {
        let expected = if plot_type == PlotType::Heatmap {
            Support::Yes
        } else {
            Support::No
        };
        assert_eq!(state(plot_type).support().aggregation, expected);
    }
    // Composites skip the pyramid, but it must stay possible to turn it off.
    let composite = PlotState {
        composite: true,
        ..state(PlotType::Heatmap)
    };
    assert_eq!(composite.support().aggregation, Support::Yes);
}

#[test]
fn camera_controls_are_for_3d_plots() {
    for plot_type in ALL_TYPES {
        assert_eq!(
            state(plot_type).support().camera.is_yes(),
            plot_type.is_3d(),
            "{plot_type:?}"
        );
    }
}

#[test]
fn an_overlay_menu_follows_its_own_heatmap_layer() {
    let mut app = crate::app::OctantApp::default();
    app.selected.plot_type = PlotType::Volume;
    let id = app.layers.push(crate::app::layers::Source::default());
    let overlay = PlotState::of_layer(&app, id);
    assert_eq!(overlay.plot_type, PlotType::Heatmap);
    assert!(!overlay.translucent);
    if let Some(layer) = app.layers.get_mut(id) {
        layer.color.opacity = 0.5;
    }
    assert!(PlotState::of_layer(&app, id).translucent);
    assert!(!PlotState::of_layer(&app, crate::app::layers::LayerId::BASE).translucent);
    assert_eq!(
        PlotState::of_layer(&app, crate::app::layers::LayerId::BASE).plot_type,
        PlotType::Volume
    );
}
