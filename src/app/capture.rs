use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

/// Standard aspect ratio presets for publication figures, presentations, and video formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AspectRatioPreset {
    FullCanvas,
    Widescreen16x9,
    Standard4x3,
    Square1x1,
    NaturePhoto3x2,
    Cinema2x1,
    Ultrawide21x9,
    DataAspect,
    Custom,
}

impl AspectRatioPreset {
    pub const ALL: [Self; 9] = [
        Self::FullCanvas,
        Self::Widescreen16x9,
        Self::Standard4x3,
        Self::Square1x1,
        Self::NaturePhoto3x2,
        Self::Cinema2x1,
        Self::Ultrawide21x9,
        Self::DataAspect,
        Self::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::FullCanvas => "Full Canvas",
            Self::Widescreen16x9 => "16:9 Widescreen (1080p/4K)",
            Self::Standard4x3 => "4:3 Classic Presentation",
            Self::Square1x1 => "1:1 Square (Journal Figure)",
            Self::NaturePhoto3x2 => "3:2 Nature/Science Column",
            Self::Cinema2x1 => "2:1 Global Cylindrical/Panorama",
            Self::Ultrawide21x9 => "21:9 Ultrawide Video",
            Self::DataAspect => "Data Aspect (Matrix Ratio)",
            Self::Custom => "Custom Ratio",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::FullCanvas => "Full",
            Self::Widescreen16x9 => "16:9",
            Self::Standard4x3 => "4:3",
            Self::Square1x1 => "1:1",
            Self::NaturePhoto3x2 => "3:2",
            Self::Cinema2x1 => "2:1",
            Self::Ultrawide21x9 => "21:9",
            Self::DataAspect => "Data",
            Self::Custom => "Custom",
        }
    }

    pub fn ratio(self, custom: (f32, f32), data_aspect: Option<f32>) -> Option<f32> {
        match self {
            Self::FullCanvas => None,
            Self::Widescreen16x9 => Some(16.0 / 9.0),
            Self::Standard4x3 => Some(4.0 / 3.0),
            Self::Square1x1 => Some(1.0),
            Self::NaturePhoto3x2 => Some(3.0 / 2.0),
            Self::Cinema2x1 => Some(2.0),
            Self::Ultrawide21x9 => Some(21.0 / 9.0),
            Self::DataAspect => data_aspect.or(Some(1.0)),
            Self::Custom => {
                let (w, h) = custom;
                if h > 0.0001 {
                    Some((w / h).clamp(0.1, 10.0))
                } else {
                    Some(1.0)
                }
            }
        }
    }
}

/// Configuration and in-flight state for canvas plot saving and interaction video recording.
#[derive(Clone)]
pub struct CaptureConfig {
    pub aspect_preset: AspectRatioPreset,
    pub custom_ratio: (f32, f32),
    pub scale_multiplier: f32,
    pub show_framing_guides: bool,
    pub output_dir: String,
    pub pending_save: bool,

    // Video recording state
    pub is_recording: bool,
    pub recording_fps: f32,
    pub recording_start_time: Option<std::time::Instant>,
    pub recorded_frames: Vec<Arc<[u8]>>,
    pub recorded_frame_size: (u32, u32),
    pub max_record_frames: usize,
    pub last_recording_time: std::time::Instant,

    // Feedback & Notifications
    pub shutter_flash_time: Option<std::time::Instant>,
    pub save_notification: Option<(String, PathBuf, std::time::Instant)>,
    pub last_canvas_rect: egui::Rect,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            aspect_preset: AspectRatioPreset::FullCanvas,
            custom_ratio: (16.0, 9.0),
            scale_multiplier: 1.0,
            show_framing_guides: false,
            output_dir: String::new(),
            pending_save: false,
            is_recording: false,
            recording_fps: 24.0,
            recording_start_time: None,
            recorded_frames: Vec::new(),
            recorded_frame_size: (0, 0),
            max_record_frames: 1800, // 75 seconds at 24 fps
            last_recording_time: std::time::Instant::now(),
            shutter_flash_time: None,
            save_notification: None,
            last_canvas_rect: egui::Rect::NOTHING,
        }
    }
}

impl CaptureConfig {
    /// Computes the effective framing/crop bounding rectangle within the canvas.
    pub fn compute_capture_rect(
        &self,
        canvas_rect: egui::Rect,
        data_aspect: Option<f32>,
    ) -> egui::Rect {
        let target_aspect = self.aspect_preset.ratio(self.custom_ratio, data_aspect);

        let Some(target_ratio) = target_aspect else {
            return canvas_rect;
        };

        let pad_margin = 16.0;
        let available_w = (canvas_rect.width() - pad_margin * 2.0).max(32.0);
        let available_h = (canvas_rect.height() - pad_margin * 2.0).max(32.0);
        let available_ratio = available_w / available_h;

        let (rect_w, rect_h) = if available_ratio > target_ratio {
            (available_h * target_ratio, available_h)
        } else {
            (available_w, available_w / target_ratio)
        };

        egui::Rect::from_center_size(canvas_rect.center(), egui::vec2(rect_w, rect_h))
    }

