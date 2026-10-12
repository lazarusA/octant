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
        PlaybackKey::CropHover => "Linhas-guia de recorte (C)",
        PlaybackKey::PlaybackSpeed => "Velocidade de reprodução",
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
    })
}
