//! Dataset activation, removal, export requests, and RGB/CMYK channel introspection.

use super::app_state::OctantApp;
use super::store_kind::StoreKind;

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
            self.store_target_input = uri;
            self.selected_store_kind = kind;
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
            let is_active = self.active_dataset_metadata.as_ref().is_some_and(|_| {
                self.store_target_input == removed.source.uri || dataset_id == removed.id
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

    /// Check if the currently plotted dataset/variable has 2 or more bands/channels available for RGB composition.
    pub fn has_rgb_bands(&self) -> bool {
        let Some(var) = self.plotted_variable_info() else {
            return false;
        };
        if var.shape.len() < 2 {
            return false;
        }
        let c_idx = self.channel_dim_index().unwrap_or(0);
        var.shape.get(c_idx).copied().unwrap_or(0) >= 2
    }

    /// Return the dimension index corresponding to channels/bands for the currently plotted variable.
    pub fn channel_dim_index(&self) -> Option<usize> {
        let var = self.plotted_variable_info()?;
        var.dimension_names
            .iter()
            .position(|d| crate::data::coordinates::naming::is_channel_dim_name(d))
            .or(if var.shape.len() >= 3 { Some(0) } else { None })
    }

    /// Return the dimension index corresponding to channels/bands for the currently selected variable.
    pub fn selected_channel_dim_index(&self) -> Option<usize> {
        let var = self.selected_variable_info()?;
        var.dimension_names
            .iter()
            .position(|d| crate::data::coordinates::naming::is_channel_dim_name(d))
            .or(if var.shape.len() >= 3 { Some(0) } else { None })
    }

    /// Return the total number of bands/channels for the currently plotted variable, if multi-band.
    pub fn num_bands(&self) -> usize {
        let Some(var) = self.plotted_variable_info() else {
            return 3;
        };
        let c_idx = self.channel_dim_index().unwrap_or(0);
        var.shape.get(c_idx).copied().unwrap_or(3) as usize
    }

    /// Returns true if the currently plotted variable represents a CMYK color space dataset.
    pub fn is_cmyk(&self) -> bool {
        let Some(var) = self.plotted_variable_info() else {
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
        if self.plotted_dataset_metadata.is_some() {
            return self.plotted_store_kind == StoreKind::LocalGeoTiff
                || self.plotted_store_kind == StoreKind::RemoteGeoTiff;
        }

        self.selected_store_kind == StoreKind::LocalGeoTiff
            || self.selected_store_kind == StoreKind::RemoteGeoTiff
    }

    /// Returns the effective `(start, end)` selected range for a given dimension index.
    pub fn get_effective_dim_range(&self, dim_idx: usize) -> (usize, usize) {
        let (configs, ranges, indices) = if !self.plotted_dim_config.is_empty() {
            (
                &self.plotted_dim_config,
                &self.plotted_selected_dim_ranges,
                &self.plotted_selected_dim_indices,
            )
        } else {
            (
                &self.dim_config,
                &self.selected_dim_ranges,
                &self.selected_dim_indices,
            )
        };
        if let Some(cfg) = configs.get(dim_idx) {
            if cfg.active {
                ranges.get(dim_idx).copied().unwrap_or((0, usize::MAX))
            } else {
                let idx = indices.get(dim_idx).copied().unwrap_or(0);
                (idx, idx)
            }
        } else {
            ranges.get(dim_idx).copied().unwrap_or((0, usize::MAX))
        }
    }
}
