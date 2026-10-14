use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "Aktiivinen aineisto",
        DatasetKey::NoDatasetLoaded => "Aineistoa ei ole ladattu",
        DatasetKey::MemoryCache => "Muisti ja välimuisti",
        DatasetKey::SampleDatasets => "Esimerkit",
        DatasetKey::DropFilesHere => "Vedä ja pudota tiedosto tai kansio",
        DatasetKey::SourceUrlOrPath => "Kohde-URL / tiedostopolku",
        DatasetKey::LoadingMetadata => "Tarkastetaan säilön metatietoja taustalla...",
        DatasetKey::OpenSource => "Avaa lähde",
        DatasetKey::DatasetManager => "Aineistojen hallinta",
        DatasetKey::CloseDatasetHover => "Sulje aineisto",
        DatasetKey::TargetUriHover => "Kohde-URI:\n{}",
        DatasetKey::VariableCountOne => "{} muuttuja",
        DatasetKey::VariableCountMany => "{} muuttujaa",
        DatasetKey::InspectVariables => "Tarkastele muuttujia",
        DatasetKey::Reload => "Lataa uudelleen",
        DatasetKey::BadgeVolume4D => "4D-tilavuus",
        DatasetKey::BadgeMatrix2D => "2D-matriisi",
        DatasetKey::BadgeStore => "Säilö",
        DatasetKey::Load => "Lataa",
        DatasetKey::TargetPathHint => "https://... tai polku (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "Luettelo...",
        DatasetKey::Clear => "Tyhjennä",
        DatasetKey::FormatLabel => "Muoto:",
        DatasetKey::SampleSeasFire => "SeasFire, maailman maastopalot (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "2D proseduraalinen matriisi (testi)",
        DatasetKey::FormatRemoteZarr => "Etä-Zarr (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "Paikallinen Zarr (tiedostojärjestelmä)",
        DatasetKey::FormatRemoteIcechunk => "Etä-Icechunk (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "Paikallinen Icechunk (tiedostojärjestelmä)",
        DatasetKey::FormatRemoteGeoTiff => "Etä-GeoTIFF / COG (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "Paikallinen GeoTIFF (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "Paikallinen NetCDF / HDF5 (.nc/.h5)",
        DatasetKey::FormatVolume4D => "4D-vertailutilavuus (proseduraalinen)",
        DatasetKey::FormatMatrix2D => "2D-matriisi (proseduraalinen)",
        DatasetKey::FormatRemoteGeoTiffShort => "Etä-GeoTIFF / COG",
        DatasetKey::FormatLocalGeoTiffShort => "Paikallinen GeoTIFF",
        DatasetKey::FormatLocalNetCdfShort => "Paikallinen NetCDF / HDF5",
        DatasetKey::FormatVolume4DShort => "4D-vertailutilavuus",
        DatasetKey::FormatMatrix2DShort => "2D proseduraalinen matriisi",
        DatasetKey::NoDatasetsRegistered => "Hallinnassa ei ole rekisteröityjä aineistoja.",
        DatasetKey::LoadedCount => "Ladattu ({}):",
        DatasetKey::ClearAll => "Tyhjennä kaikki",
        DatasetKey::ClearAllHover => "Poista kaikki aineistot hallinnasta",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "Poista tämä aineisto",
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "{} lohkoa | osumaprosentti {}%",
        DatasetKey::CacheMenuTitle => "Yhteinen monimuuttujainen muistivälimuisti",
        DatasetKey::GlobalMemoryUsage => "RAM-muistin kokonaiskäyttö:",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "Muistinkäyttö lajeittain:",
        DatasetKey::ResidentHyperslabs => "Muistissa olevat hyperslabit: {} MB ({} lohkoa)",
        DatasetKey::MultiResPyramid => "Moniresoluutiopyramidi: {} MB",
        DatasetKey::CacheHitsMisses => "Osumat: {} | Hudit: {} (osumaprosentti: {}%)",
        DatasetKey::PrefetchInFlight => "Taustaesihaku: {} käynnissä",
        DatasetKey::AllBlocksCached => "Puskuri lämmin / kaikki lohkot välimuistissa",
        DatasetKey::SummaryPerVariable => "Yhteenveto muuttujittain:",
        DatasetKey::NoVariablesCached => "(Ei muuttujia välimuistissa)",
        DatasetKey::VariableCacheSize => "({} MB, {} lohk.)",
        DatasetKey::ClearVariableCacheHover => "Tyhjennä tämän muuttujan välimuisti",
        DatasetKey::PrefetchCapacitySettings => "Esihaku- ja kapasiteettiasetukset",
        DatasetKey::EnablePrefetching => "Ota taustaesihaku käyttöön",
        DatasetKey::CacheLimitSuffix => " MB raja",
        DatasetKey::WindowSizeSuffix => " ikkunan koko (askelta)",
        DatasetKey::PrefetchWorkersSuffix => " esihakusäiettä",
        DatasetKey::FlushAllCaches => "Tyhjennä kaikki välimuistit",
        DatasetKey::CatalogTitle => "Aineistoluettelo",
        DatasetKey::CatalogStoreCount => "{} säilöä",
        DatasetKey::CatalogBlurb => {
            "Valikoituja Zarr- ja Icechunk-pilviaineistoja sekä proseduraalisia vertailuaineistoja. Valitse aineisto ladataksesi sen."
        }
        DatasetKey::CatalogSearchHint => "Suodata nimen, kuvauksen tai URL:n mukaan...",
        DatasetKey::CatalogCategory => "Luokka:",
        DatasetKey::CategoryAll => "Kaikki",
        DatasetKey::CategoryProcedural => "Proseduraalinen",
        DatasetKey::NoCatalogMatches => "Hakuasi vastaavia aineistoja ei löytynyt.",
        DatasetKey::ClearSearch => "Tyhjennä haku",
        DatasetKey::SelectAndLoadDataset => "Valitse ja lataa aineisto",
        DatasetKey::SelectAndLoad => "Valitse ja lataa",
        DatasetKey::DropTypeNotSupported => "Tyyppiä ei tueta",
        DatasetKey::DropReleaseToLoad => "Vapauta ladataksesi aineiston",
        DatasetKey::DropSupportedList => "Tuetut: {}",
        DatasetKey::HeroTryPrefix => "kokeile:",
        DatasetKey::HeroSeasFireHover => {
            "Maailman maastopalo- ja ilmastoaineisto, uudelleen lohkottu (Zarr)"
        }
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A, luonnolliset värit, COG (AWS S3)",
        DatasetKey::HeroVolume4DHover => "Synteettinen 4D-aika-avaruustilavuus",
        DatasetKey::HeroCatalogHover => "Selaa aineistoluetteloa",
        DatasetKey::LoadSampleHover => "Lataa esimerkki: {}",
        DatasetKey::HeroHintLong => {
            "URL / paikallinen polku / vedä ja pudota tiedostoja mihin tahansa"
        }
        DatasetKey::HeroHintMedium => "URL / polku / vedä ja pudota tiedostoja",
        DatasetKey::HeroHintShort => "URL / pudota tiedostoja",
        DatasetKey::HeroDropLong => {
            "Pudota aineisto ladataksesi (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroDropMedium => "Pudota aineisto (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "Pudota aineisto ladataksesi",
        DatasetKey::HeroUnsupportedLong => {
            "Tyyppiä ei tueta. Tuetut: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => "Muotoa ei tueta (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroUnsupportedShort => "Muotoa ei tueta",
        DatasetKey::HeroInputHintLong => {
            "https://... tai polku (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... tai polku (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL tai polku...",
        DatasetKey::ClearInputHover => "Tyhjennä syöte",
        DatasetKey::LoadDatasetHover => "Lataa aineisto ja avaa muuttujat",
        DatasetKey::HopOctantHover => "Napsauta hypätäksesi oktanttiin",
        DatasetKey::HeroLoadingSource => "ladataan: {}",
        DatasetKey::HeroLoading => "ladataan...",
        DatasetKey::HeroLoaded => "ladattu: {}",
        DatasetKey::BrowserLocalPaths => {
            "Paikallisia tiedostopolkuja ei voi lukea suoraan selaimessa verkon hiekkalaatikon tietoturvan vuoksi.\n\nPaikallisten aineistojen katselu selaimessa:\n1. Jaa kansio tai tiedosto paikallisella HTTP-palvelimella: `npx serve` tai `python3 -m http.server`\n2. Syötä URL: `http://localhost:8000/my_dataset`\n\nTai käytä Octantin natiivia työpöytäversiota (`cargo run --release`)."
        }
        DatasetKey::Plate => "Kuoppalevy",
        DatasetKey::PlateAcquisition => "Kuvauskerta",
        DatasetKey::PlateAllAcquisitions => "Kaikki",
        DatasetKey::PlateFields => "Kentät:",
        DatasetKey::PlatePickWell => "Valitse kuoppa nähdäksesi sen kentät",
        DatasetKey::PlateLoadingFields => "Luetaan kuopan kenttiä",
        DatasetKey::PlateNoFields => "Tällä kuvauskerralla ei ole kenttiä",
        DatasetKey::PlateWellUnreadable => {
            "Kuopan kenttiä ei voitu lukea. Yritä uudelleen napsauttamalla"
        }
        DatasetKey::CategoryMicroscopy => "Mikroskopia",
    })
}
