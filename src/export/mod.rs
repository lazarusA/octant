//! Figure and Canvas Export Subsystem.

pub mod clipboard;
pub mod raster;
pub mod vector;

pub use clipboard::reveal_in_file_manager;
pub use raster::{crop_rgba_buffer, encode_raster_image, inject_display_p3_chunks};
pub use vector::{generate_pdf, generate_svg};

use std::path::{Path, PathBuf};

/// Supported export formats for saving figures and canvases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ExportFormat {
    #[default]
    Png,
    Jpeg,
    Webp,
    Svg,
    Pdf,
}

impl ExportFormat {
    pub const ALL: [Self; 5] = [Self::Png, Self::Jpeg, Self::Webp, Self::Svg, Self::Pdf];

    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Svg => "svg",
            Self::Pdf => "pdf",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Png => "PNG (Lossless)",
            Self::Jpeg => "JPEG (Lossy)",
            Self::Webp => "WebP (Modern)",
            Self::Svg => "SVG (Vector / Hybrid)",
            Self::Pdf => "PDF (Publication)",
        }
    }
}

/// Target area of the canvas to export.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ExportTarget {
    #[default]
    FullCanvas,
    RoiCrop,
}

/// Aspect ratio presets for the Region of Interest (ROI) crop box.
#[derive(Clone, Copy, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum AspectPreset {
    #[default]
    Freeform,
    Ratio16x9,
    Ratio4x3,
    Ratio1x1,
}

impl AspectPreset {
    pub const ALL: [Self; 4] = [
        Self::Freeform,
        Self::Ratio16x9,
        Self::Ratio4x3,
        Self::Ratio1x1,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Freeform => "Freeform",
            Self::Ratio16x9 => "16:9 (Slides)",
            Self::Ratio4x3 => "4:3 (Figure)",
            Self::Ratio1x1 => "1:1 (Square)",
        }
    }

    pub fn ratio(self) -> Option<f32> {
        match self {
            Self::Freeform => None,
            Self::Ratio16x9 => Some(16.0 / 9.0),
            Self::Ratio4x3 => Some(4.0 / 3.0),
            Self::Ratio1x1 => Some(1.0),
        }
    }
}

/// Interactive Region of Interest (ROI) crop box in normalized coordinates [0.0..=1.0].
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RoiCropBox {
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
    pub aspect: AspectPreset,
}

impl Default for RoiCropBox {
    fn default() -> Self {
        Self {
            u_min: 0.1,
            v_min: 0.1,
            u_max: 0.9,
            v_max: 0.9,
            aspect: AspectPreset::Freeform,
        }
    }
}

impl RoiCropBox {
    pub fn clamp_bounds(&mut self) {
        self.u_min = self.u_min.clamp(0.0, 0.95);
        self.v_min = self.v_min.clamp(0.0, 0.95);
        self.u_max = self.u_max.clamp(self.u_min + 0.05, 1.0);
        self.v_max = self.v_max.clamp(self.v_min + 0.05, 1.0);
    }
}

/// Generates a standardized timestamped export filename for a variable.
pub fn generate_export_filename(var_name: &str, format: ExportFormat) -> String {
    let now = web_time::SystemTime::now()
        .duration_since(web_time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let safe_var = var_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
    format!("octant_{}_{}.{}", safe_var, now, format.extension())
}

/// Returns default Downloads directory path or fallback.
pub fn default_downloads_dir() -> String {
    let dl = crate::utils::expand_tilde("~/Downloads");
    if dl.exists() {
        dl.to_string_lossy().to_string()
    } else {
        crate::utils::expand_tilde("~")
            .to_string_lossy()
            .to_string()
    }
}

/// Resolves user export directory strings (expanding ~ and relative paths).
pub fn resolve_export_path(dir: &str, filename: &str) -> PathBuf {
    crate::utils::expand_tilde(dir).join(filename)
}

/// User configurable export settings.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExportSettings {
    pub format: ExportFormat,
    pub target: ExportTarget,
    pub jpeg_quality: u8,
    pub export_dir: String,
    pub custom_filename: String,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: ExportFormat::Png,
            target: ExportTarget::FullCanvas,
            jpeg_quality: 90,
            export_dir: default_downloads_dir(),
            custom_filename: String::new(),
        }
    }
}

/// In-flight screenshot / export request dispatched to the frame lifecycle.
#[derive(Clone, Debug)]
pub struct PendingExportRequest {
    pub format: ExportFormat,
    pub target: ExportTarget,
    pub roi: RoiCropBox,
    pub jpeg_quality: u8,
    pub copy_to_clipboard: bool,
    pub output_path: Option<PathBuf>,
    pub canvas_rect_in_points: egui::Rect,
    pub pixels_per_point: f32,
}

