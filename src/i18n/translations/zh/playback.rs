use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "播放",
        PlaybackKey::Pause => "暂停",
        PlaybackKey::Loop => "循环",
        PlaybackKey::FirstStep => "首个时间步",
        PlaybackKey::PreviousStep => "上一时间步",
        PlaybackKey::NextStep => "下一时间步",
        PlaybackKey::LastStep => "最后时间步",
        PlaybackKey::Crop => "裁剪",
        PlaybackKey::CropHover => "裁剪辅助线 (C)",
        PlaybackKey::PlaybackSpeed => "播放速度",
        PlaybackKey::PlaybackBuffer => "播放前缓冲",
        PlaybackKey::SaveExportHover => "保存 / 导出图像（Cmd+S 快速保存，Cmd+Shift+S 打开设置）",
        PlaybackKey::CollapsedHint => "播放（点击或拖动以展开）",
        PlaybackKey::TimelineDetails => "时间轴详情",
        PlaybackKey::TimelineDimension => "维度：{}",
        PlaybackKey::TimelineStep => "时间步：{} / {}",
        PlaybackKey::TimelineCurrent => "当前：{}",
        PlaybackKey::TimelineRange => "范围：{} -> {}",
        PlaybackKey::TimelineStepSize => "步长：{}",
        PlaybackKey::DefaultStepSize => "步长：1",
    })
}
