//! Which plot types the staged (else plotted) variable can be drawn as, and
//! why the others cannot: shared by the top bar's menu and the centered
//! picker.

use crate::app::{OctantApp, SpatialRole};
use crate::data::coordinates::{CoordinateGrid, HealpixOrder};
use crate::i18n::{PlotKey, fill, tr};
use crate::plots::PlotType;
use crate::plots::common::{MAX_2D_SURFACE_ELEMENTS, MAX_GPU_STORAGE_BUFFER_BYTES};
use crate::ui::variables_panel::{
    calculate_selected_2d_elements, calculate_selected_volume_elements,
};
use crate::utils::stack_str;

/// Every plot type, in the order the menus list them.
pub const ORDER: [PlotType; 6] = [
    PlotType::Heatmap,
    PlotType::Line,
    PlotType::Sphere,
    PlotType::Surface,
    PlotType::Volume,
    PlotType::PointCloud,
];

/// Why a plot type cannot be picked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Blocked {
    /// Label images draw only on the heatmap.
    LabelsOnly,
    /// The grid, by name, does not support it.
    Unsupported(&'static str),
    /// 2D aggregation (resampling) is on.
    Aggregation,
    /// Its data, in MB, is past the GPU limit.
    GpuLimit(f64),
}

/// A plot type and, when it cannot be picked, why.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotOption {
    pub plot_type: PlotType,
    pub blocked: Option<Blocked>,
}

/// What decides the plot types of the variable shown.
struct Facts {
    label_image: bool,
    volume_mb: f64,
    volume_fits: bool,
    surface_mb: f64,
    surface_fits: bool,
    supported: Option<&'static [PlotType]>,
    grid_name: &'static str,
    aggregated: bool,
}

impl Facts {
    fn of(app: &OctantApp) -> Self {
        let var = app
            .selected_variable_info()
            .or_else(|| app.plotted_variable_info());
        let has_3d = var.is_some_and(|v| v.shape.len() >= 3 || v.dimension_names.len() >= 3);
        let volume = var.map_or(0, |_| calculate_selected_volume_elements(app));
        let surface = calculate_selected_2d_elements(app);
        let discrete = app
            .selected
            .dim_config
            .iter()
            .any(|c| c.spatial == SpatialRole::Grid);
        let (supported, grid_name) = supported_plots(app, discrete);
        let aggregated = if app.is_exploring_unplotted_variable() {
            !has_3d && surface > crate::plots::limits::max_storage_elements()
        } else {
            app.base_resampled()
        };
        Self {
            label_image: var.is_some_and(crate::data::VariableInfo::is_label_image),
            volume_mb: megabytes(volume),
            volume_fits: volume <= crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS,
            surface_mb: megabytes(surface),
            surface_fits: surface <= MAX_2D_SURFACE_ELEMENTS,
            supported,
            grid_name,
            aggregated,
        }
    }

    /// Why `plot_type` cannot be picked, `None` when it can.
    fn blocked(&self, plot_type: PlotType) -> Option<Blocked> {
        let volumetric = matches!(plot_type, PlotType::Volume | PlotType::PointCloud);
        let surface = matches!(plot_type, PlotType::Sphere | PlotType::Surface);
        Some(if self.label_image && plot_type != PlotType::Heatmap {
            Blocked::LabelsOnly
        } else if !self.supported.is_none_or(|p| p.contains(&plot_type)) {
            Blocked::Unsupported(self.grid_name)
        } else if plot_type == PlotType::Heatmap {
            return None;
        } else if self.aggregated {
            Blocked::Aggregation
        } else if volumetric && !self.volume_fits {
            Blocked::GpuLimit(self.volume_mb)
        } else if surface && !self.surface_fits {
            Blocked::GpuLimit(self.surface_mb)
        } else {
            return None;
        })
    }
}

fn megabytes(elements: usize) -> f64 {
    (elements as f64 * 4.0) / (1024.0 * 1024.0)
}

/// The plot types a discrete grid, or the plotted variable's grid, supports
/// and the grid's name; `None` when any goes.
fn supported_plots(app: &OctantApp, discrete: bool) -> (Option<&'static [PlotType]>, &'static str) {
    if discrete {
        let healpix = CoordinateGrid::Healpix {
            nside: 1,
            ordering: HealpixOrder::Ring,
            npix: 12,
            coords_lon: None,
            coords_lat: None,
        };
        return (
            Some(healpix.supported_plot_types()),
            tr(PlotKey::GridHealpixDiscrete),
        );
    }
    let var = app
        .selected_variable_info()
        .or_else(|| app.plotted_variable_info());
    let plotted = app.plotted_variable_info().map(|p| &p.name);
    match &app.layers.base.data.matrix {
        Some(m) if var.map(|v| &v.name) == plotted => {
            (Some(m.grid.supported_plot_types()), m.grid.name())
        }
        _ => (None, "active"),
    }
}

/// Every plot type and whether it can be picked. A plotted type whose data
/// is gone (or that a label image or aggregation rules out) falls back to
/// the heatmap first.
pub fn plot_options(app: &mut OctantApp) -> [PlotOption; 6] {
    let facts = Facts::of(app);
    fall_back_to_heatmap(app, facts.label_image);
    ORDER.map(|plot_type| PlotOption {
        plot_type,
        blocked: facts.blocked(plot_type),
    })
}

/// The heatmap, once the plotted type lost what it draws (and no new
/// variable is staged or loading).
fn fall_back_to_heatmap(app: &mut OctantApp, label_image: bool) {
    let loading =
        app.layers.base.load.block_key.is_some() || app.block_prefetcher.pending_count() > 0;
    if app.is_exploring_unplotted_variable() || loading {
        return;
    }
    let base = &app.layers.base;
    let lost = match app.selected.plot_type {
        PlotType::Heatmap => false,
        _ if app.base_resampled() || label_image => true,
        PlotType::Volume | PlotType::PointCloud => base.data.volume.is_none(),
        PlotType::Sphere | PlotType::Surface => {
            base.renderers.sphere.is_none() && base.data.matrix.is_none()
        }
        PlotType::Line => false,
    };
    if lost {
        app.selected.plot_type = PlotType::Heatmap;
    }
}

/// Why a plot type is blocked, for the menus.
pub fn blocked_text(buf: &mut [u8; 256], blocked: Blocked) -> &str {
    match blocked {
        Blocked::LabelsOnly => tr(PlotKey::LabelsHeatmapOnly),
        Blocked::Unsupported(grid) => fill(buf, tr(PlotKey::UnsupportedForGrid), &[&grid]),
        Blocked::Aggregation => tr(PlotKey::DisabledAggregation),
        Blocked::GpuLimit(mb) => {
            let mut mb_buf = [0; 24];
            let mb = stack_str(&mut mb_buf, format_args!("{mb:.0}"));
            // Volumes, point clouds and meshes keep the default budget.
            let limit = MAX_GPU_STORAGE_BUFFER_BYTES / (1024 * 1024);
            fill(buf, tr(PlotKey::DisabledGpuLimit), &[&mb, &limit])
        }
    }
}
