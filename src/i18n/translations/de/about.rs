use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "Über Octant",
        AboutKey::IconsWindowTitle => "Native Vektorsymbole",
        AboutKey::IconsTab => "Vektorsymbole",
        AboutKey::Subtitle => "N-dimensionaler Daten-Explorer",
        AboutKey::Description => {
            "Octant ist ein interaktiver Betrachter für N-dimensionale Datensätze mit nativer Unterstützung für lokalen und Cloud-Objektspeicher, Zarr (v2/v3) und Icechunk. Entwickelt in Rust mit GPU-beschleunigtem Rendering über WGPU. Octant läuft nativ auf macOS, Linux und Windows."
        }
        AboutKey::HyperslabTitle => "Hyperslab-Slicing",
        AboutKey::HyperslabDesc => "Asynchroner LRU-Chunk-Cache und Multi-Resolution-Pyramiden.",
        AboutKey::StorageTitle => "Nativ für Zarr & Icechunk",
        AboutKey::StorageDesc => "Lokale, S3-, GCS-, Azure- und HTTP-Streaming-Backends.",
        AboutKey::ShadersTitle => "Hardware-beschleunigte WGPU-Shader",
        AboutKey::ShadersDesc => "2D-Karten, 3D-Kugeln, Höhenflächen, Volumina & 1D-Profile.",
        AboutKey::LinksHeading => "Links & Ressourcen",
        AboutKey::DocsLink => "Octant-Dokumentation",
        AboutKey::GoodFaithNotice => {
            "Octant würdigt Werke Dritter nach bestem Wissen. Sollte etwas fehlen oder falsch zugeordnet sein, teilen Sie es uns bitte mit, damit wir es korrigieren oder entfernen."
        }
        AboutKey::BrowseIcons => "Native Vektorsymbole durchsuchen",
        AboutKey::BrowseIconsHover => "Prozedurale Vektorsymbole in einem Fenster erkunden",
        AboutKey::Licensed => "Lizenziert unter MIT oder Apache-2.0",
        AboutKey::OctantTeaser => {
            "Die Vorzeichen von x, y und z (+/-, +/-, +/-) teilen den Raum in acht Oktanten. Erkunden Sie Ihre Daten auf dieselbe Weise: Achsen wählen, eine Scheibe schneiden und eine Region nach der anderen betrachten."
        }
        AboutKey::OctantHover => "Klicken, um zum nächsten Oktanten zu wechseln",
        AboutKey::ColormapCredits => "Farbtabellen-Nachweise",
        AboutKey::ColormapCreditsNote => {
            "Mitgelieferte Farbtabellen behalten die Lizenzen ihrer Urheber."
        }
        AboutKey::LicenseTexts => "Lizenztexte",
        AboutKey::ThirdPartyLicenses => "Lizenzen Dritter",
        AboutKey::ThirdPartyNote => {
            "Octant enthält Open-Source-Crates, Schriften und C-Bibliotheken (netCDF, HDF5), jeweils unter eigener Lizenz."
        }
        AboutKey::ViewOnGithub => "Auf GitHub ansehen",
        AboutKey::IconsSearchHint => "Symbole suchen...",
        AboutKey::IconsAll => "Alle",
        AboutKey::IconsCopied => "{} kopiert",
        AboutKey::IconsNoMatch => "Keine passenden Vektorsymbole gefunden.",
        AboutKey::IconsCategory => "Kategorie: {}",
        AboutKey::IconsClickToCopy => "Klicken, um den Enum-Bezeichner zu kopieren",
        AboutKey::IconsSizesTones => "Größen & Töne",
        AboutKey::ClearSearchHover => "Suche leeren",
    })
}
