use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "关闭",
        NoticesKey::CopyDetails => "复制详情",
        NoticesKey::RevealInFinder => "在 Finder 中显示",
        NoticesKey::OpenFolder => "打开文件夹",
        NoticesKey::NoVariablesFound => "未找到变量",
        NoticesKey::CouldntOpenDataset => "无法打开数据集",
        NoticesKey::UnsupportedFile => "不支持的文件",
        NoticesKey::CouldntLoadData => "无法加载数据",
        NoticesKey::CoordinatesUnavailable => "坐标不可用",
        NoticesKey::CoordinatesFailedDetail => "'{}' 的坐标加载失败（{}）；坐标轴显示索引",
        NoticesKey::CoastlinesUnavailable => "海岸线不可用",
        NoticesKey::NoDataAtStep => "此时间步没有数据",
        NoticesKey::NoDataAtStepDetail => "'{}' 在此时间步没有数值：存储中此处没有数据。请选择其他时间步。",
        NoticesKey::LargeChunks => "分块过大",
        NoticesKey::LargeChunksDetail => "'{}' 以 {} 的分块存储：每个时间步都要下载完整分块，因此加载较慢。",
        NoticesKey::VolumeTooLargeForGpu => "体数据过大，GPU 无法处理",
        NoticesKey::VolumeLodOff => "此体数据的按视图细化已关闭",
        NoticesKey::VolumeLodOffDetail => {
            "其显示层级需要 {} 个砖块；图集可容纳 {} 个，并为更精细的层级保留一半。体数据以单一完整层级绘制。"
        }
        NoticesKey::VolumeLodUnevenDetail => {
            "其各级之间不是单元对齐的整数倍，不满足分块的要求。体积将以单一完整级别绘制。"
        }
        NoticesKey::VolumeLodWideIntegersDetail => {
            "其整数值（如标签 ID）范围过宽，分块无法区分。体积将以单一完整级别绘制。"
        }
        NoticesKey::VolumeLodUnsupportedDetail => {
            "此 GPU 无法从体积着色器写入，不满足分块的要求。体积将以单一完整级别绘制。"
        }
        NoticesKey::VolumeLodBrickFailed => "部分体数据无法以更精细的层级读取",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "体数据缓冲区大小（{} 个元素 / {} 字节）超出 GPU 存储缓冲区上限（{}）"
        }
        NoticesKey::VolumeTextureTooLargeDetail => "体数据 {}x{}x{} 超出 GPU 3D 纹理上限（{}）",
        NoticesKey::VolumeUploadSkipped => "已跳过体数据上传",
        NoticesKey::VolumeUploadMismatchDetail => "{} 个值，而纹理为 {}x{}x{}",
        NoticesKey::DataTooLargeForGpuBuffer => "数据过大，超出 GPU 缓冲区",
        NoticesKey::BufferTooLargeDetail => {
            "{}：数据大小（{} 字节）超出 GPU 缓冲区容量（{} 字节），跳过写入"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}：从元素 {} 起的范围（+{} 个元素）超出 GPU 缓冲区容量（{} 字节），跳过写入"
        }
        NoticesKey::LinePlotTooLargeForGpu => "线图过大，GPU 无法处理",
        NoticesKey::LinePayloadTooLargeDetail => "线条占用 {} 字节；此 GPU 最多可绑定 {}。",
        NoticesKey::FontLoadFailed => "无法加载字体",
        NoticesKey::FontLoadFailedDetail => "{} 文本可能显示为方框：{}",
        NoticesKey::OmeMetadataNonConforming => "OME-Zarr 元数据不符合规范",
        NoticesKey::LabelIdsBeyond32Bits => "超过 32 位的标签 ID 与较小的 ID 共用颜色",
        NoticesKey::OmeMetadataNonConformingDetail => {
            "{} 版本的元数据（{}）不符合规范；仅显示可读取的部分。"
        }
        NoticesKey::PlateWellStopped => "孔 {}：读取中断",
        NoticesKey::PlateWellNotJson => "孔 {}：元数据不是有效的 JSON",
        NoticesKey::PlateWellNotOme => "孔 {}：不是 OME-Zarr 孔",
        NoticesKey::PlateWellNoMetadata => "孔 {}：没有孔的元数据",
    })
}
