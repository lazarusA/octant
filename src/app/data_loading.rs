use super::OctantApp;
use super::state::StoreKind;

impl OctantApp {
    pub fn get_line_profile_payload(&self) -> (Vec<f32>, u32, u32) {
        if self.line_profile_dim_idx == 2
            && let Some(vdata) = &self.volume_data
            && vdata.depth > 1
        {
            if self.line_plot_all_series {
                vdata.extract_all_z_lines_payload()
            } else {
                let (nx, ny, nz) = (vdata.width, vdata.height, vdata.depth);
                let num_pixels = nx * ny;
                let target_pixel = self
                    .line_profile_slice_idx
                    .min(num_pixels.saturating_sub(1));
                let target_y = target_pixel / nx.max(1);
                let target_x = target_pixel % nx.max(1);
                (
                    vdata.extract_z_line_profile(target_x, target_y),
                    nz as u32,
                    1,
                )
            }
        } else if let Some(matrix) = &self.matrix_data {
            if self.line_plot_all_series {
                matrix.extract_all_lines_payload(self.line_profile_dim_idx)
            } else {
                let (profile_length, max_slices) = if self.line_profile_dim_idx == 0 {
                    (matrix.width, matrix.height)
                } else {
                    (matrix.height, matrix.width)
                };
                let slice_idx = self
                    .line_profile_slice_idx
                    .min(max_slices.saturating_sub(1));
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
        self.active_dataset_metadata = None;
        self.cached_variable_tree = None;
        self.variable_search.clear();
        self.status_message = format!("Inspecting {:?} metadata...", self.selected_store_kind);

        let store_kind = self.selected_store_kind;
        self.store_target_input = self.store_target_input.trim().to_string();
        let target_input = self.store_target_input.clone();

        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        self.metadata_rx = Some(rx);

        #[cfg(not(target_arch = "wasm32"))]
        crate::utils::executor::TaskExecutor::spawn_background(move || {
            let kind = store_kind.to_data_source_kind();
            let source_id = StoreKind::make_source_id(store_kind, &target_input);
            let source = crate::data::DataSource::new(&source_id, kind, &target_input, "Store");

            let res = crate::data::SourceFactory::open(source)
                .and_then(|handle| {
                    let meta = handle.inspect()?;
                    Ok((meta, handle))
                })
                .map_err(|e| e.to_string());

            if let Err(err) = &res {
                log::error!("Store inspect failed for '{target_input}': {err}");
            }

            let _ = tx.send(res);
        });

        #[cfg(target_arch = "wasm32")]
        {
            let target_clone = target_input.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let kind = store_kind.to_data_source_kind();
                let source_id = StoreKind::make_source_id(store_kind, &target_clone);
                let source = crate::data::DataSource::new(&source_id, kind, &target_clone, "Store");

                let res: Result<(crate::data::DatasetMetadata, crate::data::StoreHandle), String> =
                    async {
                        match store_kind {
                            StoreKind::RemoteZarr => {
                                let meta = crate::data::backends::zarr::inspect_wasm_remote_zarr(
                                    &target_clone,
                                )
                                .await?;
                                let handle = crate::data::SourceFactory::open(source)
                                    .map_err(|e| e.to_string())?;
                                Ok((meta, handle))
                            }
                            StoreKind::RemoteIcechunk => {
                                let meta = crate::data::backends::icechunk::wasm::inspect_wasm_remote_icechunk(
                                    &target_clone,
                                )
                                .await?;
                                let handle = crate::data::SourceFactory::open(source)
                                    .map_err(|e| e.to_string())?;
                                Ok((meta, handle))
                            }
                            StoreKind::RemoteGeoTiff => {
                                let meta = crate::data::backends::geotiff::wasm::inspect_wasm_remote_geotiff(
                                    &target_clone,
                                )
                                .await?;
                                let handle = crate::data::SourceFactory::open(source)
                                    .map_err(|e| e.to_string())?;
                                Ok((meta, handle))
                            }
                            StoreKind::LocalZarr
                            | StoreKind::LocalIcechunk
                            | StoreKind::LocalGeoTiff
                            | StoreKind::LocalNetCdf => {
                                Err("Direct local file paths cannot be read in a browser due to web sandbox security.\n\nTo view local datasets in the browser:\n1. Serve your directory or file with a local HTTP server: `npx serve` or `python3 -m http.server`\n2. Enter the URL: `http://localhost:8000/my_dataset`\n\nOr run the native desktop version of Octant (`cargo run --release`).".to_string())
                            }
                            _ => {
                                let handle = crate::data::SourceFactory::open(source)
                                    .map_err(|e| e.to_string())?;
                                let meta = handle.inspect().map_err(|e| e.to_string())?;
                                Ok((meta, handle))
                            }
                        }
                    }
                    .await;

                if let Err(err) = &res {
                    log::error!("Store inspect failed for '{target_clone}': {err}");
                }

                let _ = tx.send(res);
            });
        }
    }

    /// Activates an existing dataset from the dataset manager if already loaded,
    /// or initiates store inspection in the background if not yet loaded.
    pub fn submit_or_activate_source(&mut self, target: &str, explicit_kind: Option<StoreKind>) {
        let trimmed = target.trim();
        if trimmed.is_empty() {
            return;
        }

        self.selected_store_kind =
            StoreKind::resolve_with_inferred(explicit_kind, trimmed, self.selected_store_kind);

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
