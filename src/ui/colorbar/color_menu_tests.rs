//! The colorbar's Color window keeps its width in every language, for every
//! plot type whose base layer has a colorbar, with every optional row
//! showing.

use super::{can_show, color_menu, show_colorbar_overlay};
use crate::app::layers::LayerId;
use crate::i18n::Language;
use crate::ui::panel_layout::PANEL_W;
use crate::ui::panel_width_tests::{
    PLOT_TYPES, SCREEN, SLACK, area_width_of, frame_margins, plotted,
};
use egui::Ui;

#[test]
fn color_window_keeps_its_width_in_every_language() {
    let limit = PANEL_W + frame_margins() + SLACK;
    let mut wide = Vec::new();
    for plot_type in PLOT_TYPES {
        for &lang in Language::ALL {
            let mut app = plotted(plot_type);
            if !can_show(&app, LayerId::BASE) {
                continue;
            }
            let mut run = |ui: &mut Ui| {
                color_menu::set_open(ui.ctx(), LayerId::BASE, true);
                show_colorbar_overlay(&mut app, ui.ctx(), SCREEN);
            };
            let id = color_menu::area_id(LayerId::BASE);
            let width = area_width_of(lang, id, &[], &mut run);
            assert!(width >= PANEL_W, "{plot_type:?} {lang:?}: drawn, {width}");
            if width > limit {
                wide.push(format!("{plot_type:?} {lang:?}: {width:.0}"));
            }
        }
    }
    assert!(
        wide.is_empty(),
        "Color window wider than {limit}: {}",
        wide.join(", ")
    );
}
