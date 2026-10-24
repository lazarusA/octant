//! Indonesian phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Goto, &["pergi ke", "lompat ke", "atur"]),
        (Open, &["buka", "membuka", "muat", "beralih ke"]),
        (Select, &["pilih", "memilih"]),
        (
            Plot,
            &[
                "tampilkan",
                "plot",
                "gambar",
                "gambarkan",
                "visualisasikan",
                "perlihatkan",
            ],
        ),
        (
            Play,
            &["putar", "mainkan", "mulai", "jalankan", "animasikan"],
        ),
        (Pause, &["jeda", "hentikan", "berhenti"]),
        (
            Next,
            &["berikutnya", "selanjutnya", "maju", "langkah berikutnya"],
        ),
        (
            Previous,
            &["sebelumnya", "mundur", "kembali", "langkah sebelumnya"],
        ),
        (Colormap, &["peta warna", "skala warna", "palet", "warna"]),
        (
            ResetView,
            &[
                "atur ulang tampilan",
                "reset tampilan",
                "kembalikan tampilan",
            ],
        ),
    ],
    months: [
        &["januari"],
        &["februari"],
        &["maret"],
        &["april"],
        &["mei"],
        &["juni"],
        &["juli"],
        &["agustus"],
        &["september"],
        &["oktober"],
        &["november"],
        &["desember"],
    ],
    axes: [
        &["waktu", "tanggal"],
        &["lintang"],
        &["bujur"],
        &["tingkat", "kedalaman", "ketinggian"],
    ],
    catalog: &["katalog"],
    then: &["lalu", "kemudian", "setelah itu"],
};
