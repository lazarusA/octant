use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "Aineisto",
        CommonKey::Variables => "Muuttujat",
        CommonKey::Dimensions => "Ulottuvuudet",
        CommonKey::Colormaps => "Värikartat",
        CommonKey::Settings => "Asetukset",
        CommonKey::About => "Tietoja",
        CommonKey::Cache => "Välimuisti",
        CommonKey::Language => "Kieli",
        CommonKey::Loading => "Ladataan...",
        CommonKey::CloseEsc => "Sulje (Esc)",
        CommonKey::Cancel => "Peruuta",
        CommonKey::Save => "Tallenna",
        CommonKey::Reset => "Palauta",
        CommonKey::Move => "Siirrä",
        CommonKey::MoveHover => "Siirrä vetämällä; palauta kaksoisnapsautuksella",
        CommonKey::Light => "Vaalea",
        CommonKey::Dark => "Tumma",
        CommonKey::SwitchToLightHover => "Vaihda vaaleaan tilaan",
        CommonKey::SwitchToDarkHover => "Vaihda tummaan tilaan",
        CommonKey::ToggleHeroHover => "Näytä/piilota aloitusnäkymä (oikea napsautus: Tietoja)",
        CommonKey::AboutOctantHover => "Tietoja Octantista",
        CommonKey::ToggleVariableControlsHover => "Näytä/piilota muuttujapaneeli",
        CommonKey::FetchingProgress => "Haetaan {} / {} ({}%)",
        CommonKey::Abort => "Keskeytä",
        CommonKey::AbortHover => "Keskeytä käynnissä oleva tiedonsiirto",
    })
}
