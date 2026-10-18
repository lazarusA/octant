//! The landing screen, shown in the plot's place until something is plotted
//! or whenever it is asked for (the logo, `H`).

use super::OctantApp;

impl OctantApp {
    /// Whether a plot has data to draw.
    pub fn has_plot(&self) -> bool {
        let data = &self.layers.base.data;
        data.matrix.is_some() || data.volume.is_some()
    }

    /// Whether the landing screen fills the canvas: asked for, or nothing
    /// is plotted yet.
    pub fn hero_shown(&self) -> bool {
        self.layout.show_hero || !self.has_plot()
    }

    /// Shows the landing screen, or the plot again.
    pub fn toggle_hero(&mut self) {
        self.layout.show_hero = !self.layout.show_hero;
    }
}
