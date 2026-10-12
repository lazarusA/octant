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
        NoticesKey::DataFontMissing => "Tidak ada font untuk sebagian teks dataset",
        NoticesKey::DataFontMissingDetail => {
            "Teks dalam {} di dataset ini mungkin tampil sebagai kotak. Pasang font seperti Noto Sans CJK."
        }
    })
}
