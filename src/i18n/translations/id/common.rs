use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "Dataset",
        CommonKey::Variables => "Variabel",
        CommonKey::Dimensions => "Dimensi",
        CommonKey::Colormaps => "Peta Warna",
        CommonKey::Settings => "Pengaturan",
        CommonKey::About => "Tentang",
        CommonKey::Cache => "Cache",
        CommonKey::Language => "Bahasa",
        CommonKey::Loading => "Memuat...",
        CommonKey::CloseEsc => "Tutup (Esc)",
        CommonKey::Cancel => "Batal",
        CommonKey::Save => "Simpan",
        CommonKey::Reset => "Atur Ulang",
        CommonKey::Move => "Pindah",
        CommonKey::MoveHover => "Seret untuk memindahkan; klik ganda untuk mengatur ulang",
        CommonKey::Light => "Terang",
        CommonKey::Dark => "Gelap",
        CommonKey::SwitchToLightHover => "Beralih ke mode Terang",
        CommonKey::SwitchToDarkHover => "Beralih ke mode Gelap",
        CommonKey::ToggleHeroHover => "Tampilkan/sembunyikan Layar Awal (klik kanan untuk Tentang)",
        CommonKey::AboutOctantHover => "Tentang Octant",
        CommonKey::ToggleVariableControlsHover => "Tampilkan/sembunyikan Panel Kontrol Variabel",
        CommonKey::FetchingProgress => "Mengambil {} / {} ({}%)",
        CommonKey::Abort => "Hentikan",
        CommonKey::AbortHover => "Interupsi dan hentikan transfer data yang sedang berjalan",
    })
}
