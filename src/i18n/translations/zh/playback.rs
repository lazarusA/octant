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
        PlaybackKey::CropHover => "裁剪辅助线",
        PlaybackKey::PlaybackSpeed => "播放速度",
        PlaybackKey::PlaybackBuffer => "播放前缓冲",
        PlaybackKey::Buffering => "缓冲中 {}/{}",
        PlaybackKey::SaveExportHover => "保存 / 导出图像（Cmd+S 快速保存，Cmd+Shift+S 打开设置）",
        PlaybackKey::CollapsedHint => "播放（点击或拖动以展开）",
        PlaybackKey::TimelineDetails => "时间轴详情",
        PlaybackKey::TimelineDimension => "维度：{}",
        PlaybackKey::TimelineStep => "时间步：{} / {}",
        PlaybackKey::TimelineCurrent => "当前：{}",
        PlaybackKey::TimelineRange => "范围：{} -> {}",
        PlaybackKey::TimelineStepSize => "步长：{}",
        PlaybackKey::DefaultStepSize => "步长：1",
        PlaybackKey::VoiceListen => "开始收听",
        PlaybackKey::VoiceStop => "停止收听",
        PlaybackKey::VoiceListening => "正在收听...",
        PlaybackKey::VoiceMicOff => "麦克风已关闭",
        PlaybackKey::VoiceMicError => "麦克风不可用",
        PlaybackKey::VoiceSilent => "麦克风只传来静音",
        PlaybackKey::VoiceSilentHint => {
            "请在系统隐私设置中允许启动 Octant 的应用访问麦克风，然后重新启动该应用。"
        }
        PlaybackKey::VoiceCloseHover => "关闭语音输入",
    })
}
