use crate::app::OctantApp;
use crate::catalog::{
    CatalogCategoryFilter, GEOTIFF_CATALOG, ICECHUNK_CATALOG, PROCEDURAL_CATALOG, ZARR_CATALOG,
};
use crate::ui::icons::UiIconExt;
use crate::utils::stack_str;

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

    ui.search_field(
        &mut app.catalog_search_query,
        "Filter by name, description, or URL...",
        None,
    );

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
            stack_str(&mut buf_all, format_args!("All ({total_count})")),
        );
        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::Zarr,
            stack_str(&mut buf_zarr, format_args!("Zarr ({zarr_count})")),
        );
        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::Icechunk,
            stack_str(&mut buf_ice, format_args!("Icechunk ({icechunk_count})")),
        );
        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::GeoTiff,
            stack_str(
                &mut buf_geo,
                format_args!("GeoTIFF / COG ({geotiff_count})"),
            ),
        );
        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::Procedural,
            stack_str(
                &mut buf_proc,
                format_args!("Procedural ({procedural_count})"),
            ),
        );
    });
}
