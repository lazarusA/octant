use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "组",
        VariablesKey::DataType => "类型",
        VariablesKey::Shape => "形状",
        VariablesKey::Description => "描述",
        VariablesKey::SearchVariables => "搜索变量...",
        VariablesKey::NoVariablesMatching => "没有与 '{}' 匹配的变量",
        VariablesKey::CloseVariablesWindow => "关闭变量窗口",
        VariablesKey::NoMetadataLoaded => "尚未加载存储元数据。",
        VariablesKey::FetchStoreMetadata => "获取 / 加载存储元数据",
        VariablesKey::NoVariablesFound => "此数据集存储中没有找到变量。",
        VariablesKey::RefreshStoreMetadata => "刷新存储元数据",
        VariablesKey::TruncatedVariables => {
            "此文件夹中显示了 {} 个变量（共 {} 个）。使用搜索查找全部。"
        }
        VariablesKey::TruncatedFolders => "显示了 {} 个文件夹（共 {} 个）。使用搜索查找全部。",
        VariablesKey::NoVariableSelected => "未选择变量。",
        VariablesKey::DimensionSliders => "维度滑块",
        VariablesKey::CloseDimensionPanel => "关闭维度面板",
        VariablesKey::PlotData => "绘制数据",
        VariablesKey::PlotDataHover => "获取此变量并绘制",
        VariablesKey::PlotDataOverlayHover => "获取此变量并将其作为叠加层添加到图上",
        VariablesKey::AddOverlay => "添加叠加层",
        VariablesKey::AddOverlayHover => "开启时，绘制数据会将此变量添加为叠加层",
        VariablesKey::InfoPath => "路径：",
        VariablesKey::InfoUnits => "单位：{}",
        VariablesKey::InfoDescription => "描述：{}",
        VariablesKey::InfoShape => "形状：{}",
        VariablesKey::InfoDimensions => "维度：{}",
        VariablesKey::InfoTime => "时间：{} -> {}",
        VariablesKey::InfoResolution => "分辨率：{}",
        VariablesKey::InfoSize => "大小：{} MB",
        VariablesKey::InfoAttributes => "属性 (.zattrs)",
        VariablesKey::Download => "下载",
        VariablesKey::DownloadCells => "{}（{} 个单元格）",
        VariablesKey::LargeSelectionPyramid => {
            "大型 2D 选区（{} 个单元格，{} MB）：已启用自动多分辨率金字塔聚合。"
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "此大型选区（{} 个单元格，{} MB）已禁用 3D 球面和 3D 曲面网格。2D 平面和 1D 线图仍可使用。"
        }
        VariablesKey::LargeVolumeDisabled => {
            "此选区已禁用 3D 体渲染和点云：体数据大小（{} MB）超出 {} MB 的 GPU 存储缓冲区上限。2D 平面、1D 线图和 3D 球面仍可使用。"
        }
        VariablesKey::RoleNone => "无",
        VariablesKey::RoleAnimated => "动画",
        VariablesKey::RoleGrid => "网格",
        VariablesKey::Index => "索引：",
        VariablesKey::ChannelN => "通道 {}",
        VariablesKey::LevelN => "第 {} 级",
        VariablesKey::LevelAuto => "自动",
        VariablesKey::LevelOf => "第 {} 级（共 {} 级）",
        VariablesKey::AutoLevelOf => "自动，第 {} 级（共 {} 级）",
    })
}
