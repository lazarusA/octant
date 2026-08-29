use std::io::Cursor;
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

/// Resolution / DPI multiplier for export.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ResolutionScale {
    #[default]
    Scale1x,
    Scale2x,
    Scale4x,
}

impl ResolutionScale {
    pub fn multiplier(self) -> u32 {
        match self {
            Self::Scale1x => 1,
            Self::Scale2x => 2,
            Self::Scale4x => 4,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Scale1x => "1× (Viewport)",
            Self::Scale2x => "2× (High-DPI / 2K)",
            Self::Scale4x => "4× (Print / 4K UHD)",
        }
    }
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

/// Returns default Downloads directory path or fallback.
pub fn default_downloads_dir() -> String {
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let p = PathBuf::from(&home).join("Downloads");
        if p.exists() {
            return p.to_string_lossy().to_string();
        }
        return home;
    }
    ".".to_string()
}

/// Resolves user export directory strings (expanding ~ and relative paths).
pub fn resolve_export_path(dir: &str, filename: &str) -> PathBuf {
    let get_home = || {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .ok()
    };
    let base = if let Some(stripped) = dir.strip_prefix("~/") {
        if let Some(home) = get_home() {
            home.join(stripped)
        } else {
            PathBuf::from(dir)
        }
    } else if dir == "~" {
        if let Some(home) = get_home() {
            home
        } else {
            PathBuf::from(dir)
        }
    } else {
        PathBuf::from(dir)
    };
    base.join(filename)
}

/// User configurable export settings.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExportSettings {
    pub format: ExportFormat,
    pub target: ExportTarget,
    pub scale: ResolutionScale,
    pub jpeg_quality: u8,
    pub export_dir: String,
    pub custom_filename: String,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: ExportFormat::Png,
            target: ExportTarget::FullCanvas,
            scale: ResolutionScale::Scale1x,
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

/// Crops a sub-rectangle from a raw RGBA8 image buffer.
pub fn crop_rgba_buffer(
    source: &[u8],
    src_w: u32,
    src_h: u32,
    crop_x: u32,
    crop_y: u32,
    crop_w: u32,
    crop_h: u32,
) -> (Vec<u8>, u32, u32) {
    let crop_x = crop_x.min(src_w.saturating_sub(1));
    let crop_y = crop_y.min(src_h.saturating_sub(1));
    let crop_w = crop_w.min(src_w.saturating_sub(crop_x)).max(1);
    let crop_h = crop_h.min(src_h.saturating_sub(crop_y)).max(1);

    let bytes_per_pixel = 4usize;
    let mut out = vec![0u8; (crop_w as usize) * (crop_h as usize) * bytes_per_pixel];

    for row in 0..crop_h {
        let src_row = crop_y + row;
        let src_start = ((src_row * src_w + crop_x) as usize) * bytes_per_pixel;
        let src_end = src_start + (crop_w as usize) * bytes_per_pixel;

        let dst_start = (row as usize) * (crop_w as usize) * bytes_per_pixel;
        let dst_end = dst_start + (crop_w as usize) * bytes_per_pixel;

        if src_end <= source.len() && dst_end <= out.len() {
            out[dst_start..dst_end].copy_from_slice(&source[src_start..src_end]);
        }
    }

    (out, crop_w, crop_h)
}

/// Encodes raw RGBA8 image pixels into PNG, JPEG, or WebP byte streams.
pub fn encode_raster_image(
    rgba: &[u8],
    width: u32,
    height: u32,
    format: ExportFormat,
    quality: u8,
) -> Result<Vec<u8>, String> {
    if rgba.len() != (width as usize) * (height as usize) * 4 {
        return Err(format!(
            "Invalid RGBA buffer length: expected {}, got {}",
            width * height * 4,
            rgba.len()
        ));
    }

    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);

    match format {
        ExportFormat::Png => {
            let img = image::RgbaImage::from_raw(width, height, rgba.to_vec())
                .ok_or_else(|| "Failed to construct RGBA image for PNG encoding".to_string())?;
            img.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)
                .map_err(|e| format!("PNG encoding error: {}", e))?;
            buffer = inject_display_p3_chunks(&buffer)?;
        }
        ExportFormat::Jpeg => {
            let img = image::RgbaImage::from_raw(width, height, rgba.to_vec())
                .ok_or_else(|| "Failed to construct RGBA image for JPEG encoding".to_string())?;
            let rgb = image::DynamicImage::ImageRgba8(img).to_rgb8();
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(cursor, quality.clamp(1, 100));
            encoder
                .encode_image(&rgb)
                .map_err(|e| format!("JPEG encoding error: {}", e))?;
        }
        ExportFormat::Webp => {
            let img = image::RgbaImage::from_raw(width, height, rgba.to_vec())
                .ok_or_else(|| "Failed to construct RGBA image for WebP encoding".to_string())?;
            img.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::WebP)
                .map_err(|e| format!("WebP encoding error: {}", e))?;
        }
        ExportFormat::Svg | ExportFormat::Pdf => {
            return Err("Vector formats (SVG/PDF) use dedicated generators".to_string());
        }
    }

    Ok(buffer)
}

