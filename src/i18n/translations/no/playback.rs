use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Spill av",
        PlaybackKey::Pause => "Pause",
        PlaybackKey::Loop => "Gjenta",
        PlaybackKey::FirstStep => "Første steg",
        PlaybackKey::PreviousStep => "Forrige steg",
        PlaybackKey::NextStep => "Neste steg",
        PlaybackKey::LastStep => "Siste steg",
        PlaybackKey::Crop => "Beskjær",
        PlaybackKey::CropHover => "Hjelpelinjer for beskjæring (C)",
        PlaybackKey::PlaybackSpeed => "Avspillingshastighet",
        PlaybackKey::PlaybackBuffer => "Buffer før avspilling",
        PlaybackKey::SaveExportHover => {
            "Lagre / eksporter figur (Cmd+S for hurtiglagring, Cmd+Shift+S for innstillinger)"
        }
        PlaybackKey::CollapsedHint => "Avspilling (klikk eller dra for å utvide)",
        PlaybackKey::TimelineDetails => "Tidslinjedetaljer",
        PlaybackKey::TimelineDimension => "Dimensjon: {}",
        PlaybackKey::TimelineStep => "Steg: {} / {}",
        PlaybackKey::TimelineCurrent => "Nåværende: {}",
        PlaybackKey::TimelineRange => "Område: {} -> {}",
        PlaybackKey::TimelineStepSize => "Stegstørrelse: {}",
        PlaybackKey::DefaultStepSize => "Steg: 1",
    })
}
