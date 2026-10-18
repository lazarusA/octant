use crate::i18n::keys::KeymapKey;

pub fn text(key: KeymapKey) -> Option<&'static str> {
    Some(match key {
        KeymapKey::Title => "Kortkommandon",
        KeymapKey::CloseHover => "Stäng kortkommandon (Esc)",
        KeymapKey::ScopeGlobal => "Överallt",
        KeymapKey::ScopePanel => "I en panel",
        KeymapKey::ScopeFree => "Utanför textfält",
        KeymapKey::ScopeCanvas => "På diagrammet",
        KeymapKey::ScopePlayback => "Uppspelning",
        KeymapKey::Arrows => "Piltangenter",
        KeymapKey::Hint => "{} ({})",
        KeymapKey::QuickSave => "Spara figuren",
        KeymapKey::ExportDialog => "Exportera figuren med inställningar",
        KeymapKey::ToggleCrop => "Visa/dölj beskärningen",
        KeymapKey::ToggleDataset => "Visa/dölj datasetpanelen",
        KeymapKey::ToggleVariables => "Visa/dölj variabelpanelen",
        KeymapKey::ToggleDimensions => "Visa/dölj dimensionspanelen",
        KeymapKey::ToggleSettings => "Visa/dölj inställningar",
        KeymapKey::SearchVariables => "Sök variabler",
        KeymapKey::ClosePanel => "Stäng panelen",
        KeymapKey::NextRegion => "Gå till nästa område",
        KeymapKey::PreviousRegion => "Gå till föregående område",
        KeymapKey::FocusTopBar => "Gå till toppfältet",
        KeymapKey::PlayPause => "Spela upp eller pausa",
        KeymapKey::StepPrev => "Föregående steg",
        KeymapKey::StepNext => "Nästa steg",
        KeymapKey::FirstStep => "Första steget",
        KeymapKey::LastStep => "Sista steget",
        KeymapKey::ToggleLoop => "Slå på/av upprepning",
        KeymapKey::Move => {
            "Panorera (2D) eller vrid (3D) diagrammet; flytta sonden eller beskärningen"
        }
        KeymapKey::MoveAlt => {
            "Panorera längre (2D) eller panorera (3D); ändra storlek på beskärningen"
        }
        KeymapKey::ZoomIn => "Zooma in",
        KeymapKey::ZoomOut => "Zooma ut",
        KeymapKey::ResetView => "Återställ vyn",
        KeymapKey::ToggleProbe => "Läs värden med piltangenterna",
        KeymapKey::LeaveProbe => "Sluta läsa värden",
        KeymapKey::ShowShortcuts => "Visa kortkommandon",
    })
}
