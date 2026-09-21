//! Unit tests for color picker widgets and conversions.

use egui::ecolor::{Hsva, Rgba};

#[test]
fn test_shape_color_picker_hsva_conversion() {
    let rgba = [1.0, 0.0, 0.0, 1.0];
    let hsva = Hsva::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]);
    assert!((hsva.h - 0.0).abs() < 1e-3 || (hsva.h - 1.0).abs() < 1e-3);
    assert!((hsva.s - 1.0).abs() < 1e-3);
    assert!((hsva.v - 1.0).abs() < 1e-3);
    assert!((hsva.a - 1.0).abs() < 1e-3);

    let roundtrip = Rgba::from(hsva).to_rgba_unmultiplied();
    assert!((roundtrip[0] - 1.0).abs() < 1e-3);
    assert!((roundtrip[1] - 0.0).abs() < 1e-3);
    assert!((roundtrip[2] - 0.0).abs() < 1e-3);
    assert!((roundtrip[3] - 1.0).abs() < 1e-3);
}
