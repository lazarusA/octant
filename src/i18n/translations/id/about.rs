use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "Tentang Octant",
        AboutKey::IconsWindowTitle => "Ikon Vektor Bawaan",
        AboutKey::IconsTab => "Ikon Vektor",
        AboutKey::Subtitle => "Penjelajah Data N-Dimensi",
        AboutKey::Description => {
            "Octant adalah penampil interaktif untuk dataset N-dimensi dengan dukungan bawaan untuk penyimpanan objek lokal dan cloud, Zarr (v2/v3), dan Icechunk. Dibuat dengan Rust dengan perenderan berakselerasi GPU melalui WGPU. Octant berjalan secara native di macOS, Linux, dan Windows."
        }
        AboutKey::HyperslabTitle => "Pemotongan Hyperslab",
        AboutKey::HyperslabDesc => "Cache chunk LRU asinkron & piramida multi-resolusi.",
        AboutKey::StorageTitle => "Dukungan Asli Zarr & Icechunk",
        AboutKey::StorageDesc => "Backend lokal, S3, GCS, Azure, dan streaming HTTP.",
        AboutKey::ShadersTitle => "Shader WGPU Terakselerasi Perangkat Keras",
        AboutKey::ShadersDesc => "Peta datar 2D, bola 3D, permukaan elevasi, volume & profil 1D.",
        AboutKey::LinksHeading => "Tautan & Sumber Daya",
        AboutKey::DocsLink => "Dokumentasi Octant",
        AboutKey::GoodFaithNotice => {
            "Octant mencantumkan kredit karya pihak ketiga dengan itikad baik. Jika menurut Anda ada yang terlewat atau atribusinya keliru, beri tahu kami dan kami akan memperbaiki atau menghapusnya."
        }
        AboutKey::BrowseIcons => "Jelajahi Ikon Vektor Bawaan",
        AboutKey::BrowseIconsHover => "Jelajahi ikon vektor prosedural di jendela sembul",
        AboutKey::Licensed => "Dilisensikan di bawah MIT atau Apache-2.0",
        AboutKey::OctantTeaser => {
            "Tanda x, y, dan z (+/-, +/-, +/-) membagi ruang menjadi delapan oktan. Jelajahi data Anda dengan cara yang sama: pilih sumbu, potong irisan, dan amati satu wilayah setiap kali."
        }
        AboutKey::OctantHover => "Klik untuk berpindah ke oktan berikutnya",
        AboutKey::ColormapCredits => "Kredit peta warna",
        AboutKey::ColormapCreditsNote => "Peta warna bawaan tetap memakai lisensi dari pembuatnya.",
        AboutKey::LicenseTexts => "Teks lisensi",
        AboutKey::ThirdPartyLicenses => "Lisensi pihak ketiga",
        AboutKey::ThirdPartyNote => {
            "Octant menyertakan crate, fon, dan pustaka C (netCDF, HDF5) sumber terbuka, masing-masing dengan lisensinya sendiri."
        }
        AboutKey::ViewOnGithub => "Lihat di GitHub",
        AboutKey::IconsSearchHint => "Cari ikon...",
        AboutKey::IconsAll => "Semua",
        AboutKey::IconsCopied => "{} disalin",
        AboutKey::IconsNoMatch => "Tidak ada ikon vektor yang cocok.",
        AboutKey::IconsCategory => "Kategori: {}",
        AboutKey::IconsClickToCopy => "Klik untuk menyalin pengenal enum",
        AboutKey::IconsSizesTones => "Ukuran & nada",
        AboutKey::ClearSearchHover => "Hapus pencarian",
        AboutKey::FontLicensesNote => {
            "Noto Sans SC, JP, dan KR (teks Tionghoa, Jepang, dan Korea):"
        }
    })
}
