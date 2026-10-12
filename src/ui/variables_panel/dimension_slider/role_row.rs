//! A dimension's header row: active checkbox, name and role selects.

use crate::app::{AnimationRole, SpatialRole};
use crate::i18n::{VariablesKey as K, tr};
use crate::ui::flow_row;
use egui::{RichText, TextStyle, Ui};

/// Dimension `i`'s row: its active checkbox and `name`, then its spatial
/// and animation role selects pinned to the right edge, or below the name
/// when they don't fit beside it.
pub(super) fn role_row(
    ui: &mut Ui,
    i: usize,
    name: &str,
    active: &mut bool,
    spatial: &mut SpatialRole,
    anim: &mut AnimationRole,
) {
    let spatial_text = match spatial {
        SpatialRole::None => tr(K::RoleNone),
        SpatialRole::Grid => tr(K::RoleGrid),
        SpatialRole::X => "X",
        SpatialRole::Y => "Y",
        SpatialRole::Z => "Z",
    };
    let anim_text = match anim {
        AnimationRole::None => tr(K::RoleNone),
        AnimationRole::Animated => tr(K::RoleAnimated),
    };
    let widths =
        flow_row::cached_widths(ui, ("role_row", i), (name, anim_text, spatial_text), |ui| {
            [
                flow_row::checkbox_width(ui)
                    + ui.spacing().item_spacing.x
                    + flow_row::text_width(ui, name, TextStyle::Small),
                flow_row::combo_width(ui, anim_text),
                flow_row::combo_width(ui, spatial_text),
            ]
        });
    let [leading_w, trailing @ ..] = widths;
    flow_row::split(
        ui,
        leading_w,
        &trailing,
        |ui| {
            ui.checkbox(active, "");
            ui.label(RichText::new(name).strong().small());
        },
        // Item 0, the animation select, is the rightmost.
        |ui, item| {
            if item == 0 {
                egui::ComboBox::from_id_salt(("anim_role", i))
                    .selected_text(anim_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(anim, AnimationRole::None, tr(K::RoleNone));
                        ui.selectable_value(anim, AnimationRole::Animated, tr(K::RoleAnimated));
                    });
                return;
            }
            egui::ComboBox::from_id_salt(("spatial_role", i))
                .selected_text(spatial_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(spatial, SpatialRole::None, tr(K::RoleNone));
                    ui.selectable_value(spatial, SpatialRole::Grid, tr(K::RoleGrid));
                    ui.selectable_value(spatial, SpatialRole::X, "X");
                    ui.selectable_value(spatial, SpatialRole::Y, "Y");
                    ui.selectable_value(spatial, SpatialRole::Z, "Z");
                });
        },
    );
}
