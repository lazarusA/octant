use crate::app::OctantApp;
use crate::i18n::{PlotKey, fill, tr};
use crate::plots::PlotType;
use crate::ui::icons::{Icon, IconSize, IconTone, ToolbarButton, UiIconExt};
use crate::utils::stack_str;

/// Map a PlotType to its corresponding vector Icon.
#[inline]
pub fn plot_type_icon(plot_type: PlotType) -> Icon {
    match plot_type {
        PlotType::Heatmap => Icon::PlotPlane,
        PlotType::Line => Icon::PlotLine,
        PlotType::Sphere => Icon::PlotGlobe,
        PlotType::Surface => Icon::PlotSurface,
        PlotType::Volume => Icon::PlotVolume,
        PlotType::PointCloud => Icon::PlotPointCloud,
    }
}

/// The plot type button and its popup; the button's response.
pub fn show_plot_type_menu(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    compact: bool,
) -> egui::Response {
    let current_icon = plot_type_icon(app.selected.plot_type);
    let current_label = app.selected.plot_type.display_name();
    let button_response = ui.add(
        ToolbarButton::new(current_icon, current_label)
            .compact(compact)
            .owns_popup(),
    );
    egui::Popup::from_toggle_button_response(&button_response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            render_plot_type_contents(app, ui);
        });
    button_response
}

