use muxide::api::{MuxerBuilder, VideoCodec};
use openh264::encoder::Encoder;
use openh264::formats::{RgbaSliceU8, YUVBuffer};
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// Encodes a sequence of raw RGBA pixel buffers into a standard H.264 MP4 video file.
///
/// Each buffer in `frames` must contain `width * height * 4` bytes.
/// `width` and `height` are automatically aligned to even dimensions required by H.264 YUV420.
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

    let mut encoder =
        Encoder::new().map_err(|e| format!("Failed to initialize OpenH264 encoder: {}", e))?;

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

        let rgba_source = RgbaSliceU8::new(&formatted_rgba, (even_width, even_height));
        let yuv = YUVBuffer::from_rgb_source(rgba_source);

        let bitstream = encoder
            .encode(&yuv)
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
