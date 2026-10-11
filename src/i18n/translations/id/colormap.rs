use crate::i18n::keys::ColormapKey;

pub fn text(key: ColormapKey) -> Option<&'static str> {
    Some(match key {
        ColormapKey::Sequential => "Sekuensial",
        ColormapKey::Diverging => "Divergen",
        ColormapKey::Cyclic => "Siklik",
        ColormapKey::Categorical => "Kategoris",
        ColormapKey::Other => "Lainnya",
        ColormapKey::InterpolationLinear => "Linear",
        ColormapKey::BlendLinearRgb => "RGB Linear",
        ColormapKey::FilterAll => "Semua",
        ColormapKey::FamilyCustom => "Kustom",
        ColormapKey::FamilyTitle => "Keluarga: {}",
        ColormapKey::Colormap => "Peta Warna",
        ColormapKey::Smooth => "Halus",
        ColormapKey::SmoothHover => "Padukan warna palet ini menjadi gradien kontinu (Oklab)",
        ColormapKey::Reversed => "Dibalik",
        ColormapKey::SearchHint => "Cari peta warna...",
        ColormapKey::NoMatches => "Tidak ada peta warna yang cocok",
        ColormapKey::ShowColorbar => "Tampilkan Bilah Warna",
        ColormapKey::BoxTransparency => "Transparansi kotak",
        ColormapKey::CustomColormap => "Peta warna kustom",
        ColormapKey::Name => "Nama",
        ColormapKey::Colors => "Warna",
        ColormapKey::ColorsHover => "Hex, rgb(), hsl() atau nama warna CSS, opsional dengan posisi",
        ColormapKey::Interpolation => "Interpolasi",
        ColormapKey::Blend => "Paduan",
        ColormapKey::Classes => "Kelas",
        ColormapKey::ClassesHover => "0 membuat gradien tetap kontinu",
        ColormapKey::RenameAndApply => "Ganti nama & terapkan",
        ColormapKey::SaveAndApply => "Simpan & terapkan",
        ColormapKey::SaveAsNew => "Simpan sebagai baru",
        ColormapKey::NameTaken => "Peta warna tersimpan sudah memakai nama ini",
        ColormapKey::NameRequired => "Nama wajib diisi",
        ColormapKey::Saved => "Tersimpan",
        ColormapKey::BrokenHover => {
            "Tidak dapat dibangun dengan versi ini: edit untuk memperbaiki, atau hapus"
        }
        ColormapKey::Edit => "Edit",
        ColormapKey::Delete => "Hapus",
        ColormapKey::DeleteHover => "Hapus peta warna",
        ColormapKey::FlipOrientation => "Balik orientasi",
        ColormapKey::RangeMinHover => {
            "Batas bawah rentang (Min). Seret untuk menyesuaikan atau klik untuk mengetik."
        }
        ColormapKey::RangeMaxHover => {
            "Batas atas rentang (Max). Seret untuk menyesuaikan atau klik untuk mengetik."
        }
        ColormapKey::LowClipTitle => "Warna Potong Bawah (< Min)",
        ColormapKey::LowClipHover => "Warna potong bawah (< Min). Klik untuk memilih warna.",
        ColormapKey::HighClipTitle => "Warna Potong Atas (> Max)",
        ColormapKey::HighClipHover => "Warna potong atas (> Max). Klik untuk memilih warna.",
        ColormapKey::HoveredValue => "Nilai: {}",
        ColormapKey::TitleHover => "Judul bilah warna. Klik untuk mengedit.",
        ColormapKey::ColorPicker => "Pemilih Warna",
        ColormapKey::Copy => "Salin",
        ColormapKey::CopyHover => "Salin nilai warna ke papan klip",
        ColormapKey::AlphaNotANumber => "'{}' bukan angka",
        ColormapKey::AlphaOutOfRange => "{} berada di luar 0..1",
        ColormapKey::AlphaMixedForms => "gunakan nilai saja atau titik henti posisi:alpha saja",
        ColormapKey::AlphaUnsorted => "posisi titik henti harus naik",
        ColormapKey::PrefsUnreadable => "Preferensi peta warna tidak dapat dibaca",
        ColormapKey::PrefsBackedUp => "Dicadangkan sebagai `{}`",
        ColormapKey::PrefsNotBackedUp => "Preferensi peta warna tidak dicadangkan",
        ColormapKey::BackupsFull => "Semua {} cadangan preferensi peta warna sudah terpakai",
        ColormapKey::CustomNotLoaded => "Peta warna kustom tidak dimuat",
        ColormapKey::CustomDoesNotBuild => "`{}` tidak dapat dibangun: {}",
        ColormapKey::NotRegistered => "Peta warna {} tidak terdaftar",
        ColormapKey::BundledUnreadable => "Peta warna bawaan tidak dapat dibaca",
        ColormapKey::FallingBackToGray => "{}; beralih ke skala abu-abu",
    })
}
