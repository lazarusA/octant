//! The desktop command line (`Startup::parse`).

use web_time::Duration;

use super::{Request, Startup};
use crate::export::ExportFormat;

const LINK: &str = "?src=https://x.org/a.zarr";

fn parse(args: &[&str]) -> Result<Startup, String> {
    Startup::parse(args.iter().map(|a| a.to_string()))
}

fn request(args: &[&str]) -> Option<Request> {
    parse(args).ok().and_then(|s| s.snapshot)
}

#[test]
fn a_source_alone_opens_as_before() {
    let startup = parse(&["https://x.org/a.zarr", "ignored"]);
    assert_eq!(
        startup,
        Ok(Startup {
            source: Some("https://x.org/a.zarr".into()),
            snapshot: None,
        })
    );
    assert_eq!(parse(&[]), Ok(Startup::default()));
}

#[test]
fn snapshot_takes_its_file_before_or_after_the_source() {
    let defaults = Some(Request {
        path: "out.png".into(),
        format: ExportFormat::Png,
        size: None,
        timeout: Duration::from_secs(120),
        colorbar: true,
        axes: true,
    });
    assert_eq!(request(&["--snapshot", "out.png", LINK]), defaults);
    assert_eq!(request(&[LINK, "--snapshot", "out.png"]), defaults);
    assert_eq!(
        parse(&[LINK, "--snapshot", "out.png"]).map(|s| s.source),
        Ok(Some(LINK.into()))
    );
}

#[test]
fn options_set_size_timeout_format_and_what_is_drawn() {
    let args = [
        "--snapshot",
        "thumbs/a.JPEG",
        "--size",
        "800x450",
        "--timeout",
        "30",
        "--no-colorbar",
        "--no-axes",
        LINK,
    ];
    let r = request(&args);
    assert_eq!(r.as_ref().map(|r| r.format), Some(ExportFormat::Jpeg));
    assert_eq!(r.as_ref().map(|r| r.size), Some(Some([800, 450])));
    assert_eq!(r.as_ref().map(|r| r.timeout.as_secs()), Some(30));
    assert_eq!(r.map(|r| (r.colorbar, r.axes)), Some((false, false)));
    let webp = request(&["--snapshot", "a.webp", LINK]);
    assert_eq!(webp.map(|r| r.format), Some(ExportFormat::Webp));
}

#[test]
fn wrong_command_lines_are_refused() {
    for args in [
        &[LINK, "--snapshot"][..],
        &["--snapshot", "out.png"],
        &["--snapshot", "out.gif", LINK],
        &["--snapshot", "out", LINK],
        &["--snapshot", "out.png", "--size", "1600", LINK],
        &["--snapshot", "out.png", "--size", "0x900", LINK],
        &["--snapshot", "out.png", "--timeout", "0", LINK],
        &["--snapshot", "out.png", "--timeout", "-1", LINK],
        &["--no-axes", LINK],
    ] {
        assert!(parse(args).is_err(), "{args:?}");
    }
}
