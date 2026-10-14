use crate::app::OctantApp;
use crate::data::AggregationOp;
use crate::i18n::{SettingsKey as K, tr};

/// Each aggregation, its name and what it does.
const AGGREGATIONS: [(AggregationOp, K, K); 4] = [
    (AggregationOp::Mean, K::AggMean, K::AggMeanHover),
    (AggregationOp::Max, K::AggMax, K::AggMaxHover),
    (AggregationOp::Min, K::AggMin, K::AggMinHover),
    (AggregationOp::Nearest, K::AggNearest, K::AggNearestHover),
];

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
            egui::RichText::new(tr(K::Aggregation)).strong(),
        );
        let resp = ui.add_enabled(!is_oversized, checkbox);
        if is_oversized {
            resp.on_hover_text(tr(K::AggregationMandatoryHover));
        } else {
            resp.on_hover_text(tr(K::AggregationHover));
        }
    });
    if app.enable_pyramid_resampling {
        ui.horizontal(|ui| {
            ui.label(tr(K::AggregationMode));
            egui::ComboBox::from_id_salt("settings_resampling_mode_dropdown")
                .selected_text(tr(aggregation_name(app.pyramid_aggregation_op)))
                .show_ui(ui, |ui| {
                    for (op, name, hover) in AGGREGATIONS {
                        ui.selectable_value(&mut app.pyramid_aggregation_op, op, tr(name))
                            .on_hover_text(tr(hover));
                    }
                });
        });
    }

    if (app.enable_pyramid_resampling != prev_resampling || app.pyramid_aggregation_op != prev_op)
        && let Some(mdata) = &app.layers.base.data.matrix
    {
        if app.enable_pyramid_resampling {
            // Aggregation is heatmap only; keep a staged plot type in line.
            app.selected.plot_type = crate::plots::PlotType::Heatmap;
            if mdata.height > 1 {
                let pyramid = std::sync::Arc::new(crate::data::MatrixPyramid::new(
                    &mdata.values,
                    mdata.width,
                    mdata.height,
                    &mdata.dataset_name,
                    app.layers.base.pyramid_op(app.pyramid_aggregation_op),
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

/// The name of aggregation `op`.
fn aggregation_name(op: AggregationOp) -> K {
    match op {
        AggregationOp::Mean => K::AggMean,
        AggregationOp::Max => K::AggMax,
        AggregationOp::Min => K::AggMin,
        AggregationOp::Nearest => K::AggNearest,
    }
}
