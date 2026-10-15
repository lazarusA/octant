//! The resolution level a multiscale plot reads: Auto (the finest level that
//! fits the canvas) or one level pinned.

use crate::app::OctantApp;
use crate::app::layers::LevelChoice;
use crate::i18n::{SettingsKey, VariablesKey, fill, tr};

/// The level select, while the plot shows a multiscale variable. Changing it
/// plots again; it is disabled while another variable is staged.
pub(crate) fn show_level_select(app: &mut OctantApp, ui: &mut egui::Ui) {
    let Some(set) = app
        .plotted()
        .variable_info()
        .and_then(|v| v.multiscale.as_ref())
        .map(|r| r.set.clone())
    else {
        return;
    };
    let staged = !app.is_exploring_unplotted_variable();
    let current = app.selected.level;
    let mut chosen = current;
    ui.add_enabled_ui(staged, |ui| {
        ui.horizontal(|ui| {
            ui.label(tr(SettingsKey::ResolutionLevel))
                .on_hover_text(tr(SettingsKey::ResolutionLevelHover));
            let mut shown = [0; 64];
            egui::ComboBox::from_id_salt("settings_resolution_level")
                .selected_text(choice_text(&mut shown, current))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut chosen,
                        LevelChoice::Auto,
                        tr(VariablesKey::LevelAuto),
                    );
                    for level in 0..set.len() {
                        let mut buf = [0; 64];
                        let text = choice_text(&mut buf, LevelChoice::Pinned(level));
                        ui.selectable_value(&mut chosen, LevelChoice::Pinned(level), text);
                    }
                });
        });
    });
    if staged && chosen != current {
        app.selected.level = chosen;
        app.plot_selection();
    }
}

/// The name of level choice `choice`, in `buf`.
fn choice_text(buf: &mut [u8], choice: LevelChoice) -> &str {
    match choice {
        LevelChoice::Auto => tr(VariablesKey::LevelAuto),
        LevelChoice::Pinned(level) => fill(buf, tr(VariablesKey::LevelN), &[&level]),
    }
}
