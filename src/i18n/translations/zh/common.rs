use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "数据集",
        CommonKey::Variables => "变量",
        CommonKey::Dimensions => "维度",
        CommonKey::Settings => "设置",
        CommonKey::About => "关于",
        CommonKey::Cache => "缓存",
        CommonKey::Language => "语言",
        CommonKey::Loading => "正在加载...",
        CommonKey::CloseEsc => "关闭 (Esc)",
        CommonKey::Cancel => "取消",
        CommonKey::Save => "保存",
        CommonKey::Reset => "重置",
        CommonKey::Move => "移动",
        CommonKey::MoveHover => "拖动以移动；双击以重置",
        CommonKey::Light => "浅色",
        CommonKey::Dark => "深色",
        CommonKey::SwitchToLightHover => "切换到浅色模式",
        CommonKey::SwitchToDarkHover => "切换到深色模式",
        CommonKey::ToggleHeroHover => "切换首页视图（右键查看关于）",
        CommonKey::AboutOctantHover => "关于 Octant",
        CommonKey::ToggleVariableControlsHover => "显示/隐藏变量控制面板",
        CommonKey::FetchingProgress => "正在获取 {} / {}（{}%）",
        CommonKey::Abort => "中止",
        CommonKey::AbortHover => "中断并中止正在进行的数据传输",
        CommonKey::Voice => "语音",
        CommonKey::VoiceHover => "语音输入：麦克风与频谱图",
    })
}
