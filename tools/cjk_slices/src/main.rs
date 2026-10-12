//! # cjk_slices
//!
//! Cuts Noto Sans SC, JP and KR (read from noto-cjk, pinned and verified by
//! MD5 in Octant's `subset_fonts` tool) into slices for the web build, so a
//! dataset with Chinese, Japanese or Korean text fetches only the slices its
//! characters need. Slice 0 of each font holds the language's common
//! characters (its national standard's basic set, `common.rs`), which most
//! names stay within; the rest follow in codepoint order. Writes
//! `<out>/<sc|jp|kr>/NNN.otf` and `index.bin` (`slice_index.rs`):
//!
//! ```sh
//! cargo run --release --manifest-path tools/cjk_slices/Cargo.toml -- dist/fonts/cjk
//! ```

#[path = "../../../src/bin/subset_fonts/download.rs"]
mod download;
#[allow(dead_code)]
#[path = "../../../src/bin/subset_fonts/pins.rs"]
mod pins;
#[allow(dead_code)]
#[path = "../../../src/ui/fonts/slice_index.rs"]
mod slice_index;
#[allow(dead_code)]
#[path = "../../../src/bin/subset_fonts/subset.rs"]
mod subset;

mod common;
mod coverage;

use download::Error;
use slice_index::{INDEX_FILE, SliceIndex, slice_file};
use std::path::Path;

/// Characters per slice after the common one (about 400 KB each).
const REST_SLICE: usize = 1500;

fn main() {
    let Some(out) = std::env::args().nth(1) else {
        eprintln!("usage: cjk_slices <output folder, e.g. dist/fonts/cjk>");
        std::process::exit(2);
    };
    if let Err(e) = run(Path::new(&out)) {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}

fn run(out: &Path) -> Result<(), Error> {
    let downloader = download::Downloader::new()?;
    for pin in &pins::FONTS {
        let font = downloader.cached(&pin.remote, pin.file_name)?;
        let covered = coverage::codepoints(&font)?;
        let slices = plan(pin.code, &covered);
        let dir = out.join(pin.code);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        std::fs::create_dir_all(&dir)?;
        let mut total = 0;
        for (i, codepoints) in slices.iter().enumerate() {
            let chars = codepoints
                .iter()
                .filter_map(|&c| char::from_u32(c))
                .collect();
            let bytes = subset::subset(&font, &chars)?;
            total += bytes.len();
            let name = slice_file(u16::try_from(i)?);
            std::fs::write(dir.join(&name), &bytes)?;
            if i == 0 {
                println!(
                    "{}: common slice, {} characters, {} bytes",
                    pin.code,
                    codepoints.len(),
                    bytes.len()
                );
            }
        }
        let index = SliceIndex::from_slices(&slices);
        let encoded = index.encode();
        if SliceIndex::decode(&encoded).as_ref() != Some(&index) {
            return Err(format!("{}: the index does not read back", pin.code).into());
        }
        std::fs::write(dir.join(INDEX_FILE), &encoded)?;
        println!(
            "{}: {} characters in {} slices, {} bytes, index {} bytes",
            pin.code,
            covered.len(),
            slices.len(),
            total,
            encoded.len()
        );
    }
    Ok(())
}

/// The slices of a font covering `covered`: its common characters first,
/// then the rest in codepoint order, `REST_SLICE` at a time.
fn plan(code: &str, covered: &[u32]) -> Vec<Vec<u32>> {
    let (common, rest): (Vec<u32>, Vec<u32>) = covered
        .iter()
        .partition(|&&c| char::from_u32(c).is_some_and(|c| common::is_common(code, c)));
    std::iter::once(common)
        .chain(rest.chunks(REST_SLICE).map(<[u32]>::to_vec))
        .collect()
}
