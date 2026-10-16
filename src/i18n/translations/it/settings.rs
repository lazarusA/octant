use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "I colori provengono dai canali del composito.",
        SettingsKey::ReasonCustomColor => {
            "'Colore personalizzato' disegna ogni punto con un solo colore."
        }
        SettingsKey::ReasonAllSeries => "'Tutte le serie' colora ogni linea in base al suo indice.",
        SettingsKey::ReasonIndexed => {
            "'RGBA indicizzato' colora i voxel in base al loro indice di etichetta."
        }
        SettingsKey::ReasonIndexedMissing => "'RGBA indicizzato' salta i voxel mancanti.",
        SettingsKey::ReasonClassicOpaque => {
            "Solo 'Raymarching del volume (DVR)' disegna colori traslucidi."
        }
        SettingsKey::ReasonNotTranslucent => {
            "Ha effetto con 'Opacità' inferiore a 1 o una 'Curva alfa'."
        }
        SettingsKey::ReasonCompositeOpaque => "I compositi sono disegnati opachi.",
        SettingsKey::ReasonLabelOpaque => {
            "'Superficie di etichette categoriali' è sempre disegnata opaca."
        }
        SettingsKey::ReasonAlwaysBlends => "Questo algoritmo fonde sempre lungo il raggio.",
        SettingsKey::CloseSettingsHover => "Chiudi impostazioni",
        SettingsKey::SectionOverlays => "Sovrapposizioni",
        SettingsKey::SectionResolution => "Risoluzione",
        SettingsKey::SectionView => "Vista",
        SettingsKey::Layers => "Livelli",
        SettingsKey::BaseLayerHover => {
            "Livello base: decide l'area di disegno, gli assi e la vista"
        }
        SettingsKey::BaseTag => "base",
        SettingsKey::Color => "Colore",
        SettingsKey::HideLayerHover => "Nascondi livello",
        SettingsKey::ShowLayerHover => "Mostra livello",
        SettingsKey::RemoveLayerHover => "Rimuovi livello",
        SettingsKey::MoveDownHover => "Sposta giù",
        SettingsKey::MoveUpHover => "Sposta su",
        SettingsKey::NoVariable => "(nessuna variabile)",
        SettingsKey::ColorbarLabel => "Etichetta della barra",
        SettingsKey::ResetLabelHover => "Reimposta l'etichetta della barra dei colori",
        SettingsKey::ColorRange => "Intervallo di colore",
        SettingsKey::Categorical => "Categoriale",
        SettingsKey::CategoricalHover => {
            "Barra dei colori discreta (rileva i valori unici o 10 classi uguali)."
        }
        SettingsKey::LockBoundsHover => {
            "Blocca min/max perché la mappatura dei colori resti fissa tra i passi temporali."
        }
        SettingsKey::ResetBoundsHover => {
            "Reimposta i limiti ai valori min e max predefiniti della sezione/del set di dati"
        }
        SettingsKey::ResetAllColorbar => "Reimposta tutta la barra dei colori",
        SettingsKey::ResetAllColorbarHover => {
            "Reimposta etichetta e intervallo della barra dei colori ai valori predefiniti"
        }
        SettingsKey::RangeMin => "Min:",
        SettingsKey::RangeMax => "Max:",
        SettingsKey::NanColor => "Colore NaN",
        SettingsKey::NanColorHover => "Se disattivo, i valori NaN/Inf sono trasparenti.",
        SettingsKey::NanColorPickerHover => "Colore NaN. Clicca per scegliere il colore.",
        SettingsKey::LowClip => "Sotto l'intervallo",
        SettingsKey::LowClipHover => "Valori < cmin resi con questo colore.",
        SettingsKey::LowClipPickerHover => {
            "Colore sotto l'intervallo (< Min). Clicca per scegliere il colore."
        }
        SettingsKey::HighClip => "Sopra l'intervallo",
        SettingsKey::HighClipHover => "Valori > cmax resi con questo colore.",
        SettingsKey::HighClipPickerHover => {
            "Colore sopra l'intervallo (> Max). Clicca per scegliere il colore."
        }
        SettingsKey::Scale => "Scala",
        SettingsKey::ScaleLinear => "Lineare",
        SettingsKey::ScaleLog => "Logaritmica",
        SettingsKey::ScaleSymlog => "Symlog",
        SettingsKey::ScaleSqrt => "Radice",
        SettingsKey::ScaleExponential => "Esponenziale",
        SettingsKey::ScaleLogHover => "Scala logaritmica (dati non negativi)",
        SettingsKey::ScaleLogDisabledHover => {
            "Disattivata: richiede min >= 0. Usa Symlog per dati negativi."
        }
        SettingsKey::ScaleParam => "Param.:",
        SettingsKey::Opacity => "Opacità",
        SettingsKey::OpacityHover => {
            "Opacità dei valori mappati sui colori. I colori NaN e fuori intervallo mantengono il proprio alfa. \
             Doppio clic per reimpostare."
        }
        SettingsKey::AlphaCurve => "Curva alfa",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  o  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "Alfa lungo la barra dei colori, dall'inizio alla fine. I valori sono \
             equidistanti (Gradino: una classe uguale ciascuno); i punti posizione:alfa \
             si collocano lungo la barra. Vuoto per nessuna."
        }
        SettingsKey::ClearCurveHover => "Cancella curva",
        SettingsKey::Presets => "Predefinite",
        SettingsKey::InterpStep => "Gradino",
        SettingsKey::InterpLinear => "Lineare",
        SettingsKey::PresetRampUp => "Rampa su",
        SettingsKey::PresetRampDown => "Rampa giù",
        SettingsKey::PresetPeak => "Picco",
        SettingsKey::PresetSink => "Avvallamento",
        SettingsKey::PresetHideLow => "Nascondi bassi",
        SettingsKey::PresetHideCenter => "Nascondi centro",
        SettingsKey::Transparency => "Trasparenza",
        SettingsKey::TransparencyHover => {
            "Con colori traslucidi ('Opacità' o 'Curva alfa'), disegna ogni strato \
             invece di lasciare che il più vicino nasconda quelli dietro."
        }
        SettingsKey::Lighting => "Illuminazione",
        SettingsKey::LightingHover => {
            "Ombreggia i campioni in base al gradiente, così fronti e bordi acquistano forma"
        }
        SettingsKey::Coastlines => "Linee di costa",
        SettingsKey::CoastlinesHover => {
            "Sovrappone le linee di costa di Natural Earth ai grafici geografici."
        }
        SettingsKey::ResetCoastlineColor => "reimposta",
        SettingsKey::ResetCoastlineColorHover => {
            "Reimposta il colore delle linee di costa a quello del tema"
        }
        SettingsKey::CoastlineColorHover => "Colore delle linee di costa. Clicca per scegliere.",
        SettingsKey::CoastlineResolution => "Risoluzione:",
        SettingsKey::Lod110mHover => "110 m: veloce, sempre disponibile (incorporata)",
        SettingsKey::Lod50mHover => "50 m: dettaglio medio (caricata in background)",
        SettingsKey::Lod10mHover => "10 m: dettaglio elevato (caricata in background)",
        SettingsKey::LineWidth => "Spessore linea",
        SettingsKey::CropToDomain => "Ritaglia al dominio dei dati",
        SettingsKey::CropToDomainHover => {
            "Ritaglia le linee di costa esattamente ai confini spaziali del set di dati attivo."
        }
        SettingsKey::AutoRotate => "Rotazione automatica",
        SettingsKey::ResetView => "Reimposta vista",
        SettingsKey::ResetViewHover => "Reimposta l'orientamento della camera 3D",
        SettingsKey::HoverCard => "Scheda al passaggio",
        SettingsKey::HideHoverCardHover => "Nascondi la scheda al passaggio",
        SettingsKey::ShowHoverCardHover => "Mostra la scheda al passaggio",
        SettingsKey::Export => "Esporta",
        SettingsKey::ExportFormat => "Formato:",
        SettingsKey::ExportFolder => "Cartella:",
        SettingsKey::OpenSaveDialog => "Apri finestra di salvataggio",
        SettingsKey::CropTool => "Strumento ritaglio",
        SettingsKey::Aggregation => "Aggregazione 2D",
        SettingsKey::AggregationMandatoryHover => {
            "Obbligatoria: l'array supera il limite di un singolo buffer GPU ({} MB). \
             L'aggregazione non può essere disattivata."
        }
        SettingsKey::AggregationHover => {
            "Sottocampionamento a piramide multirisoluzione e ricampionamento della vista."
        }
        SettingsKey::AggregationMode => "Modalità:",
        SettingsKey::AggMean => "Media (filtro a box)",
        SettingsKey::AggMax => "Max (conserva i picchi)",
        SettingsKey::AggMin => "Min (conserva i minimi)",
        SettingsKey::AggNearest => "Vicino più prossimo",
        SettingsKey::AggMeanHover => "Media i sottopixel in celle 2x2, ignorando i NaN.",
        SettingsKey::AggMaxHover => "Conserva i valori estremi alti.",
        SettingsKey::AggMinHover => "Conserva i valori estremi bassi.",
        SettingsKey::AggNearestHover => {
            "Selezione rapida del punto più vicino senza interpolazione."
        }
        SettingsKey::ResolutionLevel => "Livello di risoluzione",
        SettingsKey::ResolutionLevelHover => {
            "Il livello dei dati multiscala letto dal grafico. Auto legge il livello più fine che entra nell'area di disegno."
        }
        SettingsKey::CmykAutoMapped => "Canali assegnati in automatico: C (1), M (2), Y (3), K (4)",
        SettingsKey::MultiChannelOverlay => "Sovrapposizione multicanale",
        SettingsKey::MultiChannelOverlayHover => {
            "Sovrappone più canali in modo additivo, ciascuno con la propria tinta."
        }
        SettingsKey::CmykComposite => "Composito CMYK",
        SettingsKey::CmykCompositeHover => {
            "Combina 4 canali Ciano, Magenta, Giallo, Nero (CMYK) in RGB a colori reali."
        }
        SettingsKey::RgbComposite => "Composito RGB",
        SettingsKey::RgbCompositeHover => "Combina i 3 canali selezionati in RGB a colori reali.",
        SettingsKey::Channels => "Canali:",
        SettingsKey::ChannelsAll => "Tutti",
        SettingsKey::ChannelsNone => "Nessuno",
        SettingsKey::ChannelTintHover => "Clicca per personalizzare la tinta del canale",
        SettingsKey::BandNumber => "Banda {}",
        SettingsKey::ScaleBar => "Barra di scala",
        SettingsKey::ShowScaleBarHover => "Mostra la barra di scala",
        SettingsKey::HideScaleBarHover => "Nascondi la barra di scala",
        SettingsKey::ChannelsAuto => "Auto",
        SettingsKey::ChannelsAutoHover => {
            "Adatta l'intervallo di visualizzazione di ogni canale ai dati mostrati (dallo 0,1 al 99,9 per cento dei suoi valori)"
        }
        SettingsKey::ChannelsAutoUnavailable => {
            "Il contrasto automatico funziona su una sezione 2D"
        }
        SettingsKey::ChannelSettingsHover => "Intervallo di visualizzazione, curva e inversione",
        SettingsKey::ChannelWindow => "Intervallo di visualizzazione",
        SettingsKey::ChannelWindowHover => {
            "I valori dall'inizio alla fine vanno dal nero al colore pieno del canale"
        }
        SettingsKey::ChannelInvert => "Inverti",
        SettingsKey::ChannelCurve => "Curva",
        SettingsKey::CurveLinear => "Lineare",
        SettingsKey::CurveLogarithmic => "Logaritmica",
        SettingsKey::CurveExponential => "Esponenziale",
        SettingsKey::CurvePolynomial => "Polinomiale",
        SettingsKey::ChannelCoefficientHover => "Coefficiente della curva",
        SettingsKey::LabelsFill => "Riempimento",
        SettingsKey::LabelsFillHover => "Colora ogni etichetta per intero",
        SettingsKey::LabelsOutline => "Contorno",
        SettingsKey::LabelsOutlineHover => "Disegna solo i bordi tra le etichette",
    })
}
