//! Spanish translations, one file per UI area.

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

use crate::i18n::keys::Key;

pub fn text(key: Key) -> Option<&'static str> {
    match key {
        Key::Common(key) => common::text(key),
        Key::Dataset(key) => dataset::text(key),
        Key::Variables(key) => variables::text(key),
        Key::Playback(key) => playback::text(key),
        Key::Plot(key) => plot::text(key),
        Key::PlotOptions(key) => plot_options::text(key),
        Key::Settings(key) => settings::text(key),
        Key::Colormap(key) => colormap::text(key),
        Key::Export(key) => export::text(key),
        Key::Hover(key) => hover::text(key),
        Key::Keymap(key) => keymap::text(key),
        Key::About(key) => about::text(key),
        Key::Notices(key) => notices::text(key),
    }
}
