use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "Fargene kommer fra komposittens kanaler.",
        SettingsKey::ReasonCustomColor => "Egendefinert farge tegner alle punkter i én farge.",
        SettingsKey::ReasonAllSeries => "Alle linjeserier farger hver linje etter indeksen.",
        SettingsKey::ReasonIndexed => "Indeksert RGBA farger vokslene etter etikettindeksen.",
        SettingsKey::ReasonIndexedMissing => "Indeksert RGBA hopper over manglende voksler.",
        SettingsKey::ReasonClassicOpaque => {
            "Bare Volum-raymarching (DVR) tegner gjennomskinnelige farger."
        }
        SettingsKey::ReasonNotTranslucent => "Gjelder med Opasitet under 1 eller en Alfakurve.",
        SettingsKey::ReasonCompositeOpaque => "Kompositter tegnes ugjennomsiktige.",
        SettingsKey::ReasonLabelOpaque => "Kategorisk etikettflate tegnes alltid ugjennomsiktig.",
        SettingsKey::ReasonAlwaysBlends => "Denne algoritmen blander alltid langs strålen.",
        SettingsKey::CloseSettingsHover => "Lukk innstillinger",
        SettingsKey::SectionOverlays => "Overlegg",
        SettingsKey::SectionResolution => "Oppløsning",
        SettingsKey::SectionView => "Visning",
        SettingsKey::Layers => "Lag",
        SettingsKey::BaseLayerHover => "Grunnlag: bestemmer lerretet, aksene og visningen",
        SettingsKey::BaseTag => "grunn",
        SettingsKey::Color => "Farge",
        SettingsKey::HideLayerHover => "Skjul lag",
        SettingsKey::ShowLayerHover => "Vis lag",
        SettingsKey::RemoveLayerHover => "Fjern lag",
        SettingsKey::MoveDownHover => "Flytt ned",
        SettingsKey::MoveUpHover => "Flytt opp",
        SettingsKey::NoVariable => "(ingen variabel)",
        SettingsKey::ColorbarLabel => "Fargeskalaetikett",
        SettingsKey::ResetLabelHover => "Tilbakestill fargeskalaetiketten",
        SettingsKey::ColorRange => "Fargeområde",
        SettingsKey::Categorical => "Kategorisk",
        SettingsKey::CategoricalHover => {
            "Diskret fargeskala (finner unike verdier automatisk, ellers 10 like intervaller)."
        }
        SettingsKey::LockBoundsHover => {
            "Lås min/maks slik at fargeleggingen er lik på tvers av tidssteg."
        }
        SettingsKey::ResetBoundsHover => {
            "Tilbakestill grensene til min og maks for gjeldende snitt/datasett"
        }
        SettingsKey::ResetAllColorbar => "Tilbakestill hele fargeskalaen",
        SettingsKey::ResetAllColorbarHover => {
            "Tilbakestill både etiketten og området til fargeskalaen"
        }
        SettingsKey::RangeMin => "Min:",
        SettingsKey::RangeMax => "Maks:",
        SettingsKey::NanColor => "NaN-farge",
        SettingsKey::NanColorHover => "Når ikke avkrysset, vises NaN/Inf-verdier gjennomsiktige.",
        SettingsKey::NanColorPickerHover => "NaN-farge. Klikk for å velge farge.",
        SettingsKey::LowClip => "Nedre klipp",
        SettingsKey::LowClipHover => "Verdier < cmin klippes til denne fargen.",
        SettingsKey::LowClipPickerHover => {
            "Farge for nedre klipp (< Min). Klikk for å velge farge."
        }
        SettingsKey::HighClip => "Øvre klipp",
        SettingsKey::HighClipHover => "Verdier > cmax klippes til denne fargen.",
        SettingsKey::HighClipPickerHover => {
            "Farge for øvre klipp (> Maks). Klikk for å velge farge."
        }
        SettingsKey::Scale => "Skala",
        SettingsKey::ScaleLinear => "Lineær",
        SettingsKey::ScaleLog => "Logaritmisk",
        SettingsKey::ScaleSymlog => "Symlog",
        SettingsKey::ScaleSqrt => "Kvadratrot",
        SettingsKey::ScaleExponential => "Eksponentiell",
        SettingsKey::ScaleLogHover => "Logaritmisk skala (ikke-negative data)",
        SettingsKey::ScaleLogDisabledHover => {
            "Slått av: krever min >= 0. Bruk Symlog for negative data."
        }
        SettingsKey::ScaleParam => "Param:",
        SettingsKey::Opacity => "Opasitet",
        SettingsKey::OpacityHover => {
            "Opasitet for fargekartlagte verdier. NaN- og klippfarger beholder sin egen alfa. \
             Dobbeltklikk for å tilbakestille."
        }
        SettingsKey::AlphaCurve => "Alfakurve",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  eller  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "Alfa langs fargeskalaen, fra start til slutt. Verdier fordeles jevnt \
             (Trinn: ett likt intervall hver); posisjon:alfa-punkter plasseres \
             langs skalaen. Tom for ingen."
        }
        SettingsKey::ClearCurveHover => "Tøm kurven",
        SettingsKey::Presets => "Forhåndsvalg",
        SettingsKey::InterpStep => "Trinn",
        SettingsKey::InterpLinear => "Lineær",
        SettingsKey::PresetRampUp => "Stigende",
        SettingsKey::PresetRampDown => "Synkende",
        SettingsKey::PresetPeak => "Topp",
        SettingsKey::PresetSink => "Bunn",
        SettingsKey::PresetHideLow => "Skjul lave",
        SettingsKey::PresetHideCenter => "Skjul midten",
        SettingsKey::Transparency => "Gjennomsiktighet",
        SettingsKey::TransparencyHover => {
            "Med gjennomskinnelige farger (Opasitet eller Alfakurve) tegnes alle lag \
             i stedet for at det nærmeste skjuler de bak."
        }
        SettingsKey::Lighting => "Belysning",
        SettingsKey::LightingHover => {
            "Skyggelegg prøvene etter gradienten slik at fronter og kanter får form"
        }
        SettingsKey::Coastlines => "Kystlinjer",
        SettingsKey::CoastlinesHover => "Legg Natural Earth-kystlinjer over geografiske plott.",
        SettingsKey::ResetCoastlineColor => "tilbakestill",
        SettingsKey::ResetCoastlineColorHover => {
            "Tilbakestill kystlinjefargen til temaets standard"
        }
        SettingsKey::CoastlineColorHover => "Farge på kystlinjene. Klikk for å velge.",
        SettingsKey::CoastlineResolution => "Oppløsning:",
        SettingsKey::Lod110mHover => "110 m - rask, alltid tilgjengelig (innebygd)",
        SettingsKey::Lod50mHover => "50 m - middels detaljnivå (lastes i bakgrunnen)",
        SettingsKey::Lod10mHover => "10 m - høyt detaljnivå (lastes i bakgrunnen)",
        SettingsKey::LineWidth => "Linjebredde",
        SettingsKey::CropToDomain => "Beskjær til datadomenet",
        SettingsKey::CropToDomainHover => {
            "Klipp kystlinjene strengt til den romlige avgrensningen av det aktive datasettet."
        }
        SettingsKey::AutoRotate => "Autorotasjon",
        SettingsKey::ResetView => "Tilbakestill visning",
        SettingsKey::ResetViewHover => "Tilbakestill orienteringen til 3D-kameraet",
        SettingsKey::HoverCard => "Infokort",
        SettingsKey::HideHoverCardHover => "Skjul infokortet",
        SettingsKey::ShowHoverCardHover => "Vis infokortet",
        SettingsKey::Export => "Eksport",
        SettingsKey::ExportFormat => "Format:",
        SettingsKey::ExportFolder => "Mappe:",
        SettingsKey::OpenSaveDialog => "Åpne lagringsdialog",
        SettingsKey::CropTool => "Beskjæringsverktøy",
        SettingsKey::Aggregation => "2D-aggregering",
        SettingsKey::AggregationMandatoryHover => {
            "Obligatorisk: matrisen overskrider grensen for én GPU-buffer (128 MB / 33,5 mill. celler). \
             Aggregering kan ikke slås av."
        }
        SettingsKey::AggregationHover => {
            "Nedsampling med flernivåpyramide og omsampling til visningen."
        }
        SettingsKey::AggregationMode => "Modus:",
        SettingsKey::AggMean => "Gjennomsnitt (boksfilter)",
        SettingsKey::AggMax => "Maks (bevar topper)",
        SettingsKey::AggMin => "Min (bevar bunner)",
        SettingsKey::AggNearest => "Nærmeste nabo",
        SettingsKey::AggMeanHover => {
            "Tar gjennomsnittet av delpiksler i 2x2-celler og ignorerer NaN."
        }
        SettingsKey::AggMaxHover => "Bevarer ekstreme høye verdier.",
        SettingsKey::AggMinHover => "Bevarer ekstreme lave verdier.",
        SettingsKey::AggNearestHover => "Raskt valg av nærmeste punkt uten interpolasjon.",
        SettingsKey::CmykAutoMapped => "Automatisk tilordnede kanaler: C (1), M (2), Y (3), K (4)",
        SettingsKey::MultiChannelOverlay => "Flerkanalsoverlegg",
        SettingsKey::MultiChannelOverlayHover => {
            "Legger flere kanaler additivt oppå hverandre, hver med sin egen fargetone."
        }
        SettingsKey::CmykComposite => "CMYK-kompositt",
        SettingsKey::CmykCompositeHover => {
            "Setter sammen 4 kanaler cyan, magenta, gul og svart (CMYK) til ekte RGB-farger."
        }
        SettingsKey::RgbComposite => "RGB-kompositt",
        SettingsKey::RgbCompositeHover => "Setter sammen 3 valgte kanaler til ekte RGB-farger.",
        SettingsKey::Channels => "Kanaler:",
        SettingsKey::ChannelsAll => "Alle",
        SettingsKey::ChannelsNone => "Ingen",
        SettingsKey::ChannelTintHover => "Klikk for å endre kanalens fargetone",
        SettingsKey::BandNumber => "Bånd {}",
        SettingsKey::ScaleBar => "Målestokk",
        SettingsKey::ShowScaleBarHover => "Vis målestokken",
        SettingsKey::HideScaleBarHover => "Skjul målestokken",
        SettingsKey::ChannelsAuto => "Auto",
        SettingsKey::ChannelsAutoHover => {
            "Tilpass hver kanals vindu til de viste dataene (0,1 til 99,9 prosent av verdiene)"
        }
        SettingsKey::ChannelsAutoUnavailable => "Automatisk kontrast leser det viste 2D-snittet",
        SettingsKey::ChannelSettingsHover => "Vindu, kurve og invertering",
        SettingsKey::ChannelWindow => "Vindu",
        SettingsKey::ChannelWindowHover => {
            "Verdier fra start til slutt går fra svart til kanalens fulle farge"
        }
        SettingsKey::ChannelInvert => "Inverter",
        SettingsKey::ChannelCurve => "Kurve",
        SettingsKey::CurveLinear => "Lineær",
        SettingsKey::CurveLogarithmic => "Logaritmisk",
        SettingsKey::CurveExponential => "Eksponentiell",
        SettingsKey::CurvePolynomial => "Polynomisk",
        SettingsKey::ChannelCoefficientHover => "Kurvens koeffisient",
        SettingsKey::LabelsFill => "Fyll",
        SettingsKey::LabelsFillHover => "Fargelegg hver etikett helt",
        SettingsKey::LabelsOutline => "Kontur",
        SettingsKey::LabelsOutlineHover => "Tegn bare grensene mellom etiketter",
    })
}