impl PendingExportRequest {
    /// Computes the pixel-space crop rectangle `(crop_x, crop_y, crop_w, crop_h)` clamped to source bounds.
    pub fn compute_crop_rect(&self, full_w: u32, full_h: u32) -> (u32, u32, u32, u32) {
        let ppp = self.pixels_per_point.max(1.0);
        let cr = self.canvas_rect_in_points;
        let (u0, v0, u1, v1) = match self.target {
            ExportTarget::FullCanvas => (0.0, 0.0, 1.0, 1.0),
            ExportTarget::RoiCrop => (
                self.roi.u_min,
                self.roi.v_min,
                self.roi.u_max,
                self.roi.v_max,
            ),
        };
        let cx = ((cr.left() + u0 * cr.width()) * ppp).round() as u32;
        let cy = ((cr.top() + v0 * cr.height()) * ppp).round() as u32;
        let cw = (((u1 - u0) * cr.width()) * ppp).round() as u32;
        let ch = (((v1 - v0) * cr.height()) * ppp).round() as u32;
        let crop_x = cx.min(full_w.saturating_sub(1));
        let crop_y = cy.min(full_h.saturating_sub(1));
        let crop_w = cw.min(full_w.saturating_sub(crop_x)).max(1);
        let crop_h = ch.min(full_h.saturating_sub(crop_y)).max(1);
        (crop_x, crop_y, crop_w, crop_h)
    }
}

/// Encodes raw RGBA8 image pixels into any supported figure export format (PNG, JPEG, WebP, SVG, PDF).
pub fn encode_figure(
    rgba: &[u8],
    width: u32,
    height: u32,
    format: ExportFormat,
    quality: u8,
    title: &str,
    var_name: &str,
) -> Result<Vec<u8>, String> {
    match format {
        ExportFormat::Png | ExportFormat::Jpeg | ExportFormat::Webp => {
            encode_raster_image(rgba, width, height, format, quality)
        }
        ExportFormat::Svg => generate_svg(rgba, width, height, title, var_name),
        ExportFormat::Pdf => generate_pdf(rgba, width, height, title),
    }
}

/// Helper to write an exported file safely to disk (creates directories if missing).
pub fn save_exported_file(data: &[u8], path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create export directory: {}", e))?;
    }
    std::fs::write(path, data).map_err(|e| format!("Failed to write export file: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crop_rgba_buffer() {
        let src = vec![
            10, 10, 10, 255, 20, 20, 20, 255, 30, 30, 30, 255, 40, 40, 40, 255,
        ];
        let (cropped, w, h) = crop_rgba_buffer(&src, 2, 2, 1, 0, 1, 2);
        assert_eq!(w, 1);
        assert_eq!(h, 2);
        assert_eq!(cropped.len(), 8);
        assert_eq!(&cropped[0..4], &[20, 20, 20, 255]);
        assert_eq!(&cropped[4..8], &[40, 40, 40, 255]);
    }

    #[test]
    fn test_encode_png_and_jpeg() {
        let rgba = vec![255u8, 0, 0, 255, 0, 255, 0, 255];
        let png = encode_raster_image(&rgba, 2, 1, ExportFormat::Png, 90).unwrap();
        assert!(!png.is_empty());
        assert_eq!(&png[1..4], b"PNG");
        // Verify cHRM and gAMA chunks are injected
        assert!(png.windows(4).any(|w| w == b"cHRM"));
        assert!(png.windows(4).any(|w| w == b"gAMA"));

        let jpeg = encode_raster_image(&rgba, 2, 1, ExportFormat::Jpeg, 90).unwrap();
        assert!(!jpeg.is_empty());
        assert_eq!(&jpeg[0..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn test_generate_svg_and_pdf() {
        let rgba = vec![0u8, 0, 255, 255];
        let svg = generate_svg(&rgba, 1, 1, "Test Plot", "temp").unwrap();
        assert!(String::from_utf8(svg).unwrap().contains("<svg"));

        let pdf = generate_pdf(&rgba, 1, 1, "Test PDF").unwrap();
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n"));
        let pdf_str = String::from_utf8_lossy(&pdf);
        assert!(pdf_str.contains("/Type /Catalog"));
        assert!(pdf_str.contains("/Type /Pages"));
        assert!(pdf_str.contains("/Type /Page"));
        assert!(pdf_str.contains("xref"));
    }
}
