//! Icon catalogue: the full list, display names and gallery categories.

use super::Icon;

impl Icon {
    pub const ALL: &'static [Icon] = &[
        // Nav
        Icon::Globe,
        Icon::Variables,
        Icon::Dimensions,
        Icon::Settings,
        Icon::Cache,
        Icon::Sun,
        Icon::Moon,
        Icon::Overflow,
        // Playback
        Icon::Play,
        Icon::Pause,
        Icon::Stop,
        Icon::StepBackward,
        Icon::StepForward,
        Icon::SeekStart,
        Icon::SeekEnd,
        Icon::Loop,
        Icon::Reset,
        // Plots
        Icon::PlotPlane,
        Icon::PlotLine,
        Icon::PlotSurface,
        Icon::PlotGlobe,
        Icon::PlotVolume,
        Icon::PlotPointCloud,
        Icon::Colormap,
        // Store
        Icon::Dataset,
        Icon::Folder,
        Icon::FolderOpen,
        Icon::VariableDoc,
        Icon::Icechunk,
        Icon::Catalog,
        Icon::Save,
        Icon::Snapshot,
        Icon::DropTray,
        Icon::Search,
        Icon::Trash,
        Icon::Clipboard,
        // Status
        Icon::Scissors,
        Icon::Check,
        Icon::Cross,
        Icon::Lock,
        Icon::Unlock,
        Icon::Bolt,
        Icon::Hourglass,
        Icon::Warning,
        Icon::Info,
        Icon::Bullet,
        Icon::ChevronRight,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Icon::Globe => "Globe",
            Icon::Variables => "Variables",
            Icon::Dimensions => "Dimensions",
            Icon::Settings => "Settings",
            Icon::Cache => "Cache",
            Icon::Sun => "Sun",
            Icon::Moon => "Moon",
            Icon::Overflow => "Overflow",
            Icon::Play => "Play",
            Icon::Pause => "Pause",
            Icon::Stop => "Stop",
            Icon::StepBackward => "StepBackward",
            Icon::StepForward => "StepForward",
            Icon::SeekStart => "SeekStart",
            Icon::SeekEnd => "SeekEnd",
            Icon::Loop => "Loop",
            Icon::Reset => "Reset",
            Icon::PlotPlane => "PlotPlane",
            Icon::PlotLine => "PlotLine",
            Icon::PlotSurface => "PlotSurface",
            Icon::PlotGlobe => "PlotGlobe",
            Icon::PlotVolume => "PlotVolume",
            Icon::PlotPointCloud => "PlotPointCloud",
            Icon::Colormap => "Colormap",
            Icon::Dataset => "Dataset",
            Icon::Folder => "Folder",
            Icon::FolderOpen => "FolderOpen",
            Icon::VariableDoc => "VariableDoc",
            Icon::Icechunk => "Icechunk",
            Icon::Catalog => "Catalog",
            Icon::Save => "Save",
            Icon::Snapshot => "Snapshot",
            Icon::DropTray => "DropTray",
            Icon::Search => "Search",
            Icon::Trash => "Trash",
            Icon::Clipboard => "Clipboard",
            Icon::Scissors => "Scissors",
            Icon::Check => "Check",
            Icon::Cross => "Cross",
            Icon::Lock => "Lock",
            Icon::Unlock => "Unlock",
            Icon::Bolt => "Bolt",
            Icon::Hourglass => "Hourglass",
            Icon::Warning => "Warning",
            Icon::Info => "Info",
            Icon::Bullet => "Bullet",
            Icon::ChevronRight => "ChevronRight",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Icon::Globe
            | Icon::Variables
            | Icon::Dimensions
            | Icon::Settings
            | Icon::Cache
            | Icon::Sun
            | Icon::Moon
            | Icon::Overflow => "Navigation & Menus",

            Icon::Play
            | Icon::Pause
            | Icon::Stop
            | Icon::StepBackward
            | Icon::StepForward
            | Icon::SeekStart
            | Icon::SeekEnd
            | Icon::Loop
            | Icon::Reset => "Playback & Timeline",

            Icon::PlotPlane
            | Icon::PlotLine
            | Icon::PlotSurface
            | Icon::PlotGlobe
            | Icon::PlotVolume
            | Icon::PlotPointCloud
            | Icon::Colormap => "Plot Types & Colormaps",

            Icon::Dataset
            | Icon::Folder
            | Icon::FolderOpen
            | Icon::VariableDoc
            | Icon::Icechunk
            | Icon::Catalog
            | Icon::Save
            | Icon::Snapshot
            | Icon::DropTray
            | Icon::Search
            | Icon::Trash
            | Icon::Clipboard => "Data Store & Files",

            Icon::Scissors
            | Icon::Check
            | Icon::Cross
            | Icon::Lock
            | Icon::Unlock
            | Icon::Bolt
            | Icon::Hourglass
            | Icon::Warning
            | Icon::Info
            | Icon::Bullet
            | Icon::ChevronRight => "Tools & Status Badges",
        }
    }
}
