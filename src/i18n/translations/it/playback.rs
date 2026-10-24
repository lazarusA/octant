use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Riproduci",
        PlaybackKey::Pause => "Pausa",
        PlaybackKey::Loop => "Ripeti",
        PlaybackKey::FirstStep => "Primo passo",
        PlaybackKey::PreviousStep => "Passo precedente",
        PlaybackKey::NextStep => "Passo successivo",
        PlaybackKey::LastStep => "Ultimo passo",
        PlaybackKey::Crop => "Ritaglia",
        PlaybackKey::CropHover => "Linee guida di ritaglio",
        PlaybackKey::PlaybackSpeed => "Velocità di riproduzione",
        PlaybackKey::PlaybackBuffer => "Buffer prima della riproduzione",
        PlaybackKey::Buffering => "Caricamento buffer {}/{}",
        PlaybackKey::SaveExportHover => {
            "Salva / esporta figura (Cmd+S per il salvataggio rapido, Cmd+Shift+S per le impostazioni)"
        }
        PlaybackKey::CollapsedHint => "Riproduzione (clicca o trascina per espandere)",
        PlaybackKey::TimelineDetails => "Dettagli della sequenza",
        PlaybackKey::TimelineDimension => "Dimensione: {}",
        PlaybackKey::TimelineStep => "Passo: {} / {}",
        PlaybackKey::TimelineCurrent => "Attuale: {}",
        PlaybackKey::TimelineRange => "Intervallo: {} -> {}",
        PlaybackKey::TimelineStepSize => "Ampiezza del passo: {}",
        PlaybackKey::DefaultStepSize => "Passo: 1",
        PlaybackKey::VoiceListen => "Inizia ad ascoltare",
        PlaybackKey::VoiceStop => "Smetti di ascoltare",
        PlaybackKey::VoiceListening => "In ascolto...",
        PlaybackKey::VoiceMicOff => "Microfono spento",
        PlaybackKey::VoiceMicError => "Microfono non disponibile",
        PlaybackKey::VoiceSilent => "Il microfono invia solo silenzio",
        PlaybackKey::VoiceSilentHint => {
            "Consenti l'accesso al microfono all'app che ha avviato Octant nelle impostazioni di privacy del sistema, poi riavviala."
        }
        PlaybackKey::VoiceSttFailed => "Trascrizione non riuscita",
        PlaybackKey::VoiceDownloading => "Download del modello vocale: {}%",
        PlaybackKey::VoiceLoading => "Caricamento del modello vocale...",
        PlaybackKey::VoiceModelHover => "Modello di riconoscimento vocale",
        PlaybackKey::VoiceModelSize => "{} ({} MB)",
        PlaybackKey::VoiceCloseHover => "Chiudi input vocale",
        PlaybackKey::VoiceDidSelect => "{} selezionato",
        PlaybackKey::VoiceDidPlot => "{} tracciato come {}",
        PlaybackKey::VoiceDidColormap => "Mappa di colori {}",
        PlaybackKey::VoiceNotUnderstood => "Comando non compreso",
        PlaybackKey::VoiceNoDataset => "Apri prima un dataset",
        PlaybackKey::VoiceNoVariable => "Nessuna variabile corrisponde a '{}'",
        PlaybackKey::VoiceNoPlotType => "Nessun tipo di grafico corrisponde a '{}'",
        PlaybackKey::VoiceNoColormap => "Nessuna mappa di colori corrisponde a '{}'",
        PlaybackKey::VoiceAmbiguous => "Più variabili corrispondono a '{}'",
        PlaybackKey::VoiceDidYouMean => "Intendevi: {}",
        PlaybackKey::VoiceNothingToAnimate => "Il grafico non ha una dimensione animata",
        PlaybackKey::VoiceCannotPlot => "Impossibile tracciare come {}",
    })
}
