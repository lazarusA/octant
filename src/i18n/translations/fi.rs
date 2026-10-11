//! Finnish (Suomi) translations for Octant.

use crate::i18n::keys::Key;

pub fn translate(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::Dataset => "Tietoaineisto",
        Key::Variables => "Muuttujat",
        Key::Dimensions => "Ulottuvuudet",
        Key::Plot => "Kuvaaja",
        Key::Colormaps => "Värikartat",
        Key::Settings => "Asetukset",
        Key::About => "Tietoja",

        Key::InspectMetadata => "Tarkista metatiedot",
        Key::ActiveDatasets => "Aktiiviset tietoaineistot",
        Key::NoActiveDatasets => "Ei aktiivisia aineistoja. Avaa tietoaineisto aloittaaksesi.",
        Key::CacheUsage => "Välimuistin käyttö",
        Key::ClearCache => "Tyhjennä välimuisti",
        Key::OpenLocal => "Avaa paikallinen tiedosto",
        Key::CloudStore => "Pilvitallennus",
        Key::SampleDatasets => "Esimerkkiaineistot",
        Key::DropFilesHere => "Pudota tiedostot tai kansiot tähän",
        Key::SourceUrlOrPath => "Tallennus-URI tai paikallinen polku",
        Key::Inspect => "Tarkista",
        Key::LoadingMetadata => "Tarkistetaan metatietoja taustalla...",

        Key::Group => "Ryhmä:",
        Key::DataType => "Tyyppi",
        Key::Shape => "Muoto",
        Key::Description => "Kuvaus",
        Key::SearchVariables => "Etsi muuttujia...",
        Key::NoVariablesMatching => "Muuttujia ei löytynyt",
        Key::ShowingItemsOfTotal => "Näytetään kohteet kokonaismäärästä",

        Key::Ready => "Valmis",
        Key::Loading => "Ladataan...",
        Key::Close => "Sulje",
        Key::Cancel => "Peruuta",
        Key::Save => "Tallenna",
        Key::Reset => "Palauta",
        Key::Error => "Virhe",

        Key::Play => "Toista",
        Key::Pause => "Tauko",
        Key::StepForward => "Askel eteenpäin",
        Key::StepBackward => "Askel taaksepäin",
        Key::Speed => "Nopeus",
        Key::Loop => "Silmukka",

        Key::AboutSubtitle => "N-Ulotteinen Datan visualisoija",
        Key::AboutDescription => {
            "Octant on interaktiivinen katseluohjelma N-ulotteisille tietoaineistoille, jossa on natiivi tuki paikalliselle ja pilvitallennukselle, Zarr- (v2/v3) ja Icechunk-formaateille. Rakennettu Rustilla GPU-kiihdytetyllä WGPU-renderöinnillä. Toimii macOS-, Linux- ja Windows-käyttöjärjestelmissä."
        }
        Key::AboutHyperslabTitle => "Hyperslab-viipalointi",
        Key::AboutHyperslabDesc => "Asynkroninen LRU-välimuisti ja moniresoluutiopyramidit.",
        Key::AboutStorageTitle => "Natiivi Zarr ja Icechunk",
        Key::AboutStorageDesc => "Paikalliset, S3-, GCS-, Azure- ja HTTP-suoratoistotaustat.",
        Key::AboutShadersTitle => "Laitteistokiihdytetyt WGPU-varjostimet",
        Key::AboutShadersDesc => "2D-kartat, 3D-pallot, korkeusmallit, tilavuudet ja 1D-profiilit.",
        Key::AboutLinksHeading => "Linkit ja resurssit",
        Key::AboutGoodFaithNotice => {
            "Octant tunnustaa kolmansien osapuolten työn vilpittömässä mielessä. Jos jokin puuttuu tai on virheellisesti merkitty, ilmoitathan siitä meille."
        }
        Key::AboutBrowseIcons => "Selaa natiiveja vektori-ikoneita",
    })
}
