//! The browser's zstd shim decodes what zstd encodes against a trained dictionary, as
//! Icechunk 2 manifests store their virtual chunks' locations (dynamical.org's HRRR).

use super::zstd_shim::dictionary_decoder;

/// GRIB file URLs like the ones a virtual HRRR store points at.
fn locations() -> Vec<String> {
    let mut urls = Vec::new();
    for day in 1..=28 {
        for hour in 0..24 {
            urls.push(format!(
                "s3://noaa-hrrr-bdp-pds/hrrr.201807{day:02}/conus/hrrr.t{hour:02}z.wrfprsf00.grib2"
            ));
        }
    }
    urls
}

#[test]
fn locations_compressed_against_a_trained_dictionary_decode() {
    let urls = locations();
    let dict = zstd::dict::from_samples(&urls, 4096).expect("dictionary trains");
    let mut compressor = zstd::bulk::Compressor::with_dictionary(3, &dict).expect("compressor");
    let mut frames = dictionary_decoder(&dict)
        .expect("ruzstd reads a trained dictionary")
        .expect("a dictionary, not none");
    for url in urls.iter().step_by(37) {
        let compressed = compressor.compress(url.as_bytes()).expect("compresses");
        let mut out = [0u8; 1024];
        let len = (frames.decode_all(&compressed, &mut out)).expect("decodes with the dictionary");
        assert_eq!(&out[..len], url.as_bytes());
    }
}

#[test]
fn bytes_that_are_no_dictionary_are_refused() {
    assert!(dictionary_decoder(b"not a zstd dictionary").is_err());
}

/// `zstd::Decoder` loads an empty dictionary with every stream (async-tiff's own zstd
/// decoder): it means none, as in zstd, never a failure.
#[test]
fn an_empty_dictionary_is_none() {
    assert!(matches!(dictionary_decoder(&[]), Ok(None)));
}
