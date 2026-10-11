use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "Garis",
        PlotOptionsKey::LinesHover => "Tampilkan garis kontinu yang menghubungkan titik",
        PlotOptionsKey::Scatter => "Sebaran",
        PlotOptionsKey::ScatterHover => "Tampilkan penanda sebaran di setiap lokasi titik",
        PlotOptionsKey::PointSize => "Ukuran Titik:",
        PlotOptionsKey::CustomColor => "Warna Kustom",
        PlotOptionsKey::CustomColorHover => {
            "Gunakan satu warna garis/sebaran alih-alih evaluasi peta warna"
        }
        PlotOptionsKey::LineColorHover => "Warna garis / sebaran. Klik untuk memilih warna.",
        PlotOptionsKey::LineProfile => "Profil Garis",
        PlotOptionsKey::AllLinesSeries => "Semua Seri Garis",
        PlotOptionsKey::ProfileIndex => "Indeks Profil",
        PlotOptionsKey::SingleProfile => "Hanya satu profil tersedia.",
        PlotOptionsKey::AspectRatio => "Rasio Aspek",
        PlotOptionsKey::AspectRatioHover => {
            "Jika dicentang, plot 2D mempertahankan rasio aspek data matriks (lebar/tinggi). Jika tidak dicentang, plot 2D melebar memenuhi seluruh kanvas."
        }
        PlotOptionsKey::Algorithm => "Algoritma",
        PlotOptionsKey::AlgoDvr => "Raymarching Volume (DVR)",
        PlotOptionsKey::AlgoMip => "Intensitas Maksimum (MIP)",
        PlotOptionsKey::AlgoMinIp => "Intensitas Minimum (MinIP)",
        PlotOptionsKey::AlgoXray => "Proyeksi Rata-rata (Sinar-X)",
        PlotOptionsKey::AlgoLabelSurface => "Permukaan Label Kategoris",
        PlotOptionsKey::AlgoAbsorption => "RGBA Absorpsi",
        PlotOptionsKey::AlgoAdditive => "RGBA Aditif",
        PlotOptionsKey::AlgoIndexed => "RGBA Terindeks",
        PlotOptionsKey::Quality => "Kualitas",
        PlotOptionsKey::QualityHover => "Sampel per voxel di sepanjang setiap sinar",
        PlotOptionsKey::Density => "Kepadatan",
        PlotOptionsKey::Attenuation => "Atenuasi",
        PlotOptionsKey::ZScale => "Skala Z",
        PlotOptionsKey::ModeSmooth => "Halus",
        PlotOptionsKey::ModeBumpy => "Bergelombang",
        PlotOptionsKey::ModeSteps => "Tangga",
        PlotOptionsKey::ModeVoxel => "Voxel",
        PlotOptionsKey::Height => "Tinggi",
        PlotOptionsKey::FlipHeights => "Balik ketinggian",
        PlotOptionsKey::FlipHeightsHover => {
            "Nilai tertinggi terletak datar dan nilai yang lebih rendah naik di atasnya"
        }
        PlotOptionsKey::Size => "Ukuran",
    })
}
