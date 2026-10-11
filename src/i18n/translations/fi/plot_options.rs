use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "Viivat",
        PlotOptionsKey::LinesHover => "Näytä pisteitä yhdistävät yhtenäiset viivat",
        PlotOptionsKey::Scatter => "Hajonta",
        PlotOptionsKey::ScatterHover => "Näytä merkki jokaisen pisteen kohdalla",
        PlotOptionsKey::PointSize => "Pistekoko:",
        PlotOptionsKey::CustomColor => "Mukautettu väri",
        PlotOptionsKey::CustomColorHover => "Käytä yhtä viiva-/pisteväriä värikartan sijaan",
        PlotOptionsKey::LineColorHover => "Viivan / pisteiden väri. Valitse väri napsauttamalla.",
        PlotOptionsKey::LineProfile => "Viivaprofiili",
        PlotOptionsKey::AllLinesSeries => "Kaikki viivasarjat",
        PlotOptionsKey::ProfileIndex => "Profiilin indeksi",
        PlotOptionsKey::SingleProfile => "Vain yksi profiili saatavilla.",
        PlotOptionsKey::AspectRatio => "Kuvasuhde",
        PlotOptionsKey::AspectRatioHover => {
            "Kun valittu, 2D-kuvaajat säilyttävät matriisin kuvasuhteen (leveys/korkeus). \
             Kun ei valittu, 2D-kuvaajat täyttävät koko piirtoalueen."
        }
        PlotOptionsKey::Algorithm => "Algoritmi",
        PlotOptionsKey::AlgoDvr => "Tilavuuden säteenseuranta (DVR)",
        PlotOptionsKey::AlgoMip => "Maksimi-intensiteetti (MIP)",
        PlotOptionsKey::AlgoMinIp => "Minimi-intensiteetti (MinIP)",
        PlotOptionsKey::AlgoXray => "Keskiarvoprojektio (röntgen)",
        PlotOptionsKey::AlgoLabelSurface => "Kategorinen tunnistepinta",
        PlotOptionsKey::AlgoAbsorption => "Absorptio-RGBA",
        PlotOptionsKey::AlgoAdditive => "Additiivinen RGBA",
        PlotOptionsKey::AlgoIndexed => "Indeksoitu RGBA",
        PlotOptionsKey::Quality => "Laatu",
        PlotOptionsKey::QualityHover => "Näytteitä vokselia kohden kunkin säteen suunnassa",
        PlotOptionsKey::Density => "Tiheys",
        PlotOptionsKey::Attenuation => "Vaimennus",
        PlotOptionsKey::ZScale => "Z-skaala",
        PlotOptionsKey::ModeSmooth => "Sileä",
        PlotOptionsKey::ModeBumpy => "Kuhmurainen",
        PlotOptionsKey::ModeSteps => "Portaat",
        PlotOptionsKey::ModeVoxel => "Vokseli",
        PlotOptionsKey::Height => "Korkeus",
        PlotOptionsKey::FlipHeights => "Käännä korkeudet",
        PlotOptionsKey::FlipHeightsHover => {
            "Suurin arvo on tasainen ja pienemmät arvot nousevat sen yläpuolelle"
        }
        PlotOptionsKey::Size => "Koko",
    })
}
