use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "Dataset",
        CommonKey::Variables => "Variabler",
        CommonKey::Dimensions => "Dimensioner",
        CommonKey::Settings => "Inställningar",
        CommonKey::About => "Om",
        CommonKey::Cache => "Cache",
        CommonKey::Language => "Språk",
        CommonKey::Loading => "Laddar...",
        CommonKey::CloseEsc => "Stäng (Esc)",
        CommonKey::Cancel => "Avbryt",
        CommonKey::Save => "Spara",
        CommonKey::Reset => "Återställ",
        CommonKey::Move => "Flytta",
        CommonKey::MoveHover => "Dra för att flytta; dubbelklicka för att återställa",
        CommonKey::Light => "Ljust",
        CommonKey::Dark => "Mörkt",
        CommonKey::SwitchToLightHover => "Byt till ljust läge",
        CommonKey::SwitchToDarkHover => "Byt till mörkt läge",
        CommonKey::ToggleHeroHover => "Visa/dölj startvyn (högerklicka för Om)",
        CommonKey::AboutOctantHover => "Om Octant",
        CommonKey::ToggleVariableControlsHover => "Visa/dölj panelen för variabelkontroller",
        CommonKey::FetchingProgress => "Hämtar {} / {} ({}%)",
        CommonKey::Abort => "Avbryt",
        CommonKey::AbortHover => "Avbryt pågående dataöverföring",
        CommonKey::Voice => "Röst",
        CommonKey::VoiceHover => "Röstinmatning: mikrofon och spektrogram",
    })
}
