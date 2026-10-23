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
        PlaybackKey::CropHover => "Hjelpelinjer for beskjæring",
        PlaybackKey::PlaybackSpeed => "Avspillingshastighet",
        PlaybackKey::PlaybackBuffer => "Buffer før avspilling",
        PlaybackKey::Buffering => "Bufrer {}/{}",
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
        PlaybackKey::VoiceListen => "Start lytting",
        PlaybackKey::VoiceStop => "Stopp lytting",
        PlaybackKey::VoiceListening => "Lytter...",
        PlaybackKey::VoiceMicOff => "Mikrofon av",
        PlaybackKey::VoiceMicError => "Mikrofon utilgjengelig",
        PlaybackKey::VoiceSilent => "Mikrofonen sender bare stillhet",
        PlaybackKey::VoiceSilentHint => {
            "Gi appen som startet Octant tilgang til mikrofonen i systemets personverninnstillinger, og start den på nytt."
        }
        PlaybackKey::VoiceSttFailed => "Transkripsjonen mislyktes",
        PlaybackKey::VoiceDownloading => "Laster ned talemodellen: {} %",
        PlaybackKey::VoiceLoading => "Laster inn talemodellen...",
        PlaybackKey::VoiceModelHover => "Modell for talegjenkjenning",
        PlaybackKey::VoiceModelSize => "{} ({} MB)",
        PlaybackKey::VoiceCloseHover => "Lukk taleinndata",
    })
}
