//! Translation keys of the colormap area.

crate::i18n::keys::define_keys! {
    ColormapKey {
    // Colormap kinds
    Sequential,
    Diverging,
    Cyclic,
    Categorical,
    Other,
    // Custom colormap interpolation and blend spaces
    InterpolationLinear,
    BlendLinearRgb,
    // Picker: kind and family filters
    FilterAll,
    FamilyCustom,
    FamilyTitle,
    // Picker: active row, search and list
    Smooth,
    SmoothHover,
    Reversed,
    SearchHint,
    NoMatches,
    // Picker and colorbar: colorbar options
    ShowColorbar,
    ChangeColormapHover,
    ColorSettingsHover,
    BoxTransparency,
    // Custom colormap editor
    CustomColormap,
    Name,
    Colors,
    ColorsHover,
    Interpolation,
    Blend,
    Classes,
    ClassesHover,
    RenameAndApply,
    SaveAndApply,
    SaveAsNew,
    NameTaken,
    NameRequired,
    Saved,
    BrokenHover,
    Edit,
    Delete,
    DeleteHover,
    // Colorbar
    FlipOrientation,
    RangeMinHover,
    RangeMaxHover,
    LowClipTitle,
    LowClipHover,
    HighClipTitle,
    HighClipHover,
    HoveredValue,
    TitleHover,
    // Color picker popup
    ColorPicker,
    Copy,
    CopyHover,
    // Alpha curve errors
    AlphaNotANumber,
    AlphaOutOfRange,
    AlphaMixedForms,
    AlphaUnsorted,
    // Notices
    PrefsUnreadable,
    PrefsBackedUp,
    PrefsNotBackedUp,
    BackupsFull,
    CustomNotLoaded,
    CustomDoesNotBuild,
    NotRegistered,
    BundledUnreadable,
    FallingBackToGray,
    }
}
