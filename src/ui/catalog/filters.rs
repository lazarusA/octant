use crate::app::OctantApp;
use crate::catalog::{
    CatalogCategoryFilter, GEOTIFF_CATALOG, ICECHUNK_CATALOG, PROCEDURAL_CATALOG, ZARR_CATALOG,
};
use crate::ui::icons::{Icon, UiIconExt};

pub fn render_search_and_filters(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    is_mobile: bool,
    total_count: usize,
) {
    let zarr_count = ZARR_CATALOG.len();
    let icechunk_count = ICECHUNK_CATALOG.len();
    let geotiff_count = GEOTIFF_CATALOG.len();
    let procedural_count = PROCEDURAL_CATALOG.len();

    // Search input row
    ui.horizontal(|ui| {
        ui.icon(Icon::Search, 13.0);
        let search_has_text = !app.catalog_search_query.is_empty();
        let right_pad = if search_has_text { 28.0 } else { 0.0 };
        let search_w = (ui.available_width() - right_pad).max(80.0);

        ui.add(
            egui::TextEdit::singleline(&mut app.catalog_search_query)
                .hint_text("Filter by name, description, or URL...")
                .desired_width(search_w),
        );

        if search_has_text
            && ui
                .icon_button(Icon::Cross, "")
                .on_hover_text("Clear search")
                .clicked()
        {
            app.catalog_search_query.clear();
        }
    });

    ui.add_space(4.0);

    // Filter categories row with stack-formatted labels
    let spacing = if is_mobile {
        egui::vec2(4.0, 4.0)
    } else {
        egui::vec2(6.0, 4.0)
    };
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = spacing;
        ui.label(egui::RichText::new("Category:").strong().small());

        let mut buf_all = [0u8; 32];
        let mut buf_zarr = [0u8; 32];
        let mut buf_ice = [0u8; 32];
        let mut buf_geo = [0u8; 32];
        let mut buf_proc = [0u8; 32];

        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::All,
            format_tab(&mut buf_all, "All", total_count),
        );
        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::Zarr,
            format_tab(&mut buf_zarr, "Zarr", zarr_count),
        );
        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::Icechunk,
            format_tab(&mut buf_ice, "Icechunk", icechunk_count),
        );
        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::GeoTiff,
            format_tab(&mut buf_geo, "GeoTIFF / COG", geotiff_count),
        );
        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::Procedural,
            format_tab(&mut buf_proc, "Procedural", procedural_count),
        );
    });
}

pub fn format_tab<'a>(buf: &'a mut [u8; 32], label: &str, count: usize) -> &'a str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cursor, "{} ({})", label, count);
    let len = cursor.position() as usize;
    std::str::from_utf8(&buf[..len]).unwrap_or("")
}
