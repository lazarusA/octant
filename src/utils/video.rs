use muxide::api::{MuxerBuilder, VideoCodec};
use openh264::encoder::{Encoder, EncoderConfig};
use openh264::formats::YUVSource;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// A full-range ITU-R BT.709 YUV420 frame implementing `YUVSource`.
pub struct Bt709YuvFrame {
    pub width: usize,
    pub height: usize,
    pub y: Vec<u8>,
    pub u: Vec<u8>,
    pub v: Vec<u8>,
}

impl YUVSource for Bt709YuvFrame {
    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn strides(&self) -> (usize, usize, usize) {
        (self.width, self.width / 2, self.width / 2)
    }

    fn y(&self) -> &[u8] {
        &self.y
    }

    fn u(&self) -> &[u8] {
        &self.u
    }

    fn v(&self) -> &[u8] {
        &self.v
    }
}

/// Converts a raw RGBA buffer into ITU-R BT.709 Full-Range YUV420 planes.
///
/// Unlike standard WebRTC BT.601 limited-range (16-235), BT.709 Full-Range preserves 100%
/// of the original RGB dynamic range [0..255] and vivid colormap saturation.
pub fn rgba_to_bt709_full_range_frame(rgba: &[u8], width: usize, height: usize) -> Bt709YuvFrame {
    let half_w = width / 2;
    let half_h = height / 2;
    let mut y_plane = vec![0u8; width * height];
    let mut u_plane = vec![0u8; half_w * half_h];
    let mut v_plane = vec![0u8; half_w * half_h];

    // Compute Y plane (full resolution)
    for row in 0..height {
        let row_offset = row * width;
        for col in 0..width {
            let px_idx = (row_offset + col) * 4;
            if px_idx + 2 < rgba.len() {
                let r = rgba[px_idx] as i32;
                let g = rgba[px_idx + 1] as i32;
                let b = rgba[px_idx + 2] as i32;

                // ITU-R BT.709 Full-Range Y: 0.2126 R + 0.7152 G + 0.0722 B
                let y = (13933 * r + 46871 * g + 4732 * b + 32768) >> 16;
                y_plane[row_offset + col] = y.clamp(0, 255) as u8;
            }
        }
    }

    // Compute U and V planes with 2x2 box filtering for smooth chroma
    for uv_row in 0..half_h {
        let y_row0 = uv_row * 2;
        let y_row1 = y_row0 + 1;
        let uv_row_offset = uv_row * half_w;

        for uv_col in 0..half_w {
            let x0 = uv_col * 2;
            let x1 = x0 + 1;

            // Sample 2x2 block
            let mut r_sum = 0i32;
            let mut g_sum = 0i32;
            let mut b_sum = 0i32;
            let mut count = 0i32;

            for &(r_idx, c_idx) in &[(y_row0, x0), (y_row0, x1), (y_row1, x0), (y_row1, x1)] {
                if r_idx < height && c_idx < width {
                    let px_idx = (r_idx * width + c_idx) * 4;
                    if px_idx + 2 < rgba.len() {
                        r_sum += rgba[px_idx] as i32;
                        g_sum += rgba[px_idx + 1] as i32;
                        b_sum += rgba[px_idx + 2] as i32;
                        count += 1;
                    }
                }
            }

            if count > 0 {
                let r = r_sum / count;
                let g = g_sum / count;
                let b = b_sum / count;

                // ITU-R BT.709 Full-Range U: (B - Y) * 0.5389 + 128
                let u = (-7510 * r - 25258 * g + 32768 * b + 8388608) >> 16;
                // ITU-R BT.709 Full-Range V: (R - Y) * 0.6350 + 128
                let v = (32768 * r - 29766 * g - 3002 * b + 8388608) >> 16;

                let uv_idx = uv_row_offset + uv_col;
                u_plane[uv_idx] = u.clamp(0, 255) as u8;
                v_plane[uv_idx] = v.clamp(0, 255) as u8;
            }
        }
    }

    Bt709YuvFrame {
        width,
        height,
        y: y_plane,
        u: u_plane,
        v: v_plane,
    }
}

