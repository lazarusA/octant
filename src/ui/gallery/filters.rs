use crate::app::OctantApp;
use crate::gallery::{Domain, Format, Gallery, all_entries};
use crate::i18n::{DatasetKey, tr};
use crate::ui::icons::UiIconExt;
use crate::ui::keyboard::list_nav::Above;
use crate::utils::stack_str;

/// The search field (focused once the gallery `opened`), then a row of
/// domain chips and one of format chips, each with its count in `gallery`
/// (and the procedural entries); only those with entries show. The search
/// field, leading into the list.
pub fn render_search_and_filters(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    is_mobile: bool,
    gallery: &Gallery,
    opened: bool,
) -> Above {
    let search = search_field(app, ui, opened);
    ui.add_space(4.0);
    let spacing = if is_mobile {
        egui::vec2(4.0, 4.0)
    } else {
        egui::vec2(6.0, 4.0)
    };
    let all = all_entries(gallery).count();
    chips(ui, spacing, tr(DatasetKey::GalleryCategory), |ui, buf| {
        chip(ui, buf, &mut app.layout.gallery_domain, None, all);
        for domain in Domain::ALL {
            let count = all_entries(gallery).filter(|e| e.domain == domain).count();
            if count > 0 {
                chip(ui, buf, &mut app.layout.gallery_domain, Some(domain), count);
            }
        }
    });
    chips(ui, spacing, tr(DatasetKey::FormatLabel), |ui, buf| {
        chip(ui, buf, &mut app.layout.gallery_format, None, all);
        // Procedural entries are a domain of their own.
        for format in Format::ALL.into_iter().filter(|&f| f != Format::Procedural) {
            let count = all_entries(gallery).filter(|e| e.format == format).count();
            if count > 0 {
                chip(ui, buf, &mut app.layout.gallery_format, Some(format), count);
            }
        }
    });
    search
}

/// A wrapped row of chips after its `title`.
fn chips(
    ui: &mut egui::Ui,
    spacing: egui::Vec2,
    title: &str,
    add: impl FnOnce(&mut egui::Ui, &mut [u8; 64]),
) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = spacing;
        ui.label(egui::RichText::new(title).strong().small());
        add(ui, &mut [0u8; 64]);
    });
}

/// The chip picking `value` (`None`: All) with its `count`.
fn chip<T: Chip>(
    ui: &mut egui::Ui,
    buf: &mut [u8; 64],
    picked: &mut Option<T>,
    value: Option<T>,
    count: usize,
) {
    let name = value.map_or(tr(DatasetKey::CategoryAll), T::name);
    let label = stack_str(buf, format_args!("{name} ({count})"));
    ui.selectable_value(picked, value, label);
}

/// What a chip picks, by its shown name.
trait Chip: Copy + PartialEq {
    fn name(self) -> &'static str;
}

impl Chip for Format {
    fn name(self) -> &'static str {
        match self {
            Format::Other => tr(DatasetKey::BadgeStore),
            format => format.name(),
        }
    }
}

impl Chip for Domain {
    fn name(self) -> &'static str {
        match self {
            Domain::Climate => tr(DatasetKey::DomainClimate),
            Domain::Weather => tr(DatasetKey::DomainWeather),
            Domain::Ocean => tr(DatasetKey::DomainOcean),
            Domain::EarthObservation => tr(DatasetKey::DomainEarthObservation),
            Domain::Microscopy => tr(DatasetKey::CategoryMicroscopy),
            Domain::Medical => tr(DatasetKey::DomainMedical),
            // A field's name, kept as written.
            Domain::Scivis => "SciVis",
            Domain::Other => tr(DatasetKey::DomainOther),
            Domain::Procedural => tr(DatasetKey::CategoryProcedural),
        }
    }
}

/// The gallery's search field, focused once the gallery `opened`.
fn search_field(app: &mut OctantApp, ui: &mut egui::Ui, opened: bool) -> Above {
    let search = ui.search_field_response(
        &mut app.layout.gallery_search_query,
        tr(DatasetKey::GallerySearchHint),
        None,
    );
    if opened {
        search.request_focus();
    }
    Above::of(&search)
}
