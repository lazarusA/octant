//! Button items of the bottom bar: one builder shared by measuring and
//! drawing, plus each button's click action.

use super::labels;
use super::layout::BottomBarItem as Item;
use super::timeline::Timeline;
use crate::app::OctantApp;
use crate::ui::icons::{Icon, ToolbarButton};
use crate::ui::toolbar::{BarItem, ItemWidths, SEPARATOR_WIDTH};
use crate::utils::stack_str;

/// Stack buffer for the "N FPS" label.
type FpsBuf = [u8; 16];

const SAVE_HOVER: &str = "Save / Export Figure (Cmd+S for Quick Save, Cmd+Shift+S for Settings)";

/// Items preceded by a separator, opening a new group.
fn opens_group(item: Item) -> bool {
    matches!(item, Item::DateInfo | Item::Crop)
}

fn fps_label<'a>(buf: &'a mut FpsBuf, app: &OctantApp) -> &'a str {
    let fps = app.playback_fps.round() as u32;
    stack_str(buf, format_args!("{fps} FPS"))
}

/// Toolbar button for a button item, or `None` for text items. Step buttons
/// are always icon-only; the rest follow `compact`.
fn button<'a>(
    item: Item,
    app: &OctantApp,
    fps: &'a str,
    compact: bool,
) -> Option<ToolbarButton<'a>> {
    let icon_only = |icon, label| Some(ToolbarButton::new(icon, label).compact(true));
    let labelled = |icon, label| Some(ToolbarButton::new(icon, label).compact(compact));
    match item {
        Item::First => icon_only(Icon::SeekStart, "First Step"),
        Item::Prev => icon_only(Icon::StepBackward, "Previous Step"),
        Item::Next => icon_only(Icon::StepForward, "Next Step"),
        Item::Last => icon_only(Icon::SeekEnd, "Last Step"),
        Item::PlayPause if app.is_playing => labelled(Icon::Pause, "Pause"),
        Item::PlayPause => labelled(Icon::Play, "Play"),
        Item::Loop => labelled(Icon::Loop, "Loop").map(|b| b.toggled(app.loop_playback)),
        Item::Crop => labelled(Icon::Scissors, "Crop").map(|b| {
            b.toggled(app.show_crop_overlay)
                .hover("Crop Guiding Lines (C)")
        }),
        // Collapsed, the FPS value itself becomes the tooltip.
        Item::Fps => labelled(Icon::Gauge, fps).map(|b| {
            let b = b.owns_popup();
            if compact {
                b
            } else {
                b.hover("Playback Speed")
            }
        }),
        Item::Save => labelled(Icon::Snapshot, "Save").map(|b| b.hover(SAVE_HOVER)),
        Item::DateInfo | Item::StartBadge | Item::StepSize | Item::EndBadge => None,
    }
}

/// The "N FPS" label for the FPS item; other items need no label buffer.
fn label_for<'a>(buf: &'a mut FpsBuf, item: Item, app: &OctantApp) -> &'a str {
    if item == Item::Fps {
        fps_label(buf, app)
    } else {
        ""
    }
}

/// Widths of `item` in both modes, including spacing and any leading
/// separator. The only place that zeroes the width of items that hide when
/// compact; [`show_item`] skips drawing them with the same predicate.
pub(super) fn item_widths(item: Item, app: &OctantApp, tl: &Timeline, ui: &egui::Ui) -> ItemWidths {
    let spacing = ui.spacing().item_spacing.x;
    let lead = if opens_group(item) {
        SEPARATOR_WIDTH + spacing
    } else {
        0.0
    };
    let mut buf = FpsBuf::default();
    let fps = label_for(&mut buf, item, app);
    let (full, compact) = match button(item, app, fps, false) {
        Some(full) => {
            let compact = button(item, app, fps, true).map_or(0.0, |b| b.width(ui));
            (full.width(ui), compact)
        }
        None => labels::widths(item, tl, ui),
    };
    ItemWidths {
        full: full + spacing + lead,
        compact: if item.hides_when_compact() {
            0.0
        } else {
            compact + spacing + lead
        },
    }
}

/// Draw `item` in full or compact mode and run its action when clicked.
pub(super) fn show_item(
    item: Item,
    compact: bool,
    app: &mut OctantApp,
    tl: &Timeline,
    ui: &mut egui::Ui,
) {
    if compact && item.hides_when_compact() {
        return;
    }
    if opens_group(item) {
        ui.separator();
    }
    // A fixed id per item, so items hiding elsewhere in the bar never shift
    // this one's auto ids (which would drop drags or close its popup).
    ui.push_id(("bottom_bar_item", item.index()), |ui| {
        let mut buf = FpsBuf::default();
        let fps = label_for(&mut buf, item, app);
        match button(item, app, fps, compact) {
            Some(btn) => {
                let response = ui.add(btn);
                on_click(item, &response, app, tl);
            }
            None => labels::show(item, compact, tl, app, ui),
        }
    });
}

fn on_click(item: Item, response: &egui::Response, app: &mut OctantApp, tl: &Timeline) {
    if item == Item::Fps {
        egui::Popup::from_toggle_button_response(response)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.label(egui::RichText::new("Playback Speed").strong());
                ui.add(egui::Slider::new(&mut app.playback_fps, 1.0..=60.0).suffix(" FPS"));
            });
        return;
    }
    if !response.clicked() {
        return;
    }
    match item {
        Item::First => app.request_step_or_load(0),
        Item::Prev => app.step_prev(),
        Item::Next => app.step_next(),
        Item::Last => app.request_step_or_load(tl.last_step),
        Item::PlayPause => {
            app.is_playing = !app.is_playing;
            app.last_step_time = web_time::Instant::now();
            if app.is_playing
                && let Some(var) = app.effective_variable_info()
            {
                let shape = var.shape.clone();
                app.prefetch_selected_animated_range(&shape);
            }
        }
        Item::Loop => app.loop_playback = !app.loop_playback,
        Item::Crop => app.show_crop_overlay = !app.show_crop_overlay,
        Item::Save => app.show_export_modal = true,
        Item::Fps | Item::DateInfo | Item::StartBadge | Item::StepSize | Item::EndBadge => {}
    }
}
