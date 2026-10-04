//! Custom colormap editor: colors in CSS gradient syntax, interpolation, blend
//! space and optional hard classes, with a live preview built by `colorgrad`.

use super::swatch::PreviewSwatch;
use crate::app::OctantApp;
use crate::ui::icons::{Icon, IconSize, IconTone, ToolbarButton, UiIconExt};
use crate::utils::colormap::{BlendSpace, CustomColormapSpec, Interpolation};

#[derive(Default)]
pub struct EditorState {
    pub spec: CustomColormapSpec,
    pub preview: PreviewSwatch,
    pub error: Option<String>,
    /// Spec the preview was last built from; `None` forces a rebuild.
    built: Option<CustomColormapSpec>,
}

pub fn show(app: &mut OctantApp, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Custom colormap")
        .id_salt(("colormap_custom_editor", 0))
        .show(ui, |ui| {
            edit_spec(ui, &mut app.colormaps.picker.editor);
            show_actions(app, ui);
            show_saved(app, ui);
        });
}

fn edit_spec(ui: &mut egui::Ui, editor: &mut EditorState) {
    let spec = &mut editor.spec;
    egui::Grid::new(("colormap_editor_grid", 0))
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("Name");
            ui.add(egui::TextEdit::singleline(&mut spec.name).hint_text("my_colormap"));
            ui.end_row();
            ui.label("Colors");
            ui.add(
                egui::TextEdit::singleline(&mut spec.colors).hint_text("navy, 30%, gold, white"),
            )
            .on_hover_text("Hex, rgb(), hsl() or CSS color names, optionally with positions");
            ui.end_row();
            ui.label("Interpolation");
            combo(
                ui,
                "colormap_editor_interp",
                &mut spec.interpolation,
                &Interpolation::ALL,
                |i| i.label(),
            );
            ui.end_row();
            ui.label("Blend");
            combo(
                ui,
                "colormap_editor_blend",
                &mut spec.blend,
                &BlendSpace::ALL,
                |b| b.label(),
            );
            ui.end_row();
            ui.label("Classes");
            ui.add(egui::DragValue::new(&mut spec.classes).range(0..=64))
                .on_hover_text("0 keeps the gradient continuous");
            ui.end_row();
        });
    refresh_preview(ui.ctx(), editor);

    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 14.0), egui::Sense::hover());
    editor.preview.paint(ui.painter(), rect);
    if let Some(err) = &editor.error {
        ui.horizontal(|ui| {
            ui.icon_toned(Icon::Warning, IconSize::Xs, IconTone::Warning);
            ui.label(egui::RichText::new(err).small());
        });
    }
}

fn combo<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    salt: &'static str,
    value: &mut T,
    options: &[T],
    label: fn(T) -> &'static str,
) {
    egui::ComboBox::from_id_salt((salt, 0))
        .selected_text(label(*value))
        .show_ui(ui, |ui| {
            for &opt in options {
                ui.selectable_value(value, opt, label(opt));
            }
        });
}

/// Rebuilds the preview LUT only when the spec changed.
fn refresh_preview(ctx: &egui::Context, editor: &mut EditorState) {
    if editor.built.as_ref() == Some(&editor.spec) {
        return;
    }
    match editor.spec.build_lut() {
        Ok(lut) => {
            editor.preview.set(ctx, &lut);
            editor.error = None;
        }
        Err(e) => {
            editor.preview.clear();
            editor.error = Some(e);
        }
    }
    editor.built = Some(editor.spec.clone());
}

fn show_actions(app: &mut OctantApp, ui: &mut egui::Ui) {
    let can_save = app.colormaps.picker.editor.error.is_none();
    let save = ui
        .add_enabled_ui(can_save, |ui| {
            ui.outlined_icon_button(Icon::Save, "Save & apply", IconTone::Accent)
        })
        .inner;
    if save.clicked() {
        let spec = app.colormaps.picker.editor.spec.clone();
        match app.add_custom_colormap(spec) {
            Ok(id) => super::select_colormap(app, id),
            Err(e) => app.colormaps.picker.editor.error = Some(e),
        }
    }
}

/// Saved custom maps: click a name to edit it, trash to delete it.
fn show_saved(app: &mut OctantApp, ui: &mut egui::Ui) {
    if app.colormaps.custom.is_empty() {
        return;
    }
    ui.add_space(4.0);
    ui.label(egui::RichText::new("Saved").small().weak());
    let mut edit = None;
    let mut delete = None;
    for (i, spec) in app.colormaps.custom.iter().enumerate() {
        ui.horizontal(|ui| {
            if ui.link(&spec.name).on_hover_text("Edit").clicked() {
                edit = Some(i);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let trash = ToolbarButton::new(Icon::Trash, "Delete")
                    .compact(true)
                    .hover("Delete colormap");
                if ui.add(trash).clicked() {
                    delete = Some(i);
                }
            });
        });
    }
    if let Some(spec) = edit.and_then(|i| app.colormaps.custom.get(i)).cloned() {
        app.colormaps.picker.editor.spec = spec;
    }
    if let Some(key) = delete
        .and_then(|i| app.colormaps.custom.get(i))
        .map(CustomColormapSpec::key)
    {
        app.remove_custom_colormap(&key);
    }
}
