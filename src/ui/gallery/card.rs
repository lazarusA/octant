//! A gallery card: thumbnail, title and format pill, two lines of
//! description, the credit. The whole card opens the entry (a click, or
//! Enter under keyboard focus); its hover and keyboard focus show the
//! details (`details.rs`); its buttons copy the store's address or the
//! details, or open the provider's page.

use super::details;
use super::thumbs;
use crate::gallery::{Format, GalleryEntry};
use crate::i18n::{DatasetKey, tr};
use crate::ui::icons::{Icon, IconSize, IconTone, ToolbarButton};
use crate::ui::keyboard::focus;
use egui::{
    Color32, CornerRadius, FontId, Galley, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2,
    vec2,
};
use std::sync::Arc;

/// Space between cards.
pub const GAP: f32 = 10.0;
/// Narrower than this, a row holds one card fewer.
const MIN_WIDTH: f32 = 220.0;
/// The text under the thumbnail: title, two lines, credit.
const TEXT_HEIGHT: f32 = 84.0;
const PAD: f32 = 8.0;
const RADIUS: u8 = 6;

/// The columns and card width of a grid filling `width`: up to three
/// columns, one on mobile.
pub fn grid(width: f32, is_mobile: bool) -> (usize, f32) {
    let fit = ((width + GAP) / (MIN_WIDTH + GAP)).floor();
    let columns = if is_mobile { 1.0 } else { fit.clamp(1.0, 3.0) };
    let card = ((width - GAP * (columns - 1.0)) / columns).floor();
    (columns as usize, card)
}

