//! Tests for the GeoTIFF and TIFF backend.

use super::store::GeoTiffBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::slice_request::SliceRequest;
use std::path::PathBuf;

fn fixture_path(relative: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("dev/async-tiff/fixtures").join(relative)
}

#[test]
fn test_open_and_inspect_striped_geotiff() {
    let path = fixture_path("image-tiff/geo-5b.tif");
    if !path.exists() {
        return;
    }

    let path_str = path.to_str().expect("valid path string");
    let store = GeoTiffBlockStore::open(path_str).expect("open geotiff");

    assert_eq!(store.backend_name(), "GeoTIFF");

    let vars = store.variables().expect("list variables");
    assert!(vars.contains(&"band_1".to_string()));
    assert!(vars.contains(&"raster".to_string()));

    let meta = store.inspect().expect("inspect metadata");
    assert_eq!(meta.store_type, "GeoTIFF");

    let band1 = meta
        .variables
        .iter()
        .find(|v| v.name == "band_1")
        .expect("find band_1");
    assert_eq!(band1.shape, vec![10, 10]);
    assert_eq!(band1.dimension_names, vec!["y", "x"]);

    let raster = meta
        .variables
        .iter()
        .find(|v| v.name == "raster")
        .expect("find raster");
    assert_eq!(raster.shape, vec![5, 10, 10]);
    assert_eq!(raster.dimension_names, vec!["band", "y", "x"]);
}

#[test]
fn test_fetch_block_striped_geotiff() {
    let path = fixture_path("image-tiff/geo-5b.tif");
    if !path.exists() {
        return;
    }

    let path_str = path.to_str().expect("valid path string");
    let store = GeoTiffBlockStore::open(path_str).expect("open geotiff");

    // Single band fetch
    let req = SliceRequest::full_range("band_1", &[10, 10]);
    let block = store.fetch_block(&req).expect("fetch band_1 block");

    assert_eq!(block.shape, vec![10, 10]);
    assert_eq!(block.values.len(), 100);
    assert_eq!(block.dimension_names, vec!["y", "x"]);

    // Multi-band raster fetch
    let req_raster = SliceRequest::full_range("raster", &[5, 10, 10]);
    let block_raster = store.fetch_block(&req_raster).expect("fetch raster block");

    assert_eq!(block_raster.shape, vec![5, 10, 10]);
    assert_eq!(block_raster.values.len(), 500);
    assert_eq!(block_raster.dimension_names, vec!["band", "y", "x"]);
}

#[test]
fn test_open_and_fetch_tiled_tiff() {
    let path = fixture_path("image-tiff/tiled-rgb-u8.tif");
    if !path.exists() {
        return;
    }

    let path_str = path.to_str().expect("valid path string");
    let store = GeoTiffBlockStore::open(path_str).expect("open tiled tiff");

    let meta = store.inspect().expect("inspect");
    assert!(meta.variables.iter().any(|v| v.name == "band_1"));
    assert!(meta.variables.iter().any(|v| v.name == "raster"));

    let req = SliceRequest::full_range("band_1", &[32, 32]);
    let block = store
        .fetch_block(&req)
        .expect("fetch block from tiled tiff");

    assert_eq!(block.values.len(), 32 * 32);
    assert!(block.values.iter().all(|v| v.is_finite()));
}

#[test]
fn test_open_and_fetch_single_band_int16() {
    let path = fixture_path("image-tiff/int16.tif");
    if !path.exists() {
        return;
    }

    let path_str = path.to_str().expect("valid path string");
    let store = GeoTiffBlockStore::open(path_str).expect("open int16 tiff");

    let vars = store.variables().expect("list variables");
    assert!(vars.contains(&"band_1".to_string()));

    let meta = store.inspect().expect("inspect");
    let band = &meta.variables[0];
    assert_eq!(band.shape, vec![64, 64]);

    let req = SliceRequest::full_range("band_1", &[64, 64]);
    let block = store.fetch_block(&req).expect("fetch int16 block");
    assert_eq!(block.values.len(), 64 * 64);
}

