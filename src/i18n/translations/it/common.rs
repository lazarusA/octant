use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "Set di dati",
        CommonKey::Variables => "Variabili",
        CommonKey::Dimensions => "Dimensioni",
        CommonKey::Settings => "Impostazioni",
        CommonKey::About => "Informazioni",
        CommonKey::Cache => "Cache",
        CommonKey::Language => "Lingua",
        CommonKey::Loading => "Caricamento...",
        CommonKey::CloseEsc => "Chiudi (Esc)",
        CommonKey::Cancel => "Annulla",
        CommonKey::Save => "Salva",
        CommonKey::Reset => "Reimposta",
        CommonKey::Move => "Sposta",
        CommonKey::MoveHover => "Trascina per spostare; doppio clic per reimpostare",
        CommonKey::Light => "Chiaro",
        CommonKey::Dark => "Scuro",
        CommonKey::SwitchToLightHover => "Passa alla modalità chiara",
        CommonKey::SwitchToDarkHover => "Passa alla modalità scura",
        CommonKey::ToggleHeroHover => {
            "Mostra/nascondi la schermata iniziale (clic destro per Informazioni)"
        }
        CommonKey::AboutOctantHover => "Informazioni su Octant",
        CommonKey::ToggleVariableControlsHover => {
            "Mostra/nascondi il pannello dei controlli della variabile"
        }
        CommonKey::FetchingProgress => "Recupero {} / {} ({}%)",
        CommonKey::Abort => "Interrompi",
        CommonKey::AbortHover => "Interrompi il trasferimento di dati in corso",
    })
}
