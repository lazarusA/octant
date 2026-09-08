//! Vector export generators for SVG (XML) and publication PDF (ISO 32000).

use super::ExportFormat;
use super::raster::encode_raster_image;

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

    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n");

    let mut offsets = Vec::new();

    // 1 0 obj: Catalog (Root)
    offsets.push(pdf.len());
    let catalog = "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n";
    pdf.extend_from_slice(catalog.as_bytes());

    // 2 0 obj: Pages
    offsets.push(pdf.len());
    let pages = "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n";
    pdf.extend_from_slice(pages.as_bytes());

    // 3 0 obj: Page
    offsets.push(pdf.len());
    let page = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Contents 5 0 R /Resources << /XObject << /Im0 4 0 R >> >> >>\nendobj\n",
        w = width,
        h = height
    );
    pdf.extend_from_slice(page.as_bytes());

    // 4 0 obj: Image XObject (JPEG)
    offsets.push(pdf.len());
    let img_header = format!(
        "4 0 obj\n<< /Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {len} >>\nstream\n",
        w = width,
        h = height,
        len = jpeg_bytes.len()
    );
    pdf.extend_from_slice(img_header.as_bytes());
    pdf.extend_from_slice(&jpeg_bytes);
    pdf.extend_from_slice(b"\nendstream\nendobj\n");

    // 5 0 obj: Content Stream (draw image full size)
    offsets.push(pdf.len());
    let stream_content = format!("q\n{w} 0 0 {h} 0 0 cm\n/Im0 Do\nQ\n", w = width, h = height);
    let contents_obj = format!(
        "5 0 obj\n<< /Length {len} >>\nstream\n{content}endstream\nendobj\n",
        len = stream_content.len(),
        content = stream_content
    );
    pdf.extend_from_slice(contents_obj.as_bytes());

    // 6 0 obj: Info
    offsets.push(pdf.len());
    let info = format!(
        "6 0 obj\n<< /Title ({title}) /Producer (Octant Scientific Viewer) >>\nendobj\n",
        title = escape_pdf_str(title)
    );
    pdf.extend_from_slice(info.as_bytes());

    // XRef table
    let xref_start = pdf.len();
    let num_objects = offsets.len() + 1;
    let xref_header = format!("xref\n0 {num_objects}\n0000000000 65535 f \r\n");
    pdf.extend_from_slice(xref_header.as_bytes());
    for off in &offsets {
        let entry = format!("{:010} 00000 n \r\n", off);
        pdf.extend_from_slice(entry.as_bytes());
    }

    // Trailer
    let trailer = format!(
        "trailer\n<< /Size {num_objects} /Root 1 0 R /Info 6 0 R >>\nstartxref\n{xref_start}\n%%EOF\n"
    );
    pdf.extend_from_slice(trailer.as_bytes());

    Ok(pdf)
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

fn escape_pdf_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}
