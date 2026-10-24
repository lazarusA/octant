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
        PlaybackKey::CropHover => "Rajauksen apuviivat",
        PlaybackKey::PlaybackSpeed => "Toistonopeus",
        PlaybackKey::PlaybackBuffer => "Puskurointi ennen toistoa",
        PlaybackKey::Buffering => "Puskuroidaan {}/{}",
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
        PlaybackKey::VoiceListen => "Aloita kuuntelu",
        PlaybackKey::VoiceStop => "Lopeta kuuntelu",
        PlaybackKey::VoiceListening => "Kuunnellaan...",
        PlaybackKey::VoiceMicOff => "Mikrofoni pois",
        PlaybackKey::VoiceMicError => "Mikrofoni ei käytettävissä",
        PlaybackKey::VoiceSilent => "Mikrofoni lähettää vain hiljaisuutta",
        PlaybackKey::VoiceSilentHint => {
            "Salli mikrofonin käyttö Octantin käynnistäneelle sovellukselle järjestelmän tietosuoja-asetuksissa ja käynnistä se uudelleen."
        }
        PlaybackKey::VoiceSttFailed => "Litterointi epäonnistui",
        PlaybackKey::VoiceDownloading => "Ladataan puhemallia: {}%",
        PlaybackKey::VoiceLoading => "Avataan puhemallia...",
        PlaybackKey::VoiceModelHover => "Puheentunnistusmalli",
        PlaybackKey::VoiceModelSize => "{} ({} Mt)",
        PlaybackKey::VoiceCloseHover => "Sulje puhesyöte",
        PlaybackKey::VoiceDidSelect => "{} valittu",
        PlaybackKey::VoiceDidPlot => "{} piirretty muodossa {}",
        PlaybackKey::VoiceDidColormap => "Väriskaala {}",
        PlaybackKey::VoiceNotUnderstood => "Komentoa ei ymmärretty",
        PlaybackKey::VoiceNoDataset => "Avaa ensin aineisto",
        PlaybackKey::VoiceNoVariable => "Mikään muuttuja ei vastaa kohdetta '{}'",
        PlaybackKey::VoiceNoPlotType => "Mikään kuvaajatyyppi ei vastaa kohdetta '{}'",
        PlaybackKey::VoiceNoColormap => "Mikään väriskaala ei vastaa kohdetta '{}'",
        PlaybackKey::VoiceAmbiguous => "Useat muuttujat vastaavat kohdetta '{}'",
        PlaybackKey::VoiceDidYouMean => "Tarkoititko: {}",
        PlaybackKey::VoiceNothingToAnimate => "Kuvaajassa ei ole animoitua ulottuvuutta",
        PlaybackKey::VoiceCannotPlot => "Ei voi piirtää muodossa {}",
    })
}
