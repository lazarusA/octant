use crate::i18n::keys::ColormapKey;

pub fn text(key: ColormapKey) -> Option<&'static str> {
    Some(match key {
        ColormapKey::Sequential => "Sequenziale",
        ColormapKey::Diverging => "Divergente",
        ColormapKey::Cyclic => "Ciclica",
        ColormapKey::Categorical => "Categoriale",
        ColormapKey::Other => "Altro",
        ColormapKey::InterpolationLinear => "Lineare",
        ColormapKey::BlendLinearRgb => "RGB lineare",
        ColormapKey::FilterAll => "Tutte",
        ColormapKey::FamilyCustom => "Personalizzate",
        ColormapKey::FamilyTitle => "Famiglia: {}",
        ColormapKey::Colormap => "Mappa di colore",
        ColormapKey::Smooth => "Sfumata",
        ColormapKey::SmoothHover => {
            "Fondi i colori di questa tavolozza in un gradiente continuo (Oklab)"
        }
        ColormapKey::Reversed => "Invertita",
        ColormapKey::SearchHint => "Cerca mappe di colore...",
        ColormapKey::NoMatches => "Nessuna mappa di colore corrispondente",
        ColormapKey::ShowColorbar => "Mostra barra dei colori",
        ColormapKey::BoxTransparency => "Trasparenza del riquadro",
        ColormapKey::CustomColormap => "Mappa di colore personalizzata",
        ColormapKey::Name => "Nome",
        ColormapKey::Colors => "Colori",
        ColormapKey::ColorsHover => {
            "Hex, rgb(), hsl() o nomi di colori CSS, eventualmente con posizioni"
        }
        ColormapKey::Interpolation => "Interpolazione",
        ColormapKey::Blend => "Fusione",
        ColormapKey::Classes => "Classi",
        ColormapKey::ClassesHover => "0 mantiene il gradiente continuo",
        ColormapKey::RenameAndApply => "Rinomina e applica",
        ColormapKey::SaveAndApply => "Salva e applica",
        ColormapKey::SaveAsNew => "Salva come nuova",
        ColormapKey::NameTaken => "Esiste già una mappa di colore salvata con questo nome",
        ColormapKey::NameRequired => "Il nome è obbligatorio",
        ColormapKey::Saved => "Salvata",
        ColormapKey::BrokenHover => {
            "Non si costruisce con questa versione: modificala per correggerla, o eliminala"
        }
        ColormapKey::Edit => "Modifica",
        ColormapKey::Delete => "Elimina",
        ColormapKey::DeleteHover => "Elimina mappa di colore",
        ColormapKey::FlipOrientation => "Inverti orientamento",
        ColormapKey::RangeMinHover => {
            "Estremo inferiore (Min). Trascina per regolare o clicca per digitare."
        }
        ColormapKey::RangeMaxHover => {
            "Estremo superiore (Max). Trascina per regolare o clicca per digitare."
        }
        ColormapKey::LowClipTitle => "Colore di taglio inferiore (< Min)",
        ColormapKey::LowClipHover => {
            "Colore di taglio inferiore (< Min). Clicca per scegliere il colore."
        }
        ColormapKey::HighClipTitle => "Colore di taglio superiore (> Max)",
        ColormapKey::HighClipHover => {
            "Colore di taglio superiore (> Max). Clicca per scegliere il colore."
        }
        ColormapKey::HoveredValue => "Val: {}",
        ColormapKey::TitleHover => "Titolo della barra dei colori. Clicca per modificarlo.",
        ColormapKey::ColorPicker => "Selettore colore",
        ColormapKey::Copy => "Copia",
        ColormapKey::CopyHover => "Copia i valori del colore negli appunti",
        ColormapKey::AlphaNotANumber => "'{}' non è un numero",
        ColormapKey::AlphaOutOfRange => "{} è fuori da 0..1",
        ColormapKey::AlphaMixedForms => "usa valori oppure punti posizione:alfa",
        ColormapKey::AlphaUnsorted => "le posizioni dei punti devono crescere",
        ColormapKey::PrefsUnreadable => "Preferenze delle mappe di colore illeggibili",
        ColormapKey::PrefsBackedUp => "Copia di sicurezza salvata in `{}`",
        ColormapKey::PrefsNotBackedUp => {
            "Nessuna copia di sicurezza delle preferenze delle mappe di colore"
        }
        ColormapKey::BackupsFull => {
            "Tutte le {} copie di sicurezza delle preferenze delle mappe di colore sono in uso"
        }
        ColormapKey::CustomNotLoaded => "Mappa di colore personalizzata non caricata",
        ColormapKey::CustomDoesNotBuild => "`{}` non si costruisce: {}",
        ColormapKey::NotRegistered => "La mappa di colore {} non è stata registrata",
        ColormapKey::BundledUnreadable => "Mappe di colore incluse illeggibili",
        ColormapKey::FallingBackToGray => "{}; si usa la scala di grigi",
    })
}
