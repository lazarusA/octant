use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "Les couleurs proviennent des canaux de la composition.",
        SettingsKey::ReasonCustomColor => {
            "Couleur personnalisée dessine chaque point d'une seule couleur."
        }
        SettingsKey::ReasonAllSeries => "Toutes les séries colore chaque ligne selon son indice.",
        SettingsKey::ReasonIndexed => {
            "RGBA indexé colore les voxels selon leur indice d'étiquette."
        }
        SettingsKey::ReasonIndexedMissing => "RGBA indexé ignore les voxels manquants.",
        SettingsKey::ReasonClassicOpaque => {
            "Seul Lancer de rayons volumique (DVR) dessine des couleurs translucides."
        }
        SettingsKey::ReasonNotTranslucent => {
            "S'applique avec une Opacité inférieure à 1 ou une Courbe alpha."
        }
        SettingsKey::ReasonCompositeOpaque => "Les compositions sont dessinées opaques.",
        SettingsKey::ReasonLabelOpaque => {
            "Surface d'étiquettes catégorielles est toujours dessinée opaque."
        }
        SettingsKey::ReasonAlwaysBlends => "Cet algorithme mélange toujours le long du rayon.",
        SettingsKey::CloseSettingsHover => "Fermer les paramètres",
        SettingsKey::SectionOverlays => "Superpositions",
        SettingsKey::SectionResolution => "Résolution",
        SettingsKey::SectionView => "Vue",
        SettingsKey::Layers => "Couches",
        SettingsKey::BaseLayerHover => "Couche de base : définit le canevas, les axes et la vue",
        SettingsKey::BaseTag => "base",
        SettingsKey::Color => "Couleur",
        SettingsKey::HideLayerHover => "Masquer la couche",
        SettingsKey::ShowLayerHover => "Afficher la couche",
        SettingsKey::RemoveLayerHover => "Supprimer la couche",
        SettingsKey::MoveDownHover => "Descendre",
        SettingsKey::MoveUpHover => "Monter",
        SettingsKey::NoVariable => "(aucune variable)",
        SettingsKey::ColorbarLabel => "Titre de la barre",
        SettingsKey::ResetLabelHover => "Rétablir le titre par défaut de la barre de couleurs",
        SettingsKey::ColorRange => "Plage de couleurs",
        SettingsKey::Categorical => "Catégorielle",
        SettingsKey::CategoricalHover => {
            "Barre de couleurs discrète (détecte les valeurs uniques ou 10 classes égales)."
        }
        SettingsKey::LockBoundsHover => {
            "Verrouiller min/max pour garder la correspondance des couleurs entre les pas de temps."
        }
        SettingsKey::ResetBoundsHover => {
            "Rétablir les bornes au min et max par défaut de la tranche ou du jeu de données"
        }
        SettingsKey::ResetAllColorbar => "Tout réinitialiser",
        SettingsKey::ResetAllColorbarHover => {
            "Rétablir le titre et la plage de la barre de couleurs par défaut"
        }
        SettingsKey::RangeMin => "Min :",
        SettingsKey::RangeMax => "Max :",
        SettingsKey::NanColor => "Couleur NaN",
        SettingsKey::NanColorHover => "Décoché, les valeurs NaN/Inf sont transparentes.",
        SettingsKey::NanColorPickerHover => "Couleur NaN. Cliquez pour choisir la couleur.",
        SettingsKey::LowClip => "Sous la plage",
        SettingsKey::LowClipHover => "Les valeurs < cmin prennent cette couleur.",
        SettingsKey::LowClipPickerHover => {
            "Couleur sous la plage (< Min). Cliquez pour choisir la couleur."
        }
        SettingsKey::HighClip => "Au-dessus de la plage",
        SettingsKey::HighClipHover => "Les valeurs > cmax prennent cette couleur.",
        SettingsKey::HighClipPickerHover => {
            "Couleur au-dessus de la plage (> Max). Cliquez pour choisir la couleur."
        }
        SettingsKey::Scale => "Échelle",
        SettingsKey::ScaleLinear => "Linéaire",
        SettingsKey::ScaleLog => "Logarithmique",
        SettingsKey::ScaleSymlog => "Symlog",
        SettingsKey::ScaleSqrt => "Racine",
        SettingsKey::ScaleExponential => "Exponentielle",
        SettingsKey::ScaleLogHover => "Échelle logarithmique (données non négatives)",
        SettingsKey::ScaleLogDisabledHover => {
            "Désactivé : nécessite min >= 0. Utilisez Symlog pour les données négatives."
        }
        SettingsKey::ScaleParam => "Param. :",
        SettingsKey::Opacity => "Opacité",
        SettingsKey::OpacityHover => {
            "Opacité des valeurs colorées par la palette. Les couleurs NaN et hors plage gardent leur propre alpha. \
             Double-cliquez pour réinitialiser."
        }
        SettingsKey::AlphaCurve => "Courbe alpha",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  ou  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "Alpha le long de la barre de couleurs, du début à la fin. Les valeurs sont \
             réparties uniformément (Palier : une classe égale chacune) ; les arrêts \
             position:alpha sont placés le long de la barre. Vide pour aucune."
        }
        SettingsKey::ClearCurveHover => "Effacer la courbe",
        SettingsKey::Presets => "Préréglages",
        SettingsKey::InterpStep => "Palier",
        SettingsKey::InterpLinear => "Linéaire",
        SettingsKey::PresetRampUp => "Rampe montante",
        SettingsKey::PresetRampDown => "Rampe descendante",
        SettingsKey::PresetPeak => "Pic",
        SettingsKey::PresetSink => "Creux",
        SettingsKey::PresetHideLow => "Masquer le bas",
        SettingsKey::PresetHideCenter => "Masquer le centre",
        SettingsKey::Transparency => "Transparence",
        SettingsKey::TransparencyHover => {
            "Avec des couleurs translucides (Opacité ou Courbe alpha), dessiner toutes les couches \
             au lieu de laisser la plus proche masquer celles situées derrière."
        }
        SettingsKey::Lighting => "Éclairage",
        SettingsKey::LightingHover => {
            "Ombrer les échantillons selon leur gradient pour donner du relief aux fronts et aux bords"
        }
        SettingsKey::Coastlines => "Côtes",
        SettingsKey::CoastlinesHover => {
            "Superposer les côtes Natural Earth sur les graphiques géographiques."
        }
        SettingsKey::ResetCoastlineColor => "rétablir",
        SettingsKey::ResetCoastlineColorHover => {
            "Rétablir la couleur des côtes par défaut du thème"
        }
        SettingsKey::CoastlineColorHover => "Couleur des côtes. Cliquez pour choisir.",
        SettingsKey::CoastlineResolution => "Résolution :",
        SettingsKey::Lod110mHover => "110 m : rapide, toujours disponible (intégré)",
        SettingsKey::Lod50mHover => "50 m : détail moyen (chargé en arrière-plan)",
        SettingsKey::Lod10mHover => "10 m : détail élevé (chargé en arrière-plan)",
        SettingsKey::LineWidth => "Épaisseur",
        SettingsKey::CropToDomain => "Limiter au domaine des données",
        SettingsKey::CropToDomainHover => {
            "Découper les côtes strictement à l'emprise spatiale du jeu de données actif."
        }
        SettingsKey::AutoRotate => "Rotation auto",
        SettingsKey::ResetView => "Réinitialiser la vue",
        SettingsKey::ResetViewHover => "Réinitialiser l'orientation de la caméra 3D",
        SettingsKey::HoverCard => "Info-bulle",
        SettingsKey::HideHoverCardHover => "Masquer l'info-bulle",
        SettingsKey::ShowHoverCardHover => "Afficher l'info-bulle",
        SettingsKey::Export => "Export",
        SettingsKey::ExportFormat => "Format :",
        SettingsKey::ExportFolder => "Dossier :",
        SettingsKey::OpenSaveDialog => "Ouvrir la boîte d'enregistrement",
        SettingsKey::CropTool => "Outil de recadrage",
        SettingsKey::Aggregation => "Agrégation 2D",
        SettingsKey::AggregationMandatoryHover => {
            "Obligatoire : le tableau dépasse la limite d'un tampon GPU ({} MB). \
             L'agrégation ne peut pas être désactivée."
        }
        SettingsKey::AggregationHover => {
            "Sous-échantillonnage en pyramide multi-résolutions et rééchantillonnage de la vue."
        }
        SettingsKey::AggregationMode => "Mode :",
        SettingsKey::AggMean => "Moyenne (filtre boîte)",
        SettingsKey::AggMax => "Max (préserve les pics)",
        SettingsKey::AggMin => "Min (préserve les creux)",
        SettingsKey::AggNearest => "Plus proche voisin",
        SettingsKey::AggMeanHover => {
            "Moyenne des sous-pixels par cellules 2x2, en ignorant les NaN."
        }
        SettingsKey::AggMaxHover => "Préserve les valeurs extrêmes hautes.",
        SettingsKey::AggMinHover => "Préserve les valeurs extrêmes basses.",
        SettingsKey::AggNearestHover => {
            "Sélection rapide du point le plus proche, sans interpolation."
        }
        SettingsKey::ResolutionLevel => "Niveau de résolution",
        SettingsKey::ResolutionLevelHover => {
            "Le niveau des données multi-échelles que lit le graphique. Auto lit le niveau le plus fin qui tient dans le canevas."
        }
        SettingsKey::CmykAutoMapped => {
            "Canaux attribués automatiquement : C (1), M (2), Y (3), K (4)"
        }
        SettingsKey::MultiChannelOverlay => "Superposition multicanal",
        SettingsKey::MultiChannelOverlayHover => {
            "Superpose plusieurs canaux par addition, chacun rendu avec sa propre teinte."
        }
        SettingsKey::CmykComposite => "Composition CMYK",
        SettingsKey::CmykCompositeHover => {
            "Combine 4 canaux cyan, magenta, jaune, noir (CMYK) en RGB couleurs vraies."
        }
        SettingsKey::RgbComposite => "Composition RGB",
        SettingsKey::RgbCompositeHover => "Combine les 3 canaux choisis en RGB couleurs vraies.",
        SettingsKey::Channels => "Canaux :",
        SettingsKey::ChannelsAll => "Tous",
        SettingsKey::ChannelsNone => "Aucun",
        SettingsKey::ChannelTintHover => "Cliquez pour personnaliser la teinte du canal",
        SettingsKey::BandNumber => "Bande {}",
        SettingsKey::ScaleBar => "Barre d'échelle",
        SettingsKey::ShowScaleBarHover => "Afficher la barre d'échelle",
        SettingsKey::HideScaleBarHover => "Masquer la barre d'échelle",
        SettingsKey::ChannelsAuto => "Auto",
        SettingsKey::ChannelsAutoHover => {
            "Ajuster la plage d'affichage de chaque canal aux données affichées (de 0,1 à 99,9 pour cent de ses valeurs)"
        }
        SettingsKey::ChannelsAutoUnavailable => {
            "Le contraste automatique fonctionne sur une coupe 2D"
        }
        SettingsKey::ChannelSettingsHover => "Plage d'affichage, courbe et inversion",
        SettingsKey::ChannelWindow => "Plage d'affichage",
        SettingsKey::ChannelWindowHover => {
            "Les valeurs du début à la fin vont du noir à la pleine couleur du canal"
        }
        SettingsKey::ChannelInvert => "Inverser",
        SettingsKey::ChannelCurve => "Courbe",
        SettingsKey::CurveLinear => "Linéaire",
        SettingsKey::CurveLogarithmic => "Logarithmique",
        SettingsKey::CurveExponential => "Exponentielle",
        SettingsKey::CurvePolynomial => "Polynomiale",
        SettingsKey::ChannelCoefficientHover => "Coefficient de la courbe",
        SettingsKey::LabelsFill => "Remplissage",
        SettingsKey::LabelsFillHover => "Remplir chaque étiquette en entier",
        SettingsKey::LabelsOutline => "Contour",
        SettingsKey::LabelsOutlineHover => "Dessiner seulement les bords entre étiquettes",
    })
}
