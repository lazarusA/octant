//! egui layout, panel toggles, theme, and modal window states.

#[derive(Clone, Debug)]
pub struct UiLayoutState {
    pub show_left_panel: bool,
    pub show_hero: bool,
    pub hero_state: crate::ui::hero::HeroState,
    pub show_variables_overlay: bool,
    pub show_settings_panel: bool,
    pub show_variable_controls: bool,
    /// The Dimensions panel's Add Overlay toggle.
    pub plot_as_overlay: bool,
    /// Opens the Settings' Layers menu on the next frame.
    pub reveal_layers_menu: bool,
    pub variables_overlay_width: f32,
    pub variable_search: String,
    pub show_bottom_bar: bool,
    pub show_hover_card: bool,
    pub settings_overlay_width: f32,
    pub panel_positions: crate::ui::panel_layout::PanelPositions,
    pub theme_preference: egui::ThemePreference,
    pub enforce_data_aspect_ratio: bool,
    pub show_colorbar: bool,
    pub colorbar_transparency: f32,

    // Modal windows
    pub show_catalog_window: bool,
    pub show_about_window: bool,
    pub show_icon_gallery_window: bool,
    pub catalog_search_query: String,
    pub catalog_category_filter: crate::catalog::CatalogCategoryFilter,
}

impl Default for UiLayoutState {
    fn default() -> Self {
        Self {
            show_left_panel: false,
            show_hero: true,
            hero_state: crate::ui::hero::HeroState::default(),
            show_variables_overlay: false,
            show_settings_panel: false,
            show_variable_controls: false,
            plot_as_overlay: false,
            reveal_layers_menu: false,
            variables_overlay_width: 340.0,
            variable_search: String::new(),
            show_bottom_bar: true,
            show_hover_card: true,
            settings_overlay_width: 0.0,
            panel_positions: Default::default(),
            theme_preference: egui::ThemePreference::System,
            enforce_data_aspect_ratio: true,
            show_colorbar: true,
            colorbar_transparency: 0.0,
            show_catalog_window: false,
            show_about_window: false,
            show_icon_gallery_window: false,
            catalog_search_query: String::new(),
            catalog_category_filter: crate::catalog::CatalogCategoryFilter::All,
        }
    }
}

impl UiLayoutState {
    /// Opens the Settings panel and closes Store, Variables, Controls, and Catalog.
    pub fn open_only_settings_panel(&mut self) {
        self.show_settings_panel = true;
        self.show_left_panel = false;
        self.show_variables_overlay = false;
        self.show_variable_controls = false;
        self.show_catalog_window = false;
        self.show_about_window = false;
        self.show_icon_gallery_window = false;
    }
}

impl super::app_state::OctantApp {
    /// Opens the Settings panel and closes Store, Variables, Controls, and Catalog.
    pub fn open_only_settings_panel(&mut self) {
        self.layout.open_only_settings_panel();
    }
}
