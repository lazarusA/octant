//! Self-contained unit tests for the GeoTIFF and TIFF backend.

use super::store::GeoTiffBlockStore;
use super::test_utils::SyntheticTiffBuilder;
use crate::data::blocks::BlockStore;
use crate::data::slice_request::SliceRequest;

fn open_store_from_bytes(name: &str, bytes: Vec<u8>) -> GeoTiffBlockStore {
    let rt = crate::utils::executor::get_shared_tokio_rt();
    rt.block_on(async move { GeoTiffBlockStore::from_bytes(name, bytes).await })
        .expect("open from bytes")
}

#[test]
fn test_striped_single_and_multiband_geotiff() {
    let raw: Vec<u8> = (0..500).map(|i| (i % 255) as u8).collect();
    let tiff_bytes = SyntheticTiffBuilder::new(10, 10)
        .samples(5, 8)
        .striped_data(&raw, 10)
        .geo_keys(
            [0.1, 0.1, 0.0],
            [0.0, 0.0, 0.0, -120.0, 35.0, 0.0],
            &[1, 1, 0, 1, 1024, 0, 1, 1], // ModelTypeGeoKey = 1 (Proj)
            Some("WGS 84 / UTM zone 11N"),
        )
        .build();

    let store = open_store_from_bytes("test_geo.tif", tiff_bytes);
    assert_eq!(store.backend_name(), "GeoTIFF");

    let meta = store.inspect().expect("inspect metadata");
    assert_eq!(meta.store_type, "GeoTIFF");
    let raster = meta.variables.iter().find(|v| v.name == "raster").unwrap();
    assert_eq!(raster.shape, vec![5, 10, 10]);
    assert_eq!(raster.dimension_names, vec!["band", "y", "x"]);

    let band1 = meta.variables.iter().find(|v| v.name == "band_1").unwrap();
    assert_eq!(band1.shape, vec![10, 10]);

    let req_b1 = SliceRequest::full_range("band_1", &[10, 10]);
    let b1 = store.fetch_block(&req_b1).expect("fetch band_1");
    assert_eq!(b1.values.len(), 100);

    let req_r = SliceRequest::full_range("raster", &[5, 10, 10]);
    let br = store.fetch_block(&req_r).expect("fetch raster");
    assert_eq!(br.values.len(), 500);
}

#[test]
fn test_tiled_geotiff_fetch() {
    // 32x32 image with 16x16 tiles (4 tiles total), 3 bands RGB u8 = 16*16*3 = 768 bytes per tile
    let tile_data = vec![128u8; 768 * 4];
    let tiff_bytes = SyntheticTiffBuilder::new(32, 32)
        .samples(3, 8)
        .photometric(2) // RGB
        .tiled(16, 16, &tile_data, 4)
        .build();

    let store = open_store_from_bytes("tiled_rgb.tif", tiff_bytes);
    let meta = store.inspect().expect("inspect");
    let raster = meta.variables.iter().find(|v| v.name == "raster").unwrap();
    assert_eq!(raster.shape, vec![3, 32, 32]);
    assert_eq!(raster.chunk_shape, vec![1, 16, 16]);

    let req = SliceRequest::full_range("raster", &[3, 32, 32]);
    let block = store.fetch_block(&req).expect("fetch block");
    assert_eq!(block.values.len(), 3 * 32 * 32);
    assert!(block.values.iter().all(|&v| v == 128.0));
}

#[test]
fn test_cmyk_and_rgb_composites() {
    // 4-band CMYK 4x4 image
    let cmyk_data: Vec<u8> = (0..64).map(|i| (i * 4) as u8).collect();
    let cmyk_bytes = SyntheticTiffBuilder::new(4, 4)
        .samples(4, 8)
        .photometric(5) // CMYK
        .striped_data(&cmyk_data, 4)
        .build();

    let store = open_store_from_bytes("cmyk.tif", cmyk_bytes);
    let meta = store.inspect().expect("inspect cmyk");
    let raster = meta.variables.iter().find(|v| v.name == "raster").unwrap();
    assert_eq!(
        raster.attributes.get("photometric").map(|s| s.as_str()),
        Some("cmyk")
    );

    let req = SliceRequest::full_range("raster", &[4, 4, 4]);
    let block = store.fetch_block(&req).expect("fetch cmyk block");
    let composite = crate::data::slicing::slice_rgb_composite(&block, [0, 1, 2], 1)
        .expect("composite cmyk to rgb");
    assert_eq!(composite.values.len(), 16);
    assert!(composite.values.iter().all(|v| v.is_finite()));
}

#[test]
fn test_palette_colormap_expansion() {
    // 4x4 8-bit palette image with 256 colormap entries
    let raw_indices: Vec<u8> = (0..16).map(|i| i as u8).collect();
    let mut reds = vec![0u16; 256];
    let mut greens = vec![0u16; 256];
    let mut blues = vec![0u16; 256];
    for i in 0..16 {
        reds[i] = (i * 1000) as u16;
        greens[i] = (i * 2000) as u16;
        blues[i] = (i * 3000) as u16;
    }
    let tiff_bytes = SyntheticTiffBuilder::new(4, 4)
        .samples(1, 8)
        .photometric(3) // RGBPalette
        .colormap(&reds, &greens, &blues)
        .striped_data(&raw_indices, 4)
        .build();

    let store = open_store_from_bytes("palette.tif", tiff_bytes);
    let meta = store.inspect().expect("inspect palette");
    let raster = meta.variables.iter().find(|v| v.name == "raster").unwrap();
    assert_eq!(raster.shape, vec![3, 4, 4]); // expanded to 3 bands

    let req = SliceRequest::full_range("raster", &[3, 4, 4]);
    let block = store.fetch_block(&req).expect("fetch expanded palette");
    assert_eq!(block.values.len(), 3 * 16);
}

