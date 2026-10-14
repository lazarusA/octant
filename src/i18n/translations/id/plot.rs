use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "Peta Panas",
        PlotKey::Line => "Garis 1D",
        PlotKey::Surface => "Permukaan",
        PlotKey::Volume => "Volume",
        PlotKey::Sphere => "Bola",
        PlotKey::PointCloud => "Awan Titik",
        PlotKey::CompositeTrueColor => "Warna asli",
        PlotKey::CompositeFalseColor => "Warna semu",
        PlotKey::CompositeRgb => "Komposit RGB",
        PlotKey::CompositeOverlay => "Tumpukan kanal",
        PlotKey::AlignGeo => "Pada grid bujur/lintang lain (belum didukung)",
        PlotKey::AlignIndexOnly => {
            "Bentuk grid sama, tetapi tidak ada koordinat yang cocok dengan plot"
        }
        PlotKey::AlignIncompatible => "Tidak berbagi grid dengan plot",
        PlotKey::OverlayPlotFirst => "Plot sebuah variabel terlebih dahulu",
        PlotKey::OverlayHeatmapOnly => "Overlay digambar di atas peta panas",
        PlotKey::OverlayStackFull => "Paling banyak 4 overlay",
        PlotKey::OverlayAlreadyLayer => "Sudah menjadi lapisan plot",
        PlotKey::OverlayCantAdd => "Tidak dapat menambahkan overlay",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "Pilih Proyeksi Visualisasi",
        PlotKey::GridHealpixDiscrete => "HEALPix / Diskret",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "Tidak didukung untuk grid {}",
        PlotKey::DisabledAggregation => "Nonaktif: Agregasi 2D aktif",
        PlotKey::Requires3D => "Memerlukan Data 3D",
        PlotKey::DisabledGpuLimit => "Nonaktif: {} MB > batas GPU 128 MB",
        PlotKey::ScalarField => "Medan Skalar",
        PlotKey::LabelsHeatmapOnly => "Gambar label hanya digambar pada peta panas",
    })
}
