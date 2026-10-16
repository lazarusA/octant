use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "Grup",
        VariablesKey::DataType => "Tipe",
        VariablesKey::Shape => "Bentuk",
        VariablesKey::Description => "Deskripsi",
        VariablesKey::SearchVariables => "Cari variabel...",
        VariablesKey::NoVariablesMatching => "Tidak ada variabel yang cocok dengan '{}'",
        VariablesKey::CloseVariablesWindow => "Tutup Jendela Variabel",
        VariablesKey::NoMetadataLoaded => "Belum ada metadata penyimpanan yang dimuat.",
        VariablesKey::FetchStoreMetadata => "Ambil / Muat Metadata Penyimpanan",
        VariablesKey::NoVariablesFound => {
            "Tidak ada variabel ditemukan di penyimpanan dataset ini."
        }
        VariablesKey::RefreshStoreMetadata => "Segarkan Metadata Penyimpanan",
        VariablesKey::TruncatedVariables => {
            "Menampilkan {} dari {} variabel di folder ini. Gunakan pencarian untuk menemukan semuanya."
        }
        VariablesKey::TruncatedFolders => {
            "Menampilkan {} dari {} folder. Gunakan pencarian untuk menemukan semuanya."
        }
        VariablesKey::NoVariableSelected => "Tidak ada variabel yang dipilih.",
        VariablesKey::DimensionSliders => "Penggeser Dimensi",
        VariablesKey::CloseDimensionPanel => "Tutup Panel Dimensi",
        VariablesKey::PlotData => "Plot Data",
        VariablesKey::PlotDataHover => "Ambil variabel ini dan plot",
        VariablesKey::PlotDataOverlayHover => {
            "Ambil variabel ini dan tambahkan sebagai overlay di atas plot"
        }
        VariablesKey::AddOverlay => "Tambah Overlay",
        VariablesKey::AddOverlayHover => {
            "Saat aktif, Plot Data menambahkan variabel ini sebagai overlay"
        }
        VariablesKey::InfoPath => "Jalur:",
        VariablesKey::InfoUnits => "Satuan: {}",
        VariablesKey::InfoDescription => "Deskripsi: {}",
        VariablesKey::InfoShape => "Bentuk: {}",
        VariablesKey::InfoDimensions => "Dimensi: {}",
        VariablesKey::InfoTime => "Waktu: {} -> {}",
        VariablesKey::InfoResolution => "Resolusi: {}",
        VariablesKey::InfoSize => "Ukuran: {} MB",
        VariablesKey::InfoAttributes => "Atribut (.zattrs)",
        VariablesKey::Download => "Unduhan",
        VariablesKey::DownloadCells => "{} ({} sel)",
        VariablesKey::LargeSelectionPyramid => {
            "Pilihan 2D besar ({} sel, {} MB): agregasi piramida multi-resolusi otomatis diaktifkan."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "Mesh Bola 3D & Permukaan 3D dinonaktifkan untuk pilihan besar ini ({} sel, {} MB). Plot Bidang 2D dan Garis 1D tetap aktif sepenuhnya."
        }
        VariablesKey::LargeVolumeDisabled => {
            "Volume 3D & Awan Titik dinonaktifkan untuk pilihan ini: ukuran volume ({} MB) melebihi batas buffer penyimpanan GPU {} MB. Bidang 2D, Garis 1D, dan Bola 3D tetap aktif."
        }
        VariablesKey::RoleNone => "Tidak ada",
        VariablesKey::RoleAnimated => "Animasi",
        VariablesKey::RoleGrid => "Grid",
        VariablesKey::Index => "Indeks:",
        VariablesKey::ChannelN => "Kanal {}",
        VariablesKey::LevelN => "Level {}",
        VariablesKey::LevelAuto => "Otomatis",
        VariablesKey::LevelOf => "Level {} dari {}",
        VariablesKey::AutoLevelOf => "Otomatis, level {} dari {}",
    })
}
