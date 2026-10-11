use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> &'static str {
    match key {
        SettingsKey::ReasonComposite => "Colors come from the composite channels.",
        SettingsKey::ReasonCustomColor => "Custom Color draws every point in one color.",
        SettingsKey::ReasonAllSeries => "All Lines Series colors each line by its index.",
        SettingsKey::ReasonIndexed => "Indexed RGBA colors voxels by their label index.",
        SettingsKey::ReasonIndexedMissing => "Indexed RGBA skips missing voxels.",
        SettingsKey::ReasonClassicOpaque => {
            "Only Volume Raymarching (DVR) draws translucent colors."
        }
        SettingsKey::ReasonNotTranslucent => "Takes effect with Opacity below 1 or an Alpha curve.",
        SettingsKey::ReasonCompositeOpaque => "Composites are drawn opaque.",
        SettingsKey::ReasonLabelOpaque => "Categorical Label Surface is always drawn opaque.",
        SettingsKey::ReasonAlwaysBlends => "This algorithm always blends along the ray.",
        SettingsKey::CloseSettingsHover => "Close Settings",
        SettingsKey::SectionOverlays => "Overlays",
        SettingsKey::SectionResolution => "Resolution",
        SettingsKey::SectionView => "View",
        SettingsKey::Layers => "Layers",
        SettingsKey::BaseLayerHover => "Base layer: decides the canvas, axes and view",
        SettingsKey::BaseTag => "base",
        SettingsKey::Color => "Color",
        SettingsKey::HideLayerHover => "Hide layer",
        SettingsKey::ShowLayerHover => "Show layer",
        SettingsKey::RemoveLayerHover => "Remove layer",
        SettingsKey::MoveDownHover => "Move down",
        SettingsKey::MoveUpHover => "Move up",
        SettingsKey::NoVariable => "(no variable)",
        SettingsKey::ColorbarLabel => "Colorbar Label",
        SettingsKey::ResetLabelHover => "Reset colorbar label to default",
        SettingsKey::ColorRange => "Color Range",
        SettingsKey::Categorical => "Categorical",
        SettingsKey::CategoricalHover => {
            "Discrete colorbar (auto-detects unique values or 10 equal bins)."
        }
        SettingsKey::LockBoundsHover => {
            "Lock min/max so color mapping stays fixed across timesteps."
        }
        SettingsKey::ResetBoundsHover => {
            "Reset bounds to current slice/dataset min and max defaults"
        }
        SettingsKey::ResetAllColorbar => "Reset All Colorbar Defaults",
        SettingsKey::ResetAllColorbarHover => {
            "Reset both colorbar label and range to default values"
        }
        SettingsKey::RangeMin => "Min:",
        SettingsKey::RangeMax => "Max:",
        SettingsKey::NanColor => "NaN Color",
        SettingsKey::NanColorHover => "If unchecked, NaN/Inf values render transparently.",
        SettingsKey::NanColorPickerHover => "NaN color. Click to select color.",
        SettingsKey::LowClip => "Low Clip",
        SettingsKey::LowClipHover => "Values < cmin clipped to this color.",
        SettingsKey::LowClipPickerHover => "Low Clip color (< Min). Click to select color.",
        SettingsKey::HighClip => "High Clip",
        SettingsKey::HighClipHover => "Values > cmax clipped to this color.",
        SettingsKey::HighClipPickerHover => "High Clip color (> Max). Click to select color.",
        SettingsKey::Scale => "Scale",
        SettingsKey::ScaleLinear => "Linear",
        SettingsKey::ScaleLog => "Logarithmic",
        SettingsKey::ScaleSymlog => "Symlog",
        SettingsKey::ScaleSqrt => "Sqrt",
        SettingsKey::ScaleExponential => "Exponential",
        SettingsKey::ScaleLogHover => "Log scale (non-negative data)",
        SettingsKey::ScaleLogDisabledHover => {
            "Disabled: requires min >= 0. Use Symlog for negative data."
        }
        SettingsKey::ScaleParam => "Param:",
        SettingsKey::Opacity => "Opacity",
        SettingsKey::OpacityHover => {
            "Opacity of colormapped values. NaN and clip colors keep their own alpha. \
             Double-click to reset."
        }
        SettingsKey::AlphaCurve => "Alpha curve",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  or  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "Alpha over the colorbar, from its start to its end. Values are evenly \
             spaced (Step: one equal bin each); position:alpha stops are placed \
             along the bar. Empty for none."
        }
        SettingsKey::ClearCurveHover => "Clear curve",
        SettingsKey::Presets => "Presets",
        SettingsKey::InterpStep => "Step",
        SettingsKey::InterpLinear => "Linear",
        SettingsKey::PresetRampUp => "Ramp up",
        SettingsKey::PresetRampDown => "Ramp down",
        SettingsKey::PresetPeak => "Peak",
        SettingsKey::PresetSink => "Sink",
        SettingsKey::PresetHideLow => "Hide low",
        SettingsKey::PresetHideCenter => "Hide center",
        SettingsKey::Transparency => "Transparency",
        SettingsKey::TransparencyHover => {
            "With translucent colors (Opacity or Alpha curve), draw every layer \
             instead of letting the nearest one hide those behind it."
        }
        SettingsKey::Lighting => "Lighting",
        SettingsKey::LightingHover => {
            "Shade samples by their gradient so fronts and edges gain shape"
        }
        SettingsKey::Coastlines => "Coastlines",
        SettingsKey::CoastlinesHover => "Overlay Natural Earth coastlines on geographic plots.",
        SettingsKey::ResetCoastlineColor => "reset",
        SettingsKey::ResetCoastlineColorHover => "Reset coastline color to theme default",
        SettingsKey::CoastlineColorHover => "Coastline line color. Click to select.",
        SettingsKey::CoastlineResolution => "Resolution:",
        SettingsKey::Lod110mHover => "110 m - fast, always available (embedded)",
        SettingsKey::Lod50mHover => "50 m - medium detail (loaded in background)",
        SettingsKey::Lod10mHover => "10 m - high detail (loaded in background)",
        SettingsKey::LineWidth => "Line Width",
        SettingsKey::CropToDomain => "Crop to data domain",
        SettingsKey::CropToDomainHover => {
            "Clip coastlines strictly to the spatial boundary of the active dataset."
        }
        SettingsKey::AutoRotate => "Auto Rotate",
        SettingsKey::ResetView => "Reset View",
        SettingsKey::ResetViewHover => "Reset 3D camera orientation",
        SettingsKey::HoverCard => "Hover Card",
        SettingsKey::HideHoverCardHover => "Hide hover card",
        SettingsKey::ShowHoverCardHover => "Show hover card",
        SettingsKey::Export => "Export",
        SettingsKey::ExportFormat => "Format:",
        SettingsKey::ExportFolder => "Folder:",
        SettingsKey::OpenSaveDialog => "Open Save Dialog",
        SettingsKey::CropTool => "Crop Tool",
        SettingsKey::Aggregation => "2D Aggregation",
        SettingsKey::AggregationMandatoryHover => {
            "Mandatory: Array exceeds single GPU buffer limit (128 MB / 33.5M cells). \
             Aggregation cannot be disabled."
        }
        SettingsKey::AggregationHover => {
            "Multi-resolution pyramid downsampling and viewport resampling."
        }
        SettingsKey::AggregationMode => "Mode:",
        SettingsKey::AggMean => "Mean (Box Filter)",
        SettingsKey::AggMax => "Max (Peak Preserve)",
        SettingsKey::AggMin => "Min (Trough Preserve)",
        SettingsKey::AggNearest => "Nearest Neighbor",
        SettingsKey::AggMeanHover => "Averages sub-pixels in 2x2 cells, ignoring NaNs.",
        SettingsKey::AggMaxHover => "Preserves extreme high values.",
        SettingsKey::AggMinHover => "Preserves extreme low values.",
        SettingsKey::AggNearestHover => "Fast nearest point selection without interpolation.",
        SettingsKey::CmykAutoMapped => "Auto-mapped channels: C (1), M (2), Y (3), K (4)",
        SettingsKey::MultiChannelOverlay => "Multi-Channel Overlay",
        SettingsKey::MultiChannelOverlayHover => {
            "Overlays multiple channels additively, each rendered with its unique color tint."
        }
        SettingsKey::CmykComposite => "CMYK Composite",
        SettingsKey::CmykCompositeHover => {
            "Composites 4-channel Cyan, Magenta, Yellow, Black (CMYK) into Truecolor RGB."
        }
        SettingsKey::RgbComposite => "RGB Composite",
        SettingsKey::RgbCompositeHover => "Composites selected 3 channels into Truecolor RGB.",
        SettingsKey::Channels => "Channels:",
        SettingsKey::ChannelsAll => "All",
        SettingsKey::ChannelsNone => "None",
        SettingsKey::ChannelTintHover => "Click to customize channel tint color",
        SettingsKey::BandNumber => "Band {}",
    }
}
