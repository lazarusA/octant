use crate::i18n::keys::ExportKey;

pub fn text(key: ExportKey) -> Option<&'static str> {
    Some(match key {
        ExportKey::Png => "PNG (sans perte)",
        ExportKey::Jpeg => "JPEG (avec perte)",
        ExportKey::Webp => "WebP (moderne)",
        ExportKey::Svg => "SVG (vectoriel / hybride)",
        ExportKey::Pdf => "PDF (publication)",
        ExportKey::AspectFreeform => "Libre",
        ExportKey::Aspect16x9 => "16:9 (diapositives)",
        ExportKey::Aspect4x3 => "4:3 (figure)",
        ExportKey::Aspect1x1 => "1:1 (carré)",
        ExportKey::DialogTitle => "Enregistrer et exporter la figure",
        ExportKey::FormatHeading => "Format d'export",
        ExportKey::JpegQuality => "Qualité JPEG :",
        ExportKey::AreaHeading => "Zone d'export",
        ExportKey::FullCanvas => "Canevas entier (avec superpositions)",
        ExportKey::FramedRoi => "ROI cadrée",
        ExportKey::HideGuides => "Masquer les lignes de recadrage",
        ExportKey::ShowGuides => "Afficher les lignes de recadrage sur le canevas",
        ExportKey::DestinationHeading => "Destination et nom de fichier",
        ExportKey::Folder => "Dossier :",
        ExportKey::Preview => "Aperçu : {}/{}",
        ExportKey::SaveFigure => "Enregistrer",
        ExportKey::SaveFigureHover => "Enregistrer la figure sur le disque (Cmd+S)",
        ExportKey::CopyToClipboard => "Copier dans le presse-papiers",
        ExportKey::CopyToClipboardHover => {
            "Copier l'image directement dans le presse-papiers du système"
        }
        ExportKey::RoiLabel => "ROI :",
        ExportKey::RoiSize => "{}x{} px",
        ExportKey::CropSaveHover => "Enregistrer la figure de la ROI recadrée (Cmd+S)",
        ExportKey::CropResetHover => "Ajuster le cadre au canevas entier",
        ExportKey::CropDone => "Terminé",
        ExportKey::CropDoneHover => "Fermer le recadrage",
        ExportKey::ExportFailed => "Échec de l'export",
        ExportKey::Saved => "Enregistré",
        ExportKey::FallbackFilename => "figure",
        ExportKey::EncodingError => "Erreur d'encodage : {}",
        ExportKey::CreateDirFailed => "Impossible de créer le dossier d'export : {}",
        ExportKey::WriteFileFailed => "Impossible d'écrire le fichier d'export : {}",
    })
}
