use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "Värit tulevat yhdistelmän kanavista.",
        SettingsKey::ReasonCustomColor => {
            "'Mukautettu väri' piirtää kaikki pisteet yhdellä värillä."
        }
        SettingsKey::ReasonAllSeries => {
            "'Kaikki viivasarjat' värittää jokaisen viivan sen indeksin mukaan."
        }
        SettingsKey::ReasonIndexed => {
            "'Indeksoitu RGBA' värittää vokselit niiden tunnisteindeksin mukaan."
        }
        SettingsKey::ReasonIndexedMissing => "'Indeksoitu RGBA' ohittaa puuttuvat vokselit.",
        SettingsKey::ReasonClassicOpaque => {
            "Vain 'Tilavuuden säteenseuranta (DVR)' piirtää läpikuultavia värejä."
        }
        SettingsKey::ReasonNotTranslucent => {
            "Vaikuttaa, kun 'Peittävyys' on alle 1 tai 'Alfakäyrä' on käytössä."
        }
        SettingsKey::ReasonCompositeOpaque => "Yhdistelmät piirretään peittävinä.",
        SettingsKey::ReasonLabelOpaque => "'Kategorinen tunnistepinta' piirretään aina peittävänä.",
        SettingsKey::ReasonAlwaysBlends => "Tämä algoritmi sekoittaa aina säteen suuntaisesti.",
        SettingsKey::CloseSettingsHover => "Sulje asetukset",
        SettingsKey::SectionOverlays => "Päällysteet",
        SettingsKey::SectionResolution => "Resoluutio",
        SettingsKey::SectionView => "Näkymä",
        SettingsKey::Layers => "Tasot",
        SettingsKey::BaseLayerHover => "Perustaso: määrää piirtoalueen, akselit ja näkymän",
        SettingsKey::BaseTag => "perus",
        SettingsKey::Color => "Väri",
        SettingsKey::HideLayerHover => "Piilota taso",
        SettingsKey::ShowLayerHover => "Näytä taso",
        SettingsKey::RemoveLayerHover => "Poista taso",
        SettingsKey::MoveDownHover => "Siirrä alas",
        SettingsKey::MoveUpHover => "Siirrä ylös",
        SettingsKey::NoVariable => "(ei muuttujaa)",
        SettingsKey::ColorbarLabel => "Väripalkin otsikko",
        SettingsKey::ResetLabelHover => "Palauta väripalkin oletusotsikko",
        SettingsKey::ColorRange => "Väriväli",
        SettingsKey::Categorical => "Kategorinen",
        SettingsKey::CategoricalHover => {
            "Diskreetti väripalkki (tunnistaa yksilölliset arvot tai käyttää 10 tasavälistä luokkaa)."
        }
        SettingsKey::LockBoundsHover => {
            "Lukitse min/max, jotta värien kuvaus pysyy samana kaikissa aika-askelissa."
        }
        SettingsKey::ResetBoundsHover => {
            "Palauta rajat nykyisen viipaleen/aineiston min- ja max-oletuksiin"
        }
        SettingsKey::ResetAllColorbar => "Palauta väripalkin oletukset",
        SettingsKey::ResetAllColorbarHover => "Palauta väripalkin otsikko ja väli oletusarvoihin",
        SettingsKey::RangeMin => "Min:",
        SettingsKey::RangeMax => "Max:",
        SettingsKey::NanColor => "NaN-väri",
        SettingsKey::NanColorHover => "Jos ei valittu, NaN/Inf-arvot piirretään läpinäkyvinä.",
        SettingsKey::NanColorPickerHover => "NaN-väri. Valitse väri napsauttamalla.",
        SettingsKey::LowClip => "Alaleikkaus",
        SettingsKey::LowClipHover => "Arvot < cmin leikataan tähän väriin.",
        SettingsKey::LowClipPickerHover => {
            "Alaleikkauksen väri (< Min). Valitse väri napsauttamalla."
        }
        SettingsKey::HighClip => "Yläleikkaus",
        SettingsKey::HighClipHover => "Arvot > cmax leikataan tähän väriin.",
        SettingsKey::HighClipPickerHover => {
            "Yläleikkauksen väri (> Max). Valitse väri napsauttamalla."
        }
        SettingsKey::Scale => "Asteikko",
        SettingsKey::ScaleLinear => "Lineaarinen",
        SettingsKey::ScaleLog => "Logaritminen",
        SettingsKey::ScaleSymlog => "Symlog",
        SettingsKey::ScaleSqrt => "Neliöjuuri",
        SettingsKey::ScaleExponential => "Eksponentiaalinen",
        SettingsKey::ScaleLogHover => "Logaritminen asteikko (ei-negatiivinen data)",
        SettingsKey::ScaleLogDisabledHover => {
            "Pois käytöstä: vaatii min >= 0. Käytä negatiiviselle datalle Symlog-asteikkoa."
        }
        SettingsKey::ScaleParam => "Param.:",
        SettingsKey::Opacity => "Peittävyys",
        SettingsKey::OpacityHover => {
            "Värikartalla väritettyjen arvojen peittävyys. NaN- ja leikkausvärit pitävät oman alfansa. \
             Palauta kaksoisnapsautuksella."
        }
        SettingsKey::AlphaCurve => "Alfakäyrä",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  tai  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "Alfa väripalkin alusta loppuun. Arvot ovat tasavälein (Porras: kullekin \
             yksi tasakokoinen luokka); position:alpha-pisteet sijoitetaan palkin \
             varrelle. Tyhjä tarkoittaa, ettei käyrää ole."
        }
        SettingsKey::ClearCurveHover => "Tyhjennä käyrä",
        SettingsKey::Presets => "Esiasetukset",
        SettingsKey::InterpStep => "Porras",
        SettingsKey::InterpLinear => "Lineaarinen",
        SettingsKey::PresetRampUp => "Nouseva",
        SettingsKey::PresetRampDown => "Laskeva",
        SettingsKey::PresetPeak => "Huippu",
        SettingsKey::PresetSink => "Kuoppa",
        SettingsKey::PresetHideLow => "Piilota matalat",
        SettingsKey::PresetHideCenter => "Piilota keskiosa",
        SettingsKey::Transparency => "Läpinäkyvyys",
        SettingsKey::TransparencyHover => {
            "Läpikuultavilla väreillä ('Peittävyys' tai 'Alfakäyrä') piirrä jokainen kerros \
             sen sijaan, että lähin kerros peittäisi takanaan olevat."
        }
        SettingsKey::Lighting => "Valaistus",
        SettingsKey::LightingHover => {
            "Varjosta näytteet gradientin mukaan, jotta rintamat ja reunat saavat muotoa"
        }
        SettingsKey::Coastlines => "Rantaviivat",
        SettingsKey::CoastlinesHover => {
            "Näytä Natural Earth -rantaviivat maantieteellisten kuvaajien päällä."
        }
        SettingsKey::ResetCoastlineColor => "palauta",
        SettingsKey::ResetCoastlineColorHover => "Palauta rantaviivan väri teeman oletukseen",
        SettingsKey::CoastlineColorHover => "Rantaviivan väri. Valitse napsauttamalla.",
        SettingsKey::CoastlineResolution => "Resoluutio:",
        SettingsKey::Lod110mHover => "110 m - nopea, aina saatavilla (sisäänrakennettu)",
        SettingsKey::Lod50mHover => "50 m - keskitason tarkkuus (ladataan taustalla)",
        SettingsKey::Lod10mHover => "10 m - suuri tarkkuus (ladataan taustalla)",
        SettingsKey::LineWidth => "Viivan leveys",
        SettingsKey::CropToDomain => "Rajaa datan alueeseen",
        SettingsKey::CropToDomainHover => {
            "Rajaa rantaviivat tarkasti aktiivisen aineiston spatiaalisiin rajoihin."
        }
        SettingsKey::AutoRotate => "Automaattinen kierto",
        SettingsKey::ResetView => "Palauta näkymä",
        SettingsKey::ResetViewHover => "Palauta 3D-kameran suunta",
        SettingsKey::HoverCard => "Tietokortti",
        SettingsKey::HideHoverCardHover => "Piilota tietokortti",
        SettingsKey::ShowHoverCardHover => "Näytä tietokortti",
        SettingsKey::Export => "Vienti",
        SettingsKey::ExportFormat => "Muoto:",
        SettingsKey::ExportFolder => "Kansio:",
        SettingsKey::OpenSaveDialog => "Avaa tallennusikkuna",
        SettingsKey::CropTool => "Rajaustyökalu",
        SettingsKey::Aggregation => "2D-aggregointi",
        SettingsKey::AggregationMandatoryHover => {
            "Pakollinen: taulukko ylittää yksittäisen GPU-puskurin rajan (128 MB / 33.5M solua). \
             Aggregointia ei voi poistaa käytöstä."
        }
        SettingsKey::AggregationHover => {
            "Moniresoluutiopyramidin alasnäytteistys ja näkymän uudelleennäytteistys."
        }
        SettingsKey::AggregationMode => "Tila:",
        SettingsKey::AggMean => "Keskiarvo (laatikkosuodin)",
        SettingsKey::AggMax => "Maksimi (säilyttää huiput)",
        SettingsKey::AggMin => "Minimi (säilyttää pohjat)",
        SettingsKey::AggNearest => "Lähin naapuri",
        SettingsKey::AggMeanHover => {
            "Laskee 2x2-solujen alipikselien keskiarvon NaN-arvot ohittaen."
        }
        SettingsKey::AggMaxHover => "Säilyttää äärimmäiset suuret arvot.",
        SettingsKey::AggMinHover => "Säilyttää äärimmäiset pienet arvot.",
        SettingsKey::AggNearestHover => "Nopea lähimmän pisteen valinta ilman interpolointia.",
        SettingsKey::CmykAutoMapped => {
            "Automaattisesti kohdistetut kanavat: C (1), M (2), Y (3), K (4)"
        }
        SettingsKey::MultiChannelOverlay => "Monikanavapäällyste",
        SettingsKey::MultiChannelOverlayHover => {
            "Yhdistää useita kanavia additiivisesti, kukin omalla värisävyllään."
        }
        SettingsKey::CmykComposite => "CMYK-yhdistelmä",
        SettingsKey::CmykCompositeHover => {
            "Yhdistää 4 kanavaa, syaanin, magentan, keltaisen ja mustan (CMYK), luonnollisiksi RGB-väreiksi."
        }
        SettingsKey::RgbComposite => "RGB-yhdistelmä",
        SettingsKey::RgbCompositeHover => "Yhdistää valitut 3 kanavaa luonnollisiksi RGB-väreiksi.",
        SettingsKey::Channels => "Kanavat:",
        SettingsKey::ChannelsAll => "Kaikki",
        SettingsKey::ChannelsNone => "Ei mitään",
        SettingsKey::ChannelTintHover => "Mukauta kanavan sävyä napsauttamalla",
        SettingsKey::BandNumber => "Kaista {}",
    })
}
