use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "Färgerna kommer från kompositens kanaler.",
        SettingsKey::ReasonCustomColor => "Anpassad färg ritar alla punkter i en färg.",
        SettingsKey::ReasonAllSeries => "Alla linjeserier färgar varje linje efter dess index.",
        SettingsKey::ReasonIndexed => "Indexerad RGBA färgar voxlar efter deras etikettindex.",
        SettingsKey::ReasonIndexedMissing => "Indexerad RGBA hoppar över saknade voxlar.",
        SettingsKey::ReasonClassicOpaque => {
            "Endast Volymstrålföljning (DVR) ritar genomskinliga färger."
        }
        SettingsKey::ReasonNotTranslucent => "Gäller med Opacitet under 1 eller en Alfakurva.",
        SettingsKey::ReasonCompositeOpaque => "Kompositer ritas ogenomskinliga.",
        SettingsKey::ReasonLabelOpaque => "Kategorisk etikettyta ritas alltid ogenomskinlig.",
        SettingsKey::ReasonAlwaysBlends => "Den här algoritmen blandar alltid längs strålen.",
        SettingsKey::CloseSettingsHover => "Stäng inställningar",
        SettingsKey::SectionOverlays => "Överlägg",
        SettingsKey::SectionResolution => "Upplösning",
        SettingsKey::SectionView => "Vy",
        SettingsKey::Layers => "Lager",
        SettingsKey::BaseLayerHover => "Baslager: bestämmer ytan, axlarna och vyn",
        SettingsKey::BaseTag => "bas",
        SettingsKey::Color => "Färg",
        SettingsKey::HideLayerHover => "Dölj lager",
        SettingsKey::ShowLayerHover => "Visa lager",
        SettingsKey::RemoveLayerHover => "Ta bort lager",
        SettingsKey::MoveDownHover => "Flytta ned",
        SettingsKey::MoveUpHover => "Flytta upp",
        SettingsKey::NoVariable => "(ingen variabel)",
        SettingsKey::ColorbarLabel => "Färgskaleetikett",
        SettingsKey::ResetLabelHover => "Återställ färgskalans etikett till standard",
        SettingsKey::ColorRange => "Färgintervall",
        SettingsKey::Categorical => "Kategorisk",
        SettingsKey::CategoricalHover => {
            "Diskret färgskala (hittar unika värden automatiskt eller 10 lika stora klasser)."
        }
        SettingsKey::LockBoundsHover => {
            "Lås min/max så att färgmappningen är densamma över alla tidssteg."
        }
        SettingsKey::ResetBoundsHover => {
            "Återställ gränserna till aktuellt snitts/datasets standardvärden för min och max"
        }
        SettingsKey::ResetAllColorbar => "Återställ alla färgskaleinställningar",
        SettingsKey::ResetAllColorbarHover => {
            "Återställ både färgskalans etikett och intervall till standardvärden"
        }
        SettingsKey::RangeMin => "Min:",
        SettingsKey::RangeMax => "Max:",
        SettingsKey::NanColor => "NaN-färg",
        SettingsKey::NanColorHover => "Om omarkerad ritas NaN/Inf-värden genomskinliga.",
        SettingsKey::NanColorPickerHover => "NaN-färg. Klicka för att välja färg.",
        SettingsKey::LowClip => "Nedre klippning",
        SettingsKey::LowClipHover => "Värden < cmin klipps till denna färg.",
        SettingsKey::LowClipPickerHover => {
            "Färg för nedre klippning (< Min). Klicka för att välja färg."
        }
        SettingsKey::HighClip => "Övre klippning",
        SettingsKey::HighClipHover => "Värden > cmax klipps till denna färg.",
        SettingsKey::HighClipPickerHover => {
            "Färg för övre klippning (> Max). Klicka för att välja färg."
        }
        SettingsKey::Scale => "Skala",
        SettingsKey::ScaleLinear => "Linjär",
        SettingsKey::ScaleLog => "Logaritmisk",
        SettingsKey::ScaleSymlog => "Symlog",
        SettingsKey::ScaleSqrt => "Kvadratrot",
        SettingsKey::ScaleExponential => "Exponentiell",
        SettingsKey::ScaleLogHover => "Logaritmisk skala (icke-negativa data)",
        SettingsKey::ScaleLogDisabledHover => {
            "Avaktiverad: kräver min >= 0. Använd Symlog för negativa data."
        }
        SettingsKey::ScaleParam => "Param:",
        SettingsKey::Opacity => "Opacitet",
        SettingsKey::OpacityHover => {
            "Opacitet för färgmappade värden. NaN- och klippfärger behåller sin egen alfa. \
             Dubbelklicka för att återställa."
        }
        SettingsKey::AlphaCurve => "Alfakurva",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  eller  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "Alfa längs färgskalan, från dess början till dess slut. Värden fördelas jämnt \
             (Steg: en lika stor klass var); position:alfa-stopp placeras \
             längs skalan. Tomt för ingen."
        }
        SettingsKey::ClearCurveHover => "Rensa kurva",
        SettingsKey::Presets => "Förval",
        SettingsKey::InterpStep => "Steg",
        SettingsKey::InterpLinear => "Linjär",
        SettingsKey::PresetRampUp => "Stigande",
        SettingsKey::PresetRampDown => "Fallande",
        SettingsKey::PresetPeak => "Topp",
        SettingsKey::PresetSink => "Sänka",
        SettingsKey::PresetHideLow => "Dölj låga",
        SettingsKey::PresetHideCenter => "Dölj mitten",
        SettingsKey::Transparency => "Transparens",
        SettingsKey::TransparencyHover => {
            "Med genomskinliga färger (Opacitet eller Alfakurva) ritas varje lager \
             i stället för att det närmaste döljer de bakomliggande."
        }
        SettingsKey::Lighting => "Belysning",
        SettingsKey::LightingHover => {
            "Skugga sampel efter deras gradient så att fronter och kanter får form"
        }
        SettingsKey::Coastlines => "Kustlinjer",
        SettingsKey::CoastlinesHover => {
            "Visa kustlinjer från Natural Earth över geografiska diagram."
        }
        SettingsKey::ResetCoastlineColor => "återställ",
        SettingsKey::ResetCoastlineColorHover => "Återställ kustlinjens färg till temats standard",
        SettingsKey::CoastlineColorHover => "Kustlinjens färg. Klicka för att välja.",
        SettingsKey::CoastlineResolution => "Upplösning:",
        SettingsKey::Lod110mHover => "110 m - snabb, alltid tillgänglig (inbäddad)",
        SettingsKey::Lod50mHover => "50 m - medelhög detaljnivå (läses in i bakgrunden)",
        SettingsKey::Lod10mHover => "10 m - hög detaljnivå (läses in i bakgrunden)",
        SettingsKey::LineWidth => "Linjebredd",
        SettingsKey::CropToDomain => "Beskär till dataområdet",
        SettingsKey::CropToDomainHover => {
            "Klipp kustlinjerna strikt till det aktiva datasetets rumsliga gräns."
        }
        SettingsKey::AutoRotate => "Rotera automatiskt",
        SettingsKey::ResetView => "Återställ vy",
        SettingsKey::ResetViewHover => "Återställ 3D-kamerans orientering",
        SettingsKey::HoverCard => "Infokort",
        SettingsKey::HideHoverCardHover => "Dölj infokortet",
        SettingsKey::ShowHoverCardHover => "Visa infokortet",
        SettingsKey::Export => "Exportera",
        SettingsKey::ExportFormat => "Format:",
        SettingsKey::ExportFolder => "Mapp:",
        SettingsKey::OpenSaveDialog => "Öppna sparadialog",
        SettingsKey::CropTool => "Beskärningsverktyg",
        SettingsKey::Aggregation => "2D-aggregering",
        SettingsKey::AggregationMandatoryHover => {
            "Obligatorisk: matrisen överskrider gränsen för en enskild GPU-buffert ({} MB). \
             Aggregeringen kan inte avaktiveras."
        }
        SettingsKey::AggregationHover => {
            "Nedsampling i flerupplösta pyramider och omsampling efter vyn."
        }
        SettingsKey::AggregationMode => "Läge:",
        SettingsKey::AggMean => "Medel (boxfilter)",
        SettingsKey::AggMax => "Max (bevarar toppar)",
        SettingsKey::AggMin => "Min (bevarar dalar)",
        SettingsKey::AggNearest => "Närmaste granne",
        SettingsKey::AggMeanHover => "Medelvärdesbildar delpixlar i 2x2-celler och ignorerar NaN.",
        SettingsKey::AggMaxHover => "Bevarar extremt höga värden.",
        SettingsKey::AggMinHover => "Bevarar extremt låga värden.",
        SettingsKey::AggNearestHover => "Snabbt val av närmaste punkt utan interpolering.",
        SettingsKey::ResolutionLevel => "Upplösningsnivå",
        SettingsKey::ResolutionLevelHover => {
            "Nivån i flerskaliga data som diagrammet läser. Auto läser den finaste nivån som får plats på ytan."
        }
        SettingsKey::CmykAutoMapped => "Automatiskt mappade kanaler: C (1), M (2), Y (3), K (4)",
        SettingsKey::MultiChannelOverlay => "Flerkanalsöverlägg",
        SettingsKey::MultiChannelOverlayHover => {
            "Lägger ihop flera kanaler additivt, var och en med sin egen färgton."
        }
        SettingsKey::CmykComposite => "CMYK-komposit",
        SettingsKey::CmykCompositeHover => {
            "Sammanfogar 4 kanaler cyan, magenta, gul och svart (CMYK) till RGB i äkta färg."
        }
        SettingsKey::RgbComposite => "RGB-komposit",
        SettingsKey::RgbCompositeHover => "Sammanfogar 3 valda kanaler till RGB i äkta färg.",
        SettingsKey::Channels => "Kanaler:",
        SettingsKey::ChannelsAll => "Alla",
        SettingsKey::ChannelsNone => "Inga",
        SettingsKey::ChannelTintHover => "Klicka för att anpassa kanalens färgton",
        SettingsKey::BandNumber => "Band {}",
        SettingsKey::ScaleBar => "Skalstreck",
        SettingsKey::ShowScaleBarHover => "Visa skalstrecket",
        SettingsKey::HideScaleBarHover => "Dölj skalstrecket",
        SettingsKey::ChannelsAuto => "Auto",
        SettingsKey::ChannelsAutoHover => {
            "Anpassa varje kanals visningsintervall till visade data (0,1 till 99,9 procent av dess värden)"
        }
        SettingsKey::ChannelsAutoUnavailable => "Automatisk kontrast fungerar på ett 2D-snitt",
        SettingsKey::ChannelSettingsHover => "Visningsintervall, kurva och invertering",
        SettingsKey::ChannelWindow => "Visningsintervall",
        SettingsKey::ChannelWindowHover => {
            "Värden från start till slut går från svart till kanalens fulla färg"
        }
        SettingsKey::ChannelInvert => "Invertera",
        SettingsKey::ChannelCurve => "Kurva",
        SettingsKey::CurveLinear => "Linjär",
        SettingsKey::CurveLogarithmic => "Logaritmisk",
        SettingsKey::CurveExponential => "Exponentiell",
        SettingsKey::CurvePolynomial => "Polynomisk",
        SettingsKey::ChannelCoefficientHover => "Kurvans koefficient",
        SettingsKey::LabelsFill => "Fyllning",
        SettingsKey::LabelsFillHover => "Färga varje etikett helt",
        SettingsKey::LabelsOutline => "Kontur",
        SettingsKey::LabelsOutlineHover => "Rita bara gränserna mellan etiketter",
    })
}
