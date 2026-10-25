//! A dimension's header row: active checkbox, name and size, and its role
//! selects.

use crate::app::{AnimationRole, SpatialRole};
use crate::i18n::{VariablesKey as K, tr};
use crate::ui::flow_row;
use crate::utils::stack_str;
use egui::{RichText, TextStyle, Ui};

/// Gap between a dimension's name and its size, about a space wide.
const NAME_GAP: f32 = 3.0;

/// A dimension's name and size, as its row shows them.
#[derive(Clone, Copy)]
pub(super) struct DimName<'a> {
    pub name: &'a str,
    pub size: usize,
}

/// Dimension `i`'s row: its active checkbox, name and size, then its spatial
/// and animation role selects pinned to the right edge, or below the name
/// when they don't fit beside it.
pub(super) fn role_row(
    ui: &mut Ui,
    i: usize,
    dim: DimName<'_>,
    active: &mut bool,
    spatial: &mut SpatialRole,
    anim: &mut AnimationRole,
) {
    let spatial_text = spatial_name(*spatial);
    let anim_text = anim_name(*anim);
    let mut buf = [0; 24];
    let size = stack_str(&mut buf, format_args!("({})", dim.size));
    let key = (dim.name, size, anim_text, spatial_text);
    let widths = flow_row::cached_widths(ui, ("role_row", i), key, |ui| {
        [
            flow_row::checkbox_width(ui)
                + ui.spacing().item_spacing.x
                + flow_row::text_width(ui, dim.name, TextStyle::Small)
                + NAME_GAP
                + flow_row::text_width(ui, size, TextStyle::Small),
            flow_row::combo_width(ui, anim_text),
            flow_row::combo_width(ui, spatial_text),
        ]
    });
    let [leading_w, trailing @ ..] = widths;
    let leading = |ui: &mut Ui| {
        ui.checkbox(active, "");
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = NAME_GAP;
            ui.label(RichText::new(dim.name).strong().small());
            ui.label(RichText::new(size).strong().small());
        });
    };
    // Item 0, the animation select, is the rightmost.
    flow_row::split(ui, leading_w, &trailing, leading, |ui, item| {
        if item == 0 {
            anim_select(ui, i, anim_text, anim);
        } else {
            spatial_select(ui, i, spatial_text, spatial);
        }
    });
}

pub(crate) fn spatial_name(role: SpatialRole) -> &'static str {
    match role {
        SpatialRole::None => tr(K::RoleNone),
        SpatialRole::Grid => tr(K::RoleGrid),
        SpatialRole::X => "X",
        SpatialRole::Y => "Y",
        SpatialRole::Z => "Z",
    }
}

pub(crate) fn anim_name(role: AnimationRole) -> &'static str {
    match role {
        AnimationRole::None => tr(K::RoleNone),
        AnimationRole::Animated => tr(K::RoleAnimated),
    }
}

/// Dimension `i`'s animation role select, showing `selected`.
fn anim_select(ui: &mut Ui, i: usize, selected: &str, anim: &mut AnimationRole) {
    egui::ComboBox::from_id_salt(("anim_role", i))
        .selected_text(selected)
        .show_ui(ui, |ui| {
            for role in [AnimationRole::None, AnimationRole::Animated] {
                ui.selectable_value(anim, role, anim_name(role));
            }
        });
}

/// Dimension `i`'s spatial role select, showing `selected`.
fn spatial_select(ui: &mut Ui, i: usize, selected: &str, spatial: &mut SpatialRole) {
    egui::ComboBox::from_id_salt(("spatial_role", i))
        .selected_text(selected)
        .show_ui(ui, |ui| {
            let roles = [
                SpatialRole::None,
                SpatialRole::Grid,
                SpatialRole::X,
                SpatialRole::Y,
                SpatialRole::Z,
            ];
            for role in roles {
                ui.selectable_value(spatial, role, spatial_name(role));
            }
        });
}
