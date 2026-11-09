//! Typed translation keys for Octant UI elements, one enum per UI area.
//!
//! Keys whose English text holds `{}` placeholders are templates, filled with
//! [`crate::i18n::fill`]; every translation keeps the same number of them.

/// Declares an area's key enum and its `ALL` list.
macro_rules! define_keys {
    ($area:ident { $($name:ident),* $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $area {
            $($name),*
        }

        impl $area {
            /// Every key of the area, in declaration order.
            pub const ALL: &'static [$area] = &[$($area::$name),*];
        }
    };
}
pub(crate) use define_keys;

mod about;
mod colormap;
mod common;
mod dataset;
mod export;
mod hover;
mod keymap;
mod notices;
mod playback;
mod plot;
mod plot_options;
mod settings;
mod variables;

pub use about::AboutKey;
pub use colormap::ColormapKey;
pub use common::CommonKey;
pub use dataset::DatasetKey;
pub use export::ExportKey;
pub use hover::HoverKey;
pub use keymap::KeymapKey;
pub use notices::NoticesKey;
pub use playback::PlaybackKey;
pub use plot::PlotKey;
pub use plot_options::PlotOptionsKey;
pub use settings::SettingsKey;
pub use variables::VariablesKey;

/// Declares `Key`, one variant per area wrapping that area's key enum, each
/// enum's `From` into `Key`, and `Key::all`.
macro_rules! key_areas {
    ($($area:ident($keys:ident)),* $(,)?) => {
        /// Any translatable text, by area.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Key {
            $($area($keys)),*
        }

        $(impl From<$keys> for Key {
            fn from(key: $keys) -> Self {
                Key::$area(key)
            }
        })*

        impl Key {
            /// Every key of every area.
            pub fn all() -> impl Iterator<Item = Key> {
                std::iter::empty()$(.chain($keys::ALL.iter().copied().map(Key::from)))*
            }
        }
    };
}

key_areas! {
    Common(CommonKey),
    Dataset(DatasetKey),
    Variables(VariablesKey),
    Playback(PlaybackKey),
    Plot(PlotKey),
    PlotOptions(PlotOptionsKey),
    Settings(SettingsKey),
    Colormap(ColormapKey),
    Export(ExportKey),
    Hover(HoverKey),
    Keymap(KeymapKey),
    About(AboutKey),
    Notices(NoticesKey),
}
