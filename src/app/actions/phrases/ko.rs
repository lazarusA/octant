//! Korean phrasings of the verbs.
//! Matched as substrings, as words are not set apart by spaces.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Goto, &["이동", "설정"]),
        (Open, &["열어", "열기", "불러와", "로드"]),
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
    months: [
        &["1월"],
        &["2월"],
        &["3월"],
        &["4월"],
        &["5월"],
        &["6월"],
        &["7월"],
        &["8월"],
        &["9월"],
        &["10월"],
        &["11월"],
        &["12월"],
    ],
    axes: [
        &["시간", "날짜"],
        &["위도"],
        &["경도"],
        &["레벨", "깊이", "높이"],
    ],
    catalog: &["카탈로그"],
    then: &["그다음", "그 다음", "그리고"],
};
