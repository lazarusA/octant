use super::OctantApp;
use super::state::StoreKind;
use crate::data::{DatasetMetadata, StoreHandle};
use crate::i18n::{NoticesKey as N, tr};
use crate::ui::toast::Severity;

impl OctantApp {
    pub fn inspect_active_store(&mut self) {
        self.is_loading = true;
        self.clear_active_metadata();
        self.status_message = format!("Inspecting {:?} metadata...", self.selected.store_kind);

        let store_kind = self.selected.store_kind;
        self.selected.store_target = self.selected.store_target.trim().to_string();
        let target_input = self.selected.store_target.clone();

        // Fresh metadata holds no coordinates: its variables are read again when chosen.
        self.coordinate_loader
            .forget(&StoreKind::make_source_id(store_kind, &target_input));

        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        self.metadata_rx = Some(rx);

        #[cfg(not(target_arch = "wasm32"))]
        crate::utils::executor::TaskExecutor::spawn_background(move || {
            let kind = store_kind.to_data_source_kind();
            let source_id = StoreKind::make_source_id(store_kind, &target_input);
            let source = crate::data::DataSource::new(&source_id, kind, &target_input, "Store");

            let res = crate::utils::executor::catch_panic(|| {
                crate::data::SourceFactory::open(source)
                    .and_then(|handle| {
                        let meta = handle.inspect()?;
                        Ok((meta, handle))
                    })
                    .map_err(|e| e.to_string())
            });

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
                                Err(crate::i18n::tr(crate::i18n::DatasetKey::BrowserLocalPaths).to_owned())
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
                // The web inspects stores directly, not through `StoreHandle::inspect`.
                let res = res.map(|(mut meta, handle)| {
                    crate::data::crs::assign(&mut meta);
                    (meta, handle)
                });

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

        self.selected.store_kind =
            StoreKind::resolve_with_inferred(explicit_kind, trimmed, self.selected.store_kind);

        if self.try_activate_dataset(trimmed) {
            if let Some(meta) = &self.selected.metadata {
                self.layout.hero_state.source_label = meta.name.clone();
            }
            self.layout.hero_state.loaded = true;
            self.layout.hero_state.loading = false;
        } else {
            self.layout.hero_state.begin_submit(trimmed);
            self.selected.store_target = trimmed.to_string();
            self.inspect_active_store();
        }
    }

    /// Takes a finished background inspection (`inspect_active_store`): its
    /// dataset becomes the active one, or its failure a toast.
    pub(crate) fn poll_inspection(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.metadata_rx else {
            return;
        };
        let result = match rx.try_recv() {
            Ok(result) => result,
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
                return;
            }
            // The inspection ended without a result (it panicked): a failure,
            // so the landing screen and queued commands stop waiting.
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                Err(self.selected.store_target.clone())
            }
        };
        self.metadata_rx = None;
        self.is_loading = false;
        self.layout.hero_state.loading = false;
        match result {
            Ok((metadata, handle)) => self.activate_inspected(metadata, handle),
            Err(err) => {
                self.layout.hero_state.loaded = false;
                self.clear_active_metadata();
                self.status_message = format!("Store inspect error: {}", err);
                self.notify(Severity::Error, tr(N::CouldntOpenDataset), &err);
            }
        }
    }

    /// Adds an inspected dataset to the open ones and makes it active.
    fn activate_inspected(&mut self, metadata: DatasetMetadata, handle: StoreHandle) {
        let is_plate = self.follow_plate(&metadata);
        if metadata.variables.is_empty() && !is_plate {
            self.status_message = format!("No variables discovered in '{}'", metadata.name);
            self.notify(Severity::Warning, tr(N::NoVariablesFound), &metadata.name);
        } else {
            self.status_message = format!(
                "Inspected '{}' (Found {} variables)",
                metadata.name,
                metadata.variables.len()
            );
        }
        self.layout.hero_state.loaded = true;
        self.layout.hero_state.source_label = metadata.name.clone();
        self.layout.show_variables_overlay = !metadata.variables.is_empty();

        let source_id = self.selected_source_id();
        let mut dataset = crate::data::Dataset::new(&source_id, handle.source().clone(), handle);
        dataset.metadata = Some(metadata.clone());
        self.dataset_manager.add(dataset);

        self.load_new_metadata(metadata);
    }
}
