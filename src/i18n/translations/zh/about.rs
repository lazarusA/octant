use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "关于 Octant",
        AboutKey::IconsWindowTitle => "原生矢量图标",
        AboutKey::IconsTab => "矢量图标",
        AboutKey::Subtitle => "N 维数据浏览器",
        AboutKey::Description => {
            "Octant 是一款交互式 N 维数据集查看器，原生支持本地与云端对象存储、Zarr (v2/v3) 和 Icechunk。它使用 Rust 编写，通过 WGPU 进行 GPU 加速渲染，可在 macOS、Linux 和 Windows 上原生运行。"
        }
        AboutKey::HyperslabTitle => "超切片读取",
        AboutKey::HyperslabDesc => "异步 LRU 分块缓存与多分辨率金字塔。",
        AboutKey::StorageTitle => "原生支持 Zarr 与 Icechunk",
        AboutKey::StorageDesc => "本地、S3、GCS、Azure 和 HTTP 流式后端。",
        AboutKey::ShadersTitle => "硬件加速的 WGPU 着色器",
        AboutKey::ShadersDesc => "2D 平面图、3D 球面、高程曲面、体渲染与 1D 剖面。",
        AboutKey::LinksHeading => "链接与资源",
        AboutKey::DocsLink => "Octant 文档",
        AboutKey::GoodFaithNotice => {
            "Octant 本着诚信原则注明第三方作品的来源。如果你认为有遗漏或注明有误，请告诉我们，我们会更正或移除。"
        }
        AboutKey::BrowseIcons => "浏览原生矢量图标",
        AboutKey::BrowseIconsHover => "在弹出窗口中查看程序生成的矢量图标",
        AboutKey::Licensed => "采用 MIT 或 Apache-2.0 许可证",
        AboutKey::OctantTeaser => {
            "x、y、z 的正负号 (+/-, +/-, +/-) 把空间分成八个卦限。用同样的方式探索你的数据：选定坐标轴，切出一层，每次只看一个区域。"
        }
        AboutKey::OctantHover => "点击前往下一个卦限",
        AboutKey::ColormapCredits => "色图来源",
        AboutKey::ColormapCreditsNote => "内置色图沿用其作者的许可证。",
        AboutKey::LicenseTexts => "许可证文本",
        AboutKey::ThirdPartyLicenses => "第三方许可证",
        AboutKey::ThirdPartyNote => {
            "Octant 包含开源 crate、字体和 C 库 (netCDF、HDF5)，各自采用其自己的许可证。"
        }
        AboutKey::ViewOnGithub => "在 GitHub 上查看",
        AboutKey::IconsSearchHint => "搜索图标...",
        AboutKey::IconsAll => "全部",
        AboutKey::IconsCopied => "已复制 {}",
        AboutKey::IconsNoMatch => "没有找到匹配的矢量图标。",
        AboutKey::IconsCategory => "类别：{}",
        AboutKey::IconsClickToCopy => "点击以复制枚举标识符",
        AboutKey::IconsSizesTones => "尺寸与色调",
        AboutKey::ClearSearchHover => "清除搜索",
        AboutKey::FontLicensesNote => "Noto Sans SC、JP 和 KR（中文、日文和韩文文本）：",
    })
}
