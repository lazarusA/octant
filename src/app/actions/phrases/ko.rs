//! Korean phrasings of the verbs.
//! Matched as substrings, as words are not set apart by spaces.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Select, &["선택"]),
        (Plot, &["표시", "그려", "플롯", "보여", "시각화"]),
        (Play, &["재생", "시작", "애니메이션"]),
        (Pause, &["일시정지", "일시 정지", "정지", "멈춰"]),
        (Next, &["다음", "앞으로"]),
        (Previous, &["이전", "뒤로"]),
        (Colormap, &["컬러맵", "색상표", "색상 맵", "팔레트"]),
        (
            ResetView,
            &["뷰 초기화", "보기 초기화", "화면 초기화", "초기화"],
        ),
    ],
    then: &["그다음", "그 다음", "그리고"],
};
