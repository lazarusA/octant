//! Opening dropped files and folders: on the desktop their paths, on the
//! web the local sources dropped or picked (`data::backends::local`).

use super::{OctantApp, StoreKind};
use crate::i18n::{NoticesKey as N, tr};
use crate::ui::toast::Severity;

impl OctantApp {
    /// Opens the first file or folder dropped this frame.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn open_dropped(&mut self, ctx: &egui::Context) {
        let dropped_file = ctx.input(|i| i.raw.dropped_files.first().cloned());
        let Some(file) = dropped_file else {
            return;
        };
        let path_str = file.path().to_string_lossy().trim().to_string();
        if path_str.is_empty() {
            return;
        }
        match crate::utils::infer_store_kind_from_target(&path_str) {
            Ok(_) => self.open_dropped_source(ctx, &path_str, None),
            Err(_) => self.refuse_dropped(ctx, &path_str),
        }
    }

    /// Opens the local sources dropped or picked since the last frame. The
    /// page's own drop listener reads drops, so eframe's dropped files (no
    /// folders, read whole) are left alone.
    #[cfg(target_arch = "wasm32")]
    pub(super) fn open_dropped(&mut self, ctx: &egui::Context) {
        for opened in crate::data::backends::local::take_opened() {
            match opened {
                Ok((url, kind)) => {
                    let kind = StoreKind::from_data_source_kind(&kind);
                    self.open_dropped_source(ctx, &url, Some(kind));
                }
                Err(name) => self.refuse_dropped(ctx, &name),
            }
        }
    }

    fn open_dropped_source(&mut self, ctx: &egui::Context, target: &str, kind: Option<StoreKind>) {
        crate::ui::drop_zone::clear_drop_zone_warning(ctx);
        self.layout.hero_state.input = target.to_string();
        self.selected.store_target = target.to_string();
        self.submit_or_activate_source(target, kind);
    }

    /// Warns that the file or folder `name` cannot be opened.
    fn refuse_dropped(&mut self, ctx: &egui::Context, name: &str) {
        crate::ui::drop_zone::trigger_drop_zone_warning(ctx);
        self.notify(Severity::Warning, tr(N::UnsupportedFile), name.to_string());
    }
}
