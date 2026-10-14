use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "재생",
        PlaybackKey::Pause => "일시 정지",
        PlaybackKey::Loop => "반복",
        PlaybackKey::FirstStep => "첫 스텝",
        PlaybackKey::PreviousStep => "이전 스텝",
        PlaybackKey::NextStep => "다음 스텝",
        PlaybackKey::LastStep => "마지막 스텝",
        PlaybackKey::Crop => "자르기",
        PlaybackKey::CropHover => "자르기 안내선 (C)",
        PlaybackKey::PlaybackSpeed => "재생 속도",
        PlaybackKey::PlaybackBuffer => "재생 전 버퍼링",
        PlaybackKey::SaveExportHover => {
            "그림 저장 / 내보내기 (Cmd+S: 빠른 저장, Cmd+Shift+S: 설정)"
        }
        PlaybackKey::CollapsedHint => "재생 (클릭하거나 드래그하여 펼치기)",
        PlaybackKey::TimelineDetails => "타임라인 세부 정보",
        PlaybackKey::TimelineDimension => "차원: {}",
        PlaybackKey::TimelineStep => "스텝: {} / {}",
        PlaybackKey::TimelineCurrent => "현재: {}",
        PlaybackKey::TimelineRange => "범위: {} -> {}",
        PlaybackKey::TimelineStepSize => "스텝 간격: {}",
        PlaybackKey::DefaultStepSize => "스텝 간격: 1",
    })
}
