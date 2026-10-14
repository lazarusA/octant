use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Riproduci",
        PlaybackKey::Pause => "Pausa",
        PlaybackKey::Loop => "Ripeti",
        PlaybackKey::FirstStep => "Primo passo",
        PlaybackKey::PreviousStep => "Passo precedente",
        PlaybackKey::NextStep => "Passo successivo",
        PlaybackKey::LastStep => "Ultimo passo",
        PlaybackKey::Crop => "Ritaglia",
        PlaybackKey::CropHover => "Linee guida di ritaglio (C)",
        PlaybackKey::PlaybackSpeed => "Velocità di riproduzione",
        PlaybackKey::PlaybackBuffer => "Buffer prima della riproduzione",
        PlaybackKey::Buffering => "Caricamento buffer {}/{}",
        PlaybackKey::SaveExportHover => {
            "Salva / esporta figura (Cmd+S per il salvataggio rapido, Cmd+Shift+S per le impostazioni)"
        }
        PlaybackKey::CollapsedHint => "Riproduzione (clicca o trascina per espandere)",
        PlaybackKey::TimelineDetails => "Dettagli della sequenza",
        PlaybackKey::TimelineDimension => "Dimensione: {}",
        PlaybackKey::TimelineStep => "Passo: {} / {}",
        PlaybackKey::TimelineCurrent => "Attuale: {}",
        PlaybackKey::TimelineRange => "Intervallo: {} -> {}",
        PlaybackKey::TimelineStepSize => "Ampiezza del passo: {}",
        PlaybackKey::DefaultStepSize => "Passo: 1",
    })
}
