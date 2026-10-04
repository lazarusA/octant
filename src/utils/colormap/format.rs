//! Compact binary container for the bundled colormap catalog.
//!
//! Written by `cargo run --bin fetch_colormaps` and decoded at startup, so the
//! encoder and decoder share this single definition. Layout (little endian):
//!
//! ```text
//! b"OCMAPS01"
//! u16 family_count, then per family: key, name, license, source, attribution (strings)
//! u16 map_count,    then per map:    name (string), u8 family, u8 kind, u16 n, n × [u8; 3]
//! string = u16 byte length + UTF-8 bytes
//! ```

use super::kind::ColormapKind;

const MAGIC: &[u8; 8] = b"OCMAPS01";

/// Provenance and license of one colormap family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamilyRecord {
    pub key: String,
    pub name: String,
    pub license: String,
    pub source: String,
    pub attribution: String,
}

/// One colormap as its original color stops (not yet resampled).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapRecord {
    pub name: String,
    pub family: u8,
    pub kind: ColormapKind,
    pub stops: Vec<[u8; 3]>,
}

/// Decoded catalog contents.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CatalogRecords {
    pub families: Vec<FamilyRecord>,
    pub maps: Vec<MapRecord>,
}

pub fn encode(records: &CatalogRecords) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(records.maps.len() * 800 + 64);
    out.extend_from_slice(MAGIC);
    put_u16(&mut out, records.families.len())?;
    for f in &records.families {
        for s in [&f.key, &f.name, &f.license, &f.source, &f.attribution] {
            put_str(&mut out, s)?;
        }
    }
    put_u16(&mut out, records.maps.len())?;
    for m in &records.maps {
        put_str(&mut out, &m.name)?;
        out.push(m.family);
        out.push(m.kind as u8);
        put_u16(&mut out, m.stops.len())?;
        for rgb in &m.stops {
            out.extend_from_slice(rgb);
        }
    }
    Ok(out)
}

pub fn decode(bytes: &[u8]) -> Result<CatalogRecords, String> {
    let mut r = Reader { bytes, pos: 0 };
    if r.take(MAGIC.len())? != MAGIC {
        return Err("colormap catalog: bad magic header".into());
    }
    let family_count = r.u16()?;
    let mut families = Vec::with_capacity(family_count);
    for _ in 0..family_count {
        families.push(FamilyRecord {
            key: r.string()?,
            name: r.string()?,
            license: r.string()?,
            source: r.string()?,
            attribution: r.string()?,
        });
    }
    let map_count = r.u16()?;
    let mut maps = Vec::with_capacity(map_count);
    for _ in 0..map_count {
        let name = r.string()?;
        let family = r.u8()?;
        let kind_byte = r.u8()?;
        let kind = ColormapKind::from_u8(kind_byte)
            .ok_or_else(|| format!("colormap '{name}' has unknown kind {kind_byte}"))?;
        let n = r.u16()?;
        let raw = r.take(n * 3)?;
        let stops = raw.as_chunks::<3>().0.to_vec();
        if usize::from(family) >= families.len() {
            return Err(format!(
                "colormap '{name}' references unknown family {family}"
            ));
        }
        maps.push(MapRecord {
            name,
            family,
            kind,
            stops,
        });
    }
    if r.pos != bytes.len() {
        return Err(format!("{} trailing bytes", bytes.len() - r.pos));
    }
    Ok(CatalogRecords { families, maps })
}

fn put_u16(out: &mut Vec<u8>, v: usize) -> Result<(), String> {
    let v = u16::try_from(v).map_err(|_| format!("value {v} exceeds u16"))?;
    out.extend_from_slice(&v.to_le_bytes());
    Ok(())
}

fn put_str(out: &mut Vec<u8>, s: &str) -> Result<(), String> {
    put_u16(out, s.len())?;
    out.extend_from_slice(s.as_bytes());
    Ok(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or("colormap catalog: overflow")?;
        let slice = self
            .bytes
            .get(self.pos..end)
            .ok_or("colormap catalog: truncated")?;
        self.pos = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<usize, String> {
        let b = self.take(2)?;
        Ok(usize::from(u16::from_le_bytes([b[0], b[1]])))
    }

    fn string(&mut self) -> Result<String, String> {
        let n = self.u16()?;
        String::from_utf8(self.take(n)?.to_vec()).map_err(|e| e.to_string())
    }
}