/// Generates a clean hybrid or pure vector SVG XML document.
pub fn generate_svg(
    rgba: &[u8],
    width: u32,
    height: u32,
    title: &str,
    var_name: &str,
) -> Result<Vec<u8>, String> {
    let png_bytes = encode_raster_image(rgba, width, height, ExportFormat::Png, 100)?;
    let base64_png = format!("data:image/png;base64,{}", base64_encode(&png_bytes));

    let svg = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 {width} {height}" width="{width}" height="{height}">
  <title>{title}</title>
  <desc>Octant Scientific Visualization - {var_name}</desc>
  <image width="{width}" height="{height}" xlink:href="{base64_png}" />
</svg>
"#,
        width = width,
        height = height,
        title = escape_xml(title),
        var_name = escape_xml(var_name),
        base64_png = base64_png
    );

    Ok(svg.into_bytes())
}

/// Generates a standard single-page publication PDF embedding the figure.
pub fn generate_pdf(rgba: &[u8], width: u32, height: u32, title: &str) -> Result<Vec<u8>, String> {
    let jpeg_bytes = encode_raster_image(rgba, width, height, ExportFormat::Jpeg, 95)?;

    // Minimal compliant PDF 1.4 document with embedded JPEG XObject
    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n");

    let mut offsets = Vec::new();

    // 1 0 obj: Pages
    offsets.push(pdf.len());
    let pages = "1 0 obj\n<< /Type /Pages /Kids [2 0 R] /Count 1 >>\nendobj\n";
    pdf.extend_from_slice(pages.as_bytes());

    // 2 0 obj: Page
    offsets.push(pdf.len());
    let page = format!(
        "2 0 obj\n<< /Type /Page /Parent 1 0 R /MediaBox [0 0 {w} {h}] /Contents 4 0 R /Resources << /XObject << /Im0 3 0 R >> >> >>\nendobj\n",
        w = width,
        h = height
    );
    pdf.extend_from_slice(page.as_bytes());

    // 3 0 obj: Image XObject (JPEG)
    offsets.push(pdf.len());
    let img_header = format!(
        "3 0 obj\n<< /Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {len} >>\nstream\n",
        w = width,
        h = height,
        len = jpeg_bytes.len()
    );
    pdf.extend_from_slice(img_header.as_bytes());
    pdf.extend_from_slice(&jpeg_bytes);
    pdf.extend_from_slice(b"\nendstream\nendobj\n");

    // 4 0 obj: Content Stream (draw image full size)
    offsets.push(pdf.len());
    let stream_content = format!("q\n{w} 0 0 {h} 0 0 cm\n/Im0 Do\nQ\n", w = width, h = height);
    let contents_obj = format!(
        "4 0 obj\n<< /Length {len} >>\nstream\n{content}endstream\nendobj\n",
        len = stream_content.len(),
        content = stream_content
    );
    pdf.extend_from_slice(contents_obj.as_bytes());

    // 5 0 obj: Info
    offsets.push(pdf.len());
    let info = format!(
        "5 0 obj\n<< /Title ({title}) /Producer (Octant Scientific Viewer) >>\nendobj\n",
        title = escape_pdf_str(title)
    );
    pdf.extend_from_slice(info.as_bytes());

    // XRef table
    let xref_start = pdf.len();
    pdf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for off in &offsets {
        let entry = format!("{:010} 00000 n \n", off);
        pdf.extend_from_slice(entry.as_bytes());
    }

    // Trailer
    let trailer = format!(
        "trailer\n<< /Size 6 /Root 1 0 R /Info 5 0 R >>\nstartxref\n{xref_start}\n%%EOF\n",
        xref_start = xref_start
    );
    pdf.extend_from_slice(trailer.as_bytes());

    Ok(pdf)
}

/// Helper to write an exported file safely to disk (creates directories if missing).
pub fn save_exported_file(data: &[u8], path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create export directory: {}", e))?;
    }
    std::fs::write(path, data).map_err(|e| format!("Failed to write export file: {}", e))
}

