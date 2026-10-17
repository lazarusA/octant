//! Frame lifecycle handling for in-flight figure export and screenshot requests.

use crate::app::OctantApp;
use crate::i18n::{ExportKey, filled, tr};
#[cfg(not(target_arch = "wasm32"))]
use crate::ui::toast::ToastAction;
use crate::ui::toast::{Notice, Severity};

impl OctantApp {
    /// Encodes the pending export once its screenshot has arrived.
    pub(crate) fn process_pending_export(&mut self) {
        let Some(req) = self.pending_export.take() else {
            return;
        };

        if req.canvas_rect_in_points == egui::Rect::NOTHING {
            self.pending_export = Some(req);
            return;
        }

        let screenshot = req
            .screenshot
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take();
        let Some(image) = screenshot else {
            self.pending_export = Some(req);
            return;
        };

        self.export_flash_timer = Some(web_time::Instant::now());
        let (crop_x, crop_y, crop_w, crop_h) =
            req.compute_crop_rect(image.width() as u32, image.height() as u32);
        let rgba: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();

        let (cropped_rgba, final_w, final_h) = crate::export::crop_rgba_buffer(
            &rgba,
            image.width() as u32,
            image.height() as u32,
            crop_x,
            crop_y,
            crop_w,
            crop_h,
        );

        let var_name = self
            .plotted_variable_info()
            .map(|v| v.name.as_str())
            .unwrap_or("plot");
        let title = format!("Octant - {}", var_name);

        match crate::export::encode_figure(
            &cropped_rgba,
            final_w,
            final_h,
            req.format,
            req.jpeg_quality,
            &title,
            var_name,
        ) {
            Ok(data) => {
                if req.copy_to_clipboard {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let img_data = arboard::ImageData {
                                width: final_w as usize,
                                height: final_h as usize,
                                bytes: std::borrow::Cow::Borrowed(&cropped_rgba),
                            };
                            let _ = clipboard.set_image(img_data);
                        }
                    }
                    self.status_message = "Copied figure to clipboard".to_string();
                } else if let Some(ref path) = req.output_path {
                    if let Err(e) = crate::export::save_exported_file(&data, path, req.format) {
                        self.status_message = format!("Export error: {}", e);
                        self.notify(Severity::Error, tr(ExportKey::ExportFailed), e);
                    } else {
                        let filename = path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| tr(ExportKey::FallbackFilename).to_string());
                        self.status_message = format!("Saved figure to {}", path.display());
                        let notice = Notice::new(Severity::Success, tr(ExportKey::Saved), filename);
                        // The browser keeps its downloads: there is no file to reveal.
                        #[cfg(not(target_arch = "wasm32"))]
                        let notice = notice.with_action(ToastAction::RevealFile(path.clone()));
                        self.push_notice(notice);
                    }
                }
            }
            Err(err) => {
                self.status_message = format!("Encoding error: {}", err);
                self.notify(
                    Severity::Error,
                    tr(ExportKey::ExportFailed),
                    filled(tr(ExportKey::EncodingError), &[&err]),
                );
            }
        }
    }
}
