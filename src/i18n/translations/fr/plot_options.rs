use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "Lignes",
        PlotOptionsKey::LinesHover => "Afficher des lignes continues reliant les points",
        PlotOptionsKey::Scatter => "Points",
        PlotOptionsKey::ScatterHover => "Afficher un marqueur à l'emplacement de chaque point",
        PlotOptionsKey::PointSize => "Taille des points :",
        PlotOptionsKey::CustomColor => "Couleur personnalisée",
        PlotOptionsKey::CustomColorHover => {
            "Utiliser une couleur unie pour les lignes/points au lieu de la palette"
        }
        PlotOptionsKey::LineColorHover => {
            "Couleur des lignes / points. Cliquez pour choisir la couleur."
        }
        PlotOptionsKey::LineProfile => "Profil de ligne",
        PlotOptionsKey::AllLinesSeries => "Toutes les séries",
        PlotOptionsKey::ProfileIndex => "Indice du profil",
        PlotOptionsKey::SingleProfile => "Un seul profil disponible.",
        PlotOptionsKey::AspectRatio => "Rapport d'aspect",
        PlotOptionsKey::AspectRatioHover => {
            "Coché, les graphiques 2D conservent le rapport d'aspect de la matrice (largeur/hauteur). \
             Décoché, les graphiques 2D occupent tout le canevas."
        }
        PlotOptionsKey::Algorithm => "Algorithme",
        PlotOptionsKey::AlgoDvr => "Lancer de rayons volumique (DVR)",
        PlotOptionsKey::AlgoMip => "Intensité maximale (MIP)",
        PlotOptionsKey::AlgoMinIp => "Intensité minimale (MinIP)",
        PlotOptionsKey::AlgoXray => "Projection moyenne (rayons X)",
        PlotOptionsKey::AlgoLabelSurface => "Surface d'étiquettes catégorielles",
        PlotOptionsKey::AlgoAbsorption => "RGBA par absorption",
        PlotOptionsKey::AlgoAdditive => "RGBA additif",
        PlotOptionsKey::AlgoIndexed => "RGBA indexé",
        PlotOptionsKey::Quality => "Qualité",
        PlotOptionsKey::QualityHover => "Échantillons par voxel le long de chaque rayon",
        PlotOptionsKey::Density => "Densité",
        PlotOptionsKey::Attenuation => "Atténuation",
        PlotOptionsKey::ZScale => "Échelle Z",
        PlotOptionsKey::ModeSmooth => "Lisse",
        PlotOptionsKey::ModeBumpy => "Relief",
        PlotOptionsKey::ModeSteps => "Marches",
        PlotOptionsKey::ModeVoxel => "Voxel",
        PlotOptionsKey::Height => "Hauteur",
        PlotOptionsKey::FlipHeights => "Inverser les hauteurs",
        PlotOptionsKey::FlipHeightsHover => {
            "La valeur la plus haute est à plat et les valeurs plus basses s'élèvent au-dessus"
        }
        PlotOptionsKey::Size => "Taille",
    })
}
