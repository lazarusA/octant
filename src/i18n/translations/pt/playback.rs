use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Reproduzir",
        PlaybackKey::Pause => "Pausar",
        PlaybackKey::Loop => "Repetir",
        PlaybackKey::FirstStep => "Primeiro passo",
        PlaybackKey::PreviousStep => "Passo anterior",
        PlaybackKey::NextStep => "Próximo passo",
        PlaybackKey::LastStep => "Último passo",
        PlaybackKey::Crop => "Recortar",
        PlaybackKey::CropHover => "Linhas-guia de recorte",
        PlaybackKey::PlaybackSpeed => "Velocidade de reprodução",
        PlaybackKey::PlaybackBuffer => "Buffer antes de reproduzir",
        PlaybackKey::Buffering => "Carregando buffer {}/{}",
        PlaybackKey::SaveExportHover => {
            "Salvar / exportar figura (Cmd+S para salvar rápido, Cmd+Shift+S para configurações)"
        }
        PlaybackKey::CollapsedHint => "Reprodução (clique ou arraste para expandir)",
        PlaybackKey::TimelineDetails => "Detalhes da linha do tempo",
        PlaybackKey::TimelineDimension => "Dimensão: {}",
        PlaybackKey::TimelineStep => "Passo: {} / {}",
        PlaybackKey::TimelineCurrent => "Atual: {}",
        PlaybackKey::TimelineRange => "Intervalo: {} -> {}",
        PlaybackKey::TimelineStepSize => "Tamanho do passo: {}",
        PlaybackKey::DefaultStepSize => "Passo: 1",
        PlaybackKey::VoiceListen => "Começar a ouvir",
        PlaybackKey::VoiceStop => "Parar de ouvir",
        PlaybackKey::VoiceListening => "Ouvindo...",
        PlaybackKey::VoiceMicOff => "Microfone desligado",
        PlaybackKey::VoiceMicError => "Microfone indisponível",
        PlaybackKey::VoiceSilent => "O microfone envia apenas silêncio",
        PlaybackKey::VoiceSilentHint => {
            "Permita o acesso ao microfone para o aplicativo que iniciou o Octant nas configurações de privacidade do sistema e reinicie-o."
        }
        PlaybackKey::VoiceSttFailed => "A transcrição falhou",
        PlaybackKey::VoiceDownloading => "Baixando o modelo de voz: {}%",
        PlaybackKey::VoiceLoading => "Carregando o modelo de voz...",
        PlaybackKey::VoiceModelHover => "Modelo de fala para texto",
        PlaybackKey::VoiceModelSize => "{} ({} MB)",
        PlaybackKey::VoiceCloseHover => "Fechar entrada de voz",
        PlaybackKey::VoiceDidOpen => "Abrindo {}",
        PlaybackKey::VoiceNoSource => "Nenhum conjunto de dados corresponde a '{}'",
        PlaybackKey::VoiceDidSelect => "{} selecionado",
        PlaybackKey::VoiceDidSetDim => "{}: {}",
        PlaybackKey::VoiceDidSetRange => "{}: {} - {}",
        PlaybackKey::VoiceNoDimension => "Nenhuma dimensão corresponde a '{}'",
        PlaybackKey::VoiceNoValue => "Nenhum valor de {} corresponde a '{}'",
        PlaybackKey::VoiceDidPlot => "{} plotado como {}",
        PlaybackKey::VoiceDidColormap => "Mapa de cores {}",
        PlaybackKey::VoiceNotUnderstood => "Comando não entendido",
        PlaybackKey::VoiceNoDataset => "Abra primeiro um conjunto de dados",
        PlaybackKey::VoiceNoVariable => "Nenhuma variável corresponde a '{}'",
        PlaybackKey::VoiceNoPlotType => "Nenhum tipo de gráfico corresponde a '{}'",
        PlaybackKey::VoiceNoColormap => "Nenhum mapa de cores corresponde a '{}'",
        PlaybackKey::VoiceAmbiguous => "Várias variáveis correspondem a '{}'",
        PlaybackKey::VoiceDidYouMean => "Você quis dizer: {}",
        PlaybackKey::VoiceNothingToAnimate => "O gráfico não tem dimensão animada",
        PlaybackKey::VoiceCannotPlot => "Não é possível plotar como {}",
    })
}
