//! Dataset activation, removal, export requests, and RGB/CMYK channel introspection.

use super::app_state::OctantApp;
use super::store_kind::StoreKind;
use crate::app::layers::LayerId;

impl OctantApp {
    /// Checks if a dataset matching `target` (by URI, ID, or display name) is already in `dataset_manager`.
    /// If found and it has metadata, activates it, updates the variable tree cache, and opens the variables overlay.
    pub fn try_activate_dataset(&mut self, target: &str) -> bool {
        let input_target = target.trim().trim_end_matches('/');
        if input_target.is_empty() {
            return false;
        }
        let expanded = crate::utils::expand_tilde_str(input_target);
        let clean_expanded = expanded.trim_end_matches('/');

        let existing = self.dataset_manager.iter().find(|d| {
            let d_uri = d.source.uri.trim().trim_end_matches('/');
            let d_id = d.id.trim().trim_end_matches('/');
            d_uri == input_target
                || d_uri == clean_expanded
                || d_id == input_target
                || d_id == clean_expanded
                || d.source.display_name == input_target
                || d_id.ends_with(input_target)
        });

        if let Some(dataset) = existing {
            let uri = dataset.source.uri.clone();
            let kind = StoreKind::resolve_with_inferred(
                None,
                &uri,
                StoreKind::from_data_source_kind(&dataset.source.kind),
            );
            let meta = dataset.metadata.clone();
            self.selected.store_target = uri;
            self.selected.store_kind = kind;
            if let Some(meta) = meta {
                self.status_message = format!(
                    "Activated dataset '{}' (Found {} variables)",
                    meta.name,
                    meta.variables.len()
                );
                self.show_variables_overlay = true;
                self.load_new_metadata(meta);
                return true;
            }
        }
        false
    }

    /// Removes a dataset from `dataset_manager` by ID.
    /// If it is currently active, resets active dataset metadata and variable tree.
    pub fn remove_dataset(&mut self, dataset_id: &str) {
        if let Some(removed) = self.dataset_manager.remove(dataset_id) {
            crate::data::backends::coord_bounds::evict_coord_values(Some(&removed.source.uri));
            self.coordinate_loader.forget(&removed.id);
            let is_active = self.selected.metadata.as_ref().is_some_and(|_| {
                self.selected.store_target == removed.source.uri || dataset_id == removed.id
            });
            if is_active {
                self.clear_active_metadata();
            }
            self.status_message = format!("Removed dataset '{}'", removed.source.display_name);
        }
    }

    /// Clears all datasets from `dataset_manager` and resets active dataset state.
    pub fn clear_all_datasets(&mut self) {
        self.dataset_manager.clear();
        crate::data::backends::coord_bounds::evict_coord_values(None);
        self.coordinate_loader = crate::data::blocks::CoordinateLoader::default();
        self.clear_active_metadata();
        self.status_message = "Cleared all datasets from Dataset Manager".to_string();
    }

    /// Triggers an export request for the canvas / figure.
    pub fn request_canvas_export(
        &mut self,
        output_path: std::path::PathBuf,
        copy_to_clipboard: bool,
    ) {
        let target = if self.show_crop_overlay {
            crate::export::ExportTarget::RoiCrop
        } else {
            self.export_settings.target
        };

        self.pending_export = Some(crate::export::PendingExportRequest {
            format: self.export_settings.format,
            target,
            roi: self.roi_crop_box,
            jpeg_quality: self.export_settings.jpeg_quality,
            copy_to_clipboard,
            output_path: if output_path.as_os_str().is_empty() {
                None
            } else {
                Some(output_path)
            },
            canvas_rect_in_points: egui::Rect::NOTHING,
            pixels_per_point: 1.0,
        });
    }

    /// Quick save shortcut (Cmd+S): saves to export_dir with auto-generated filename.
    pub fn quick_save_canvas(&mut self) {
        let var_name = self
            .plotted_variable_info()
            .map(|v| v.name.as_str())
            .unwrap_or("plot");
        let filename =
            crate::export::generate_export_filename(var_name, self.export_settings.format);
        let out_path =
            crate::export::resolve_export_path(&self.export_settings.export_dir, &filename);
        self.request_canvas_export(out_path, false);
    }

    /// Whether the plotted variable has 2 or more bands for an RGB composite.
    pub fn has_rgb_bands(&self) -> bool {
        self.layer_has_rgb_bands(LayerId::BASE)
    }

