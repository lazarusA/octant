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

mod keys {
    use super::super::ShapeColorPicker;
    use super::super::shape::ColorShape;
    use crate::ui::keyboard::focus;
    use crate::ui::test_input::Harness;
    use egui::{Key, Modifiers, Rect, Ui, pos2};

    const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));
    const SWATCH: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(40.0, 40.0));

    fn run(color: &mut [f32; 4]) -> impl FnMut(&mut Ui) + '_ {
        |ui| {
            focus::begin_frame(ui.ctx());
            ShapeColorPicker::new("test_picker", color, ColorShape::Rect(3.0)).show_at(ui, SWATCH);
        }
    }

    fn popup_open(h: &Harness) -> bool {
        egui::Popup::is_any_open(&h.ctx)
    }

    #[test]
    fn escape_closes_the_popup_and_arrows_change_the_color() {
        let mut color = [1.0, 0.0, 0.0, 1.0];
        let mut h = Harness::new(SCREEN);
        h.settle(1, &mut run(&mut color));
        h.click(SWATCH.center(), &mut run(&mut color));
        h.settle(1, &mut run(&mut color));
        assert!(popup_open(&h), "egui knows the picker is open");
        // Tab through the popup to the alpha strip, its last focusable
        // widget: Shift+Tab from nothing lands there.
        h.key(Key::Tab, Modifiers::SHIFT, &mut run(&mut color));
        for _ in 0..20 {
            h.key(Key::ArrowLeft, Modifiers::NONE, &mut run(&mut color));
        }
        // Each press moves alpha by one hundredth of its range.
        assert!(
            (color[3] - 0.8).abs() < 1e-3,
            "the alpha strip took the arrows: {color:?}"
        );
        h.key(Key::Escape, Modifiers::NONE, &mut run(&mut color));
        h.settle(2, &mut run(&mut color));
        assert!(!popup_open(&h));
    }
}
