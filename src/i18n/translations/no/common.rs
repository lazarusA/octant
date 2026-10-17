use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "Datasett",
        CommonKey::Variables => "Variabler",
        CommonKey::Dimensions => "Dimensjoner",
        CommonKey::Settings => "Innstillinger",
        CommonKey::About => "Om",
        CommonKey::Cache => "Hurtigbuffer",
        CommonKey::Language => "Språk",
        CommonKey::Loading => "Laster...",
        CommonKey::CloseEsc => "Lukk (Esc)",
        CommonKey::Cancel => "Avbryt",
        CommonKey::Save => "Lagre",
        CommonKey::Reset => "Tilbakestill",
        CommonKey::Move => "Flytt",
        CommonKey::MoveHover => "Dra for å flytte; dobbeltklikk for å tilbakestille",
        CommonKey::Light => "Lys",
        CommonKey::Dark => "Mørk",
        CommonKey::SwitchToLightHover => "Bytt til lys modus",
        CommonKey::SwitchToDarkHover => "Bytt til mørk modus",
        CommonKey::ToggleHeroHover => "Vis/skjul startsiden (høyreklikk for Om)",
        CommonKey::AboutOctantHover => "Om Octant",
        CommonKey::ToggleVariableControlsHover => "Vis/skjul variabelpanelet",
        CommonKey::FetchingProgress => "Henter {} / {} ({}%)",
        CommonKey::Abort => "Stopp",
        CommonKey::AbortHover => "Avbryt pågående dataoverføring",
    })
}