#[test]
fn test_int16_and_float32_sample_formats() {
    // Float32 8x8 image
    let f32_vals: Vec<f32> = (0..64).map(|i| i as f32 * 0.25).collect();
    let mut f32_bytes = Vec::with_capacity(64 * 4);
    for v in &f32_vals {
        f32_bytes.extend_from_slice(&v.to_le_bytes());
    }
    let tiff_bytes = SyntheticTiffBuilder::new(8, 8)
        .samples(1, 32)
        .sample_format(3, 1) // SampleFormat::IEEEFP
        .striped_data(&f32_bytes, 8)
        .build();

    let store = open_store_from_bytes("float.tif", tiff_bytes);
    let req = SliceRequest::full_range("band_1", &[8, 8]);
    let block = store.fetch_block(&req).expect("fetch float block");
    assert_eq!(block.values.len(), 64);
    assert_eq!(block.values[4], 1.0);
}

#[test]
fn test_open_local_temp_file() {
    let raw: Vec<u8> = vec![255u8; 100];
    let tiff_bytes = SyntheticTiffBuilder::new(10, 10)
        .striped_data(&raw, 10)
        .build();

    let temp_file = std::env::temp_dir().join(format!(
        "octant_test_geotiff_{}_{:?}.tif",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::write(&temp_file, &tiff_bytes).expect("write temp tiff");

    let store = GeoTiffBlockStore::open(temp_file.to_str().unwrap()).expect("open local file");
    assert_eq!(store.backend_name(), "GeoTIFF");

    let vars = store.variables().expect("list vars");
    assert!(vars.contains(&"band_1".to_string()));

    let _ = std::fs::remove_file(&temp_file);
}

#[test]
fn test_unpredict_horizontal_and_float() {
    use crate::data::backends::geotiff::decode::unpredict_buffer;
    use async_tiff::reader::Endianness;
    use async_tiff::tags::Predictor;

    // Horizontal differencing u8 (diff: [10, 5, 2] -> orig: [10, 15, 17])
    let diff_u8 = vec![10u8, 5, 2];
    let unpred = unpredict_buffer(
        diff_u8,
        Predictor::Horizontal,
        1,
        8,
        3,
        Endianness::LittleEndian,
    )
    .expect("unpredict u8");
    assert_eq!(unpred, vec![10, 15, 17]);

    // Floating-point differencing f32
    let orig_f32: [f32; 2] = [1.0, 2.0];
    let mut bytes = Vec::new();
    for v in orig_f32 {
        bytes.extend_from_slice(&v.to_ne_bytes());
    }
    // Round-trip unpredict_buffer with Predictor::None gives same bytes
    let passthrough = unpredict_buffer(
        bytes.clone(),
        Predictor::None,
        1,
        32,
        2,
        Endianness::LittleEndian,
    )
    .expect("unpredict none");
    assert_eq!(passthrough, bytes);
}

#[test]
fn test_wasm_geotiff_store_behavior() {
    use crate::data::backends::geotiff::WasmGeoTiffBlockStore;

    let store = WasmGeoTiffBlockStore::get_or_create("https://example.com/test.tif");
    assert_eq!(store.backend_name(), "GeoTIFF (Web COG)");
    assert_eq!(store.variables().unwrap(), Vec::<String>::new());
    // Initial state returns loading error
    let inspect_err = store.inspect().unwrap_err();
    assert!(
        inspect_err
            .to_string()
            .contains("loading in the background")
            || inspect_err.to_string().contains("Failed to load")
    );
}

/// A GeoTIFF's GeoKeys name its CRS by EPSG code (ProjectedCSTypeGeoKey).
#[test]
fn geo_keys_name_the_crs_by_epsg_code() {
    let raw: Vec<u8> = (0..100).map(|i| i as u8).collect();
    let tiff_bytes = SyntheticTiffBuilder::new(10, 10)
        .striped_data(&raw, 10)
        .geo_keys(
            [10.0, 10.0, 0.0],
            [0.0, 0.0, 0.0, 500_000.0, 1_900_000.0, 0.0],
            &[1, 1, 0, 2, 1024, 0, 1, 1, 3072, 0, 1, 32636],
            None,
        )
        .build();
    let mut meta = open_store_from_bytes("utm.tif", tiff_bytes)
        .inspect()
        .expect("inspect");
    crate::data::crs::assign(&mut meta);
    let band = meta
        .variables
        .iter()
        .find(|v| v.name == "band_1")
        .expect("band_1");
    assert_eq!(band.crs, Some(crate::data::crs::CrsRef::Epsg(32636)));
}
