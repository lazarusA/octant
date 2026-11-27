//! The desktop command line:
//! `octant [--snapshot <file> [--size WxH] [--timeout <s>] [--no-colorbar]
//! [--no-axes]] [<source>]`.

use std::path::PathBuf;

use web_time::Duration;

use crate::export::ExportFormat;

const USAGE: &str = "usage: octant [--snapshot <file.png|jpg|webp|svg|pdf> \
    [--size WxH] [--timeout <seconds>] [--no-colorbar] [--no-axes]] \
    [<share link or dataset>]";

/// What the desktop app was started with.
#[derive(Debug, Default, PartialEq)]
pub struct Startup {
    /// A share link, address or path to open, as typed in the Dataset field.
    pub source: Option<String>,
    pub snapshot: Option<Request>,
}

/// What `--snapshot` saves and how.
#[derive(Debug, PartialEq)]
pub struct Request {
    pub path: PathBuf,
    /// From the file's extension.
    pub format: ExportFormat,
    /// The image's exact pixels, the canvas scaled to cover them and
    /// centre-cropped; `None` keeps the canvas's own (the window less the top
    /// bar, times the display's scale).
    pub size: Option<[u32; 2]>,
    /// Longest wait for the plot.
    pub timeout: Duration,
    pub colorbar: bool,
    pub axes: bool,
}

impl Startup {
    /// The process's arguments; exits with 2 and the usage when they are
    /// wrong.
    pub fn from_args() -> Self {
        Self::parse(std::env::args().skip(1)).unwrap_or_else(|why| {
            eprintln!("octant: {why}\n{USAGE}");
            std::process::exit(2);
        })
    }

    /// `args` without the program's name. The first other argument is the
    /// source; any after it are ignored, as before `--snapshot` existed.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut source = None;
        let (mut path, mut size, mut timeout) = (None, None, None);
        let (mut no_colorbar, mut no_axes) = (false, false);
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            let mut value = || args.next().ok_or(format!("{arg} needs a value"));
            match arg.as_str() {
                "--snapshot" => path = Some(PathBuf::from(value()?)),
                "--size" => size = Some(size_of(&value()?)?),
                "--timeout" => timeout = Some(seconds(&value()?)?),
                "--no-colorbar" => no_colorbar = true,
                "--no-axes" => no_axes = true,
                _ if source.is_none() => source = Some(arg),
                _ => {}
            }
        }
        let Some(path) = path else {
            if size.is_some() || timeout.is_some() || no_colorbar || no_axes {
                return Err("these options need --snapshot".into());
            }
            return Ok(Self {
                source,
                snapshot: None,
            });
        };
        if source.is_none() {
            return Err("--snapshot needs a link or dataset to open".into());
        }
        let format = format_of(&path)?;
        let snapshot = Some(Request {
            path,
            format,
            size,
            timeout: timeout.unwrap_or(Duration::from_secs(120)),
            colorbar: !no_colorbar,
            axes: !no_axes,
        });
        Ok(Self { source, snapshot })
    }
}

/// The export format named by `path`'s extension.
fn format_of(path: &std::path::Path) -> Result<ExportFormat, String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    ExportFormat::from_extension(ext).ok_or(format!("unknown image type: {}", path.display()))
}

/// `WxH`, both positive.
fn size_of(text: &str) -> Result<[u32; 2], String> {
    let bad = || format!("--size takes WxH, such as 1600x900, not {text}");
    let (w, h) = text.split_once('x').ok_or_else(bad)?;
    let (w, h) = (w.parse::<u16>(), h.parse::<u16>());
    match (w, h) {
        (Ok(w), Ok(h)) if w > 0 && h > 0 => Ok([u32::from(w), u32::from(h)]),
        _ => Err(bad()),
    }
}

/// Whole seconds, at least one.
fn seconds(text: &str) -> Result<Duration, String> {
    match text.parse::<u64>() {
        Ok(s) if s > 0 => Ok(Duration::from_secs(s)),
        _ => Err(format!("--timeout takes whole seconds, not {text}")),
    }
}
