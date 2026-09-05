use super::OctantApp;
use super::state::StoreKind;

impl OctantApp {
    pub fn get_line_profile_payload(&self) -> (Vec<f32>, u32, u32) {
        if self.line_profile_dim_idx == 2
            && let Some(vdata) = &self.volume_data
            && vdata.depth > 1
        {
            // Along Z: profile length = nz (all spatial rays across depth)
            let (nx, ny, nz) = (vdata.width, vdata.height, vdata.depth);
            let num_pixels = nx * ny;
            if self.line_plot_all_series {
                let mut payload = Vec::with_capacity(nx * ny * nz);
                let mut valid_lines = 0u32;
                for y in 0..ny {
                    for x in 0..nx {
                        let mut has_valid = false;
                        for z in 0..nz {
                            let idx = z * (nx * ny) + y * nx + x;
                            if let Some(&v) = vdata.values.get(idx)
                                && !v.is_nan()
                                && v.is_finite()
                            {
                                has_valid = true;
                                break;
                            }
                        }
                        if has_valid {
                            valid_lines += 1;
                            for z in 0..nz {
                                let idx = z * (nx * ny) + y * nx + x;
                                payload.push(vdata.values.get(idx).copied().unwrap_or(f32::NAN));
                            }
                        }
                    }
                }
                (payload, nz as u32, valid_lines)
            } else {
                let target_pixel = self
                    .line_profile_slice_idx
                    .min(num_pixels.saturating_sub(1));
                let target_y = target_pixel / nx.max(1);
                let target_x = target_pixel % nx.max(1);

                let mut profile = Vec::with_capacity(nz);
                for z in 0..nz {
                    let idx = z * (nx * ny) + target_y * nx + target_x;
                    profile.push(vdata.values.get(idx).copied().unwrap_or(f32::NAN));
                }
                (profile, nz as u32, 1)
            }
        } else if let Some(matrix) = &self.matrix_data {
            // Along X (dim 0) or Along Y (dim 1):
            // Always extracted from current timestep slice (matrix_data), so lines update dynamically during playback!
            let (profile_length, line_count, slice_idx) = if self.line_profile_dim_idx == 0 {
                (
                    matrix.width,
                    matrix.height,
                    self.line_profile_slice_idx
                        .min(matrix.height.saturating_sub(1)),
                )
            } else {
                (
                    matrix.height,
                    matrix.width,
                    self.line_profile_slice_idx
                        .min(matrix.width.saturating_sub(1)),
                )
            };

            if self.line_plot_all_series {
                if self.line_profile_dim_idx == 0 {
                    let mut payload = Vec::with_capacity(profile_length * line_count);
                    let mut valid_lines = 0u32;
                    for row in 0..line_count {
                        let start = row * profile_length;
                        let end = (start + profile_length).min(matrix.values.len());
                        let row_slice = &matrix.values[start..end];
                        if row_slice.iter().any(|v| !v.is_nan() && v.is_finite()) {
                            valid_lines += 1;
                            payload.extend_from_slice(row_slice);
                        }
                    }
                    (payload, profile_length as u32, valid_lines)
                } else {
                    let mut payload = Vec::with_capacity(profile_length * line_count);
                    let mut valid_lines = 0u32;
                    for col in 0..line_count {
                        let mut has_valid = false;
                        for row in 0..profile_length {
                            let idx = row * matrix.width + col;
                            if let Some(&v) = matrix.values.get(idx)
                                && !v.is_nan()
                                && v.is_finite()
                            {
                                has_valid = true;
                                break;
                            }
                        }
                        if has_valid {
                            valid_lines += 1;
                            for row in 0..profile_length {
                                let idx = row * matrix.width + col;
                                payload.push(matrix.values.get(idx).copied().unwrap_or(f32::NAN));
                            }
                        }
                    }
                    (payload, profile_length as u32, valid_lines)
                }
            } else {
                (
                    matrix.extract_1d_line_profile(self.line_profile_dim_idx, slice_idx),
                    profile_length as u32,
                    1,
                )
            }
        } else {
            (Vec::new(), 0, 0)
        }
    }

    pub fn inspect_active_store(&mut self) {
        self.is_loading = true;
        self.status_message = format!("Inspecting {:?} metadata...", self.selected_store_kind);

        let store_kind = self.selected_store_kind;
        self.store_target_input = self.store_target_input.trim().to_string();
        let target_input = self.store_target_input.clone();

        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        self.metadata_rx = Some(rx);

        rayon::spawn(move || {
            let kind = store_kind.to_data_source_kind();
            let source_id = StoreKind::make_source_id(store_kind, &target_input);
            let source = crate::data::DataSource::new(&source_id, kind, &target_input, "Store");

            let res = crate::data::SourceFactory::open(source)
                .and_then(|store| store.inspect())
                .map_err(|e| e.to_string());

            if let Err(err) = &res {
                log::error!("Store inspect failed for '{target_input}': {err}");
            }

            let _ = tx.send(res);
        });
    }

    /// Activates an existing dataset from the dataset manager if already loaded,
    /// or initiates store inspection in the background if not yet loaded.
    pub fn submit_or_activate_source(&mut self, target: &str, explicit_kind: Option<StoreKind>) {
        let trimmed = target.trim();
        if trimmed.is_empty() {
            return;
        }

        let inferred = crate::utils::infer_store_kind_from_target(trimmed).ok();
        let effective_kind = match (explicit_kind, inferred) {
            (Some(kind), Some(inf)) => {
                // Auto-upgrade generic/default Zarr selection if target URL is specifically Icechunk or NetCDF
                if (kind == StoreKind::RemoteZarr && inf == StoreKind::RemoteIcechunk)
                    || (kind == StoreKind::LocalZarr && inf == StoreKind::LocalIcechunk)
                    || (kind == StoreKind::LocalZarr && inf == StoreKind::LocalNetCdf)
                {
                    inf
                } else {
                    kind
                }
            }
            (Some(kind), None) => kind,
            (None, Some(inf)) => inf,
            (None, None) => self.selected_store_kind,
        };
        self.selected_store_kind = effective_kind;

        if self.try_activate_dataset(trimmed) {
            if let Some(meta) = &self.active_dataset_metadata {
                self.hero_state.source_label = meta.name.clone();
            }
            self.hero_state.loaded = true;
            self.hero_state.loading = false;
        } else {
            self.hero_state.begin_submit(trimmed);
            self.store_target_input = trimmed.to_string();
            self.inspect_active_store();
        }
    }
}
