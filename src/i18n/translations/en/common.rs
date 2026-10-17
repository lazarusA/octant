use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> &'static str {
    match key {
        CommonKey::Dataset => "Dataset",
        CommonKey::Variables => "Variables",
        CommonKey::Dimensions => "Dimensions",
        CommonKey::Settings => "Settings",
        CommonKey::About => "About",
        CommonKey::Cache => "Cache",
        CommonKey::Language => "Language",
        CommonKey::Loading => "Loading...",
        CommonKey::CloseEsc => "Close (Esc)",
        CommonKey::Cancel => "Cancel",
        CommonKey::Save => "Save",
        CommonKey::Reset => "Reset",
        CommonKey::Move => "Move",
        CommonKey::MoveHover => "Drag to move; double-click to reset",
        CommonKey::Light => "Light",
        CommonKey::Dark => "Dark",
        CommonKey::SwitchToLightHover => "Switch to Light mode",
        CommonKey::SwitchToDarkHover => "Switch to Dark mode",
        CommonKey::ToggleHeroHover => "Toggle Hero / Landing View (right-click for About)",
        CommonKey::AboutOctantHover => "About Octant",
        CommonKey::ToggleVariableControlsHover => "Toggle Variable Controls Panel",
        CommonKey::FetchingProgress => "Fetching {} / {} ({}%)",
        CommonKey::Abort => "Abort",
        CommonKey::AbortHover => "Interrupt and abort ongoing data transfer",
    }
}