/// Encodes a sequence of raw RGBA pixel buffers into a standard H.264 MP4 video file.
///
/// Each buffer in `frames` must contain `width * height * 4` bytes.
/// Uses ITU-R BT.709 full dynamic range conversion for vivid color preservation.
pub fn encode_rgba_frames_to_mp4(
    frames: &[Arc<[u8]>],
    width: u32,
    height: u32,
    fps: f32,
    output_path: &Path,
) -> Result<(), String> {
    if frames.is_empty() {
        return Err("No frames to encode".to_string());
    }

    // H.264 YUV420 requires dimensions to be even numbers
    let even_width = (width.max(2) & !1) as usize;
    let even_height = (height.max(2) & !1) as usize;
    let valid_fps = fps.clamp(1.0, 120.0);

    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        let _ = std::fs::create_dir_all(parent);
    }

    let file = File::create(output_path).map_err(|e| {
        format!(
            "Failed to create video file {}: {}",
            output_path.display(),
            e
        )
    })?;

    let mut muxer = MuxerBuilder::new(file)
        .video(
            VideoCodec::H264,
            even_width as u32,
            even_height as u32,
            valid_fps as f64,
        )
        .build()
        .map_err(|e| format!("Failed to initialize MP4 muxer: {}", e))?;

    let api = openh264::OpenH264API::from_source();
    let config = EncoderConfig::new()
        .max_frame_rate(valid_fps)
        .set_bitrate_bps(30_000_000);
    let mut encoder = Encoder::with_api_config(api, config)
        .map_err(|e| format!("Failed to initialize OpenH264 encoder: {}", e))?;

    let frame_duration_secs = 1.0 / valid_fps as f64;

    for (i, frame_bytes) in frames.iter().enumerate() {
        // If dimensions need padding/cropping to even sizes, handle safely
        let formatted_rgba: Vec<u8> =
            if even_width != width as usize || even_height != height as usize {
                let mut buf = vec![0u8; even_width * even_height * 4];
                let copy_w = (width as usize).min(even_width);
                let copy_h = (height as usize).min(even_height);
                for y in 0..copy_h {
                    let src_start = y * (width as usize) * 4;
                    let src_end = src_start + copy_w * 4;
                    let dst_start = y * even_width * 4;
                    let dst_end = dst_start + copy_w * 4;
                    if src_end <= frame_bytes.len() && dst_end <= buf.len() {
                        buf[dst_start..dst_end].copy_from_slice(&frame_bytes[src_start..src_end]);
                    }
                }
                buf
            } else {
                frame_bytes.to_vec()
            };

        let yuv_frame = rgba_to_bt709_full_range_frame(&formatted_rgba, even_width, even_height);

        let bitstream = encoder
            .encode(&yuv_frame)
            .map_err(|e| format!("Failed to encode frame {}: {}", i, e))?;

        let raw_stream = bitstream.to_vec();
        if !raw_stream.is_empty() {
            let pts = (i as f64) * frame_duration_secs;
            let is_keyframe = (i % (valid_fps.round() as usize * 2).max(1)) == 0;
            muxer
                .write_video(pts, &raw_stream, is_keyframe)
                .map_err(|e| format!("Failed to write MP4 frame {}: {}", i, e))?;
        }
    }

    muxer
        .finish_with_stats()
        .map_err(|e| format!("Failed to finalize MP4 file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_rgba_frames_to_mp4() {
        let width = 64;
        let height = 64;
        let frame_size = width * height * 4;

        let mut frames = Vec::new();
        for f in 0..5 {
            let mut frame = vec![0u8; frame_size];
            for i in 0..(width * height) {
                frame[i * 4] = (f * 40) as u8; // R
                frame[i * 4 + 1] = 120; // G
                frame[i * 4 + 2] = 200; // B
                frame[i * 4 + 3] = 255; // A
            }
            frames.push(Arc::from(frame.into_boxed_slice()));
        }

        let temp_dir = std::env::temp_dir();
        let test_output = temp_dir.join("octant_test_output.mp4");

        let res =
            encode_rgba_frames_to_mp4(&frames, width as u32, height as u32, 24.0, &test_output);
        assert!(res.is_ok(), "Encoding failed: {:?}", res.err());
        assert!(test_output.exists());

        let metadata = std::fs::metadata(&test_output);
        assert!(metadata.is_ok());
        assert!(metadata.unwrap().len() > 0);

        let _ = std::fs::remove_file(&test_output);
    }
}
