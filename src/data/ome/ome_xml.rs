//! Series names from a `bioformats2raw` container's `OME/METADATA.ome.xml`.
//!
//! Only `<Image ID="Image:N" Name="...">` elements are read, streamed with
//! `quick-xml`; series `N` is the container's group `N`. Images whose `ID`
//! is not of that form name no series.

use std::collections::HashMap;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

/// Names of the series by index (`names[&n]` names group `n`).
pub fn series_names(xml: &[u8]) -> HashMap<usize, String> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut names = HashMap::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) if e.local_name().as_ref() == b"Image" => {
                if let Some((index, name)) = image_entry(&e) {
                    names.insert(index, name);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    names
}

/// `(N, Name)` of an `<Image ID="Image:N" Name="...">` element.
fn image_entry(e: &BytesStart<'_>) -> Option<(usize, String)> {
    let mut index = None;
    let mut name = None;
    for attr in e.attributes().flatten() {
        let value = attr.normalized_value(XmlVersion::Implicit1_0).ok()?;
        match attr.key.local_name().as_ref() {
            b"ID" => {
                index = value
                    .strip_prefix("Image:")
                    .and_then(|n| n.parse::<usize>().ok())
            }
            b"Name" => name = Some(value.trim().to_string()).filter(|n| !n.is_empty()),
            _ => {}
        }
    }
    index.zip(name)
}
