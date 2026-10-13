use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "Die Farben stammen aus den Kompositkanälen.",
        SettingsKey::ReasonCustomColor => "Eigene Farbe zeichnet alle Punkte in einer Farbe.",
        SettingsKey::ReasonAllSeries => "Alle Linienserien färbt jede Linie nach ihrem Index.",
        SettingsKey::ReasonIndexed => "Indiziertes RGBA färbt Voxel nach ihrem Label-Index.",
        SettingsKey::ReasonIndexedMissing => "Indiziertes RGBA überspringt fehlende Voxel.",
        SettingsKey::ReasonClassicOpaque => {
            "Nur Volumen-Raymarching (DVR) zeichnet durchscheinende Farben."
        }
        SettingsKey::ReasonNotTranslucent => "Wirkt bei Deckkraft unter 1 oder einer Alphakurve.",
        SettingsKey::ReasonCompositeOpaque => "Komposite werden deckend gezeichnet.",
        SettingsKey::ReasonLabelOpaque => {
            "Kategorische Label-Oberfläche wird immer deckend gezeichnet."
        }
        SettingsKey::ReasonAlwaysBlends => "Dieser Algorithmus mischt immer entlang des Strahls.",
        SettingsKey::CloseSettingsHover => "Einstellungen schließen",
        SettingsKey::SectionOverlays => "Überlagerungen",
        SettingsKey::SectionResolution => "Auflösung",
        SettingsKey::SectionView => "Ansicht",
        SettingsKey::Layers => "Ebenen",
        SettingsKey::BaseLayerHover => "Basisebene: bestimmt Zeichenfläche, Achsen und Ansicht",
        SettingsKey::BaseTag => "Basis",
        SettingsKey::Color => "Farbe",
        SettingsKey::HideLayerHover => "Ebene ausblenden",
        SettingsKey::ShowLayerHover => "Ebene einblenden",
        SettingsKey::RemoveLayerHover => "Ebene entfernen",
        SettingsKey::MoveDownHover => "Nach unten",
        SettingsKey::MoveUpHover => "Nach oben",
        SettingsKey::NoVariable => "(keine Variable)",
        SettingsKey::ColorbarLabel => "Farbleistentitel",
        SettingsKey::ResetLabelHover => "Titel der Farbleiste zurücksetzen",
        SettingsKey::ColorRange => "Farbbereich",
        SettingsKey::Categorical => "Kategorisch",
        SettingsKey::CategoricalHover => {
            "Diskrete Farbleiste (erkennt eindeutige Werte oder 10 gleiche Klassen)."
        }
        SettingsKey::LockBoundsHover => {
            "Min/Max sperren, damit die Farbzuordnung über alle Zeitschritte gleich bleibt."
        }
        SettingsKey::ResetBoundsHover => {
            "Grenzen auf Min und Max des aktuellen Ausschnitts/Datensatzes zurücksetzen"
        }
        SettingsKey::ResetAllColorbar => "Farbleiste zurücksetzen",
        SettingsKey::ResetAllColorbarHover => {
            "Titel und Bereich der Farbleiste auf Standardwerte zurücksetzen"
        }
        SettingsKey::RangeMin => "Min:",
        SettingsKey::RangeMax => "Max:",
        SettingsKey::NanColor => "NaN-Farbe",
        SettingsKey::NanColorHover => {
            "Wenn deaktiviert, werden NaN/Inf-Werte transparent dargestellt."
        }
        SettingsKey::NanColorPickerHover => "NaN-Farbe. Klicken, um eine Farbe zu wählen.",
        SettingsKey::LowClip => "Unter Bereich",
        SettingsKey::LowClipHover => "Werte < cmin erhalten diese Farbe.",
        SettingsKey::LowClipPickerHover => {
            "Farbe unter dem Bereich (< Min). Klicken, um eine Farbe zu wählen."
        }
        SettingsKey::HighClip => "Über Bereich",
        SettingsKey::HighClipHover => "Werte > cmax erhalten diese Farbe.",
        SettingsKey::HighClipPickerHover => {
            "Farbe über dem Bereich (> Max). Klicken, um eine Farbe zu wählen."
        }
        SettingsKey::Scale => "Skala",
        SettingsKey::ScaleLinear => "Linear",
        SettingsKey::ScaleLog => "Logarithmisch",
        SettingsKey::ScaleSymlog => "Symlog",
        SettingsKey::ScaleSqrt => "Wurzel",
        SettingsKey::ScaleExponential => "Exponentiell",
        SettingsKey::ScaleLogHover => "Log-Skala (nicht negative Daten)",
        SettingsKey::ScaleLogDisabledHover => {
            "Deaktiviert: erfordert min >= 0. Für negative Daten Symlog verwenden."
        }
        SettingsKey::ScaleParam => "Param.:",
        SettingsKey::Opacity => "Deckkraft",
        SettingsKey::OpacityHover => {
            "Deckkraft der Werte aus der Farbtabelle. NaN-Farben und Farben außerhalb des Bereichs behalten ihr eigenes Alpha. \
             Doppelklick zum Zurücksetzen."
        }
        SettingsKey::AlphaCurve => "Alphakurve",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  oder  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "Alpha entlang der Farbleiste, vom Anfang bis zum Ende. Werte sind gleichmäßig \
             verteilt (Stufe: je eine gleiche Klasse); position:alpha-Stopps werden \
             entlang der Leiste platziert. Leer für keine."
        }
        SettingsKey::ClearCurveHover => "Kurve leeren",
        SettingsKey::Presets => "Vorlagen",
        SettingsKey::InterpStep => "Stufe",
        SettingsKey::InterpLinear => "Linear",
        SettingsKey::PresetRampUp => "Ansteigend",
        SettingsKey::PresetRampDown => "Abfallend",
        SettingsKey::PresetPeak => "Spitze",
        SettingsKey::PresetSink => "Senke",
        SettingsKey::PresetHideLow => "Tiefe ausblenden",
        SettingsKey::PresetHideCenter => "Mitte ausblenden",
        SettingsKey::Transparency => "Transparenz",
        SettingsKey::TransparencyHover => {
            "Bei durchscheinenden Farben (Deckkraft oder Alphakurve) jede Ebene zeichnen, \
             statt die nächste die dahinterliegenden verdecken zu lassen."
        }
        SettingsKey::Lighting => "Beleuchtung",
        SettingsKey::LightingHover => {
            "Abtastwerte nach ihrem Gradienten schattieren, damit Fronten und Kanten Form erhalten"
        }
        SettingsKey::Coastlines => "Küstenlinien",
        SettingsKey::CoastlinesHover => "Natural-Earth-Küstenlinien über geografische Plots legen.",
        SettingsKey::ResetCoastlineColor => "zurücksetzen",
        SettingsKey::ResetCoastlineColorHover => {
            "Küstenlinienfarbe auf den Standard des Designs zurücksetzen"
        }
        SettingsKey::CoastlineColorHover => "Farbe der Küstenlinien. Klicken zum Auswählen.",
        SettingsKey::CoastlineResolution => "Auflösung:",
        SettingsKey::Lod110mHover => "110 m - schnell, immer verfügbar (eingebettet)",
        SettingsKey::Lod50mHover => "50 m - mittlere Detailstufe (im Hintergrund geladen)",
        SettingsKey::Lod10mHover => "10 m - hohe Detailstufe (im Hintergrund geladen)",
        SettingsKey::LineWidth => "Linienbreite",
        SettingsKey::CropToDomain => "Auf Datenbereich zuschneiden",
        SettingsKey::CropToDomainHover => {
            "Küstenlinien streng auf die räumliche Grenze des aktiven Datensatzes beschneiden."
        }
        SettingsKey::AutoRotate => "Auto-Rotation",
        SettingsKey::ResetView => "Ansicht zurücksetzen",
        SettingsKey::ResetViewHover => "Ausrichtung der 3D-Kamera zurücksetzen",
        SettingsKey::HoverCard => "Infokarte",
        SettingsKey::HideHoverCardHover => "Infokarte ausblenden",
        SettingsKey::ShowHoverCardHover => "Infokarte einblenden",
        SettingsKey::Export => "Export",
        SettingsKey::ExportFormat => "Format:",
        SettingsKey::ExportFolder => "Ordner:",
        SettingsKey::OpenSaveDialog => "Speicherdialog öffnen",
        SettingsKey::CropTool => "Zuschnitt",
        SettingsKey::Aggregation => "2D-Aggregation",
        SettingsKey::AggregationMandatoryHover => {
            "Erforderlich: Das Array überschreitet das Limit eines GPU-Puffers (128 MB / 33,5 Mio. Zellen). \
             Die Aggregation kann nicht deaktiviert werden."
        }
        SettingsKey::AggregationHover => {
            "Downsampling über Multi-Resolution-Pyramiden und Resampling im Ansichtsbereich."
        }
        SettingsKey::AggregationMode => "Modus:",
        SettingsKey::AggMean => "Mittelwert (Boxfilter)",
        SettingsKey::AggMax => "Max (Spitzen erhalten)",
        SettingsKey::AggMin => "Min (Senken erhalten)",
        SettingsKey::AggNearest => "Nächster Nachbar",
        SettingsKey::AggMeanHover => "Mittelt Subpixel in 2x2-Zellen, ignoriert NaNs.",
        SettingsKey::AggMaxHover => "Erhält extreme Höchstwerte.",
        SettingsKey::AggMinHover => "Erhält extreme Tiefstwerte.",
        SettingsKey::AggNearestHover => "Schnelle Auswahl des nächsten Punkts ohne Interpolation.",
        SettingsKey::CmykAutoMapped => "Automatisch zugeordnete Kanäle: C (1), M (2), Y (3), K (4)",
        SettingsKey::MultiChannelOverlay => "Mehrkanal-Überlagerung",
        SettingsKey::MultiChannelOverlayHover => {
            "Überlagert mehrere Kanäle additiv, jeden in seiner eigenen Farbtönung."
        }
        SettingsKey::CmykComposite => "CMYK-Komposit",
        SettingsKey::CmykCompositeHover => {
            "Setzt die 4 Kanäle Cyan, Magenta, Gelb, Schwarz (CMYK) zu Echtfarben-RGB zusammen."
        }
        SettingsKey::RgbComposite => "RGB-Komposit",
        SettingsKey::RgbCompositeHover => "Setzt 3 gewählte Kanäle zu Echtfarben-RGB zusammen.",
        SettingsKey::Channels => "Kanäle:",
        SettingsKey::ChannelsAll => "Alle",
        SettingsKey::ChannelsNone => "Keine",
        SettingsKey::ChannelTintHover => "Klicken, um die Tönung des Kanals anzupassen",
        SettingsKey::BandNumber => "Band {}",
        SettingsKey::ScaleBar => "Maßstabsleiste",
        SettingsKey::ShowScaleBarHover => "Maßstabsleiste einblenden",
        SettingsKey::HideScaleBarHover => "Maßstabsleiste ausblenden",
        SettingsKey::ChannelsAuto => "Auto",
        SettingsKey::ChannelsAutoHover => {
            "Fenster jedes Kanals an die angezeigten Daten anpassen (0,1 bis 99,9 Prozent seiner Werte)"
        }
        SettingsKey::ChannelsAutoUnavailable => {
            "Der automatische Kontrast liest den angezeigten 2D-Schnitt"
        }
        SettingsKey::ChannelSettingsHover => "Fenster, Kurve und Invertierung",
        SettingsKey::ChannelWindow => "Fenster",
        SettingsKey::ChannelWindowHover => {
            "Werte von Anfang bis Ende reichen von Schwarz bis zur vollen Kanalfarbe"
        }
        SettingsKey::ChannelInvert => "Invertieren",
        SettingsKey::ChannelCurve => "Kurve",
        SettingsKey::CurveLinear => "Linear",
        SettingsKey::CurveLogarithmic => "Logarithmisch",
        SettingsKey::CurveExponential => "Exponentiell",
        SettingsKey::CurvePolynomial => "Polynomiell",
        SettingsKey::ChannelCoefficientHover => "Koeffizient der Kurve",
        SettingsKey::LabelsFill => "Füllung",
        SettingsKey::LabelsFillHover => "Jedes Label ganz einfärben",
        SettingsKey::LabelsOutline => "Kontur",
        SettingsKey::LabelsOutlineHover => "Nur die Grenzen zwischen Labels zeichnen",
    })
}
