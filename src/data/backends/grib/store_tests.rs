//! GRIB files opened as the desktop app opens them (`NetCdfBlockStore::open_local`, which
//! sends them to the GRIB reader, never to netCDF-C), and messages too large to decode.

use super::scan::is_grib;
use super::test_files::RRFS_PEVAP;
use super::tests::{data_vars, open, section};
use crate::data::backends::netcdf::NetCdfBlockStore;
use crate::data::backends::netcdf::pure::read_ahead::{RangeSource, ReadAhead};
use crate::data::backends::netcdf::reader::NcReader;
use crate::data::blocks::BlockStore;

/// The variables the desktop store lists for a file holding `bytes`.
fn variables_on_disk(bytes: &[u8], name: &str) -> Vec<String> {
    let path = std::env::temp_dir().join(format!("octant_{}_{name}", std::process::id()));
    std::fs::write(&path, bytes).expect("write");
    let store = NetCdfBlockStore::open_local(path.to_str().expect("path"));
    let _ = std::fs::remove_file(&path);
    let meta = store.expect("open").inspect().expect("inspect");
    meta.variables.into_iter().map(|v| v.name).collect()
}

#[test]
fn local_grib_files_open_through_the_grib_reader() {
    let names = variables_on_disk(RRFS_PEVAP, "plain.grib2");
    assert!(names.contains(&"missing".to_owned()), "{names:?}");
    // GRIB from bulletin feeds starts with a WMO header.
    let mut bulletin = b"\x01\r\r\n123\r\r\nYTXZ98 KWBC 010000\r\r\n".to_vec();
    bulletin.extend(RRFS_PEVAP);
    let names = variables_on_disk(&bulletin, "bulletin.grib2");
    assert!(names.contains(&"missing".to_owned()), "{names:?}");
    assert!(
        !is_grib(b"CDF\x01 dimension GRIB_level"),
        "a NetCDF header naming GRIB"
    );
}

#[test]
fn messages_declaring_more_points_than_decodable_are_refused() {
    let mut bytes = RRFS_PEVAP.to_vec();
    let s5 = section(&bytes, 5);
    bytes[s5 + 5..s5 + 9].copy_from_slice(&u32::MAX.to_be_bytes());
    let reader = open(bytes);
    let [var] = data_vars(&reader).try_into().expect("one variable");
    let error = reader.read(&var, &[0, 0], &[2, 4]).err().expect("refused");
    assert!(error.to_string().contains("declares"), "{error}");

    // The grid section's own count, to which gribberish resizes the decoded values.
    let mut bytes = RRFS_PEVAP.to_vec();
    let s3 = section(&bytes, 3);
    bytes[s3 + 6..s3 + 10].copy_from_slice(&u32::MAX.to_be_bytes());
    let reader = open(bytes);
    let [var] = data_vars(&reader).try_into().expect("one variable");
    let error = reader.read(&var, &[0, 0], &[2, 4]).err().expect("refused");
    assert!(error.to_string().contains("declares"), "{error}");
}

/// A grid header claiming 4 billion columns would make opening compute 32 GB of
/// coordinates, an abort no panic handler catches: such a message is unreadable.
#[test]
fn grids_with_more_points_than_decodable_are_unreadable() {
    let mut bytes = RRFS_PEVAP.to_vec();
    let s3 = section(&bytes, 3);
    bytes[s3 + 30..s3 + 34].copy_from_slice(&u32::MAX.to_be_bytes());
    let source = super::tests::source(bytes);
    let error = super::reader::GribReader::open(source, "wide.grib2")
        .err()
        .expect("refused");
    assert!(error.to_string().contains("no GRIB2 message"), "{error}");
}

/// The RRFS message with section `n` cut to `len` bytes, the sections after it intact.
fn shortened(n: u8, len: usize) -> Vec<u8> {
    let bytes = RRFS_PEVAP;
    let (at, was) = super::sections::section_at(bytes, n).expect("section");
    let mut out = bytes[..at + len].to_vec();
    out[at..at + 4].copy_from_slice(&(len as u32).to_be_bytes());
    out.extend(&bytes[at + was..]);
    out
}

/// Product template 8 (accumulations, such as precipitation) is read up to its 58th byte;
/// a section shorter than that, or than its own header, would panic in gribberish.
#[test]
fn product_sections_shorter_than_gribberish_reads_are_refused() {
    use super::sections::fit_templates;
    let s4 = section(RRFS_PEVAP, 4);
    assert_eq!(RRFS_PEVAP[s4 + 7..s4 + 9], [0, 8], "template 4.8");
    assert!(fit_templates(RRFS_PEVAP));
    assert!(
        !fit_templates(&shortened(4, 41)),
        "template 8 reads to byte 58"
    );
    assert!(fit_templates(&shortened(4, 58)));
    assert!(!fit_templates(&shortened(4, 5)), "shorter than its header");
}

