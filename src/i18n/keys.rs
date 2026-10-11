//! Typed translation keys for Octant UI elements.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    // Navigation & Top Bar
    Dataset,
    Variables,
    Dimensions,
    Plot,
    Colormaps,
    Settings,
    About,

    // Dataset Panel & Actions
    InspectMetadata,
    ActiveDatasets,
    NoActiveDatasets,
    CacheUsage,
    ClearCache,
    OpenLocal,
    CloudStore,
    SampleDatasets,
    DropFilesHere,
    SourceUrlOrPath,
    Inspect,
    LoadingMetadata,

    // Variables Overlay
    Group,
    DataType,
    Shape,
    Description,
    SearchVariables,
    NoVariablesMatching,
    ShowingItemsOfTotal,

    // Common Actions & Status
    Ready,
    Loading,
    Close,
    Cancel,
    Save,
    Reset,
    Error,

    // Playback & Timeline
    Play,
    Pause,
    StepForward,
    StepBackward,
    Speed,
    Loop,

    // About Dialog
    AboutSubtitle,
    AboutDescription,
    AboutHyperslabTitle,
    AboutHyperslabDesc,
    AboutStorageTitle,
    AboutStorageDesc,
    AboutShadersTitle,
    AboutShadersDesc,
    AboutLinksHeading,
    AboutGoodFaithNotice,
    AboutBrowseIcons,
}
