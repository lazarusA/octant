use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "Linien",
        PlotOptionsKey::LinesHover => "Durchgehende Linien zwischen den Punkten zeigen",
        PlotOptionsKey::Scatter => "Punkte",
        PlotOptionsKey::ScatterHover => "Streumarker an jeder Punktposition zeigen",
        PlotOptionsKey::PointSize => "Punktgröße:",
        PlotOptionsKey::CustomColor => "Eigene Farbe",
        PlotOptionsKey::CustomColorHover => {
            "Eine feste Linien-/Punktfarbe statt der Farbtabelle verwenden"
        }
        PlotOptionsKey::LineColorHover => "Linien-/Punktfarbe. Klicken, um eine Farbe zu wählen.",
        PlotOptionsKey::LineProfile => "Linienprofil",
        PlotOptionsKey::AllLinesSeries => "Alle Linienserien",
        PlotOptionsKey::ProfileIndex => "Profilindex",
        PlotOptionsKey::SingleProfile => "Nur ein Profil verfügbar.",
        PlotOptionsKey::AspectRatio => "Seitenverhältnis",
        PlotOptionsKey::AspectRatioHover => {
            "Wenn aktiviert, behalten 2D-Plots das Seitenverhältnis der Matrixdaten (Breite/Höhe) bei. \
             Wenn deaktiviert, füllen 2D-Plots die gesamte Zeichenfläche."
        }
        PlotOptionsKey::Algorithm => "Algorithmus",
        PlotOptionsKey::AlgoDvr => "Volumen-Raymarching (DVR)",
        PlotOptionsKey::AlgoMip => "Maximale Intensität (MIP)",
        PlotOptionsKey::AlgoMinIp => "Minimale Intensität (MinIP)",
        PlotOptionsKey::AlgoXray => "Mittelwertprojektion (Röntgen)",
        PlotOptionsKey::AlgoLabelSurface => "Kategorische Label-Oberfläche",
        PlotOptionsKey::AlgoAbsorption => "Absorption RGBA",
        PlotOptionsKey::AlgoAdditive => "Additives RGBA",
        PlotOptionsKey::AlgoIndexed => "Indiziertes RGBA",
        PlotOptionsKey::Quality => "Qualität",
        PlotOptionsKey::QualityHover => "Abtastwerte pro Voxel entlang jedes Strahls",
        PlotOptionsKey::Density => "Dichte",
        PlotOptionsKey::Attenuation => "Dämpfung",
        PlotOptionsKey::ZScale => "Z-Skala",
        PlotOptionsKey::ModeSmooth => "Glatt",
        PlotOptionsKey::ModeBumpy => "Uneben",
        PlotOptionsKey::ModeSteps => "Stufen",
        PlotOptionsKey::ModeVoxel => "Voxel",
        PlotOptionsKey::Height => "Höhe",
        PlotOptionsKey::FlipHeights => "Höhen umkehren",
        PlotOptionsKey::FlipHeightsHover => {
            "Der höchste Wert liegt flach, niedrigere Werte steigen darüber auf"
        }
        PlotOptionsKey::Size => "Größe",
    })
}
