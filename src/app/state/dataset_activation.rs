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
                self.variable_search.clear();
                self.cached_variable_tree = Some(meta.build_variable_tree());
                self.active_dataset_metadata = Some(meta);
                self.selected_variable_idx = 0;
                return true;
            }
        }
        false
    }

    /// Removes a dataset from `dataset_manager` by ID.
    /// If it is currently active, resets active dataset metadata and variable tree.
    pub fn remove_dataset(&mut self, dataset_id: &str) {
        if let Some(removed) = self.dataset_manager.remove(dataset_id) {
            let is_active = self.active_dataset_metadata.as_ref().is_some_and(|_| {
                self.store_target_input == removed.source.uri || dataset_id == removed.id
            });
            if is_active {
                self.active_dataset_metadata = None;
                self.cached_variable_tree = None;
                self.variable_search.clear();
            }
            self.status_message = format!("Removed dataset '{}'", removed.source.display_name);
        }
    }

    /// Clears all datasets from `dataset_manager` and resets active dataset state.
    pub fn clear_all_datasets(&mut self) {
        self.dataset_manager.clear();
        self.active_dataset_metadata = None;
        self.cached_variable_tree = None;
        self.variable_search.clear();
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

    /// Check if the active dataset/variable has 3 or more bands available for RGB composition.
    pub fn has_rgb_bands(&self) -> bool {
        if let Some(var) = self.plotted_variable_info() {
            return var.shape.len() >= 3 && var.shape[0] >= 3;
        }
        if let Some(var) = self.selected_variable_info() {
            return var.shape.len() >= 3 && var.shape[0] >= 3;
        }
        false
    }

    /// Return the total number of bands for the active variable, if multi-band.
    pub fn num_bands(&self) -> usize {
        if let Some(var) = self
            .plotted_variable_info()
            .or_else(|| self.selected_variable_info())
            && var.shape.len() >= 3
        {
            var.shape[0] as usize
        } else {
            3
        }
    }

    /// Returns true if the active variable represents a CMYK color space dataset.
    pub fn is_cmyk(&self) -> bool {
        let Some(var) = self
            .plotted_variable_info()
            .or_else(|| self.selected_variable_info())
        else {
            return false;
        };

        var.attributes
            .get("photometric")
            .is_some_and(|p| p.eq_ignore_ascii_case("cmyk"))
            || var
                .attributes
                .get("color_space")
                .is_some_and(|cs| cs.eq_ignore_ascii_case("cmyk"))
            || var.long_name.as_deref().is_some_and(|l| l.contains("CMYK"))
    }
}

/// Helper function to verify dimensional compatibility between two variables
/// (matching rank, shapes, or spatial extent) for multi-layer plotting.
pub fn check_dimensional_compatibility(
    var_a: &crate::data::VariableInfo,
    var_b: &crate::data::VariableInfo,
) -> Result<(), String> {
    if var_a.shape.len() != var_b.shape.len() {
        return Err(format!(
            "Rank mismatch: '{}' (rank {}) vs '{}' (rank {})",
            var_a.name,
            var_a.shape.len(),
            var_b.name,
            var_b.shape.len()
        ));
    }
    if var_a.shape != var_b.shape {
        return Err(format!(
            "Shape mismatch: '{}' ({:?}) vs '{}' ({:?})",
            var_a.name, var_a.shape, var_b.name, var_b.shape
        ));
    }
    Ok(())
}