    /// Whether layer `id`'s variable has 2 or more bands for an RGB composite.
    pub fn layer_has_rgb_bands(&self, id: LayerId) -> bool {
        let Some(var) = self.layer_variable_info(id) else {
            return false;
        };
        let c_idx = self.layer_channel_dim(id).unwrap_or(0);
        var.shape.len() >= 2 && var.shape.get(c_idx).copied().unwrap_or(0) >= 2
    }

    /// Return the dimension index corresponding to channels/bands for the currently plotted variable.
    pub fn channel_dim_index(&self) -> Option<usize> {
        self.layer_channel_dim(LayerId::BASE)
    }

    /// The channels/bands dimension of layer `id`'s variable.
    pub fn layer_channel_dim(&self, id: LayerId) -> Option<usize> {
        channel_dim(self.layer_variable_info(id)?)
    }

    /// The number of bands of the plotted variable (3 without one).
    pub fn num_bands(&self) -> usize {
        self.layer_num_bands(LayerId::BASE)
    }

    /// The number of bands of layer `id`'s variable (3 without one).
    pub fn layer_num_bands(&self, id: LayerId) -> usize {
        let Some(var) = self.layer_variable_info(id) else {
            return 3;
        };
        let c_idx = self.layer_channel_dim(id).unwrap_or(0);
        var.shape.get(c_idx).copied().unwrap_or(3) as usize
    }

    /// Whether the plotted variable holds CMYK inks.
    pub fn is_cmyk(&self) -> bool {
        self.layer_is_cmyk(LayerId::BASE)
    }

    /// Whether layer `id`'s variable holds CMYK inks.
    pub fn layer_is_cmyk(&self, id: LayerId) -> bool {
        let Some(var) = self.layer_variable_info(id) else {
            return false;
        };
        crate::data::slicing::composite::cmyk::is_cmyk_attrs(|key| match key {
            "long_name" => var.long_name.as_deref(),
            _ => var.attributes.get(key).map(String::as_str),
        })
    }

    /// Returns true if the currently plotted variable represents an OME-Zarr / bioimaging dataset with OMERO channels.
    pub fn is_ome_dataset(&self) -> bool {
        let Some(var) = self.plotted_variable_info() else {
            return false;
        };
        var.attributes.contains_key("omero_channels") || var.attributes.contains_key("omero_colors")
    }

    /// Returns true if the active dataset represents a GeoTIFF.
    pub fn is_geotiff(&self) -> bool {
        self.layer_is_geotiff(LayerId::BASE)
    }

    /// Whether layer `id` shows (or, before its first plot, stages) a GeoTIFF.
    pub fn layer_is_geotiff(&self, id: LayerId) -> bool {
        let Some((shown, staged)) = self.layer_selections(id) else {
            return false;
        };
        let selection = if shown.metadata.is_some() {
            shown
        } else {
            staged
        };
        matches!(
            selection.store_kind,
            StoreKind::LocalGeoTiff | StoreKind::RemoteGeoTiff
        )
    }

    /// The plotted `(start, end)` selection of dimension `dim_idx`.
    pub fn get_effective_dim_range(&self, dim_idx: usize) -> (usize, usize) {
        self.layer_dim_range(LayerId::BASE, dim_idx)
    }

    /// Layer `id`'s `(start, end)` selection of dimension `dim_idx`: its
    /// range while expanded, else its single index.
    pub fn layer_dim_range(&self, id: LayerId, dim_idx: usize) -> (usize, usize) {
        let range = self.layer_dim_ranges(id).get(dim_idx).copied();
        match self.layer_dim_config(id).get(dim_idx) {
            Some(cfg) if !cfg.active => {
                let idx = self
                    .layer_dim_indices(id)
                    .get(dim_idx)
                    .copied()
                    .unwrap_or(0);
                (idx, idx)
            }
            _ => range.unwrap_or((0, usize::MAX)),
        }
    }
}

/// The channel/band dimension of `var`: one named like a channel, else the
/// first of a variable with 3 or more dimensions.
pub(crate) fn channel_dim(var: &crate::data::VariableInfo) -> Option<usize> {
    var.dimension_names
        .iter()
        .position(|d| crate::data::coordinates::naming::is_channel_dim_name(d))
        .or(if var.shape.len() >= 3 { Some(0) } else { None })
}
