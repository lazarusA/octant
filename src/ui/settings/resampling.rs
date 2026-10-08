use crate::app::OctantApp;
use crate::plots::PlotType;

pub(crate) fn show_resampling_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    let prev_resampling = app.enable_pyramid_resampling;
    let prev_op = app.pyramid_aggregation_op;

    let is_oversized = app.layers.base.data.matrix.as_ref().is_some_and(|m| {
        m.width * m.height > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS
    });

    if is_oversized {
        app.enable_pyramid_resampling = true;
    }

    ui.horizontal(|ui| {
        let checkbox = egui::Checkbox::new(
            &mut app.enable_pyramid_resampling,
            egui::RichText::new("2D Aggregation").strong(),
        );
        let resp = ui.add_enabled(!is_oversized, checkbox);
        if is_oversized {
            resp.on_hover_text("Mandatory: Array exceeds single GPU buffer limit (128 MB / 33.5M cells). Aggregation cannot be disabled.");
        } else {
            resp.on_hover_text("Multi-resolution pyramid downsampling and viewport resampling.");
        }
    });
    if app.enable_pyramid_resampling {
        ui.horizontal(|ui| {
            ui.label("Mode:");
            egui::ComboBox::from_id_salt("settings_resampling_mode_dropdown")
                .selected_text(match app.pyramid_aggregation_op {
                    crate::data::AggregationOp::Mean => "Mean (Box Filter)",
                    crate::data::AggregationOp::Max => "Max (Peak Preserve)",
                    crate::data::AggregationOp::Min => "Min (Trough Preserve)",
                    crate::data::AggregationOp::Nearest => "Nearest Neighbor",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app.pyramid_aggregation_op,
                        crate::data::AggregationOp::Mean,
                        "Mean (Box Filter)",
                    )
                    .on_hover_text("Averages sub-pixels in 2x2 cells, ignoring NaNs.");
                    ui.selectable_value(
                        &mut app.pyramid_aggregation_op,
                        crate::data::AggregationOp::Max,
                        "Max (Peak Preserve)",
                    )
                    .on_hover_text("Preserves extreme high values.");
                    ui.selectable_value(
                        &mut app.pyramid_aggregation_op,
                        crate::data::AggregationOp::Min,
                        "Min (Trough Preserve)",
                    )
                    .on_hover_text("Preserves extreme low values.");
                    ui.selectable_value(
                        &mut app.pyramid_aggregation_op,
                        crate::data::AggregationOp::Nearest,
                        "Nearest Neighbor",
                    )
                    .on_hover_text("Fast nearest point selection without interpolation.");
                });
        });
    }

    if (app.enable_pyramid_resampling != prev_resampling || app.pyramid_aggregation_op != prev_op)
        && let Some(mdata) = &app.layers.base.data.matrix
    {
        if app.enable_pyramid_resampling {
            if app.selected.plot_type != PlotType::Heatmap {
                app.selected.plot_type = PlotType::Heatmap;
            }
            if mdata.height > 1 {
                let pyramid = std::sync::Arc::new(crate::data::MatrixPyramid::new(
                    &mdata.values,
                    mdata.width,
                    mdata.height,
                    &mdata.dataset_name,
                    app.pyramid_aggregation_op,
                    512,
                ));
                app.layers
                    .base
                    .data
                    .resampler
                    .set_pyramid(Some(pyramid.clone()));
                app.layers.base.data.pyramid = Some(pyramid);
            }
        } else {
            app.layers.base.data.pyramid = None;
            app.layers.base.data.resampler.set_pyramid(None);
        }
    }
}
