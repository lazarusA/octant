//! Button items of the bottom bar: one builder shared by measuring and
//! drawing, plus each button's click action.

use super::labels;
use super::layout::BottomBarItem as Item;
use super::timeline::Timeline;
use crate::app::OctantApp;
use crate::i18n::{CommonKey, PlaybackKey as K, tr};
use crate::ui::icons::{Icon, ToolbarButton};
use crate::ui::number_input::LocalizedInput;
use crate::ui::toolbar::roving::{BarNav, Stop};
use crate::ui::toolbar::{BarItem, ItemWidths, SEPARATOR_WIDTH};
use crate::utils::stack_str;

/// Stack buffer for the "N FPS" and buffering labels.
pub(super) type LabelBuf = [u8; 64];

/// Items preceded by a separator, opening a new group.
fn opens_group(item: Item) -> bool {
    matches!(item, Item::DateInfo | Item::Crop)
}

fn fps_label<'a>(buf: &'a mut LabelBuf, app: &OctantApp) -> &'a str {
    let fps = app.playback.playback_fps.round() as u32;
    stack_str(buf, format_args!("{fps} FPS"))
}

/// Toolbar button for a button item, or `None` for text items. Step buttons
/// are always icon-only; the rest follow `compact`.
fn button<'a>(
    item: Item,
    app: &OctantApp,
    label: &'a str,
    compact: bool,
) -> Option<ToolbarButton<'a>> {
    let icon_only = |icon, label| Some(ToolbarButton::new(icon, label).compact(true));
    let labelled = |icon, label| Some(ToolbarButton::new(icon, label).compact(compact));
    match item {
        Item::First => icon_only(Icon::SeekStart, tr(K::FirstStep)),
        Item::Prev => icon_only(Icon::StepBackward, tr(K::PreviousStep)),
        Item::Next => icon_only(Icon::StepForward, tr(K::NextStep)),
        Item::Last => icon_only(Icon::SeekEnd, tr(K::LastStep)),
        // Waiting for the buffer: its progress, and a click still pauses.
        Item::PlayPause if is_buffering(app) => {
            labelled(Icon::Hourglass, label).map(|b| b.hover(tr(K::Pause)))
        }
        Item::PlayPause if app.playback.is_playing => labelled(Icon::Pause, tr(K::Pause)),
        Item::PlayPause => labelled(Icon::Play, tr(K::Play)),
        Item::Loop => {
            labelled(Icon::Loop, tr(K::Loop)).map(|b| b.toggled(app.playback.loop_playback))
        }
        Item::Crop => labelled(Icon::Scissors, tr(K::Crop))
            .map(|b| b.toggled(app.show_crop_overlay).hover(tr(K::CropHover))),
        // Collapsed, the FPS value itself becomes the tooltip.
        Item::Fps => labelled(Icon::Gauge, label).map(|b| {
            let b = b.owns_popup();
            if compact {
                b
            } else {
                b.hover(tr(K::PlaybackSpeed))
            }
        }),
        Item::Save => {
            labelled(Icon::Snapshot, tr(CommonKey::Save)).map(|b| b.hover(tr(K::SaveExportHover)))
        }
        Item::DateInfo | Item::StartBadge | Item::StepSize | Item::EndBadge => None,
    }
}

fn is_buffering(app: &OctantApp) -> bool {
    app.playback.is_playing && app.playback.buffering
}

/// The "N FPS" label of the FPS item and the "Buffering n/m" label of the
/// play button while it buffers; other items need no label buffer.
pub(super) fn label_for<'a>(buf: &'a mut LabelBuf, item: Item, app: &OctantApp) -> &'a str {
    match item {
        Item::Fps => fps_label(buf, app),
        Item::PlayPause if is_buffering(app) => {
            let (ready, wanted) = app.playback.buffer_progress;
            crate::i18n::fill(buf, tr(K::Buffering), &[&ready, &wanted])
        }
        _ => "",
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
    let mut buf: LabelBuf = [0; 64];
    let label = label_for(&mut buf, item, app);
    let (full, compact) = match button(item, app, label, false) {
        Some(full) => {
            let compact = button(item, app, label, true).map_or(0.0, |b| b.width(ui));
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

/// Draw `item` in full or compact mode and run its action when clicked; a
/// button joins `nav`.
pub(super) fn show_item(
    item: Item,
    compact: bool,
    app: &mut OctantApp,
    tl: &Timeline,
    ui: &mut egui::Ui,
    nav: &mut BarNav,
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
        let mut buf: LabelBuf = [0; 64];
        let label = label_for(&mut buf, item, app);
        match button(item, app, label, compact) {
            Some(btn) => {
                let response = ui.add(btn);
                let stop = match item {
                    Item::Fps => Stop::menu(&response),
                    _ => Stop::Button,
                };
                nav.push(&response, stop);
                on_click(item, &response, app);
            }
            None => labels::show(item, compact, tl, app, ui),
        }
    });
}

fn on_click(item: Item, response: &egui::Response, app: &mut OctantApp) {
    if item == Item::Fps {
        egui::Popup::from_toggle_button_response(response)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.label(egui::RichText::new(tr(K::PlaybackSpeed)).strong());
                ui.add(
                    egui::Slider::new(&mut app.playback.playback_fps, 1.0..=60.0)
                        .localized()
                        .suffix(" FPS"),
                );
                ui.label(egui::RichText::new(tr(K::PlaybackBuffer)).strong());
                ui.add(
                    egui::Slider::new(&mut app.playback.buffer_secs, 0.0..=10.0)
                        .localized()
                        .step_by(0.5)
                        .suffix(" s"),
                );
            });
        return;
    }
    if !response.clicked() {
        return;
    }
    match item {
        Item::First => app.step_first(),
        Item::Prev => app.step_prev(),
        Item::Next => app.step_next(),
        Item::Last => app.step_last(),
        Item::PlayPause => app.toggle_playback(),
        Item::Loop => app.playback.loop_playback = !app.playback.loop_playback,
        Item::Crop => app.show_crop_overlay = !app.show_crop_overlay,
        Item::Save => app.show_export_modal = true,
        Item::Fps | Item::DateInfo | Item::StartBadge | Item::StepSize | Item::EndBadge => {}
    }
}