fn render_plot_type_contents(app: &mut OctantApp, ui: &mut egui::Ui) {
    let target_var = app
        .selected_variable_info()
        .or_else(|| app.plotted_variable_info());

    // Label images hold ids, which only the heatmap's label colors read.
    let label_image = target_var.is_some_and(crate::data::VariableInfo::is_label_image);
    let (is_3d_available, is_size_allowed, vol_mb) = if let Some(v) = target_var {
        let has_3d = v.shape.len() >= 3 || v.dimension_names.len() >= 3;
        let vol_elements = crate::ui::variables_panel::calculate_selected_volume_elements(app);
        let size_ok = vol_elements <= crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS;
        let mb = (vol_elements as f64 * 4.0) / (1024.0 * 1024.0);
        (has_3d, size_ok, mb)
    } else {
        (false, false, 0.0)
    };
    let is_discrete_grid = app
        .selected
        .dim_config
        .iter()
        .any(|c| c.spatial == crate::app::SpatialRole::Grid);

    let (supported_plots, grid_name) = if is_discrete_grid {
        (
            Some(
                crate::data::coordinates::CoordinateGrid::Healpix {
                    nside: 1,
                    ordering: crate::data::coordinates::HealpixOrder::Ring,
                    npix: 12,
                    coords_lon: None,
                    coords_lat: None,
                }
                .supported_plot_types(),
            ),
            tr(PlotKey::GridHealpixDiscrete),
        )
    } else if let Some(m) = &app.layers.base.data.matrix
        && target_var.map(|v| &v.name) == app.plotted_variable_info().map(|p| &p.name)
    {
        (Some(m.grid.supported_plot_types()), m.grid.name())
    } else {
        (None, "active")
    };

    let is_exploring_new = app.is_exploring_unplotted_variable();
    let total_2d_elements = crate::ui::variables_panel::calculate_selected_2d_elements(app);
    let target_pyramid_disabled = if is_exploring_new {
        !is_3d_available && total_2d_elements > crate::plots::limits::max_storage_elements()
    } else {
        app.base_resampled()
    };

    let is_volume_allowed = is_3d_available
        && !target_pyramid_disabled
        && (is_size_allowed || vol_mb == 0.0)
        && supported_plots
            .is_none_or(|plots| plots.contains(&PlotType::Volume) && !is_discrete_grid);

    let is_surface_allowed = total_2d_elements <= crate::plots::common::MAX_2D_SURFACE_ELEMENTS
        && !target_pyramid_disabled;
    let surface_mb = (total_2d_elements as f64 * 4.0) / (1024.0 * 1024.0);

    // Safety fallback: revert to Heatmap only if the currently active plot lacks valid GPU data (and user is not staging an unplotted variable) or pyramid is on
    let is_loading =
        app.layers.base.load.block_key.is_some() || app.block_prefetcher.pending_count() > 0;
    if !is_exploring_new
        && !is_loading
        && (((app.base_resampled() || label_image) && app.selected.plot_type != PlotType::Heatmap)
            || ((app.selected.plot_type == PlotType::Volume
                || app.selected.plot_type == PlotType::PointCloud)
                && app.layers.base.data.volume.is_none())
            || ((app.selected.plot_type == PlotType::Sphere
                || app.selected.plot_type == PlotType::Surface)
                && app.layers.base.renderers.sphere.is_none()
                && app.layers.base.data.matrix.is_none()))
    {
        app.selected.plot_type = PlotType::Heatmap;
    }

    ui.set_min_width(220.0);

    ui.label(
        egui::RichText::new(tr(PlotKey::SelectProjection))
            .small()
            .weak(),
    );
    ui.separator();

    let pyramid_disabled = target_pyramid_disabled;

    let plot_items = [
        (
            PlotType::Heatmap,
            Icon::PlotPlane,
            PlotType::Heatmap.display_name(),
            true,
        ),
        (
            PlotType::Line,
            Icon::PlotLine,
            PlotType::Line.display_name(),
            !pyramid_disabled,
        ),
        (
            PlotType::Sphere,
            Icon::PlotGlobe,
            PlotType::Sphere.display_name(),
            is_surface_allowed,
        ),
        (
            PlotType::Surface,
            Icon::PlotSurface,
            PlotType::Surface.display_name(),
            is_surface_allowed,
        ),
        (
            PlotType::Volume,
            Icon::PlotVolume,
            PlotType::Volume.display_name(),
            is_volume_allowed,
        ),
        (
            PlotType::PointCloud,
            Icon::PlotPointCloud,
            PlotType::PointCloud.display_name(),
            is_volume_allowed,
        ),
    ];

    for (plot_type, icon, label, enabled) in plot_items {
        let is_supported = supported_plots.is_none_or(|plots| plots.contains(&plot_type));
        let labels_only = label_image && plot_type != PlotType::Heatmap;
        let is_enabled = enabled && is_supported && !labels_only;
        let is_selected = app.selected.plot_type == plot_type;

        if is_enabled {
            let clicked = ui
                .horizontal(|ui| {
                    ui.icon(icon, IconSize::Sm);
                    ui.selectable_label(is_selected, label).clicked()
                })
                .inner;
            if clicked {
                app.switch_plot_type(plot_type);
            }
        } else {
            let mut reason_buf = [0; 256];
            let mut mb_buf = [0; 24];
            let reason = if labels_only {
                tr(PlotKey::LabelsHeatmapOnly)
            } else if !is_supported {
                fill(
                    &mut reason_buf,
                    tr(PlotKey::UnsupportedForGrid),
                    &[&grid_name],
                )
            } else if pyramid_disabled {
                tr(PlotKey::DisabledAggregation)
            } else if (plot_type == PlotType::Volume || plot_type == PlotType::PointCloud)
                && !is_3d_available
            {
                tr(PlotKey::Requires3D)
            } else {
                let mb = match plot_type {
                    PlotType::Sphere | PlotType::Surface => surface_mb,
                    _ => vol_mb,
                };
                let mb = stack_str(&mut mb_buf, format_args!("{mb:.0}"));
                // Volumes, point clouds and meshes keep the default budget.
                let limit = crate::plots::common::MAX_GPU_STORAGE_BUFFER_BYTES / (1024 * 1024);
                fill(
                    &mut reason_buf,
                    tr(PlotKey::DisabledGpuLimit),
                    &[&mb, &limit],
                )
            };

            ui.horizontal(|ui| {
                ui.icon_toned(icon, IconSize::Sm, IconTone::Muted);
                let mut buf = [0; 384];
                let text = fill(&mut buf, tr(PlotKey::DisabledItem), &[&label, &reason]);
                ui.add_enabled(false, egui::Label::new(text));
            });
        }
    }
}
