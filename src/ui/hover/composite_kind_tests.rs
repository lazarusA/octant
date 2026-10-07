//! Naming RGB band mappings from band names: true color, false color, or a plain RGB
//! composite.

use super::composite::{CompositeKind, classify_rgb};

#[test]
fn band_names_name_the_rgb_mapping() {
    assert_eq!(
        classify_rgb([Some("Red"), Some(" green "), Some("BLUE")]),
        CompositeKind::TrueColor
    );
    assert_eq!(
        classify_rgb([Some("NIR"), Some("Red"), Some("Green")]),
        CompositeKind::FalseColor
    );
    assert_eq!(
        classify_rgb([Some("Red"), None, Some("Blue")]),
        CompositeKind::Rgb
    );
    // Names that contain their color, band codes that name none, colors out of place.
    assert_eq!(
        classify_rgb([Some("Red (B4)"), Some("B3 green"), Some("Blue")]),
        CompositeKind::TrueColor
    );
    assert_eq!(
        classify_rgb([Some("B04"), Some("B03"), Some("B02")]),
        CompositeKind::Rgb
    );
    assert_eq!(
        classify_rgb([Some("Blue"), Some("Green"), Some("Red")]),
        CompositeKind::FalseColor
    );
    assert_eq!(
        classify_rgb([Some("Red edge 1"), Some("Green"), Some("Blue")]),
        CompositeKind::FalseColor
    );
    assert_eq!(
        classify_rgb([Some("SWIR 1"), Some("B8A"), Some("B04")]),
        CompositeKind::FalseColor
    );
}
