use crate::i18n::keys::ColormapKey;

pub fn text(key: ColormapKey) -> Option<&'static str> {
    Some(match key {
        ColormapKey::Sequential => "Séquentielle",
        ColormapKey::Diverging => "Divergente",
        ColormapKey::Cyclic => "Cyclique",
        ColormapKey::Categorical => "Catégorielle",
        ColormapKey::Other => "Autre",
        ColormapKey::InterpolationLinear => "Linéaire",
        ColormapKey::BlendLinearRgb => "RGB linéaire",
        ColormapKey::FilterAll => "Toutes",
        ColormapKey::FamilyCustom => "Personnalisées",
        ColormapKey::FamilyTitle => "Famille : {}",
        ColormapKey::Colormap => "Palette",
        ColormapKey::Smooth => "Lisse",
        ColormapKey::SmoothHover => {
            "Fondre les couleurs de cette palette en un dégradé continu (Oklab)"
        }
        ColormapKey::Reversed => "Inversée",
        ColormapKey::SearchHint => "Rechercher des palettes...",
        ColormapKey::NoMatches => "Aucune palette ne correspond",
        ColormapKey::ShowColorbar => "Afficher la barre de couleurs",
        ColormapKey::BoxTransparency => "Transparence du cadre",
        ColormapKey::CustomColormap => "Palette personnalisée",
        ColormapKey::Name => "Nom",
        ColormapKey::Colors => "Couleurs",
        ColormapKey::ColorsHover => {
            "Hex, rgb(), hsl() ou noms de couleurs CSS, avec positions facultatives"
        }
        ColormapKey::Interpolation => "Interpolation",
        ColormapKey::Blend => "Mélange",
        ColormapKey::Classes => "Classes",
        ColormapKey::ClassesHover => "0 garde le dégradé continu",
        ColormapKey::RenameAndApply => "Renommer et appliquer",
        ColormapKey::SaveAndApply => "Enregistrer et appliquer",
        ColormapKey::SaveAsNew => "Enregistrer comme nouvelle",
        ColormapKey::NameTaken => "Une palette enregistrée porte déjà ce nom",
        ColormapKey::NameRequired => "Le nom est obligatoire",
        ColormapKey::Saved => "Enregistrée",
        ColormapKey::BrokenHover => {
            "Ne se construit pas avec cette version : modifiez-la pour la corriger, ou supprimez-la"
        }
        ColormapKey::Edit => "Modifier",
        ColormapKey::Delete => "Supprimer",
        ColormapKey::DeleteHover => "Supprimer la palette",
        ColormapKey::FlipOrientation => "Changer d'orientation",
        ColormapKey::RangeMinHover => {
            "Borne inférieure (Min). Faites glisser pour ajuster ou cliquez pour saisir."
        }
        ColormapKey::RangeMaxHover => {
            "Borne supérieure (Max). Faites glisser pour ajuster ou cliquez pour saisir."
        }
        ColormapKey::LowClipTitle => "Couleur sous la plage (< Min)",
        ColormapKey::LowClipHover => {
            "Couleur sous la plage (< Min). Cliquez pour choisir la couleur."
        }
        ColormapKey::HighClipTitle => "Couleur au-dessus de la plage (> Max)",
        ColormapKey::HighClipHover => {
            "Couleur au-dessus de la plage (> Max). Cliquez pour choisir la couleur."
        }
        ColormapKey::HoveredValue => "Val. : {}",
        ColormapKey::TitleHover => "Titre de la barre de couleurs. Cliquez pour modifier.",
        ColormapKey::ColorPicker => "Sélecteur de couleur",
        ColormapKey::Copy => "Copier",
        ColormapKey::CopyHover => "Copier les valeurs de la couleur dans le presse-papiers",
        ColormapKey::AlphaNotANumber => "'{}' n'est pas un nombre",
        ColormapKey::AlphaOutOfRange => "{} est hors de 0..1",
        ColormapKey::AlphaMixedForms => "utilisez soit des valeurs, soit des arrêts position:alpha",
        ColormapKey::AlphaUnsorted => "les positions des arrêts doivent être croissantes",
        ColormapKey::PrefsUnreadable => "Préférences de palettes illisibles",
        ColormapKey::PrefsBackedUp => "Sauvegardées sous `{}`",
        ColormapKey::PrefsNotBackedUp => "Préférences de palettes non sauvegardées",
        ColormapKey::BackupsFull => {
            "Les {} sauvegardes des préférences de palettes sont toutes utilisées"
        }
        ColormapKey::CustomNotLoaded => "Palette personnalisée non chargée",
        ColormapKey::CustomDoesNotBuild => "`{}` ne se construit pas : {}",
        ColormapKey::NotRegistered => "La palette {} n'a pas été enregistrée",
        ColormapKey::BundledUnreadable => "Palettes intégrées illisibles",
        ColormapKey::FallingBackToGray => "{} ; repli sur les niveaux de gris",
    })
}
