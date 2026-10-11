use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "Dataset Aktif",
        DatasetKey::NoDatasetLoaded => "Tidak ada dataset yang dimuat",
        DatasetKey::MemoryCache => "Memori & Cache",
        DatasetKey::SampleDatasets => "Contoh",
        DatasetKey::DropFilesHere => "Seret & Lepas Berkas atau Direktori",
        DatasetKey::SourceUrlOrPath => "URL Target / Jalur Berkas",
        DatasetKey::LoadingMetadata => "Memeriksa metadata penyimpanan di latar belakang...",
        // Dataset panel
        DatasetKey::OpenSource => "Buka Sumber",
        DatasetKey::DatasetManager => "Pengelola Dataset",
        DatasetKey::CloseDatasetHover => "Tutup Dataset",
        // Active dataset card
        DatasetKey::TargetUriHover => "URI target:\n{}",
        DatasetKey::VariableCountOne => "{} variabel",
        DatasetKey::VariableCountMany => "{} variabel",
        DatasetKey::InspectVariables => "Periksa Variabel",
        DatasetKey::Reload => "Muat Ulang",
        DatasetKey::BadgeVolume4D => "Volume 4D",
        DatasetKey::BadgeMatrix2D => "Matriks 2D",
        DatasetKey::BadgeStore => "Penyimpanan",
        // Source intake
        DatasetKey::Load => "Muat",
        DatasetKey::TargetPathHint => "https://... atau jalur (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "Katalog...",
        DatasetKey::Clear => "Hapus",
        DatasetKey::FormatLabel => "Format:",
        DatasetKey::SampleSeasFire => "SeasFire Kebakaran Hutan Global (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "Matriks Prosedural 2D (Uji)",
        // Store formats (menu entries, then the shorter selected texts)
        DatasetKey::FormatRemoteZarr => "Zarr Jarak Jauh (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "Zarr Lokal (Sistem Berkas)",
        DatasetKey::FormatRemoteIcechunk => "Icechunk Jarak Jauh (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "Icechunk Lokal (Sistem Berkas)",
        DatasetKey::FormatRemoteGeoTiff => "GeoTIFF / COG Jarak Jauh (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "GeoTIFF Lokal (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "NetCDF / HDF5 Lokal (.nc/.h5)",
        DatasetKey::FormatVolume4D => "Volume 4D Kebenaran Diketahui (Prosedural)",
        DatasetKey::FormatMatrix2D => "Matriks Prosedural 2D (Prosedural)",
        DatasetKey::FormatRemoteGeoTiffShort => "GeoTIFF / COG Jarak Jauh",
        DatasetKey::FormatLocalGeoTiffShort => "GeoTIFF Lokal",
        DatasetKey::FormatLocalNetCdfShort => "NetCDF / HDF5 Lokal",
        DatasetKey::FormatVolume4DShort => "Volume 4D Kebenaran Diketahui",
        DatasetKey::FormatMatrix2DShort => "Matriks Prosedural 2D",
        // Dataset manager
        DatasetKey::NoDatasetsRegistered => "Belum ada dataset yang terdaftar di pengelola.",
        DatasetKey::LoadedCount => "Dimuat ({}):",
        DatasetKey::ClearAll => "Hapus Semua",
        DatasetKey::ClearAllHover => "Hapus semua dataset dari pengelola",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "Hapus dataset ini",
        // Cache statistics
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "{} blok - tingkat hit {}%",
        DatasetKey::CacheMenuTitle => "Cache Memori Multi-Variabel Terpadu",
        DatasetKey::GlobalMemoryUsage => "Penggunaan Memori RAM Global:",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "Rincian Memori per Jenis:",
        DatasetKey::ResidentHyperslabs => "Hyperslab Residen: {} MB ({} blok)",
        DatasetKey::MultiResPyramid => "Piramida Multi-Resolusi: {} MB",
        DatasetKey::CacheHitsMisses => "Cache Hit: {} | Miss: {} (Tingkat Hit: {}%)",
        DatasetKey::PrefetchInFlight => "Prefetch Latar Belakang: {} berjalan",
        DatasetKey::AllBlocksCached => "Buffer Siap / Semua Blok Ter-cache",
        DatasetKey::SummaryPerVariable => "Ringkasan per Variabel:",
        DatasetKey::NoVariablesCached => "(Belum ada variabel yang ter-cache di memori)",
        DatasetKey::VariableCacheSize => "({} MB, {} blok)",
        DatasetKey::ClearVariableCacheHover => "Hapus cache untuk variabel ini",
        DatasetKey::PrefetchCapacitySettings => "Pengaturan Prefetch & Kapasitas",
        DatasetKey::EnablePrefetching => "Aktifkan Prefetch Latar Belakang",
        DatasetKey::CacheLimitSuffix => " MB (batas)",
        DatasetKey::WindowSizeSuffix => " Ukuran Jendela (langkah)",
        DatasetKey::PrefetchWorkersSuffix => " Pekerja Prefetch (thread)",
        DatasetKey::FlushAllCaches => "Kosongkan & Hapus Semua Cache",
        // Catalog
        DatasetKey::CatalogTitle => "Katalog Dataset",
        DatasetKey::CatalogStoreCount => "{} penyimpanan",
        DatasetKey::CatalogBlurb => {
            "Dataset cloud Zarr, Icechunk, dan kebenaran dasar prosedural pilihan. Pilih dataset mana pun untuk dimuat."
        }
        DatasetKey::CatalogSearchHint => "Saring menurut nama, deskripsi, atau URL...",
        DatasetKey::CatalogCategory => "Kategori:",
        DatasetKey::CategoryAll => "Semua",
        DatasetKey::CategoryProcedural => "Prosedural",
        DatasetKey::NoCatalogMatches => "Tidak ada dataset yang cocok dengan pencarian Anda.",
        DatasetKey::ClearSearch => "Hapus Pencarian",
        DatasetKey::SelectAndLoadDataset => "Pilih & Muat Dataset",
        DatasetKey::SelectAndLoad => "Pilih & Muat",
        // Drop zone
        DatasetKey::DropTypeNotSupported => "Jenis tidak didukung",
        DatasetKey::DropReleaseToLoad => "Lepaskan untuk memuat dataset",
        DatasetKey::DropSupportedList => "Didukung: {}",
        // Hero landing
        DatasetKey::HeroTryPrefix => "coba:",
        DatasetKey::HeroSeasFireHover => {
            "Dataset kebakaran hutan & iklim global yang di-rechunk (Zarr)"
        }
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A Warna Asli COG (AWS S3)",
        DatasetKey::HeroVolume4DHover => "Volume spasiotemporal 4D sintetis",
        DatasetKey::HeroCatalogHover => "Jelajahi katalog dataset",
        DatasetKey::LoadSampleHover => "Muat contoh: {}",
        DatasetKey::HeroHintLong => {
            "tempel URL, jalur lokal, atau seret & lepas berkas di mana saja"
        }
        DatasetKey::HeroHintMedium => "tempel URL, jalur, atau seret & lepas berkas",
        DatasetKey::HeroHintShort => "tempel URL atau lepas berkas",
        DatasetKey::HeroDropLong => "Lepas dataset untuk dimuat (.nc, .h5, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropMedium => "Lepas dataset (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "Lepas dataset untuk dimuat",
        DatasetKey::HeroUnsupportedLong => {
            "Jenis tidak didukung. Didukung: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => "Format tidak didukung (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroUnsupportedShort => "Format tidak didukung",
        DatasetKey::HeroInputHintLong => {
            "https://... atau jalur (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... atau jalur (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL atau jalur...",
        DatasetKey::ClearInputHover => "Hapus input",
        DatasetKey::LoadDatasetHover => "Muat Dataset & Buka Variabel",
        DatasetKey::HopOctantHover => "Klik untuk melompat oktan",
        DatasetKey::HeroLoadingSource => "memuat: {}",
        DatasetKey::HeroLoading => "memuat...",
        DatasetKey::HeroLoaded => "dimuat: {}",
        // Errors
        DatasetKey::BrowserLocalPaths => {
            "Jalur berkas lokal tidak dapat dibaca langsung di browser karena keamanan sandbox web.\n\nUntuk melihat dataset lokal di browser:\n1. Sajikan direktori atau berkas Anda dengan server HTTP lokal: `npx serve` atau `python3 -m http.server`\n2. Masukkan URL: `http://localhost:8000/my_dataset`\n\nAtau jalankan versi desktop native Octant (`cargo run --release`)."
        }
    })
}