/// The card of `entry`, `width` wide; clicked, it opens the entry.
pub fn card(ui: &mut Ui, entry: &GalleryEntry, width: f32) -> Response {
    let thumb_height = (width * 9.0 / 16.0).round();
    let size = vec2(width, thumb_height + TEXT_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let visuals = ui.visuals();
    let stroke = if response.hovered() {
        visuals.widgets.hovered.bg_stroke
    } else {
        visuals.widgets.noninteractive.bg_stroke
    };
    let fill = visuals.faint_bg_color;
    ui.painter()
        .rect(rect, RADIUS, fill, stroke, StrokeKind::Inside);
    let thumb = Rect::from_min_size(rect.min, vec2(width, thumb_height)).shrink(1.0);
    thumbnail(ui, entry, thumb);
    let text = Rect::from_min_max(pos2(rect.left(), thumb.bottom()), rect.max).shrink(PAD);
    paint_text(ui, entry, text);
    focus::paint_ring(ui, &response, RADIUS);
    buttons(ui, entry, text);
    details::attach(response, entry)
}

/// The entry's picture, or its format's icon while it loads or offline.
fn thumbnail(ui: &Ui, entry: &GalleryEntry, rect: Rect) {
    let radius = CornerRadius {
        nw: RADIUS - 1,
        ne: RADIUS - 1,
        sw: 0,
        se: 0,
    };
    let visuals = ui.visuals();
    if let Some(texture) = thumbs::texture(ui.ctx(), entry) {
        egui::Image::new((texture.id(), rect.size()))
            .corner_radius(radius)
            .paint_at(ui, rect);
    } else {
        ui.painter()
            .rect_filled(rect, radius, visuals.extreme_bg_color);
        let (icon, _, tone) = badge(entry);
        let side = IconSize::Lg.px();
        let at = Rect::from_center_size(rect.center(), vec2(side, side));
        icon.paint(ui.painter(), at, tone.color(visuals), visuals.dark_mode);
    }
    if cfg!(target_arch = "wasm32") && entry.desktop_only {
        let side = IconSize::Sm.px();
        let at = Rect::from_min_size(rect.min + vec2(6.0, 6.0), vec2(side, side));
        let color = IconTone::Warning.color(visuals);
        Icon::Lock.paint(ui.painter(), at, color, visuals.dark_mode);
    }
}

/// Title and format pill, two lines of description, the credit.
fn paint_text(ui: &Ui, entry: &GalleryEntry, rect: Rect) {
    let visuals = ui.visuals();
    let painter = ui.painter_at(rect);
    let small = FontId::proportional(11.0);
    let pill_width = paint_pill(&painter, entry, visuals, rect, &small);

    let title_width = rect.width() - pill_width - 6.0;
    let strong = visuals.strong_text_color();
    let title = lines(
        ui,
        &entry.label,
        FontId::proportional(14.0),
        strong,
        title_width,
        1,
    );
    painter.galley(rect.min, title, strong);
    let weak = visuals.weak_text_color();
    let text = visuals.text_color();
    let description = lines(ui, &entry.subtitle, small.clone(), text, rect.width(), 2);
    painter.galley(rect.min + vec2(0.0, 22.0), description, text);
    // Room on the right for the three buttons.
    let credit_width = rect.width() - 3.0 * IconSize::Lg.px() - 4.0;
    let credit = lines(
        ui,
        &entry.credit,
        FontId::proportional(10.0),
        weak,
        credit_width,
        1,
    );
    painter.galley(
        pos2(rect.left(), rect.bottom() - credit.size().y),
        credit,
        weak,
    );
}

/// The format pill at the top right of `rect`; its width.
fn paint_pill(
    painter: &egui::Painter,
    entry: &GalleryEntry,
    visuals: &egui::Visuals,
    rect: Rect,
    font: &FontId,
) -> f32 {
    let (_, name, tone) = badge(entry);
    let tone = tone.color(visuals);
    let pill = painter.layout_no_wrap(name.to_owned(), font.clone(), tone);
    let size = pill.size() + vec2(10.0, 2.0);
    let at = Rect::from_min_size(pos2(rect.right() - size.x, rect.top() + 1.0), size);
    let outline = Stroke::new(1.0, tone.gamma_multiply(0.5));
    let fill = tone.gamma_multiply(0.12);
    painter.rect(at, 8, fill, outline, StrokeKind::Inside);
    painter.galley(at.min + vec2(5.0, 1.0), pill, tone);
    size.x
}

/// `text` laid out `width` wide in at most `rows` rows, cut with an ellipsis.
fn lines(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    width: f32,
    rows: usize,
) -> Arc<Galley> {
    let mut job = egui::text::LayoutJob::simple(text.to_owned(), font, color, width.max(1.0));
    job.wrap.max_rows = rows;
    ui.painter().layout_job(job)
}

/// The buttons at the bottom right of `text`, right to left: the provider's
/// page (when it has one), copy the details, copy the store's address.
fn buttons(ui: &mut Ui, entry: &GalleryEntry, text: Rect) {
    let side = IconSize::Lg.px();
    let mut right = text.right() + 2.0;
    let mut add = |ui: &mut Ui, icon: Icon, label: &str| {
        let at = Rect::from_min_size(
            pos2(right - side, text.bottom() + 2.0 - side),
            vec2(side, side),
        );
        right -= side;
        let button = ToolbarButton::new(icon, label)
            .compact(true)
            .icon_size(IconSize::Sm);
        // A child, not `put`: the button takes no room in the row of cards.
        let salt = ("gallery_card", &entry.key, icon);
        let mut child = ui.new_child(egui::UiBuilder::new().id_salt(salt).max_rect(at));
        child.add(button).clicked()
    };
    if let Some(page) = &entry.source_page
        && add(ui, Icon::Info, tr(DatasetKey::GallerySourcePage))
    {
        ui.ctx().open_url(egui::OpenUrl::new_tab(page));
    }
    if add(ui, Icon::VariableDoc, tr(DatasetKey::GalleryCopyDetails)) {
        let title = tr(DatasetKey::GalleryDetailsCopied);
        copy(ui, title, details::text(entry), &entry.label);
    }
    if add(ui, Icon::Clipboard, tr(DatasetKey::GalleryCopyUrl)) {
        let store = entry.store.trim();
        copy(
            ui,
            tr(DatasetKey::GalleryUrlCopied),
            store.to_owned(),
            store,
        );
    }
}

/// Puts `text` on the clipboard, saying so with `title` and `shown`, one
/// line of it (the whole details would fill the toast).
fn copy(ui: &Ui, title: &str, text: String, shown: &str) {
    ui.ctx().copy_text(text);
    crate::ui::toast::report(crate::ui::toast::Severity::Success, title, shown);
}

/// The icon, name and tone of the entry's format.
fn badge(entry: &GalleryEntry) -> (Icon, &'static str, IconTone) {
    let name = entry.format.name();
    match entry.format {
        Format::OmeZarr => (Icon::Globe, name, IconTone::Success),
        Format::Zarr => (Icon::Globe, name, IconTone::Accent),
        Format::Icechunk => (Icon::Icechunk, name, IconTone::Info),
        Format::GeoTiff => (Icon::Folder, name, IconTone::Success),
        Format::Procedural if entry.store == "procedural://volume4d" => (
            Icon::PlotVolume,
            tr(DatasetKey::BadgeVolume4D),
            IconTone::Warning,
        ),
        Format::Procedural => (
            Icon::PlotPlane,
            tr(DatasetKey::BadgeMatrix2D),
            IconTone::Warning,
        ),
        Format::NetCdf | Format::Hdf5 | Format::Grib => (Icon::Folder, name, IconTone::Muted),
        Format::Other => (Icon::Folder, tr(DatasetKey::BadgeStore), IconTone::Muted),
    }
}

#[cfg(test)]
mod tests {
    use super::{GAP, MIN_WIDTH, grid};

    /// A row of cards fills the width it is given and never overflows it;
    /// narrow rooms and mobile take one column, wide ones at most three.
    #[test]
    fn a_row_fills_its_width() {
        for room in (200..=1400).step_by(7).map(|w| w as f32) {
            let (columns, card) = grid(room, false);
            let used = card * columns as f32 + GAP * (columns - 1) as f32;
            assert!(
                used <= room && room - used < columns as f32,
                "{room}: {used}"
            );
            assert!((1..=3).contains(&columns), "{room}");
            assert!(columns == 1 || card >= MIN_WIDTH, "{room}: {card}");
        }
        assert_eq!(grid(900.0, true).0, 1);
    }
}
