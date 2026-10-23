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
        PlaybackKey::CropHover => "Líneas guía de recorte",
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
        PlaybackKey::VoiceListen => "Empezar a escuchar",
        PlaybackKey::VoiceStop => "Dejar de escuchar",
        PlaybackKey::VoiceListening => "Escuchando...",
        PlaybackKey::VoiceMicOff => "Micrófono apagado",
        PlaybackKey::VoiceMicError => "Micrófono no disponible",
        PlaybackKey::VoiceSilent => "El micrófono solo envía silencio",
        PlaybackKey::VoiceSilentHint => {
            "Permita el acceso al micrófono a la aplicación que inició Octant en la configuración de privacidad del sistema y reiníciela."
        }
        PlaybackKey::VoiceSttFailed => "La transcripción falló",
        PlaybackKey::VoiceCloseHover => "Cerrar entrada de voz",
    })
}
