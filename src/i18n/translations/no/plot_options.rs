use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "Linjer",
        PlotOptionsKey::LinesHover => "Vis sammenhengende linjer mellom punktene",
        PlotOptionsKey::Scatter => "Punkter",
        PlotOptionsKey::ScatterHover => "Vis punktmarkører ved hvert punkt",
        PlotOptionsKey::PointSize => "Punktstørrelse:",
        PlotOptionsKey::CustomColor => "Egendefinert farge",
        PlotOptionsKey::CustomColorHover => {
            "Bruk én fast farge for linjer/punkter i stedet for fargekartet"
        }
        PlotOptionsKey::LineColorHover => "Farge for linjer / punkter. Klikk for å velge farge.",
        PlotOptionsKey::LineProfile => "Linjeprofil",
        PlotOptionsKey::AllLinesSeries => "Alle linjeserier",
        PlotOptionsKey::ProfileIndex => "Profilindeks",
        PlotOptionsKey::SingleProfile => "Bare én profil tilgjengelig.",
        PlotOptionsKey::AspectRatio => "Sideforhold",
        PlotOptionsKey::AspectRatioHover => {
            "Når avkrysset, beholder 2D-plott matrisens sideforhold (bredde/høyde). \
             Ellers fyller 2D-plott hele lerretet."
        }
        PlotOptionsKey::Algorithm => "Algoritme",
        PlotOptionsKey::Mode => "Modus",
        PlotOptionsKey::AlgoDvr => "Volum-raymarching (DVR)",
        PlotOptionsKey::AlgoMip => "Maksimal intensitet (MIP)",
        PlotOptionsKey::AlgoMinIp => "Minimal intensitet (MinIP)",
        PlotOptionsKey::AlgoXray => "Gjennomsnittsprojeksjon (røntgen)",
        PlotOptionsKey::AlgoLabelSurface => "Kategorisk etikettflate",
        PlotOptionsKey::AlgoAbsorption => "Absorpsjon RGBA",
        PlotOptionsKey::AlgoAdditive => "Additiv RGBA",
        PlotOptionsKey::AlgoIndexed => "Indeksert RGBA",
        PlotOptionsKey::Quality => "Kvalitet",
        PlotOptionsKey::QualityHover => "Prøver per voksel langs hver stråle",
        PlotOptionsKey::Density => "Tetthet",
        PlotOptionsKey::Attenuation => "Demping",
        PlotOptionsKey::ZScale => "Z-skala",
        PlotOptionsKey::ModeSmooth => "Glatt",
        PlotOptionsKey::ModeBumpy => "Ujevn",
        PlotOptionsKey::ModeSteps => "Trinn",
        PlotOptionsKey::ModeVoxel => "Voksel",
        PlotOptionsKey::Height => "Høyde",
        PlotOptionsKey::FlipHeights => "Snu høyder",
        PlotOptionsKey::ScaleByValue => "Skaler etter verdi",
        PlotOptionsKey::FlipHeightsHover => {
            "Den høyeste verdien ligger flatt, og lavere verdier hever seg over den"
        }
        PlotOptionsKey::MeshDetail => "Detalj",
        PlotOptionsKey::MeshDetailHover => {
            "Celler som tegnes på store rutenett: mer detalj er tregere; pekeren leser alltid hver celle; Hver celle stopper ved det én GPU-buffer rommer"
        }
        PlotOptionsKey::DetailScreen => "Skjerm",
        PlotOptionsKey::DetailDouble => "2x skjerm",
        PlotOptionsKey::DetailQuadruple => "4x skjerm",
        PlotOptionsKey::DetailFull => "Hver celle",
        PlotOptionsKey::Size => "Størrelse",
        PlotOptionsKey::Surface => "Overflate",
        PlotOptionsKey::SurfaceRange => "Fargeområde",
        PlotOptionsKey::SurfaceValue => "Verdi",
        PlotOptionsKey::SurfaceHover => {
            "Hvor det ugjennomsiktige volumet tegner overflaten: der verdiene går inn i fargeområdet, eller der de når én verdi"
        }
    })
}
