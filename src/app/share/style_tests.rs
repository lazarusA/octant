//! A plot's composite and color settings travel in its link: the channels
//! drawn with their tints, windows, curves and inversion, the color range,
//! scale, opacity, reversal and NaN and clip colors.

use super::parse_share;
use super::tests::{app_of, apply};
use crate::app::OctantApp;
use crate::app::actions::{AppAction, ChannelSetting, CompositeSetting};
use crate::data::ome::ChannelFamily;

/// `refl(band, lat, lon)`, its bands named by wavelength.
fn bands_app(target: &str) -> OctantApp {
    use crate::app::test_support::{write_array, write_coordinate};
    use serde_json::json;
    let store = std::sync::Arc::new(zarrs::storage::store::MemoryStore::new());
    write_array(
        &store,
        "refl",
        &["band", "lat", "lon"],
        &[4, 5, 4],
        &[4, 5, 4],
    );
    let nm = json!({ "units": "nm" });
    write_coordinate(&store, "band", &[443.0, 490.0, 560.0, 665.0], nm);
    write_coordinate(&store, "lat", &[10.0, 20.0, 30.0, 40.0, 50.0], json!({}));
    write_coordinate(&store, "lon", &[0.0, 90.0, 180.0, 270.0], json!({}));
    app_of(store, target)
}

#[test]
fn a_composite_and_its_colors_plot_the_same_again() {
    let src = "https://share.test/bands.zarr";
    let mut first = bands_app(src);
    apply(&mut first, &format!("?src={src}&var=refl&band=443..665"));
    let base = &mut first.layers.base;
    base.composite.enabled = true;
    let configs = &mut base.composite.channel_configs;
    assert_eq!(configs.len(), 4);
    for (i, cfg) in configs.iter_mut().enumerate() {
        cfg.visible = i != 0;
    }
    let cfg = &mut configs[2];
    cfg.color_rgb = [255, 128, 0];
    cfg.window = Some((0.5, 2.5));
    cfg.display.family = ChannelFamily::Exponential;
    cfg.display.coefficient = 2.5;
    cfg.display.inverted = true;
    let color = &mut base.color;
    (color.scale_type, color.scale_param) = (2, 0.5);
    color.opacity = 0.5;
    color.reversed = true;
    (color.use_nan_color, color.nan_color) = (true, [1.0, 0.0, 0.0, 1.0]);
    let link = first.share_link().unwrap_or_default();
    for part in [
        "band=443..665",
        "composite=on&rgb=443,490,560&channels=490:",
        "560:ff8000:0.5..2.5:exponential^2.5:inv",
        ":linear,560:",
        "scale=symlog:0.5",
        "opacity=0.5",
        "reverse=on",
        "nan=ff0000ff",
    ] {
        assert!(link.contains(part), "{part} missing from {link}");
    }
    assert!(
        !link.contains("range="),
        "a composite draws no colormap: {link}"
    );

    let mut second = bands_app(src);
    apply(&mut second, &link);
    let (a, b) = (&first.layers.base, &second.layers.base);
    assert!(b.composite.enabled);
    assert_eq!(b.composite.channel_configs, a.composite.channel_configs);
    let (ca, cb) = (&a.color, &b.color);
    assert_eq!(
        (cb.scale_type, cb.scale_param),
        (ca.scale_type, ca.scale_param)
    );
    assert_eq!((cb.opacity, cb.reversed), (ca.opacity, ca.reversed));
    assert_eq!(
        (cb.use_nan_color, cb.nan_color),
        (ca.use_nan_color, ca.nan_color)
    );
    assert_eq!(second.share_link().unwrap_or_default(), link);
}

#[test]
fn a_colormapped_plot_s_range_travels_unlocked_or_not() {
    let src = "https://share.test/range.zarr";
    let mut first = bands_app(src);
    apply(&mut first, &format!("?src={src}&var=refl&band=490"));
    let color = &mut first.layers.base.color;
    (color.range_min, color.range_max) = (-1.5, 2.25);
    let link = first.share_link().unwrap_or_default();
    let off = link.contains("composite=off") && !link.contains("rgb=");
    assert!(off && link.contains("range=-1.5..2.25"), "{link}");
    let mut second = bands_app(src);
    apply(&mut second, &link);
    let color = &second.layers.base.color;
    assert_eq!((color.range_min, color.range_max), (-1.5, 2.25));
    assert!(!second.layers.base.composite.enabled);
}

#[test]
fn composite_keys_read_into_one_action_after_the_colormap() {
    let actions =
        parse_share("?src=https://x/a.tif&var=raster&cmap=k&rgb=B04,B03,B02").unwrap_or_default();
    let cmap = actions
        .iter()
        .position(|a| matches!(a, AppAction::SetColormap { .. }));
    let rgb = actions
        .iter()
        .position(|a| matches!(a, AppAction::SetComposite { .. }));
    assert!(cmap < rgb, "{actions:?}");
    let composite = |link: &str| {
        parse_share(link)
            .unwrap_or_default()
            .into_iter()
            .find_map(|a| match a {
                AppAction::SetComposite { composite } => Some(composite),
                _ => None,
            })
    };
    let bands = composite("?src=https://x/a.tif&var=r&rgb=B04,B03,B02");
    assert_eq!(
        bands.map(|c| c.rgb),
        Some(vec!["B04".into(), "B03".into(), "B02".into()])
    );
    let off = composite("?src=https://x/a.tif&var=r&composite=off");
    assert_eq!(off, Some(CompositeSetting::default()));
    let channels = composite("?src=https://x/a.zarr&var=r&channels=DAPI:0000ff:10..900:inv,%232");
    let expected = vec![
        ChannelSetting {
            channel: "DAPI".into(),
            color: Some("0000ff".into()),
            window_from: Some(10.0),
            window_to: Some(900.0),
            inverted: true,
            ..ChannelSetting::default()
        },
        ChannelSetting {
            channel: "#2".into(),
            ..ChannelSetting::default()
        },
    ];
    assert_eq!(channels.map(|c| c.channels), Some(Some(expected)));
    let none = composite("?src=https://x/a.zarr&var=r&channels=none");
    assert_eq!(none.map(|c| c.channels), Some(Some(Vec::new())));
    let kept = composite("?src=https://x/a.zarr&var=r&composite=on");
    assert_eq!(kept.map(|c| c.channels), Some(None));
}

#[test]
fn a_label_holding_a_separator_is_written_as_its_position() {
    use super::style::separable;
    assert_eq!(separable("GFP: 488".into(), 2), "#3");
    assert_eq!(separable("a,b".into(), 0), "#1");
    assert_eq!(separable("DAPI".into(), 0), "DAPI");
}

#[test]
fn a_failing_channel_changes_nothing() {
    let src = "https://share.test/failing.zarr";
    let mut app = bands_app(src);
    apply(&mut app, &format!("?src={src}&var=refl&band=443..665"));
    let before = app.layers.base.composite.clone();
    let bad = ChannelSetting {
        channel: "490".into(),
        curve: Some("sigmoid".into()),
        ..ChannelSetting::default()
    };
    let composite = CompositeSetting {
        enabled: true,
        rgb: Vec::new(),
        channels: Some(vec![bad]),
    };
    assert!(app.dispatch(AppAction::SetComposite { composite }).is_err());
    assert_eq!(app.layers.base.composite, before);
}
