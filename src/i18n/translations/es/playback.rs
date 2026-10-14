use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Reproducir",
        PlaybackKey::Pause => "Pausar",
        PlaybackKey::Loop => "Bucle",
        PlaybackKey::FirstStep => "Primer paso",
        PlaybackKey::PreviousStep => "Paso anterior",
        PlaybackKey::NextStep => "Paso siguiente",
        PlaybackKey::LastStep => "Último paso",
        PlaybackKey::Crop => "Recortar",
        PlaybackKey::CropHover => "Líneas guía de recorte (C)",
        PlaybackKey::PlaybackSpeed => "Velocidad de reproducción",
        PlaybackKey::PlaybackBuffer => "Búfer antes de reproducir",
        PlaybackKey::Buffering => "Almacenando en búfer {}/{}",
        PlaybackKey::SaveExportHover => {
            "Guardar / exportar figura (Cmd+S para guardado rápido, Cmd+Shift+S para ajustes)"
        }
        PlaybackKey::CollapsedHint => "Reproducción (haga clic o arrastre para expandir)",
        PlaybackKey::TimelineDetails => "Detalles de la línea de tiempo",
        PlaybackKey::TimelineDimension => "Dimensión: {}",
        PlaybackKey::TimelineStep => "Paso: {} / {}",
        PlaybackKey::TimelineCurrent => "Actual: {}",
        PlaybackKey::TimelineRange => "Rango: {} -> {}",
        PlaybackKey::TimelineStepSize => "Tamaño de paso: {}",
        PlaybackKey::DefaultStepSize => "Paso: 1",
    })
}
