//! Real GRIB files opened as the app opens them (the desktop store: inspection,
//! coordinates, blocks); the files live in the git-ignored `dev/gribberish/test-data`:
//! `cargo test --lib grib_dev_files -- --ignored --nocapture`. With `GRIB_RENDER_DIR` set,
//! each file's first plane is also written there as PNGs, as gribberish decodes it and as
//! the store hands it to the plot, to look at.

use crate::data::backends::netcdf::NetCdfBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::octant_block::OctantBlock;

/// The first `y`, `x` plane of variable `name`, shaped `shape`.
fn first_plane(store: &NetCdfBlockStore, name: &str, shape: &[usize]) -> OctantBlock {
    let lead = vec![0; shape.len().saturating_sub(2)];
    let block = store.fetch_block(&super::plane_request(name, shape, &lead));
    block.expect(name)
}

/// `values` (`ny` rows of `nx`) as a grayscale PNG at `path`, NaN red.
fn render(path: &str, values: &[f32], ny: usize, nx: usize) {
    let finite = values.iter().filter(|v| v.is_finite());
    let (lo, hi) = finite.fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
    let mut img = image::RgbImage::new(nx as u32, ny as u32);
    for (i, v) in values.iter().enumerate() {
        let px = match v.is_finite() {
            true => {
                let g = (((v - lo) / (hi - lo).max(f32::MIN_POSITIVE)) * 255.0) as u8;
                [g, g, g]
            }
            false => [255, 0, 0],
        };
        img.put_pixel((i % nx) as u32, (i / nx) as u32, image::Rgb(px));
    }
    img.save(path).expect("png");
}

/// Writes the raw and plotted first plane of the file `name` (`bytes`) into `out`.
fn render_both(out: &str, name: &str, bytes: &[u8], block: &OctantBlock) {
    let message = gribberish::message::read_message(bytes, 0).expect("message");
    let (ny, nx) = message.grid_dimensions().expect("grid");
    let raw: Vec<f32> = (message.data().expect("data").iter())
        .map(|&v| v as f32)
        .collect();
    render(&format!("{out}/{name}.raw.png"), &raw, ny, nx);
    let rank = block.shape.len();
    let (by, bx) = (block.shape[rank - 2], block.shape[rank - 1]);
    render(&format!("{out}/{name}.block.png"), &block.values, by, bx);
}

#[test]
#[ignore = "local files"]
fn grib_dev_files_open_and_read() {
    let dir = format!("{}/dev/gribberish/test-data", env!("CARGO_MANIFEST_DIR"));
    let render_to = std::env::var("GRIB_RENDER_DIR").ok();
    let mut names: Vec<String> = (std::fs::read_dir(&dir).expect("test data"))
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| !n.ends_with(".idx") && !n.ends_with(".index"))
        .collect();
    names.sort();
    for name in names {
        let path = format!("{dir}/{name}");
        let bytes = std::fs::read(&path).expect("read");
        let store = match NetCdfBlockStore::open_local(&path) {
            Ok(store) => store,
            Err(e) => {
                println!("{name}: not opened: {e}");
                assert!(
                    bytes.get(7) == Some(&1),
                    "{name}: only GRIB1 may be refused"
                );
                continue;
            }
        };
        let meta = store.inspect().expect("inspect");
        let data: Vec<_> = (meta.variables.iter())
            .filter(|v| v.data_type == "float32")
            .collect();
        let lon = meta.dimension_coordinates.get("longitude");
        let ends = lon.map(|l| (l.number_for(0, l.len()), l.number_for(l.len() - 1, l.len())));
        println!("{name}: {} variables, longitude {ends:?}", data.len());
        for (i, var) in data.iter().enumerate() {
            let shape: Vec<usize> = var.shape.iter().map(|&n| n as usize).collect();
            let block = first_plane(&store, &var.name, &shape);
            let finite = block.values.iter().filter(|v| v.is_finite()).count();
            println!(
                "  {} {:?} flipped {:?}: {finite} finite",
                var.name, shape, block.flipped_dims
            );
            if let (0, Some(out)) = (i, &render_to) {
                render_both(out, &name, &bytes, &block);
            }
        }
        let messages: Vec<_> = gribberish::message::read_messages(&bytes).collect();
        if let ([message], [var]) = (messages.as_slice(), data.as_slice()) {
            // Blocks come in display orientation (`block_orientation.rs`): compare as sets.
            let shape: Vec<usize> = var.shape.iter().map(|&n| n as usize).collect();
            let mut plane = first_plane(&store, &var.name, &shape).values.to_vec();
            let mut expected: Vec<f32> = (message.data().expect("data").iter())
                .map(|&v| v as f32)
                .collect();
            plane.sort_by(f32::total_cmp);
            expected.sort_by(f32::total_cmp);
            let same = |a: &f32, b: &f32| a == b || (a.is_nan() && b.is_nan());
            assert!(plane.len() == expected.len(), "{name}");
            assert!(
                plane.iter().zip(&expected).all(|(a, b)| same(a, b)),
                "{name}"
            );
        }
    }
}
