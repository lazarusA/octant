use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Putar",
        PlaybackKey::Pause => "Jeda",
        PlaybackKey::Loop => "Ulangi",
        PlaybackKey::FirstStep => "Langkah Pertama",
        PlaybackKey::PreviousStep => "Langkah Sebelumnya",
        PlaybackKey::NextStep => "Langkah Berikutnya",
        PlaybackKey::LastStep => "Langkah Terakhir",
        PlaybackKey::Crop => "Potong",
        PlaybackKey::CropHover => "Garis Panduan Potong",
        PlaybackKey::PlaybackSpeed => "Kecepatan Putar",
        PlaybackKey::PlaybackBuffer => "Buffer Sebelum Putar",
        PlaybackKey::Buffering => "Memuat buffer {}/{}",
        PlaybackKey::SaveExportHover => {
            "Simpan / Ekspor Gambar (Cmd+S untuk Simpan Cepat, Cmd+Shift+S untuk Pengaturan)"
        }
        PlaybackKey::CollapsedHint => "Pemutaran (klik atau seret untuk membuka)",
        PlaybackKey::TimelineDetails => "Detail Linimasa",
        PlaybackKey::TimelineDimension => "Dimensi: {}",
        PlaybackKey::TimelineStep => "Langkah: {} / {}",
        PlaybackKey::TimelineCurrent => "Saat ini: {}",
        PlaybackKey::TimelineRange => "Rentang: {} -> {}",
        PlaybackKey::TimelineStepSize => "Ukuran Langkah: {}",
        PlaybackKey::DefaultStepSize => "Langkah: 1",
    })
}
