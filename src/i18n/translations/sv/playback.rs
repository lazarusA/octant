use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Spela",
        PlaybackKey::Pause => "Pausa",
        PlaybackKey::Loop => "Slinga",
        PlaybackKey::FirstStep => "Första steget",
        PlaybackKey::PreviousStep => "Föregående steg",
        PlaybackKey::NextStep => "Nästa steg",
        PlaybackKey::LastStep => "Sista steget",
        PlaybackKey::Crop => "Beskär",
        PlaybackKey::CropHover => "Stödlinjer för beskärning (C)",
        PlaybackKey::PlaybackSpeed => "Uppspelningshastighet",
        PlaybackKey::PlaybackBuffer => "Buffra före uppspelning",
        PlaybackKey::SaveExportHover => {
            "Spara / exportera figur (Cmd+S för snabbsparning, Cmd+Shift+S för inställningar)"
        }
        PlaybackKey::CollapsedHint => "Uppspelning (klicka eller dra för att expandera)",
        PlaybackKey::TimelineDetails => "Tidslinjedetaljer",
        PlaybackKey::TimelineDimension => "Dimension: {}",
        PlaybackKey::TimelineStep => "Steg: {} / {}",
        PlaybackKey::TimelineCurrent => "Aktuellt: {}",
        PlaybackKey::TimelineRange => "Intervall: {} -> {}",
        PlaybackKey::TimelineStepSize => "Stegstorlek: {}",
        PlaybackKey::DefaultStepSize => "Steg: 1",
    })
}
