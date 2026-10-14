use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> &'static str {
    match key {
        VariablesKey::Group => "Group",
        VariablesKey::DataType => "Type",
        VariablesKey::Shape => "Shape",
        VariablesKey::Description => "Description",
        VariablesKey::SearchVariables => "Search variables...",
        VariablesKey::NoVariablesMatching => "No variables matching '{}'",
        VariablesKey::CloseVariablesWindow => "Close Variables Window",
        VariablesKey::NoMetadataLoaded => "No store metadata loaded yet.",
        VariablesKey::FetchStoreMetadata => "Fetch / Load Store Metadata",
        VariablesKey::NoVariablesFound => "No variables found in this dataset store.",
        VariablesKey::RefreshStoreMetadata => "Refresh Store Metadata",
        VariablesKey::TruncatedVariables => {
            "Showing {} of {} variables in this folder. Use search to discover all."
        }
        VariablesKey::TruncatedFolders => "Showing {} of {} folders. Use search to discover all.",
        VariablesKey::NoVariableSelected => "No variable selected.",
        VariablesKey::DimensionSliders => "Dimension Sliders",
        VariablesKey::CloseDimensionPanel => "Close Dimension Panel",
        VariablesKey::PlotData => "Plot Data",
        VariablesKey::PlotDataHover => "Fetch this variable and plot it",
        VariablesKey::PlotDataOverlayHover => {
            "Fetch this variable and add it as an overlay over the plot"
        }
        VariablesKey::AddOverlay => "Add Overlay",
        VariablesKey::AddOverlayHover => "Plot Data adds this variable as an overlay while on",
        VariablesKey::InfoPath => "Path:",
        VariablesKey::InfoUnits => "Units: {}",
        VariablesKey::InfoDescription => "Description: {}",
        VariablesKey::InfoShape => "Shape: {}",
        VariablesKey::InfoDimensions => "Dimensions: {}",
        VariablesKey::InfoTime => "Time: {} -> {}",
        VariablesKey::InfoResolution => "Resolution: {}",
        VariablesKey::InfoSize => "Size: {} MB",
        VariablesKey::InfoAttributes => "Attributes (.zattrs)",
        VariablesKey::Download => "Download",
        VariablesKey::DownloadCells => "{} ({} cells)",
        VariablesKey::LargeSelectionPyramid => {
            "Large 2D selection ({} cells, {} MB): Automatic multi-resolution pyramid aggregation is enabled."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "3D Globe & 3D Surface meshes are disabled for this large selection ({} cells, {} MB). 2D Plane and 1D Line plots remain fully active."
        }
        VariablesKey::LargeVolumeDisabled => {
            "3D Volume & Point Cloud are disabled for this selection: volume size ({} MB) exceeds the {} MB GPU storage buffer limit. 2D Plane, 1D Line, and 3D Globe remain active."
        }
        VariablesKey::RoleNone => "None",
        VariablesKey::RoleAnimated => "Animated",
        VariablesKey::RoleGrid => "Grid",
        VariablesKey::Index => "Index:",
        VariablesKey::ChannelN => "Channel {}",
    }
}
