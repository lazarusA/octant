//! Series names read from `METADATA.ome.xml`.

use super::ome_xml::series_names;

#[test]
fn names_follow_image_ids() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ome:OME xmlns:ome="http://www.openmicroscopy.org/Schemas/OME/2016-06">
  <ome:Image ID="Image:1" Name="second &amp; last"><ome:Pixels ID="Pixels:1"/></ome:Image>
  <ome:Image ID="Image:0" Name="first"/>
  <ome:Image ID="urn:lsid:example:Image:7" Name="not a series"/>
  <ome:Image ID="Image:2"/>
</ome:OME>"#;
    let names = series_names(xml);
    assert_eq!(names.len(), 2);
    assert_eq!(names[&0], "first");
    assert_eq!(names[&1], "second & last");
}

#[test]
fn huge_ids_allocate_nothing_extra() {
    let names = series_names(br#"<OME><Image ID="Image:1048575" Name="far"/></OME>"#);
    assert_eq!(names.len(), 1);
    assert_eq!(names[&1048575], "far");
}

#[test]
fn malformed_or_empty_xml_names_nothing() {
    assert!(series_names(b"").is_empty());
    assert!(series_names(b"<OME><Image ID=\"Image:0\" Name=").is_empty());
    assert!(series_names(b"not xml at all").is_empty());
}
