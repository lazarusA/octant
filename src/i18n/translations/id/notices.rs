use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "Tutup",
        NoticesKey::CopyDetails => "Salin detail",
        NoticesKey::RevealInFinder => "Tampilkan di Finder",
        NoticesKey::OpenFolder => "Buka Folder",
        NoticesKey::NoVariablesFound => "Tidak ada variabel ditemukan",
        NoticesKey::CouldntOpenDataset => "Tidak dapat membuka dataset",
        NoticesKey::UnsupportedFile => "Berkas tidak didukung",
        NoticesKey::CouldntLoadData => "Tidak dapat memuat data",
        NoticesKey::CoordinatesUnavailable => "Koordinat tidak tersedia",
        NoticesKey::CoordinatesFailedDetail => {
            "Koordinat '{}' gagal dimuat ({}); sumbu menampilkan indeks"
        }
        NoticesKey::CoastlinesUnavailable => "Garis pantai tidak tersedia",
        NoticesKey::VolumeTooLargeForGpu => "Volume terlalu besar untuk GPU",
        NoticesKey::VolumeLodOff => "Detail sesuai tampilan nonaktif untuk volume ini",
        NoticesKey::VolumeLodOffDetail => {
            "Level yang diplot memerlukan {} brick; atlas memuat {} dan menyisakan setengahnya untuk yang lebih halus. Volume digambar dengan satu level utuh."
        }
        NoticesKey::VolumeLodUnevenDetail => {
            "Level-levelnya bukan kelipatan bulat satu sama lain dengan sel yang sejajar, seperti yang dibutuhkan brick. Volume digambar dengan satu level utuh."
        }
        NoticesKey::VolumeLodWideIntegersDetail => {
            "Nilai bilangan bulatnya (seperti id label) mencakup rentang terlalu lebar untuk dibedakan oleh brick. Volume digambar dengan satu level utuh."
        }
        NoticesKey::VolumeLodUnsupportedDetail => {
            "GPU ini tidak dapat menulis dari shader volume, yang dibutuhkan brick. Volume digambar dengan satu level utuh."
        }
        NoticesKey::VolumeLodBrickFailed => "Sebagian volume tidak dapat dibaca pada level yang lebih halus",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "Ukuran buffer data volume ({} elemen / {} byte) melebihi batas maksimum buffer penyimpanan GPU ({})"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "Volume {}x{}x{} melebihi batas tekstur 3D GPU ({})"
        }
        NoticesKey::VolumeUploadSkipped => "Unggahan volume dilewati",
        NoticesKey::VolumeUploadMismatchDetail => "{} nilai untuk tekstur {}x{}x{}",
        NoticesKey::DataTooLargeForGpuBuffer => "Data terlalu besar untuk buffer GPU",
        NoticesKey::BufferTooLargeDetail => {
            "{}: ukuran data ({} byte) melebihi kapasitas buffer GPU ({} byte), penulisan dilewati"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: rentang pada elemen {} (+{} elemen) melebihi kapasitas buffer GPU ({} byte), penulisan dilewati"
        }
        NoticesKey::LinePlotTooLargeForGpu => "Plot garis terlalu besar untuk GPU",
        NoticesKey::LinePayloadTooLargeDetail => {
            "Garis-garis ini memerlukan {} byte; GPU ini mengikat paling banyak {}."
        }
        NoticesKey::FontLoadFailed => "Tidak dapat memuat font",
        NoticesKey::FontLoadFailedDetail => "Teks dalam {} mungkin tampil sebagai kotak: {}",
        NoticesKey::OmeMetadataNonConforming => "Metadata OME-Zarr tidak sesuai spesifikasi",
        NoticesKey::LabelIdsBeyond32Bits => {
            "ID label di atas 32 bit berbagi warna dengan ID yang lebih kecil"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "Metadata versi {} dari {} tidak mengikuti spesifikasi; menampilkan yang dapat dibaca."
        }
        NoticesKey::PlateWellStopped => "Sumur {}: pembacaan terhenti",
        NoticesKey::PlateWellNotJson => "Sumur {}: metadatanya bukan JSON yang valid",
        NoticesKey::PlateWellNotOme => "Sumur {}: bukan sumur OME-Zarr",
        NoticesKey::PlateWellNoMetadata => "Sumur {}: tidak ada metadata sumur",
    })
}
