use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "Conjunto de dados",
        CommonKey::Variables => "Variáveis",
        CommonKey::Dimensions => "Dimensões",
        CommonKey::Colormaps => "Mapas de cores",
        CommonKey::Settings => "Configurações",
        CommonKey::About => "Sobre",
        CommonKey::Cache => "Cache",
        CommonKey::Language => "Idioma",
        CommonKey::Loading => "Carregando...",
        CommonKey::CloseEsc => "Fechar (Esc)",
        CommonKey::Cancel => "Cancelar",
        CommonKey::Save => "Salvar",
        CommonKey::Reset => "Redefinir",
        CommonKey::Move => "Mover",
        CommonKey::MoveHover => "Arraste para mover; clique duas vezes para redefinir",
        CommonKey::Light => "Claro",
        CommonKey::Dark => "Escuro",
        CommonKey::SwitchToLightHover => "Mudar para o modo claro",
        CommonKey::SwitchToDarkHover => "Mudar para o modo escuro",
        CommonKey::ToggleHeroHover => {
            "Alternar a tela inicial (clique com o botão direito para Sobre)"
        }
        CommonKey::AboutOctantHover => "Sobre o Octant",
        CommonKey::ToggleVariableControlsHover => "Alternar o painel de controles da variável",
        CommonKey::FetchingProgress => "Baixando {} / {} ({}%)",
        CommonKey::Abort => "Abortar",
        CommonKey::AbortHover => "Interromper e abortar a transferência de dados em andamento",
    })
}
