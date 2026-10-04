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
    /// Key of the saved map loaded with "Edit"; saving under a new name renames it.
    pub editing_key: Option<String>,
    /// Spec the preview was last built from; `None` forces a rebuild.
    built: Option<CustomColormapSpec>,
}

/// Id of the "Custom colormap" section's open state (stable, so tests can open it).
pub(super) fn section_id(ui: &egui::Ui) -> egui::Id {
    ui.make_persistent_id(("colormap_custom_editor", 0))
}

pub fn show(app: &mut OctantApp, ui: &mut egui::Ui) {
    egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        section_id(ui),
        false,
    )
    .show_header(ui, |ui| ui.label("Custom colormap"))
    .body(|ui| {
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
            choices(ui, &mut spec.interpolation, &Interpolation::ALL, |i| {
                i.label()
            });
            ui.end_row();
            ui.label("Blend");
            choices(ui, &mut spec.blend, &BlendSpace::ALL, |b| b.label());
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
        warning(ui, err);
    }
}

/// Inline selectable options. A dropdown would open its own popup layer, and a
/// click there counts as "outside" the colormap popup and closes it.
fn choices<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    value: &mut T,
    options: &[T],
    label: fn(T) -> &'static str,
) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
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
    let editor = &app.colormaps.picker.editor;
    let renames = editor
        .editing_key
        .as_deref()
        .is_some_and(|key| !editor.spec.has_key(key));
    // Saving an edited map under another saved map's name would replace that map.
    let clashes = renames && app.has_custom_name(&editor.spec.name);
    let can_save = editor.error.is_none() && !editor.spec.name.trim().is_empty() && !clashes;
    let label = if renames {
        "Rename & apply"
    } else {
        "Save & apply"
    };
    let mut rename = None;
    ui.add_enabled_ui(can_save, |ui| {
        ui.horizontal(|ui| {
            if ui
                .outlined_icon_button(Icon::Save, label, IconTone::Accent)
                .clicked()
            {
                rename = Some(renames);
            }
            if renames && ui.button("Save as new").clicked() {
                rename = Some(false);
            }
        });
    });
    if clashes {
        warning(ui, "A saved colormap already has this name");
    }
    if let Some(rename) = rename {
        save(app, rename);
    }
}

/// Saves the editor's spec and applies it; `rename` replaces the map being edited.
fn save(app: &mut OctantApp, rename: bool) {
    let editor = &app.colormaps.picker.editor;
    let spec = editor.spec.clone();
    let key = spec.key();
    let replaces = editor.editing_key.clone().filter(|_| rename);
    match app.save_custom_colormap(spec, replaces.as_deref()) {
        Ok(id) => {
            // Later saves keep editing the map just saved.
            app.colormaps.picker.editor.editing_key = Some(key);
            super::select_colormap(app, id);
        }
        Err(e) => app.colormaps.picker.editor.error = Some(e),
    }
}

fn warning(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.icon_toned(Icon::Warning, IconSize::Xs, IconTone::Warning);
        ui.label(egui::RichText::new(text).small());
    });
}

/// Saved custom maps: click a name to edit it, trash to delete it. Maps that no
/// longer build are listed after them with a warning, to fix or delete.
fn show_saved(app: &mut OctantApp, ui: &mut egui::Ui) {
    let colormaps = &app.colormaps;
    if colormaps.custom.is_empty() && colormaps.unloaded.is_empty() {
        return;
    }
    ui.add_space(4.0);
    ui.label(egui::RichText::new("Saved").small().weak());
    let mut edit = None;
    let mut delete = None;
    let built = colormaps.custom.iter().map(|s| (s, false));
    let broken = colormaps.unloaded.iter().map(|s| (s, true));
    for (spec, broken) in built.chain(broken) {
        ui.horizontal(|ui| {
            if broken {
                ui.icon_toned(Icon::Warning, IconSize::Xs, IconTone::Warning)
                    .on_hover_text("Does not build with this version: edit to fix, or delete");
            }
            if ui.link(&spec.name).on_hover_text("Edit").clicked() {
                edit = Some(spec.clone());
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let trash = ToolbarButton::new(Icon::Trash, "Delete")
                    .compact(true)
                    .hover("Delete colormap");
                if ui.add(trash).clicked() {
                    delete = Some(spec.key());
                }
            });
        });
    }
    if let Some(spec) = edit {
        app.colormaps.picker.editor.editing_key = Some(spec.key());
        app.colormaps.picker.editor.spec = spec;
    }
    if let Some(key) = delete {
        let editor = &mut app.colormaps.picker.editor;
        if editor.editing_key.as_deref() == Some(key.as_str()) {
            editor.editing_key = None;
        }
        app.remove_custom_colormap(&key);
    }
}