    /// Resolves the destination directory for saved files (defaults to ~/Downloads).
    pub fn resolve_output_dir(&self, _is_video: bool) -> PathBuf {
        if !self.output_dir.trim().is_empty() {
            let p = PathBuf::from(self.output_dir.trim());
            if p.is_absolute() || p.exists() {
                return p;
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(home) = std::env::var_os("HOME") {
                let base = PathBuf::from(home);
                let target = base.join("Downloads");
                if target.exists() || std::fs::create_dir_all(&target).is_ok() {
                    return target;
                }
            }
        }

        PathBuf::from(".")
    }

    /// Generates a formatted timestamp string (YYYYMMDD_HHMMSS).
    pub fn timestamp_suffix() -> String {
        let now = SystemTime::now();
        let duration = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs();

        let sec_of_day = secs % 86400;
        let hour = sec_of_day / 3600;
        let minute = (sec_of_day % 3600) / 60;
        let second = sec_of_day % 60;

        let days = secs / 86400;
        let mut y = 1970;
        let mut d = days;

        loop {
            let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
            let days_in_year = if leap { 366 } else { 365 };
            if d < days_in_year {
                break;
            }
            d -= days_in_year;
            y += 1;
        }

        let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
        let month_days = [
            31,
            if leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        let mut m = 1;
        for &md in &month_days {
            if d < md {
                break;
            }
            d -= md;
            m += 1;
        }
        let day = d + 1;

        format!(
            "{:04}{:02}{:02}_{:02}{:02}{:02}",
            y, m, day, hour, minute, second
        )
    }

    /// Generates a timestamped output filepath.
    pub fn generate_filepath(&self, is_video: bool) -> PathBuf {
        let dir = self.resolve_output_dir(is_video);
        let ext = if is_video { "mp4" } else { "png" };
        let prefix = if is_video {
            "octant_recording"
        } else {
            "octant_plot"
        };
        let name = format!("{}_{}.{}", prefix, Self::timestamp_suffix(), ext);
        dir.join(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aspect_ratio_presets() {
        assert_eq!(AspectRatioPreset::FullCanvas.ratio((1.0, 1.0), None), None);
        assert_eq!(
            AspectRatioPreset::Widescreen16x9.ratio((1.0, 1.0), None),
            Some(16.0 / 9.0)
        );
        assert_eq!(
            AspectRatioPreset::Standard4x3.ratio((1.0, 1.0), None),
            Some(4.0 / 3.0)
        );
        assert_eq!(
            AspectRatioPreset::Square1x1.ratio((1.0, 1.0), None),
            Some(1.0)
        );
        assert_eq!(
            AspectRatioPreset::NaturePhoto3x2.ratio((1.0, 1.0), None),
            Some(1.5)
        );
        assert_eq!(
            AspectRatioPreset::Cinema2x1.ratio((1.0, 1.0), None),
            Some(2.0)
        );
        assert_eq!(
            AspectRatioPreset::Ultrawide21x9.ratio((1.0, 1.0), None),
            Some(21.0 / 9.0)
        );
        assert_eq!(
            AspectRatioPreset::DataAspect.ratio((1.0, 1.0), Some(2.5)),
            Some(2.5)
        );
        assert_eq!(
            AspectRatioPreset::Custom.ratio((1920.0, 1080.0), None),
            Some(1920.0 / 1080.0)
        );
    }

    #[test]
    fn test_compute_capture_rect_fitting() {
        let config = CaptureConfig {
            aspect_preset: AspectRatioPreset::Widescreen16x9,
            ..Default::default()
        };
        let canvas =
            egui::Rect::from_min_size(egui::pos2(100.0, 100.0), egui::vec2(1920.0, 1080.0));
        let rect = config.compute_capture_rect(canvas, None);

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
        assert!(canvas.contains_rect(rect));

        let aspect = rect.width() / rect.height();
        assert!((aspect - (16.0 / 9.0)).abs() < 0.01);
    }

    #[test]
    fn test_timestamp_suffix() {
        let suffix = CaptureConfig::timestamp_suffix();
        assert_eq!(suffix.len(), 15);
        assert!(suffix.contains('_'));
    }
}
