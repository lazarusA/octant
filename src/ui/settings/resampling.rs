use crate::app::OctantApp;
use crate::data::AggregationOp;
use crate::i18n::{SettingsKey as K, fill, tr};

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

    // A base layer too large for one GPU buffer is always resampled: shown
    // ticked and locked, without changing the setting other layers follow.
    let is_oversized = app
        .layers
        .base
        .data
        .matrix
        .as_ref()
        .is_some_and(|m| crate::app::needs_pyramid(false, m.width.saturating_mul(m.height)));
    let mut resampling = app.enable_pyramid_resampling || is_oversized;

    ui.horizontal(|ui| {
        let checkbox = egui::Checkbox::new(
            &mut resampling,
            egui::RichText::new(tr(K::Aggregation)).strong(),
        );
        let resp = ui.add_enabled(!is_oversized, checkbox);
        if is_oversized {
            let mut buf = [0; 256];
            let limit = crate::plots::limits::max_storage_mb();
            let text = fill(&mut buf, tr(K::AggregationMandatoryHover), &[&limit]);
            resp.on_hover_text(text);
        } else {
            resp.on_hover_text(tr(K::AggregationHover));
        }
    });
    if !is_oversized {
        app.enable_pyramid_resampling = resampling;
    }
    if resampling {
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

    if app.enable_pyramid_resampling != prev_resampling || app.pyramid_aggregation_op != prev_op {
        if app.enable_pyramid_resampling {
            // Aggregation is heatmap only; keep a staged plot type in line.
            app.selected.plot_type = crate::plots::PlotType::Heatmap;
        }
        app.rebuild_layer_pyramids();
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
