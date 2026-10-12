use crate::i18n::keys::ExportKey;

pub fn text(key: ExportKey) -> Option<&'static str> {
    Some(match key {
        ExportKey::Png => "PNG (sem perdas)",
        ExportKey::Jpeg => "JPEG (com perdas)",
        ExportKey::Webp => "WebP (moderno)",
        ExportKey::Svg => "SVG (vetorial / híbrido)",
        ExportKey::Pdf => "PDF (publicação)",
        ExportKey::AspectFreeform => "Livre",
        ExportKey::Aspect16x9 => "16:9 (slides)",
        ExportKey::Aspect4x3 => "4:3 (figura)",
        ExportKey::Aspect1x1 => "1:1 (quadrado)",
        ExportKey::DialogTitle => "Salvar e exportar figura",
        ExportKey::FormatHeading => "Formato de exportação",
        ExportKey::JpegQuality => "Qualidade JPEG:",
        ExportKey::AreaHeading => "Área de exportação",
        ExportKey::FullCanvas => "Tela inteira (com sobreposições)",
        ExportKey::FramedRoi => "ROI enquadrada",
        ExportKey::HideGuides => "Ocultar linhas-guia",
        ExportKey::ShowGuides => "Mostrar linhas-guia na tela",
        ExportKey::DestinationHeading => "Destino e nome do arquivo",
        ExportKey::Folder => "Pasta:",
        ExportKey::Preview => "Prévia: {}/{}",
        ExportKey::SaveFigure => "Salvar figura",
        ExportKey::SaveFigureHover => "Salvar a figura no disco (Cmd+S)",
        ExportKey::CopyToClipboard => "Copiar para a área de transferência",
        ExportKey::CopyToClipboardHover => {
            "Copiar a imagem diretamente para a área de transferência do sistema"
        }
        ExportKey::RoiLabel => "ROI:",
        ExportKey::RoiSize => "{}x{} px",
        ExportKey::CropSaveHover => "Salvar a figura da ROI recortada (Cmd+S)",
        ExportKey::CropResetHover => "Ajustar a caixa de recorte à tela inteira",
        ExportKey::CropDone => "Concluído",
        ExportKey::CropDoneHover => "Fechar a sobreposição de recorte",
        ExportKey::ExportFailed => "Falha na exportação",
        ExportKey::Saved => "Salvo",
        ExportKey::FallbackFilename => "figura",
        ExportKey::EncodingError => "Erro de codificação: {}",
        ExportKey::CreateDirFailed => "Falha ao criar a pasta de exportação: {}",
        ExportKey::WriteFileFailed => "Falha ao gravar o arquivo de exportação: {}",
    })
}
