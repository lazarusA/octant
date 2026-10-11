//! Canonical English translations for Octant.

use crate::i18n::keys::Key;

pub fn translate(key: Key) -> &'static str {
    match key {
        // Navigation & Top Bar
        Key::Dataset => "Dataset",
        Key::Variables => "Variables",
        Key::Dimensions => "Dimensions",
        Key::Plot => "Plot",
        Key::Colormaps => "Colormaps",
        Key::Settings => "Settings",
        Key::About => "About",

        // Dataset Panel & Actions
        Key::InspectMetadata => "Inspect Store Metadata",
        Key::ActiveDatasets => "Active Datasets",
        Key::NoActiveDatasets => "No active datasets. Open a dataset to begin.",
        Key::CacheUsage => "Cache Usage",
        Key::ClearCache => "Clear Cache",
        Key::OpenLocal => "Open Local File",
        Key::CloudStore => "Cloud Object Store",
        Key::SampleDatasets => "Sample Datasets",
        Key::DropFilesHere => "Drop files or folders here",
        Key::SourceUrlOrPath => "Store URI or local directory",
        Key::Inspect => "Inspect",
        Key::LoadingMetadata => "Inspecting store metadata in background...",

        // Variables Overlay
        Key::Group => "Group:",
        Key::DataType => "Type",
        Key::Shape => "Shape",
        Key::Description => "Description",
        Key::SearchVariables => "Search variables...",
        Key::NoVariablesMatching => "No variables matching query",
        Key::ShowingItemsOfTotal => "Showing items of total",

        // Common Actions & Status
        Key::Ready => "Ready",
        Key::Loading => "Loading...",
        Key::Close => "Close",
        Key::Cancel => "Cancel",
        Key::Save => "Save",
        Key::Reset => "Reset",
        Key::Error => "Error",

        // Playback & Timeline
        Key::Play => "Play",
        Key::Pause => "Pause",
        Key::StepForward => "Step Forward",
        Key::StepBackward => "Step Backward",
        Key::Speed => "Speed",
        Key::Loop => "Loop",

        // About Dialog
        Key::AboutSubtitle => "N-Dimensional Data Explorer",
        Key::AboutDescription => {
            "Octant is an interactive viewer for N-dimensional datasets with native support for local and cloud object storage, Zarr (v2/v3), and Icechunk. Built in Rust with GPU-accelerated rendering via WGPU. Octant runs natively on macOS, Linux, and Windows."
        }
        Key::AboutHyperslabTitle => "Hyperslab Slicing",
        Key::AboutHyperslabDesc => "Async LRU chunk cache & multi-resolution pyramids.",
        Key::AboutStorageTitle => "Zarr & Icechunk Native",
        Key::AboutStorageDesc => "Local, S3, GCS, Azure, and HTTP streaming backends.",
        Key::AboutShadersTitle => "Hardware-Accelerated WGPU Shaders",
        Key::AboutShadersDesc => {
            "2D Flatmaps, 3D Spheres, Elevation Surfaces, Volumes & 1D Profiles."
        }
        Key::AboutLinksHeading => "Links & Resources",
        Key::AboutGoodFaithNotice => {
            "Octant credits third-party work in good faith. If you believe something is missing or incorrectly attributed, please let us know and we will correct or remove it."
        }
        Key::AboutBrowseIcons => "Browse Native Vector Icons",
    }
}
