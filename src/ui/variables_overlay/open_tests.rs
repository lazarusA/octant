//! Search focus when the real overlay opens or its dataset changes.

use crate::app::OctantApp;
use crate::data::{DatasetMetadata, VariableInfo};

fn meta(names: &[&str]) -> DatasetMetadata {
    DatasetMetadata {
        name: "test".into(),
        store_type: "zarr".into(),
        variables: names
            .iter()
            .map(|n| VariableInfo {
                name: (*n).into(),
                ..Default::default()
            })
            .collect(),
        dimension_coordinates: Default::default(),
    }
}

/// App with the overlay open on a dataset holding `names`.
fn open_app(names: &[&str]) -> OctantApp {
    OctantApp {
        active_dataset_metadata: Some(meta(names)),
        show_variables_overlay: true,
        ..Default::default()
    }
}

fn run(ctx: &egui::Context, app: &mut OctantApp) {
    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
        let rect = ui.max_rect();
        super::show_variables_overlay(app, ui.ctx(), rect);
    });
    out.textures_delta.clear();
}

fn search_focused(ctx: &egui::Context) -> bool {
    ctx.memory(|m| m.focused()).is_some()
}

fn drop_focus(ctx: &egui::Context) {
    ctx.memory_mut(|m| {
        if let Some(id) = m.focused() {
            m.surrender_focus(id);
        }
    });
}

#[test]
fn opening_focuses_search_for_root_and_folder_only_datasets() {
    for names in [&["a", "b"][..], &["ocean/sst", "land/lai"][..]] {
        let ctx = egui::Context::default();
        let mut app = open_app(names);
        run(&ctx, &mut app);
        run(&ctx, &mut app);
        assert!(search_focused(&ctx), "{names:?}: focused on open");
    }
}

#[test]
fn loading_a_new_dataset_while_open_refocuses_search() {
    let ctx = egui::Context::default();
    let mut app = open_app(&["a", "b"]);
    run(&ctx, &mut app);

    // The user clicks elsewhere, then loads a folder-only dataset; the overlay stays open.
    drop_focus(&ctx);
    app.active_dataset_metadata = None;
    run(&ctx, &mut app);
    app.active_dataset_metadata = Some(meta(&["ocean/sst", "land/lai"]));
    run(&ctx, &mut app);
    assert!(search_focused(&ctx), "new dataset takes focus");

    drop_focus(&ctx);
    run(&ctx, &mut app);
    assert!(
        !search_focused(&ctx),
        "same dataset does not steal focus back"
    );
}

#[test]
fn reopening_the_overlay_refocuses_search() {
    let ctx = egui::Context::default();
    let mut app = open_app(&["ocean/sst"]);
    run(&ctx, &mut app);
    drop_focus(&ctx);

    app.show_variables_overlay = false;
    run(&ctx, &mut app);
    app.show_variables_overlay = true;
    run(&ctx, &mut app);
    assert!(search_focused(&ctx));
}
