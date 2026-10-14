use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> &'static str {
    match key {
        PlaybackKey::Play => "Play",
        PlaybackKey::Pause => "Pause",
        PlaybackKey::Loop => "Loop",
        PlaybackKey::FirstStep => "First Step",
        PlaybackKey::PreviousStep => "Previous Step",
        PlaybackKey::NextStep => "Next Step",
        PlaybackKey::LastStep => "Last Step",
        PlaybackKey::Crop => "Crop",
        PlaybackKey::CropHover => "Crop Guiding Lines (C)",
        PlaybackKey::PlaybackSpeed => "Playback Speed",
        PlaybackKey::PlaybackBuffer => "Buffer Before Playing",
        PlaybackKey::SaveExportHover => {
            "Save / Export Figure (Cmd+S for Quick Save, Cmd+Shift+S for Settings)"
        }
        PlaybackKey::CollapsedHint => "Playback (click or drag to expand)",
        PlaybackKey::TimelineDetails => "Timeline Details",
        PlaybackKey::TimelineDimension => "Dimension: {}",
        PlaybackKey::TimelineStep => "Step: {} / {}",
        PlaybackKey::TimelineCurrent => "Current: {}",
        PlaybackKey::TimelineRange => "Range: {} -> {}",
        PlaybackKey::TimelineStepSize => "Step Size: {}",
        PlaybackKey::DefaultStepSize => "Step: 1",
    }
}
