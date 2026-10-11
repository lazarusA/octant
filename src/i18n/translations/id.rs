//! Indonesian (Bahasa Indonesia) translations for Octant.

use crate::i18n::keys::Key;

pub fn translate(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::Dataset => "Dataset",
        Key::Variables => "Variabel",
        Key::Dimensions => "Dimensi",
        Key::Plot => "Plot",
        Key::Colormaps => "Peta Warna",
        Key::Settings => "Pengaturan",
        Key::About => "Tentang",

        Key::InspectMetadata => "Periksa Metadata Penyimpanan",
        Key::ActiveDatasets => "Dataset Aktif",
        Key::NoActiveDatasets => "Tidak ada dataset aktif. Buka dataset untuk memulai.",
        Key::CacheUsage => "Penggunaan Cache",
        Key::ClearCache => "Hapus Cache",
        Key::OpenLocal => "Buka Berkas Lokal",
        Key::CloudStore => "Penyimpanan Objek Cloud",
        Key::SampleDatasets => "Contoh Dataset",
        Key::DropFilesHere => "Tarik berkas atau folder ke sini",
        Key::SourceUrlOrPath => "URI penyimpanan atau jalur lokal",
        Key::Inspect => "Periksa",
        Key::LoadingMetadata => "Memeriksa metadata penyimpanan di latar belakang...",

        Key::Group => "Grup:",
        Key::DataType => "Tipe",
        Key::Shape => "Bentuk",
        Key::Description => "Deskripsi",
        Key::SearchVariables => "Cari variabel...",
        Key::NoVariablesMatching => "Tidak ada variabel yang cocok",
        Key::ShowingItemsOfTotal => "Menampilkan item dari total",

        Key::Ready => "Siap",
        Key::Loading => "Memuat...",
        Key::Close => "Tutup",
        Key::Cancel => "Batal",
        Key::Save => "Simpan",
        Key::Reset => "Atur Ulang",
        Key::Error => "Kesalahan",

        Key::Play => "Putar",
        Key::Pause => "Jeda",
        Key::StepForward => "Maju Selangkah",
        Key::StepBackward => "Mundur Selangkah",
        Key::Speed => "Kecepatan",
        Key::Loop => "Ulangi",

        Key::AboutSubtitle => "Penjelajah Data N-Dimensi",
        Key::AboutDescription => {
            "Octant adalah penampil interaktif untuk dataset N-dimensi dengan dukungan bawaan untuk penyimpanan lokal dan cloud, Zarr (v2/v3), dan Icechunk. Dibuat dengan Rust dan akselerasi GPU via WGPU. Berjalan di macOS, Linux, dan Windows."
        }
        Key::AboutHyperslabTitle => "Pemotongan Hyperslab",
        Key::AboutHyperslabDesc => "Cache chunk LRU asinkron & piramida multi-resolusi.",
        Key::AboutStorageTitle => "Dukungan Asli Zarr & Icechunk",
        Key::AboutStorageDesc => "Backend lokal, S3, GCS, Azure, dan streaming HTTP.",
        Key::AboutShadersTitle => "Shader WGPU Terakselerasi Perangkat Keras",
        Key::AboutShadersDesc => "Peta 2D, bola 3D, permukaan elevasi, volume & profil 1D.",
        Key::AboutLinksHeading => "Tautan & Sumber Daya",
        Key::AboutGoodFaithNotice => {
            "Octant menghargai karya pihak ketiga dengan itikad baik. Jika ada yang terlewat atau keliru, beri tahu kami untuk diperbaiki."
        }
        Key::AboutBrowseIcons => "Jelajahi Ikon Vektor Asli",
    })
}
