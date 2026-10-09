//! Icon catalogue: the enum, its full list, display names and gallery
//! categories, all generated from one declaration so they cannot drift.

/// Declare every icon once, grouped by gallery category. Generates the
/// [`Icon`] enum plus `Icon::ALL`, `Icon::CATEGORIES`, `name()` and
/// `category()`.
macro_rules! define_icons {
    ($($category:literal => [$($icon:ident),+ $(,)?]),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Icon {
            $($($icon,)+)+
        }

        impl Icon {
            /// Every icon, in gallery order.
            pub const ALL: &'static [Icon] = &[$($(Icon::$icon,)+)+];

            /// Gallery categories, in display order.
            pub const CATEGORIES: &'static [&'static str] = &[$($category,)+];

            /// Identifier of the variant, e.g. `"Globe"`.
            pub fn name(&self) -> &'static str {
                match self {
                    $($(Icon::$icon => stringify!($icon),)+)+
                }
            }

            /// Gallery category the icon belongs to.
            pub fn category(&self) -> &'static str {
                match self {
                    $($(Icon::$icon => $category,)+)+
                }
            }
        }
    };
}

define_icons! {
    "Navigation & Menus" => [
        Globe, Variables, Dimensions, Settings, Cache, Sun, Moon,
    ],
    "Playback & Timeline" => [
        Play, Pause, Stop, StepBackward, StepForward, SeekStart, SeekEnd, Loop, Reset, Gauge,
    ],
    "Plot Types & Colormaps" => [
        PlotPlane, PlotLine, PlotSurface, PlotGlobe, PlotVolume, PlotPointCloud, Colormap,
        Layers,
    ],
    "Data Store & Files" => [
        Dataset, Folder, FolderOpen, VariableDoc, Icechunk, Catalog, Save, Snapshot, DropTray,
        Search, Trash, Clipboard,
    ],
    "Tools & Status Badges" => [
        Scissors, Check, Cross, Lock, Unlock, Bolt, Hourglass, Warning, Info, Bullet,
        ChevronRight, ChevronDown, ChevronUp, Eye, EyeOff, Grip, Orientation,
    ],
}
