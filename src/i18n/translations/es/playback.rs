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
        PlaybackKey::VoiceDownloading => "Descargando el modelo de voz: {}%",
        PlaybackKey::VoiceLoading => "Cargando el modelo de voz...",
        PlaybackKey::VoiceModelHover => "Modelo de voz a texto",
        PlaybackKey::VoiceModelSize => "{} ({} MB)",
        PlaybackKey::VoiceCloseHover => "Cerrar entrada de voz",
        PlaybackKey::VoiceDidOpen => "Abriendo {}",
        PlaybackKey::VoiceNoSource => "Ningún conjunto de datos coincide con '{}'",
        PlaybackKey::VoiceDidSelect => "Seleccionado {}",
        PlaybackKey::VoiceDidSetDim => "{}: {}",
        PlaybackKey::VoiceDidSetRange => "{}: {} - {}",
        PlaybackKey::VoiceNoDimension => "Ninguna dimensión coincide con '{}'",
        PlaybackKey::VoiceNoValue => "Ningún valor de {} coincide con '{}'",
        PlaybackKey::VoiceDidPlot => "{} graficado como {}",
        PlaybackKey::VoiceDidColormap => "Mapa de colores {}",
        PlaybackKey::VoiceNotUnderstood => "Orden no entendida",
        PlaybackKey::VoiceNoDataset => "Abra primero un conjunto de datos",
        PlaybackKey::VoiceNoVariable => "Ninguna variable coincide con '{}'",
        PlaybackKey::VoiceNoPlotType => "Ningún tipo de gráfico coincide con '{}'",
        PlaybackKey::VoiceNoColormap => "Ningún mapa de colores coincide con '{}'",
        PlaybackKey::VoiceAmbiguous => "Varias variables coinciden con '{}'",
        PlaybackKey::VoiceDidYouMean => "¿Quiso decir: {}?",
        PlaybackKey::VoiceNothingToAnimate => "El gráfico no tiene dimensión animada",
        PlaybackKey::VoiceCannotPlot => "No se puede graficar como {}",
    })
}
