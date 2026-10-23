use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "Jeu de données",
        CommonKey::Variables => "Variables",
        CommonKey::Dimensions => "Dimensions",
        CommonKey::Settings => "Paramètres",
        CommonKey::About => "À propos",
        CommonKey::Cache => "Cache",
        CommonKey::Language => "Langue",
        CommonKey::Loading => "Chargement...",
        CommonKey::CloseEsc => "Fermer (Esc)",
        CommonKey::Cancel => "Annuler",
        CommonKey::Save => "Enregistrer",
        CommonKey::Reset => "Réinitialiser",
        CommonKey::Move => "Déplacer",
        CommonKey::MoveHover => "Faites glisser pour déplacer ; double-cliquez pour réinitialiser",
        CommonKey::Light => "Clair",
        CommonKey::Dark => "Sombre",
        CommonKey::SwitchToLightHover => "Passer en mode clair",
        CommonKey::SwitchToDarkHover => "Passer en mode sombre",
        CommonKey::ToggleHeroHover => {
            "Afficher/masquer l'écran d'accueil (clic droit pour À propos)"
        }
        CommonKey::AboutOctantHover => "À propos d'Octant",
        CommonKey::ToggleVariableControlsHover => {
            "Afficher/masquer le panneau de contrôle des variables"
        }
        CommonKey::FetchingProgress => "Récupération {} / {} ({}%)",
        CommonKey::Abort => "Interrompre",
        CommonKey::AbortHover => "Interrompre et annuler le transfert de données en cours",
        CommonKey::Voice => "Voix",
        CommonKey::VoiceHover => "Saisie vocale : microphone et spectrogramme",
    })
}
