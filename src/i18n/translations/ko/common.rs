use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "데이터셋",
        CommonKey::Variables => "변수",
        CommonKey::Dimensions => "차원",
        CommonKey::Settings => "설정",
        CommonKey::About => "정보",
        CommonKey::Cache => "캐시",
        CommonKey::Language => "언어",
        CommonKey::Loading => "불러오는 중...",
        CommonKey::CloseEsc => "닫기 (Esc)",
        CommonKey::Cancel => "취소",
        CommonKey::Save => "저장",
        CommonKey::Reset => "초기화",
        CommonKey::Move => "이동",
        CommonKey::MoveHover => "드래그하여 이동하고, 더블클릭하여 원래 위치로 되돌립니다",
        CommonKey::Light => "라이트",
        CommonKey::Dark => "다크",
        CommonKey::SwitchToLightHover => "라이트 모드로 전환",
        CommonKey::SwitchToDarkHover => "다크 모드로 전환",
        CommonKey::ToggleHeroHover => "시작 화면 표시/숨기기 (오른쪽 클릭: 정보)",
        CommonKey::AboutOctantHover => "Octant 정보",
        CommonKey::ToggleVariableControlsHover => "변수 제어 패널 표시/숨기기",
        CommonKey::FetchingProgress => "가져오는 중 {} / {} ({}%)",
        CommonKey::Abort => "중단",
        CommonKey::AbortHover => "진행 중인 데이터 전송을 중단합니다",
        CommonKey::Voice => "음성",
        CommonKey::VoiceHover => "음성 입력: 마이크와 스펙트로그램",
    })
}
