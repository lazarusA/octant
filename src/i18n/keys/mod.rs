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
pub use notices::NoticesKey;
pub use playback::PlaybackKey;
pub use plot::PlotKey;
pub use plot_options::PlotOptionsKey;
pub use settings::SettingsKey;
pub use variables::VariablesKey;

/// Any translatable text, by area.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
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
    About(AboutKey),
    Notices(NoticesKey),
}

impl From<CommonKey> for Key {
    fn from(key: CommonKey) -> Self {
        Key::Common(key)
    }
}

impl From<DatasetKey> for Key {
    fn from(key: DatasetKey) -> Self {
        Key::Dataset(key)
    }
}

impl From<VariablesKey> for Key {
    fn from(key: VariablesKey) -> Self {
        Key::Variables(key)
    }
}

impl From<PlaybackKey> for Key {
    fn from(key: PlaybackKey) -> Self {
        Key::Playback(key)
    }
}

impl From<PlotKey> for Key {
    fn from(key: PlotKey) -> Self {
        Key::Plot(key)
    }
}

impl From<PlotOptionsKey> for Key {
    fn from(key: PlotOptionsKey) -> Self {
        Key::PlotOptions(key)
    }
}

impl From<SettingsKey> for Key {
    fn from(key: SettingsKey) -> Self {
        Key::Settings(key)
    }
}

impl From<ColormapKey> for Key {
    fn from(key: ColormapKey) -> Self {
        Key::Colormap(key)
    }
}

impl From<ExportKey> for Key {
    fn from(key: ExportKey) -> Self {
        Key::Export(key)
    }
}

impl From<HoverKey> for Key {
    fn from(key: HoverKey) -> Self {
        Key::Hover(key)
    }
}

impl From<AboutKey> for Key {
    fn from(key: AboutKey) -> Self {
        Key::About(key)
    }
}

impl From<NoticesKey> for Key {
    fn from(key: NoticesKey) -> Self {
        Key::Notices(key)
    }
}

impl Key {
    /// Every key of every area.
    pub fn all() -> impl Iterator<Item = Key> {
        CommonKey::ALL
            .iter()
            .copied()
            .map(Key::from)
            .chain(DatasetKey::ALL.iter().copied().map(Key::from))
            .chain(VariablesKey::ALL.iter().copied().map(Key::from))
            .chain(PlaybackKey::ALL.iter().copied().map(Key::from))
            .chain(PlotKey::ALL.iter().copied().map(Key::from))
            .chain(PlotOptionsKey::ALL.iter().copied().map(Key::from))
            .chain(SettingsKey::ALL.iter().copied().map(Key::from))
            .chain(ColormapKey::ALL.iter().copied().map(Key::from))
            .chain(ExportKey::ALL.iter().copied().map(Key::from))
            .chain(HoverKey::ALL.iter().copied().map(Key::from))
            .chain(AboutKey::ALL.iter().copied().map(Key::from))
            .chain(NoticesKey::ALL.iter().copied().map(Key::from))
    }
}