#[test]
fn test_all_fixture_images_pass() {
    let dir = fixture_path("image-tiff");
    if !dir.exists() {
        return;
    }

    let files = [
        "minisblack-1c-8b.tiff",
        "minisblack-1c-16b.tiff",
        "minisblack-1c-i8b.tiff",
        "minisblack-1c-i16b.tiff",
        "minisblack-2c-8b-alpha.tiff",
        "miniswhite-1c-1b.tiff",
        "12bit.cropped.tiff",
        "12bit.cropped.rgb.tiff",
        "palette-1c-1b.tiff",
        "palette-1c-4b.tiff",
        "palette-1c-8b.tiff",
        "planar-rgb-u8.tif",
        "predictor-3-gray-f32.tif",
        "predictor-3-rgb-f32.tif",
        "quad-lzw-compat.tiff",
        "random-fp16.tiff",
        "random-fp16-pred2.tiff",
        "random-fp16-pred3.tiff",
        "rgb-3c-8b.tiff",
        "rgb-3c-16b.tiff",
        "single-black-fp16.tiff",
        "tiled-jpeg-ycbcr.tif",
        "white-fp16.tiff",
        "white-fp16-pred2.tiff",
        "white-fp16-pred3.tiff",
    ];

    let mut failed = Vec::new();
    for file in &files {
        let p = dir.join(file);
        if !p.exists() {
            continue;
        }
        let p_str = p.to_str().unwrap();
        match GeoTiffBlockStore::open(p_str) {
            Ok(store) => match store.inspect() {
                Ok(meta) => {
                    for var in &meta.variables {
                        let shape_usize: Vec<usize> =
                            var.shape.iter().map(|&x| x as usize).collect();
                        let req = SliceRequest::full_range(&var.name, &shape_usize);
                        match store.fetch_block(&req) {
                            Ok(block) => {
                                let total = block.values.len();
                                if total == 0 {
                                    failed.push(format!(
                                        "{file} -> var {} returned 0 elements",
                                        var.name
                                    ));
                                }
                            }
                            Err(e) => {
                                failed.push(format!("{file} -> var {} fetch error: {e}", var.name));
                            }
                        }
                    }
                }
                Err(e) => failed.push(format!("{file} -> inspect error: {e}")),
            },
            Err(e) => failed.push(format!("{file} -> open error: {e}")),
        }
    }

    assert!(
        failed.is_empty(),
        "The following fixture tests failed:\n{}",
        failed.join("\n")
    );
}

#[test]
fn test_edge_and_compat_fixtures_correctness() {
    let dir = fixture_path("image-tiff");
    if !dir.exists() {
        return;
    }

    // 1. miniswhite-1c-1b.tiff (157x151 1-bit WhiteIsZero)
    let p = dir.join("miniswhite-1c-1b.tiff");
    if p.exists() {
        let store = GeoTiffBlockStore::open(p.to_str().unwrap()).unwrap();
        let req = SliceRequest::full_range("band_1", &[151, 157]);
        let block = store.fetch_block(&req).unwrap();
        assert_eq!(block.values.len(), 151 * 157);
        // All pixels are 0.0 or 1.0 without NaNs
        assert!(block.values.iter().all(|&v| v == 0.0 || v == 1.0));
    }

    // 2. quad-lzw-compat.tiff (512x384 3-band LZW compat mode)
    let p = dir.join("quad-lzw-compat.tiff");
    if p.exists() {
        let store = GeoTiffBlockStore::open(p.to_str().unwrap()).unwrap();
        let req = SliceRequest::full_range("raster", &[3, 384, 512]);
        let block = store.fetch_block(&req).unwrap();
        assert_eq!(block.values.len(), 3 * 384 * 512);
        assert!(block.values.iter().all(|v| v.is_finite()));
    }

    // 3. predictor-3-gray-f32.tif (200x200 f32 floating-point predictor)
    let p = dir.join("predictor-3-gray-f32.tif");
    if p.exists() {
        let store = GeoTiffBlockStore::open(p.to_str().unwrap()).unwrap();
        let req = SliceRequest::full_range("band_1", &[200, 200]);
        let block = store.fetch_block(&req).unwrap();
        assert_eq!(block.values.len(), 200 * 200);
        assert!(block.values.iter().all(|v| v.is_finite()));
    }
}

