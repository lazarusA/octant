//! Why Octant is called Octant: the About header's cube fills one of the
//! eight octants and moves to the next on each click, and the teaser under
//! the name explains the geometry behind it.

use crate::i18n::{AboutKey, tr};
use crate::ui::hero::{SEQUENCE, draw_octant_widget};

const CUBE_SIZE: f32 = 42.0;

/// The cube with its filled octant, which a click moves on to the next one
/// (the landing screen's order: one axis flips per step).
pub(super) fn show_octant_cube(ui: &mut egui::Ui) {
    let id = egui::Id::unique(("about_octant", "step"));
    let step = ui.data(|d| d.get_temp::<usize>(id)).unwrap_or(0) % SEQUENCE.len();

    let response = draw_octant_widget(ui, CUBE_SIZE, SEQUENCE[step], 0.0, 1.0)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tr(AboutKey::OctantHover));
    if response.clicked() {
        ui.data_mut(|d| d.insert_temp(id, (step + 1) % SEQUENCE.len()));
    }
}

/// The teaser on the name: how the eight octants relate to slicing data.
pub(super) fn show_why_octant(ui: &mut egui::Ui) {
    ui.label(tr(AboutKey::OctantTeaser));
}
