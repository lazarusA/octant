use crate::i18n::keys::ColormapKey;

pub fn text(key: ColormapKey) -> Option<&'static str> {
    Some(match key {
        ColormapKey::Sequential => "Sequencial",
        ColormapKey::Diverging => "Divergente",
        ColormapKey::Cyclic => "Cíclico",
        ColormapKey::Categorical => "Categórico",
        ColormapKey::Other => "Outro",
        ColormapKey::InterpolationLinear => "Linear",
        ColormapKey::BlendLinearRgb => "RGB linear",
        ColormapKey::FilterAll => "Todos",
        ColormapKey::FamilyCustom => "Personalizado",
        ColormapKey::FamilyTitle => "Família: {}",
        ColormapKey::Smooth => "Suave",
        ColormapKey::SmoothHover => {
            "Mesclar as cores desta paleta em um gradiente contínuo (Oklab)"
        }
        ColormapKey::Reversed => "Invertido",
        ColormapKey::SearchHint => "Pesquisar mapas de cores...",
        ColormapKey::NoMatches => "Nenhum mapa de cores corresponde",
        ColormapKey::ShowColorbar => "Mostrar barra de cores",
        ColormapKey::ChangeColormapHover => "Clique para alterar o mapa de cores",
        ColormapKey::ColorSettingsHover => "Configurações de cor",
        ColormapKey::BoxTransparency => "Transparência da caixa",
        ColormapKey::CustomColormap => "Mapa de cores personalizado",
        ColormapKey::Name => "Nome",
        ColormapKey::Colors => "Cores",
        ColormapKey::ColorsHover => {
            "Hex, rgb(), hsl() ou nomes de cores CSS, opcionalmente com posições"
        }
        ColormapKey::Interpolation => "Interpolação",
        ColormapKey::Blend => "Mistura",
        ColormapKey::Classes => "Classes",
        ColormapKey::ClassesHover => "0 mantém o gradiente contínuo",
        ColormapKey::RenameAndApply => "Renomear e aplicar",
        ColormapKey::SaveAndApply => "Salvar e aplicar",
        ColormapKey::SaveAsNew => "Salvar como novo",
        ColormapKey::NameTaken => "Já existe um mapa de cores salvo com este nome",
        ColormapKey::NameRequired => "O nome é obrigatório",
        ColormapKey::Saved => "Salvo",
        ColormapKey::BrokenHover => "Não funciona nesta versão: edite para corrigir ou exclua",
        ColormapKey::Edit => "Editar",
        ColormapKey::Delete => "Excluir",
        ColormapKey::DeleteHover => "Excluir mapa de cores",
        ColormapKey::FlipOrientation => "Inverter orientação",
        ColormapKey::RangeMinHover => {
            "Extremo inferior do intervalo (Mín). Arraste para ajustar ou clique para digitar."
        }
        ColormapKey::RangeMaxHover => {
            "Extremo superior do intervalo (Máx). Arraste para ajustar ou clique para digitar."
        }
        ColormapKey::LowClipTitle => "Cor abaixo do intervalo (< Mín)",
        ColormapKey::LowClipHover => {
            "Cor abaixo do intervalo (< Mín). Clique para selecionar a cor."
        }
        ColormapKey::HighClipTitle => "Cor acima do intervalo (> Máx)",
        ColormapKey::HighClipHover => {
            "Cor acima do intervalo (> Máx). Clique para selecionar a cor."
        }
        ColormapKey::HoveredValue => "Val: {}",
        ColormapKey::TitleHover => "Título da barra de cores. Clique para editar.",
        ColormapKey::ColorPicker => "Seletor de cores",
        ColormapKey::Copy => "Copiar",
        ColormapKey::CopyHover => "Copiar os valores da cor para a área de transferência",
        ColormapKey::AlphaNotANumber => "'{}' não é um número",
        ColormapKey::AlphaOutOfRange => "{} está fora de 0..1",
        ColormapKey::AlphaMixedForms => "use valores ou paradas posição:alfa, não ambos",
        ColormapKey::AlphaUnsorted => "as posições das paradas devem ser crescentes",
        ColormapKey::PrefsUnreadable => "Preferências de mapas de cores ilegíveis",
        ColormapKey::PrefsBackedUp => "Cópia de segurança salva em `{}`",
        ColormapKey::PrefsNotBackedUp => {
            "Sem cópia de segurança das preferências de mapas de cores"
        }
        ColormapKey::BackupsFull => {
            "Todas as {} cópias de segurança das preferências de mapas de cores estão em uso"
        }
        ColormapKey::CustomNotLoaded => "Mapa de cores personalizado não carregado",
        ColormapKey::CustomDoesNotBuild => "`{}` não pôde ser criado: {}",
        ColormapKey::NotRegistered => "O mapa de cores {} não foi registrado",
        ColormapKey::BundledUnreadable => "Mapas de cores incluídos ilegíveis",
        ColormapKey::FallingBackToGray => "{}; usando escala de cinza",
    })
}
