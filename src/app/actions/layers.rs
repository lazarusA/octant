//! Actions on the plotted layers, as their colorbars and the Layers menu
//! change them: an overlay added (Plot Data with Add Overlay), shown,
//! hidden or removed, a layer's colormap and color range; and the export
//! dialog, whose own button confirms an export.

use super::resolve::among;
use super::{Done, Failure, NameKind, Outcome, resolved};
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::export::ExportFormat;

/// A layer's name: its variable as listed.
fn layer_name(app: &OctantApp, id: LayerId) -> String {
    let layer = app.layers.get(id);
    let var = layer.and_then(|l| l.selection().variable_info());
    var.map(|v| v.display_name().to_string())
        .unwrap_or_default()
}

impl OctantApp {
    /// The layer `name` names by its variable; the base layer without one.
    pub(super) fn layer_named(&self, name: Option<&str>) -> Result<LayerId, Failure> {
        let Some(name) = name else {
            return Ok(LayerId::BASE);
        };
        let ids: Vec<LayerId> = self.layers.iter().map(|l| l.id()).collect();
        let names: Vec<String> = ids.iter().map(|&id| layer_name(self, id)).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        resolved(among(&names, name), NameKind::Layer, name).map(|i| ids[i])
    }

    /// Adds the staged variable as an overlay, as Plot Data does with Add
    /// Overlay on.
    pub(super) fn add_staged_overlay(&mut self) -> Outcome {
        if let Some(reason) = self.overlay_unavailable_for(self.selected.variable_idx) {
            return Err(Failure::OverlayUnavailable(reason.to_string()));
        }
        self.layout.plot_as_overlay = true;
        self.plot_from_panel();
        Ok(Done::OverlayAdded(self.staged_name()))
    }

    pub(super) fn show_layer(&mut self, name: &str, visible: bool) -> Outcome {
        let id = self.layer_named(Some(name))?;
        self.set_layer_visible(id, visible);
        Ok(Done::LayerShown {
            layer: layer_name(self, id),
            visible,
        })
    }

    pub(super) fn remove_layer(&mut self, name: &str) -> Outcome {
        let id = self.layer_named(Some(name))?;
        let layer = layer_name(self, id);
        // The base layer is replaced by plotting, never removed.
        if id == LayerId::BASE {
            return Err(Failure::NotFound {
                kind: NameKind::Layer,
                text: name.to_string(),
                closest: Vec::new(),
            });
        }
        self.remove_overlay(id);
        Ok(Done::OverlayRemoved(layer))
    }

    /// Layer `name`'s color range, locked as the colorbar's range inputs
    /// lock it.
    pub(super) fn set_color_range(&mut self, name: Option<&str>, min: f32, max: f32) -> Outcome {
        let id = self.layer_named(name)?;
        let (min, max) = (min.min(max), min.max(max));
        if let Some(layer) = self.layers.get_mut(id) {
            layer.color.range_min = min;
            layer.color.range_max = max;
            layer.color.lock_bounds = true;
        }
        Ok(Done::ColorRange {
            layer: layer_name(self, id),
            min,
            max,
        })
    }

    /// Opens the export dialog, on `format` when given.
    pub(super) fn open_export(&mut self, format: Option<&str>) -> Outcome {
        if let Some(format) = format {
            let extensions: Vec<&str> = ExportFormat::ALL.iter().map(|f| f.extension()).collect();
            let i = resolved(among(&extensions, format), NameKind::Format, format)?;
            self.export_settings.format = ExportFormat::ALL[i];
        }
        self.show_export_modal = true;
        Ok(Done::ExportOpened(
            self.export_settings.format.label().to_string(),
        ))
    }
}
