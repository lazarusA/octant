use crate::stores::{
    DataStore, DatasetMetadata, VariableInfo, icechunk_local::IcechunkLocalStore,
    icechunk_remote::IcechunkRemoteStore, zarr_local::ZarrLocalStore, zarr_remote::ZarrRemoteStore,
};

use super::OctantApp;
use super::state::StoreKind;

impl OctantApp {
    pub(super) fn get_line_profile_payload(&self) -> (Vec<f32>, u32, u32) {
        if let Some(matrix) = &self.matrix_data {
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
                let mut payload = Vec::with_capacity(profile_length.max(1) * line_count.max(1));
                for idx in 0..line_count {
                    payload.extend(matrix.extract_1d_line_profile(self.line_profile_dim_idx, idx));
                }
                (payload, profile_length as u32, line_count as u32)
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
        let target_input = self.store_target_input.clone();

        let (tx, rx) = std::sync::mpsc::channel();
        self.metadata_rx = Some(rx);

        std::thread::spawn(move || {
            let store: Box<dyn DataStore> = match store_kind {
                StoreKind::RemoteZarr => Box::new(ZarrRemoteStore::new(&target_input)),
                StoreKind::LocalZarr => Box::new(ZarrLocalStore::new(&target_input)),
                StoreKind::RemoteIcechunk => Box::new(IcechunkRemoteStore::new(&target_input)),
                StoreKind::LocalIcechunk => Box::new(IcechunkLocalStore::new(&target_input)),
                StoreKind::ProceduralRandom => {
                    let meta = DatasetMetadata {
                        name: "Procedural Test Store".to_string(),
                        store_type: "Random Procedural".to_string(),
                        variables: vec![VariableInfo {
                            name: "random_matrix".to_string(),
                            data_type: "float32".to_string(),
                            shape: vec![64, 64],
                            dimension_names: vec!["y".to_string(), "x".to_string()],
                            chunk_shape: vec![64, 64],
                            file_size: crate::utils::calculate_variable_size_bytes(
                                &[64, 64],
                                "float32",
                            ),
                            ..Default::default()
                        }],
                        dimension_coordinates: std::collections::HashMap::new(),
                    };
                    let _ = tx.send(Ok(meta));
                    return;
                }
            };

            let res = store.inspect().map_err(|e| e.to_string());
            let _ = tx.send(res);
        });
    }
}
