use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "再生",
        PlaybackKey::Pause => "一時停止",
        PlaybackKey::Loop => "ループ",
        PlaybackKey::FirstStep => "最初のステップ",
        PlaybackKey::PreviousStep => "前のステップ",
        PlaybackKey::NextStep => "次のステップ",
        PlaybackKey::LastStep => "最後のステップ",
        PlaybackKey::Crop => "切り抜き",
        PlaybackKey::CropHover => "切り抜きガイド線 (C)",
        PlaybackKey::PlaybackSpeed => "再生速度",
        PlaybackKey::SaveExportHover => {
            "図を保存 / エクスポート (Cmd+S でクイック保存、Cmd+Shift+S で設定)"
        }
        PlaybackKey::CollapsedHint => "再生 (クリックまたはドラッグで展開)",
        PlaybackKey::TimelineDetails => "タイムラインの詳細",
        PlaybackKey::TimelineDimension => "次元: {}",
        PlaybackKey::TimelineStep => "ステップ: {} / {}",
        PlaybackKey::TimelineCurrent => "現在: {}",
        PlaybackKey::TimelineRange => "範囲: {} -> {}",
        PlaybackKey::TimelineStepSize => "ステップ幅: {}",
        PlaybackKey::DefaultStepSize => "ステップ: 1",
    })
}
