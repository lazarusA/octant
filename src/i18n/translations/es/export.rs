use crate::i18n::keys::ExportKey;

pub fn text(key: ExportKey) -> Option<&'static str> {
    Some(match key {
        ExportKey::Png => "PNG (sin pérdida)",
        ExportKey::Jpeg => "JPEG (con pérdida)",
        ExportKey::Webp => "WebP (moderno)",
        ExportKey::Svg => "SVG (vectorial / híbrido)",
        ExportKey::Pdf => "PDF (publicación)",
        ExportKey::AspectFreeform => "Libre",
        ExportKey::Aspect16x9 => "16:9 (diapositivas)",
        ExportKey::Aspect4x3 => "4:3 (figura)",
        ExportKey::Aspect1x1 => "1:1 (cuadrado)",
        ExportKey::DialogTitle => "Guardar y exportar figura",
        ExportKey::FormatHeading => "Formato de exportación",
        ExportKey::JpegQuality => "Calidad JPEG:",
        ExportKey::AreaHeading => "Área de exportación",
        ExportKey::FullCanvas => "Lienzo completo (con superposiciones)",
        ExportKey::FramedRoi => "ROI enmarcada",
        ExportKey::HideGuides => "Ocultar líneas guía",
        ExportKey::ShowGuides => "Mostrar líneas guía en el lienzo",
        ExportKey::DestinationHeading => "Destino y nombre de archivo",
        ExportKey::Folder => "Carpeta:",
        ExportKey::Preview => "Vista previa: {}/{}",
        ExportKey::SaveFigure => "Guardar figura",
        ExportKey::SaveFigureHover => "Guardar la figura en el disco (Cmd+S)",
        ExportKey::CopyToClipboard => "Copiar al portapapeles",
        ExportKey::CopyToClipboardHover => {
            "Copiar la imagen directamente al portapapeles del sistema"
        }
        ExportKey::RoiLabel => "ROI:",
        ExportKey::RoiSize => "{}x{} px",
        ExportKey::CropSaveHover => "Guardar la figura de la ROI recortada (Cmd+S)",
        ExportKey::CropResetHover => "Ajustar el recuadro de recorte a todo el lienzo",
        ExportKey::CropDone => "Listo",
        ExportKey::CropDoneHover => "Cerrar el recorte",
        ExportKey::ExportFailed => "Error al exportar",
        ExportKey::Saved => "Guardado",
        ExportKey::FallbackFilename => "figura",
        ExportKey::EncodingError => "Error de codificación: {}",
        ExportKey::CreateDirFailed => "No se pudo crear el directorio de exportación: {}",
        ExportKey::WriteFileFailed => "No se pudo escribir el archivo de exportación: {}",
    })
}
