use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> &'static str {
    match key {
        NoticesKey::DismissHover => "Dismiss",
        NoticesKey::CopyDetails => "Copy details",
        NoticesKey::RevealInFinder => "Reveal in Finder",
        NoticesKey::OpenFolder => "Open Folder",
        NoticesKey::NoVariablesFound => "No variables found",
        NoticesKey::CouldntOpenDataset => "Couldn't open dataset",
        NoticesKey::UnsupportedFile => "Unsupported file",
        NoticesKey::CouldntLoadData => "Couldn't load data",
        NoticesKey::CoordinatesUnavailable => "Coordinates unavailable",
        NoticesKey::CoordinatesFailedDetail => {
            "Coordinates of '{}' failed to load ({}); axes show indices"
        }
        NoticesKey::CoastlinesUnavailable => "Coastlines unavailable",
        NoticesKey::VolumeTooLargeForGpu => "Volume too large for the GPU",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "Volume data buffer size ({} elements / {} bytes) exceeds maximum GPU storage buffer limit ({})"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "Volume {}x{}x{} exceeds GPU 3D texture limit ({})"
        }
        NoticesKey::VolumeUploadSkipped => "Volume upload skipped",
        NoticesKey::VolumeUploadMismatchDetail => "{} values for a {}x{}x{} texture",
        NoticesKey::DataTooLargeForGpuBuffer => "Data too large for the GPU buffer",
        NoticesKey::BufferTooLargeDetail => {
            "{}: data size ({} bytes) exceeds GPU buffer capacity ({} bytes), skipping write"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: range at element {} (+{} elements) exceeds GPU buffer capacity ({} bytes), skipping write"
        }
        NoticesKey::LinePlotTooLargeForGpu => "Line plot too large for the GPU",
        NoticesKey::LinePayloadTooLargeDetail => {
            "The lines take {} bytes; this GPU binds at most {}."
        }
        NoticesKey::FontLoadFailed => "Couldn't load a font",
        NoticesKey::FontLoadFailedDetail => "Text in {} may show as boxes: {}",
        NoticesKey::DataFontMissing => "No font for some dataset text",
        NoticesKey::DataFontMissingDetail => {
            "{} text in this dataset may show as boxes. Install a font such as Noto Sans CJK."
        }
    }
}