#[test]
fn test_tiled_and_cmyk_fixtures_correctness() {
    let dir = fixture_path("image-tiff");
    if !dir.exists() {
        return;
    }

    // 1. tiled-rgb-u8.tif (499x374 3-band RGB u8, sum=39528948)
    let p_rgb = dir.join("tiled-rgb-u8.tif");
    if p_rgb.exists() {
        let store = GeoTiffBlockStore::open(p_rgb.to_str().unwrap()).unwrap();
        let req = SliceRequest::full_range("raster", &[3, 499, 374]);
        let block = store.fetch_block(&req).unwrap();
        assert_eq!(block.values.len(), 3 * 499 * 374);
        let sum: f64 = block.values.iter().map(|&v| v as f64).sum();
        assert_eq!(sum as u64, 39528948);
    }

    // 2. tiled-rect-rgb-u8.tif (367x490 3-band RGB u8, sum=62081032)
    let p_rect = dir.join("tiled-rect-rgb-u8.tif");
    if p_rect.exists() {
        let store = GeoTiffBlockStore::open(p_rect.to_str().unwrap()).unwrap();
        let req = SliceRequest::full_range("raster", &[3, 367, 490]);
        let block = store.fetch_block(&req).unwrap();
        assert_eq!(block.values.len(), 3 * 367 * 490);
        let sum: f64 = block.values.iter().map(|&v| v as f64).sum();
        assert_eq!(sum as u64, 62081032);
    }

    // 3. tiled-cmyk-i8.tif (367x490 4-band CMYK i8, sum=1759101)
    let p_cmyk = dir.join("tiled-cmyk-i8.tif");
    if p_cmyk.exists() {
        let store = GeoTiffBlockStore::open(p_cmyk.to_str().unwrap()).unwrap();
        let meta = store.inspect().unwrap();
        let raster_var = meta.variables.iter().find(|v| v.name == "raster").unwrap();
        assert_eq!(
            raster_var.attributes.get("photometric").map(|s| s.as_str()),
            Some("cmyk")
        );
        let req = SliceRequest::full_range("raster", &[4, 367, 490]);
        let ifd = store.tiff().ifds().first().unwrap();
        eprintln!("=== TILED-CMYK-I8 INFO ===");
        eprintln!("Photometric: {:?}", ifd.photometric_interpretation());
        eprintln!("Planar: {:?}", ifd.planar_configuration());
        eprintln!("SamplesPerPixel: {:?}", ifd.samples_per_pixel());
        eprintln!("BitsPerSample: {:?}", ifd.bits_per_sample());
        eprintln!("SampleFormat: {:?}", ifd.sample_format());
        eprintln!("Compression: {:?}", ifd.compression());
        eprintln!("Predictor: {:?}", ifd.predictor());
        eprintln!("TileSize: {:?}x{:?}", ifd.tile_width(), ifd.tile_height());
        eprintln!("TileCount: {:?}", ifd.tile_count());
        let block = store.fetch_block(&req).unwrap();
        assert_eq!(block.values.len(), 4 * 367 * 490);
        assert_eq!(
            block.attributes.get("photometric").map(|s| s.as_str()),
            Some("cmyk")
        );
        let sum: f64 = block.values.iter().map(|&v| v as f64).sum();
        assert_eq!(sum as u64, 1759101);
    }

    // 4. cmyk-3c-8b.tiff (151x157 4-band CMYK u8, sum=8522658)
    let p_cmyk8 = dir.join("cmyk-3c-8b.tiff");
    if p_cmyk8.exists() {
        let store = GeoTiffBlockStore::open(p_cmyk8.to_str().unwrap()).unwrap();
        let meta = store.inspect().unwrap();
        let raster_var = meta.variables.iter().find(|v| v.name == "raster").unwrap();
        assert_eq!(
            raster_var.attributes.get("photometric").map(|s| s.as_str()),
            Some("cmyk")
        );
        let req = SliceRequest::full_range("raster", &[4, 151, 157]);
        let block = store.fetch_block(&req).unwrap();
        assert_eq!(block.values.len(), 4 * 151 * 157);
        assert_eq!(
            block.attributes.get("photometric").map(|s| s.as_str()),
            Some("cmyk")
        );
        let sum: f64 = block.values.iter().map(|&v| v as f64).sum();
        assert_eq!(sum as u64, 8522658);
    }
}
