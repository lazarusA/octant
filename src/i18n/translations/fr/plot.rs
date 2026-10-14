use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "Carte de chaleur",
        PlotKey::Line => "Ligne 1D",
        PlotKey::Surface => "Surface",
        PlotKey::Volume => "Volume",
        PlotKey::Sphere => "Sphère",
        PlotKey::PointCloud => "Nuage de points",
        PlotKey::CompositeTrueColor => "Couleurs vraies",
        PlotKey::CompositeFalseColor => "Fausses couleurs",
        PlotKey::CompositeRgb => "Composition RGB",
        PlotKey::CompositeOverlay => "Superposition de canaux",
        PlotKey::AlignGeo => "Sur une autre grille longitude/latitude (pas encore prise en charge)",
        PlotKey::AlignIndexOnly => {
            "Même forme de grille, mais aucune coordonnée ne correspond au graphique"
        }
        PlotKey::AlignIncompatible => "Ne partage aucune grille avec le graphique",
        PlotKey::OverlayPlotFirst => "Tracez d'abord une variable",
        PlotKey::OverlayHeatmapOnly => "Les superpositions se dessinent sur les cartes de chaleur",
        PlotKey::OverlayStackFull => "4 superpositions au maximum",
        PlotKey::OverlayAlreadyLayer => "Déjà une couche du graphique",
        PlotKey::OverlayCantAdd => "Impossible d'ajouter la superposition",
        PlotKey::OverlayNamedReason => "'{}' : {}",
        PlotKey::SelectProjection => "Choisir la projection de visualisation",
        PlotKey::GridHealpixDiscrete => "HEALPix / discret",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "Non pris en charge pour la grille {}",
        PlotKey::DisabledAggregation => "Désactivé : l'agrégation 2D est active",
        PlotKey::Requires3D => "Nécessite des données 3D",
        PlotKey::DisabledGpuLimit => "Désactivé : {} MB > limite GPU de 128 MB",
        PlotKey::ScalarField => "Champ scalaire",
        PlotKey::LabelsHeatmapOnly => {
            "Les images d'étiquettes ne s'affichent que sur la carte de chaleur"
        }
    })
}
