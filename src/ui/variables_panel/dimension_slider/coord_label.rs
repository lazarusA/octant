//! The coordinate under a dimension slider: `850 hPa` for an index, `1000 hPa - 500 hPa` for
//! a range, formatted like the hover card and cached until the selection changes.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use egui::{RichText, Ui};

use crate::app::OctantApp;
use crate::data::VariableInfo;
use crate::ui::hover::format::format_dimension_coord;

/// Shows the coordinate of `start..=end` along dimension `dim`; nothing without coordinates.
pub(super) fn show_coord_label(
    app: &OctantApp,
    ui: &mut Ui,
    var: &VariableInfo,
    dim: usize,
    (start, end): (usize, usize),
) {
    if let Some(text) = cached_label(app, ui.ctx(), var, dim, (start, end)) {
        ui.add(egui::Label::new(RichText::new(&*text).small().weak()).truncate());
    }
}

/// The label for this selection, formatted only when the selection or dataset changes.
pub(super) fn cached_label(
    app: &OctantApp,
    ctx: &egui::Context,
    var: &VariableInfo,
    dim: usize,
    range: (usize, usize),
) -> Option<Arc<str>> {
    // The loaded metadata's generation, not the editable URL field: typing there must not
    // refresh this. The plotted target gives date hints, so it is part of the key too.
    let mut hasher = DefaultHasher::new();
    let target = date_target(app).unwrap_or_default();
    let generation = app.metadata_generation;
    (
        generation,
        target,
        var.name.as_str(),
        &var.shape,
        dim,
        range,
    )
        .hash(&mut hasher);
    let key = hasher.finish();
    let id = egui::Id::new(("dim_coord_label", dim));
    let cached = ctx.data(|d| d.get_temp::<(u64, Option<Arc<str>>)>(id));
    if let Some((cached_key, label)) = cached
        && cached_key == key
    {
        return label;
    }
    let label: Option<Arc<str>> = format_label(app, var, dim, range).map(Arc::from);
    ctx.data_mut(|d| d.insert_temp(id, (key, label.clone())));
    label
}

/// The formatted coordinate of `start..=end` along dimension `dim` of `var`, or `None`
/// when the dataset has no coordinate for it.
pub(super) fn format_label(
    app: &OctantApp,
    var: &VariableInfo,
    dim: usize,
    (start, end): (usize, usize),
) -> Option<String> {
    let meta = app.active_dataset_metadata.as_ref()?;
    let name = var.dimension_names.get(dim)?;
    meta.get_dim_coords(Some(&var.name), name)?;
    let len = usize::try_from(*var.shape.get(dim)?).ok()?;
    let target = date_target(app);
    let value =
        |i: usize| format_dimension_coord(Some(meta), Some(var), target, name, i, len, None).value;
    Some(if start == end {
        value(start)
    } else {
        format!("{} - {}", value(start), value(end))
    })
}

/// Date hints come from the plotted target when the panel shows the plotted dataset.
fn date_target(app: &OctantApp) -> Option<&str> {
    let meta = app.active_dataset_metadata.as_ref()?;
    app.plotted_dataset_metadata
        .as_ref()
        .filter(|plotted| plotted.name == meta.name)
        .map(|_| app.plotted_store_target_input.as_str())
}
