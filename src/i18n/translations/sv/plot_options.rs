use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "Linjer",
        PlotOptionsKey::LinesHover => "Visa sammanhängande linjer mellan punkterna",
        PlotOptionsKey::Scatter => "Punkter",
        PlotOptionsKey::ScatterHover => "Visa punktmarkörer vid varje punkt",
        PlotOptionsKey::PointSize => "Punktstorlek:",
        PlotOptionsKey::CustomColor => "Anpassad färg",
        PlotOptionsKey::CustomColorHover => {
            "Använd en enfärgad linje-/punktfärg i stället för färgkartan"
        }
        PlotOptionsKey::LineColorHover => "Linje-/punktfärg. Klicka för att välja färg.",
        PlotOptionsKey::LineProfile => "Linjeprofil",
        PlotOptionsKey::AllLinesSeries => "Alla linjeserier",
        PlotOptionsKey::ProfileIndex => "Profilindex",
        PlotOptionsKey::SingleProfile => "En enda profil tillgänglig.",
        PlotOptionsKey::AspectRatio => "Bildförhållande",
        PlotOptionsKey::AspectRatioHover => {
            "Om markerad behåller 2D-diagram matrisdatans bildförhållande (bredd/höjd). \
             Om omarkerad fyller 2D-diagram hela ytan."
        }
        PlotOptionsKey::Algorithm => "Algoritm",
        PlotOptionsKey::AlgoDvr => "Volymstrålföljning (DVR)",
        PlotOptionsKey::AlgoMip => "Maximal intensitet (MIP)",
        PlotOptionsKey::AlgoMinIp => "Minimal intensitet (MinIP)",
        PlotOptionsKey::AlgoXray => "Medelprojektion (röntgen)",
        PlotOptionsKey::AlgoLabelSurface => "Kategorisk etikettyta",
        PlotOptionsKey::AlgoAbsorption => "Absorption RGBA",
        PlotOptionsKey::AlgoAdditive => "Additiv RGBA",
        PlotOptionsKey::AlgoIndexed => "Indexerad RGBA",
        PlotOptionsKey::Quality => "Kvalitet",
        PlotOptionsKey::QualityHover => "Sampel per voxel längs varje stråle",
        PlotOptionsKey::Density => "Densitet",
        PlotOptionsKey::Attenuation => "Dämpning",
        PlotOptionsKey::ZScale => "Z-skala",
        PlotOptionsKey::ModeSmooth => "Mjuk",
        PlotOptionsKey::ModeBumpy => "Ojämn",
        PlotOptionsKey::ModeSteps => "Steg",
        PlotOptionsKey::ModeVoxel => "Voxel",
        PlotOptionsKey::Height => "Höjd",
        PlotOptionsKey::FlipHeights => "Vänd höjder",
        PlotOptionsKey::FlipHeightsHover => {
            "Det högsta värdet ligger plant och lägre värden reser sig över det"
        }
        PlotOptionsKey::Size => "Storlek",
    })
}
