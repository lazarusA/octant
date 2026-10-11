use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "Tietoja Octantista",
        AboutKey::IconsWindowTitle => "Sisäänrakennetut vektorikuvakkeet",
        AboutKey::IconsTab => "Vektorikuvakkeet",
        AboutKey::Subtitle => "N-ulotteisen datan tutkija",
        AboutKey::Description => {
            "Octant on interaktiivinen katseluohjelma N-ulotteisille tietoaineistoille, jossa on natiivi tuki paikalliselle ja pilviobjektitallennukselle sekä Zarr- (v2/v3) ja Icechunk-formaateille. Rakennettu Rustilla GPU-kiihdytetyllä WGPU-renderöinnillä. Octant toimii natiivisti macOS-, Linux- ja Windows-käyttöjärjestelmissä."
        }
        AboutKey::HyperslabTitle => "Hyperslab-viipalointi",
        AboutKey::HyperslabDesc => "Asynkroninen LRU-välimuisti ja moniresoluutiopyramidit.",
        AboutKey::StorageTitle => "Natiivi Zarr ja Icechunk",
        AboutKey::StorageDesc => "Paikalliset, S3-, GCS-, Azure- ja HTTP-suoratoistotaustat.",
        AboutKey::ShadersTitle => "Laitteistokiihdytetyt WGPU-varjostimet",
        AboutKey::ShadersDesc => "2D-kartat, 3D-pallot, korkeuspinnat, tilavuudet ja 1D-profiilit.",
        AboutKey::LinksHeading => "Linkit ja resurssit",
        AboutKey::DocsLink => "Octantin dokumentaatio",
        AboutKey::GoodFaithNotice => {
            "Octant tunnustaa kolmansien osapuolten työn vilpittömässä mielessä. Jos jokin mielestäsi puuttuu tai on virheellisesti merkitty, ilmoitathan siitä meille, niin korjaamme tai poistamme sen."
        }
        AboutKey::BrowseIcons => "Selaa vektorikuvakkeita",
        AboutKey::BrowseIconsHover => "Selaa proseduraalisia vektorikuvakkeita ikkunassa",
        AboutKey::Licensed => "Lisensoitu MIT- tai Apache-2.0-lisenssillä",
        AboutKey::OctantTeaser => {
            "x:n, y:n ja z:n etumerkit (+/-, +/-, +/-) jakavat avaruuden kahdeksaan oktanttiin. Tutki dataasi samalla tavalla: valitse akselit, leikkaa viipale ja tarkastele yhtä aluetta kerrallaan."
        }
        AboutKey::OctantHover => "Napsauta siirtyäksesi seuraavaan oktanttiin",
        AboutKey::ColormapCredits => "Värikarttojen tekijät",
        AboutKey::ColormapCreditsNote => {
            "Mukana tulevat värikartat säilyttävät tekijöidensä lisenssit."
        }
        AboutKey::LicenseTexts => "Lisenssitekstit",
        AboutKey::ThirdPartyLicenses => "Kolmansien osapuolten lisenssit",
        AboutKey::ThirdPartyNote => {
            "Octant sisältää avoimen lähdekoodin crateja, fontteja ja C-kirjastoja (netCDF, HDF5), kukin omalla lisenssillään."
        }
        AboutKey::ViewOnGithub => "Näytä GitHubissa",
        AboutKey::IconsSearchHint => "Etsi kuvakkeita...",
        AboutKey::IconsAll => "Kaikki",
        AboutKey::IconsCopied => "Kopioitu {}",
        AboutKey::IconsNoMatch => "Vastaavia vektorikuvakkeita ei löytynyt.",
        AboutKey::IconsCategory => "Luokka: {}",
        AboutKey::IconsClickToCopy => "Kopioi enum-tunniste napsauttamalla",
        AboutKey::IconsSizesTones => "Koot ja sävyt",
        AboutKey::ClearSearchHover => "Tyhjennä haku",
        AboutKey::FontLicensesNote => {
            "Noto Sans SC, JP ja KR (kiinan-, japanin- ja koreankielinen teksti):"
        }
    })
}
