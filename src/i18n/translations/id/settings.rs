use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "Warna berasal dari kanal komposit.",
        SettingsKey::ReasonCustomColor => "Warna Kustom menggambar semua titik dengan satu warna.",
        SettingsKey::ReasonAllSeries => "Semua Seri Garis mewarnai setiap garis menurut indeksnya.",
        SettingsKey::ReasonIndexed => "RGBA Terindeks mewarnai voxel menurut indeks labelnya.",
        SettingsKey::ReasonIndexedMissing => "RGBA Terindeks melewati voxel yang hilang.",
        SettingsKey::ReasonClassicOpaque => {
            "Hanya Raymarching Volume (DVR) yang menggambar warna tembus pandang."
        }
        SettingsKey::ReasonNotTranslucent => "Berlaku dengan Opasitas di bawah 1 atau Kurva alfa.",
        SettingsKey::ReasonCompositeOpaque => "Komposit digambar buram.",
        SettingsKey::ReasonLabelOpaque => "Permukaan Label Kategoris selalu digambar buram.",
        SettingsKey::ReasonAlwaysBlends => "Algoritma ini selalu memadukan sepanjang sinar.",
        SettingsKey::CloseSettingsHover => "Tutup Pengaturan",
        SettingsKey::SectionOverlays => "Overlay",
        SettingsKey::SectionResolution => "Resolusi",
        SettingsKey::SectionView => "Tampilan",
        SettingsKey::Layers => "Lapisan",
        SettingsKey::BaseLayerHover => "Lapisan dasar: menentukan kanvas, sumbu, dan tampilan",
        SettingsKey::BaseTag => "dasar",
        SettingsKey::Color => "Warna",
        SettingsKey::HideLayerHover => "Sembunyikan lapisan",
        SettingsKey::ShowLayerHover => "Tampilkan lapisan",
        SettingsKey::RemoveLayerHover => "Hapus lapisan",
        SettingsKey::MoveDownHover => "Pindah ke bawah",
        SettingsKey::MoveUpHover => "Pindah ke atas",
        SettingsKey::NoVariable => "(tanpa variabel)",
        SettingsKey::ColorbarLabel => "Label Bilah Warna",
        SettingsKey::ResetLabelHover => "Atur ulang label bilah warna ke bawaan",
        SettingsKey::ColorRange => "Rentang Warna",
        SettingsKey::Categorical => "Kategoris",
        SettingsKey::CategoricalHover => {
            "Bilah warna diskret (mendeteksi otomatis nilai unik atau 10 bin sama besar)."
        }
        SettingsKey::LockBoundsHover => {
            "Kunci min/max agar pemetaan warna tetap sama di semua langkah waktu."
        }
        SettingsKey::ResetBoundsHover => {
            "Atur ulang batas ke nilai min dan max bawaan irisan/dataset saat ini"
        }
        SettingsKey::ResetAllColorbar => "Atur Ulang Semua Bawaan Bilah Warna",
        SettingsKey::ResetAllColorbarHover => {
            "Atur ulang label dan rentang bilah warna ke nilai bawaan"
        }
        SettingsKey::RangeMin => "Min:",
        SettingsKey::RangeMax => "Max:",
        SettingsKey::NanColor => "Warna NaN",
        SettingsKey::NanColorHover => "Jika tidak dicentang, nilai NaN/Inf digambar transparan.",
        SettingsKey::NanColorPickerHover => "Warna NaN. Klik untuk memilih warna.",
        SettingsKey::LowClip => "Klip Bawah",
        SettingsKey::LowClipHover => "Nilai < cmin dipotong ke warna ini.",
        SettingsKey::LowClipPickerHover => "Warna klip bawah (< Min). Klik untuk memilih warna.",
        SettingsKey::HighClip => "Klip Atas",
        SettingsKey::HighClipHover => "Nilai > cmax dipotong ke warna ini.",
        SettingsKey::HighClipPickerHover => "Warna klip atas (> Max). Klik untuk memilih warna.",
        SettingsKey::Scale => "Skala",
        SettingsKey::ScaleLinear => "Linear",
        SettingsKey::ScaleLog => "Logaritmik",
        SettingsKey::ScaleSymlog => "Symlog",
        SettingsKey::ScaleSqrt => "Akar",
        SettingsKey::ScaleExponential => "Eksponensial",
        SettingsKey::ScaleLogHover => "Skala log (data non-negatif)",
        SettingsKey::ScaleLogDisabledHover => {
            "Nonaktif: memerlukan min >= 0. Gunakan Symlog untuk data negatif."
        }
        SettingsKey::ScaleParam => "Param:",
        SettingsKey::Opacity => "Opasitas",
        SettingsKey::OpacityHover => {
            "Opasitas nilai yang dipetakan warna. Warna NaN dan klip tetap memakai alfanya sendiri. Klik ganda untuk mengatur ulang."
        }
        SettingsKey::AlphaCurve => "Kurva alfa",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  atau  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "Alfa di sepanjang bilah warna, dari awal hingga akhirnya. Nilai diberi jarak merata (Tangga: masing-masing satu bin sama besar); titik henti posisi:alpha ditempatkan di sepanjang bilah. Kosongkan untuk tidak ada."
        }
        SettingsKey::ClearCurveHover => "Hapus kurva",
        SettingsKey::Presets => "Preset",
        SettingsKey::InterpStep => "Tangga",
        SettingsKey::InterpLinear => "Linear",
        SettingsKey::PresetRampUp => "Naik",
        SettingsKey::PresetRampDown => "Turun",
        SettingsKey::PresetPeak => "Puncak",
        SettingsKey::PresetSink => "Lembah",
        SettingsKey::PresetHideLow => "Sembunyikan rendah",
        SettingsKey::PresetHideCenter => "Sembunyikan tengah",
        SettingsKey::Transparency => "Transparansi",
        SettingsKey::TransparencyHover => {
            "Dengan warna tembus pandang (Opasitas atau Kurva alfa), gambar setiap lapisan alih-alih membiarkan yang terdekat menyembunyikan yang di belakangnya."
        }
        SettingsKey::Lighting => "Pencahayaan",
        SettingsKey::LightingHover => {
            "Beri bayangan pada sampel menurut gradiennya agar tepi dan muka tampak berbentuk"
        }
        SettingsKey::Coastlines => "Garis Pantai",
        SettingsKey::CoastlinesHover => {
            "Tampilkan garis pantai Natural Earth di atas plot geografis."
        }
        SettingsKey::ResetCoastlineColor => "atur ulang",
        SettingsKey::ResetCoastlineColorHover => "Atur ulang warna garis pantai ke bawaan tema",
        SettingsKey::CoastlineColorHover => "Warna garis pantai. Klik untuk memilih.",
        SettingsKey::CoastlineResolution => "Resolusi:",
        SettingsKey::Lod110mHover => "110 m - cepat, selalu tersedia (tertanam)",
        SettingsKey::Lod50mHover => "50 m - detail sedang (dimuat di latar belakang)",
        SettingsKey::Lod10mHover => "10 m - detail tinggi (dimuat di latar belakang)",
        SettingsKey::LineWidth => "Lebar Garis",
        SettingsKey::CropToDomain => "Potong ke domain data",
        SettingsKey::CropToDomainHover => {
            "Potong garis pantai tepat pada batas spasial dataset aktif."
        }
        SettingsKey::AutoRotate => "Putar Otomatis",
        SettingsKey::ResetView => "Atur Ulang Tampilan",
        SettingsKey::ResetViewHover => "Atur ulang orientasi kamera 3D",
        SettingsKey::HoverCard => "Kartu Info",
        SettingsKey::HideHoverCardHover => "Sembunyikan kartu info",
        SettingsKey::ShowHoverCardHover => "Tampilkan kartu info",
        SettingsKey::Export => "Ekspor",
        SettingsKey::ExportFormat => "Format:",
        SettingsKey::ExportFolder => "Folder:",
        SettingsKey::OpenSaveDialog => "Buka Dialog Simpan",
        SettingsKey::CropTool => "Alat Potong",
        SettingsKey::Aggregation => "Agregasi 2D",
        SettingsKey::AggregationMandatoryHover => {
            "Wajib: larik melebihi batas satu buffer GPU (128 MB / 33,5 juta sel). Agregasi tidak dapat dinonaktifkan."
        }
        SettingsKey::AggregationHover => {
            "Downsampling piramida multi-resolusi dan resampling viewport."
        }
        SettingsKey::AggregationMode => "Mode:",
        SettingsKey::AggMean => "Rata-rata (Filter Kotak)",
        SettingsKey::AggMax => "Maks (Pertahankan Puncak)",
        SettingsKey::AggMin => "Min (Pertahankan Lembah)",
        SettingsKey::AggNearest => "Tetangga Terdekat",
        SettingsKey::AggMeanHover => "Merata-ratakan subpiksel dalam sel 2x2, mengabaikan NaN.",
        SettingsKey::AggMaxHover => "Mempertahankan nilai tinggi ekstrem.",
        SettingsKey::AggMinHover => "Mempertahankan nilai rendah ekstrem.",
        SettingsKey::AggNearestHover => "Pemilihan titik terdekat yang cepat tanpa interpolasi.",
        SettingsKey::CmykAutoMapped => "Kanal dipetakan otomatis: C (1), M (2), Y (3), K (4)",
        SettingsKey::MultiChannelOverlay => "Tumpukan Multi-Kanal",
        SettingsKey::MultiChannelOverlayHover => {
            "Menumpuk beberapa kanal secara aditif, masing-masing dengan warna rona uniknya."
        }
        SettingsKey::CmykComposite => "Komposit CMYK",
        SettingsKey::CmykCompositeHover => {
            "Menggabungkan 4 kanal Sian, Magenta, Kuning, Hitam (CMYK) menjadi RGB warna asli."
        }
        SettingsKey::RgbComposite => "Komposit RGB",
        SettingsKey::RgbCompositeHover => "Menggabungkan 3 kanal terpilih menjadi RGB warna asli.",
        SettingsKey::Channels => "Kanal:",
        SettingsKey::ChannelsAll => "Semua",
        SettingsKey::ChannelsNone => "Tidak ada",
        SettingsKey::ChannelTintHover => "Klik untuk menyesuaikan warna rona kanal",
        SettingsKey::BandNumber => "Band {}",
    })
}
