use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Toista",
        PlaybackKey::Pause => "Tauko",
        PlaybackKey::Loop => "Silmukka",
        PlaybackKey::FirstStep => "Ensimmäinen askel",
        PlaybackKey::PreviousStep => "Edellinen askel",
        PlaybackKey::NextStep => "Seuraava askel",
        PlaybackKey::LastStep => "Viimeinen askel",
        PlaybackKey::Crop => "Rajaa",
        PlaybackKey::CropHover => "Rajauksen apuviivat (C)",
        PlaybackKey::PlaybackSpeed => "Toistonopeus",
        PlaybackKey::PlaybackBuffer => "Puskuroi ennen toistoa",
        PlaybackKey::SaveExportHover => {
            "Tallenna / vie kuva (Cmd+S pikatallennus, Cmd+Shift+S asetukset)"
        }
        PlaybackKey::CollapsedHint => "Toisto (laajenna napsauttamalla tai vetämällä)",
        PlaybackKey::TimelineDetails => "Aikajanan tiedot",
        PlaybackKey::TimelineDimension => "Ulottuvuus: {}",
        PlaybackKey::TimelineStep => "Askel: {} / {}",
        PlaybackKey::TimelineCurrent => "Nykyinen: {}",
        PlaybackKey::TimelineRange => "Väli: {} -> {}",
        PlaybackKey::TimelineStepSize => "Askelkoko: {}",
        PlaybackKey::DefaultStepSize => "Askel: 1",
    })
}
