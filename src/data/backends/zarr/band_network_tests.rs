//! What changing bands costs on a real pyramid, timed by hand:
//! `cargo test --release --lib band_changes_timed -- --ignored --nocapture`
//! reads TESSERA dclimate's `global_rgb/9/rgb` (its coarsest level, 3515 x
//! 7031 x 4 bands in 98 chunks of 512 x 512 x 4) whole, one band at a time,
//! as the heatmap does over the whole globe.

use std::time::Instant;

use super::store::ZarrBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::{DimensionSelection, SliceRequest};

const DCLIMATE: &str = "https://s3.us-west-2.amazonaws.com/us-west-2.opendata.source.coop/tessera/tessera/zarr/v1.1-dclimate";

fn level_9(band: DimensionSelection) -> SliceRequest {
    SliceRequest {
        variable: "global_rgb/9/rgb".into(),
        selections: vec![
            DimensionSelection::Range {
                start: 0,
                end: 3515,
            },
            DimensionSelection::Range {
                start: 0,
                end: 7031,
            },
            band,
        ],
    }
}

#[test]
#[ignore = "network"]
fn band_changes_timed() {
    use DimensionSelection::{Index, Range};
    let store = ZarrBlockStore::open_remote(DCLIMATE).expect("open");
    store.inspect().expect("inspect");
    eprintln!("read of global_rgb/9/rgb      time");
    for (name, band) in [
        ("band 0, opening the array", Index(0)),
        ("band 1", Index(1)),
        ("band 2", Index(2)),
        ("band 0 again", Index(0)),
        ("bands 0-3 (multichannel)", Range { start: 0, end: 4 }),
    ] {
        let start = Instant::now();
        store.fetch_block(&level_9(band)).expect("block");
        eprintln!("{name:<28} {:>8.2?}", start.elapsed());
    }
}