/// Notification payload for a successful figure export.
#[derive(Clone, Debug)]
pub struct ExportToastNotification {
    pub file_path: PathBuf,
    pub filename: String,
    pub timestamp: std::time::Instant,
}

/// Reveals a file in the native file manager (Finder on macOS, Explorer on Windows, xdg-open on Linux).
pub fn reveal_in_file_manager(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(parent) = path.parent() {
            let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);

        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        result.push(CHARSET[((n >> 18) & 63) as usize] as char);
        result.push(CHARSET[((n >> 12) & 63) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARSET[((n >> 6) & 63) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARSET[(n & 63) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Computes standard IEEE 802.3 CRC32 used in PNG chunks.
fn png_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// Builds a self-contained PNG chunk: `[length (4B)][type (4B)][data][crc32 (4B)]`.
fn build_png_chunk(chunk_type: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut chunk = Vec::with_capacity(4 + 4 + data.len() + 4);
    chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());
    chunk.extend_from_slice(chunk_type);
    chunk.extend_from_slice(data);

    let mut crc_buf = Vec::with_capacity(4 + data.len());
    crc_buf.extend_from_slice(chunk_type);
    crc_buf.extend_from_slice(data);
    let crc = png_crc32(&crc_buf);
    chunk.extend_from_slice(&crc.to_be_bytes());

    chunk
}

/// Injects Apple Display P3 color primaries (`cHRM`) and Gamma 2.2 (`gAMA`) chunks into a raw PNG byte stream.
///
/// This ensures macOS Preview, Finder QuickLook, Safari, and Adobe Photoshop render the exact
/// wide-gamut saturation and contrast seen on Apple Retina displays.
pub fn inject_display_p3_chunks(raw_png: &[u8]) -> Result<Vec<u8>, String> {
    const PNG_HEADER: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if raw_png.len() < 33 || &raw_png[0..8] != PNG_HEADER {
        return Err("Invalid PNG data header".to_string());
    }

    // Standard IHDR chunk is at offset 8:
    // Length: 4 bytes (13) + Type: 4 bytes (IHDR) + Data: 13 bytes + CRC: 4 bytes = 25 bytes
    // Total offset after IHDR: 8 + 25 = 33 bytes.
    let ihdr_end = 33;
    if raw_png.len() < ihdr_end || &raw_png[12..16] != b"IHDR" {
        return Err("Corrupted or unexpected PNG IHDR chunk structure".to_string());
    }

    // 1. Display P3 Primary Chromaticities (`cHRM` chunk, D65 White Point + DCI-P3 Primaries)
    // Scaled by 100,000 as required by the PNG specification:
    // White point: x=0.3127, y=0.3290 -> 31270, 32900
    // Red:         x=0.6800, y=0.3200 -> 68000, 32000
    // Green:       x=0.2650, y=0.6900 -> 26500, 69000
    // Blue:        x=0.1500, y=0.0600 -> 15000,  6000
    let mut chrm_data = Vec::with_capacity(32);
    chrm_data.extend_from_slice(&31270u32.to_be_bytes());
    chrm_data.extend_from_slice(&32900u32.to_be_bytes());
    chrm_data.extend_from_slice(&68000u32.to_be_bytes());
    chrm_data.extend_from_slice(&32000u32.to_be_bytes());
    chrm_data.extend_from_slice(&26500u32.to_be_bytes());
    chrm_data.extend_from_slice(&69000u32.to_be_bytes());
    chrm_data.extend_from_slice(&15000u32.to_be_bytes());
    chrm_data.extend_from_slice(&6000u32.to_be_bytes());
    let chrm_chunk = build_png_chunk(b"cHRM", &chrm_data);

    // 2. Display Gamma (`gAMA` chunk, 1 / 2.2 = 0.45455 -> 45455)
    let mut gama_data = Vec::with_capacity(4);
    gama_data.extend_from_slice(&45455u32.to_be_bytes());
    let gama_chunk = build_png_chunk(b"gAMA", &gama_data);

    // Splice in cHRM and gAMA chunks immediately after IHDR (omitting conflicting sRGB chunk to avoid double-gamma curve)
    let mut output = Vec::with_capacity(raw_png.len() + chrm_chunk.len() + gama_chunk.len());
    output.extend_from_slice(&raw_png[..ihdr_end]);
    output.extend_from_slice(&chrm_chunk);
    output.extend_from_slice(&gama_chunk);
    output.extend_from_slice(&raw_png[ihdr_end..]);

    Ok(output)
}

fn escape_pdf_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
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
    }
}
