//! Chinese (simplified) phrasings of the verbs.
//! Matched as substrings, as words are not set apart by spaces.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Select, &["选择", "选中"]),
        (Plot, &["绘制", "显示", "展示", "可视化", "绘图", "画"]),
        (Play, &["播放", "开始", "动画", "继续"]),
        (Pause, &["暂停", "停止"]),
        (Next, &["下一步", "下一个", "下一帧", "前进"]),
        (Previous, &["上一步", "上一个", "上一帧", "后退"]),
        (Colormap, &["颜色映射", "色图", "配色", "调色板", "色标"]),
        (ResetView, &["重置视图", "复位视图", "重置"]),
    ],
    then: &["然后", "接着", "之后"],
};