/// A grid section shorter than its template (Lambert, 81 bytes) would make gribberish
/// index past it and panic, which aborts the browser's worker: such a message is unreadable.
#[test]
fn sections_shorter_than_their_templates_are_unreadable() {
    let mut bytes = RRFS_PEVAP.to_vec();
    let s3 = section(&bytes, 3);
    bytes[s3..s3 + 4].copy_from_slice(&70u32.to_be_bytes());
    let source = super::tests::source(bytes);
    let error = super::reader::GribReader::open(source, "short.grib2")
        .err()
        .expect("refused");
    assert!(error.to_string().contains("no GRIB2 message"), "{error}");
}

/// `n` messages of HRRR parameter `parameter` (dew point 6, temperature 0) sharing no
/// reference time, step or level, so every combination of the three gets a slot (`n³`).
fn sparse(n: u32, parameter: u8) -> Vec<u8> {
    use super::test_files::HRRR_DPT_CONSTANT;
    let (s1, s4) = (section(HRRR_DPT_CONSTANT, 1), section(HRRR_DPT_CONSTANT, 4));
    let mut bytes = Vec::new();
    for i in 0..n {
        let mut message = HRRR_DPT_CONSTANT.to_vec();
        (message[s1 + 16], message[s1 + 17]) = ((i / 60) as u8, (i % 60) as u8);
        message[s4 + 10] = parameter;
        message[s4 + 18..s4 + 22].copy_from_slice(&i.to_be_bytes());
        message[s4 + 24..s4 + 28].copy_from_slice(&(i + 1).to_be_bytes());
        bytes.extend(message);
    }
    bytes
}

/// The file's slots (`layout.rs` `MAX_SLOTS`, 4 Mi) are one budget: one variable of 130³
/// (2.2 Mi) slots opens, two of them do not, nor does one of 300³.
#[test]
fn messages_sharing_no_times_or_levels_are_refused() {
    let opened = |bytes| super::reader::GribReader::open(super::tests::source(bytes), "s.grib2");
    assert!(
        opened(sparse(130, 6)).is_ok(),
        "one variable within the budget"
    );
    let mut two = sparse(130, 6);
    two.extend(sparse(130, 0));
    for bytes in [two, sparse(300, 6)] {
        let error = opened(bytes).err().expect("refused");
        assert!(error.to_string().contains("too many messages"), "{error}");
    }
}

/// The bytes of `message`, its stated length 1 GiB, in a source claiming that length (a
/// remote server states its file's size itself), zeros past the message. Reading it whole
/// fails the test.
fn claiming_a_gib(mut message: Vec<u8>) -> Box<dyn RangeSource + Send + Sync> {
    const GIB: u64 = 1 << 30;
    message[8..16].copy_from_slice(&GIB.to_be_bytes());
    Box::new(ReadAhead::new(GIB, move |offset, length| {
        assert!(length <= 1 << 20, "read {length} bytes at {offset}");
        let mut out = vec![0; length as usize];
        let start = (offset as usize).min(message.len());
        let end = ((offset + length) as usize).min(message.len());
        out[..end - start].copy_from_slice(&message[start..end]);
        Ok(out)
    }))
}

/// A message's stated length is checked only against the file's size: longer than
/// `MAX_MESSAGE_BYTES`, it is never read whole, neither to describe it nor to decode it.
#[test]
fn messages_longer_than_decodable_are_never_read_whole() {
    let reader = super::reader::GribReader::open(claiming_a_gib(RRFS_PEVAP.to_vec()), "a.grib2");
    let reader = reader.expect("its head describes it");
    let [var] = data_vars(&reader).try_into().expect("one variable");
    let error = reader.read(&var, &[0, 0], &[2, 4]).err().expect("refused");
    assert!(error.to_string().contains("bytes long"), "{error}");

    // A head that does not describe its message (a grid template gribberish does not read).
    let mut bytes = RRFS_PEVAP.to_vec();
    let s3 = section(&bytes, 3);
    bytes[s3 + 12..s3 + 14].copy_from_slice(&u16::MAX.to_be_bytes());
    let error = super::reader::GribReader::open(claiming_a_gib(bytes), "b.grib2")
        .err()
        .expect("refused");
    assert!(error.to_string().contains("no GRIB2 message"), "{error}");
}
