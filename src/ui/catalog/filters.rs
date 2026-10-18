use crate::app::OctantApp;
use crate::catalog::{
    CatalogCategoryFilter as F, GEOTIFF_CATALOG, ICECHUNK_CATALOG, MICROSCOPY_CATALOG,
    PROCEDURAL_CATALOG, ZARR_CATALOG,
};
use crate::i18n::{DatasetKey, tr};
use crate::ui::icons::UiIconExt;
use crate::utils::stack_str;

/// The search field (focused once the catalog `opened`) and category chips.
pub fn render_search_and_filters(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    is_mobile: bool,
    total_count: usize,
    opened: bool,
) {
    let search = ui.search_field_response(
        &mut app.layout.catalog_search_query,
        tr(DatasetKey::CatalogSearchHint),
        None,
    );
    if opened {
        search.request_focus();
    }
    ui.add_space(4.0);
    let spacing = if is_mobile {
        egui::vec2(4.0, 4.0)
    } else {
        egui::vec2(6.0, 4.0)
    };
    // Each category with its name (format names stay untranslated) and count.
    let categories = [
        (F::All, tr(DatasetKey::CategoryAll), total_count),
        (F::Zarr, "Zarr", ZARR_CATALOG.len()),
        (F::Icechunk, "Icechunk", ICECHUNK_CATALOG.len()),
        (F::GeoTiff, "GeoTIFF / COG", GEOTIFF_CATALOG.len()),
        (
            F::Microscopy,
            tr(DatasetKey::CategoryMicroscopy),
            MICROSCOPY_CATALOG.len(),
        ),
        (
            F::Procedural,
            tr(DatasetKey::CategoryProcedural),
            PROCEDURAL_CATALOG.len(),
        ),
    ];
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = spacing;
        ui.label(
            egui::RichText::new(tr(DatasetKey::CatalogCategory))
                .strong()
                .small(),
        );
        let mut buf = [0u8; 64];
        for (filter, name, count) in categories {
            let label = stack_str(&mut buf, format_args!("{name} ({count})"));
            ui.selectable_value(&mut app.layout.catalog_category_filter, filter, label);
        }
    });
}
