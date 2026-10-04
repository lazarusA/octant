//! Minimal ZIP reader (stored and deflate entries) for the Crameri Zenodo archive.

use flate2::read::DeflateDecoder;
use std::io::Read;

fn u16_at(b: &[u8], i: usize) -> Option<usize> {
    Some(usize::from(u16::from_le_bytes([
        *b.get(i)?,
        *b.get(i + 1)?,
    ])))
}

fn u32_at(b: &[u8], i: usize) -> Option<usize> {
    let v = u32::from_le_bytes([*b.get(i)?, *b.get(i + 1)?, *b.get(i + 2)?, *b.get(i + 3)?]);
    usize::try_from(v).ok()
}

/// One central-directory entry.
pub struct Entry {
    pub name: String,
    method: usize,
    compressed: usize,
    local_offset: usize,
}

/// Lists the archive's entries from its central directory.
pub fn entries(zip: &[u8]) -> Result<Vec<Entry>, String> {
    let eocd = (0..zip.len().saturating_sub(21))
        .rev()
        .find(|&i| zip[i..].starts_with(&[0x50, 0x4b, 0x05, 0x06]))
        .ok_or("zip: end of central directory not found")?;
    let count = u16_at(zip, eocd + 10).ok_or("zip: truncated")?;
    let mut pos = u32_at(zip, eocd + 16).ok_or("zip: truncated")?;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        if !zip
            .get(pos..)
            .is_some_and(|b| b.starts_with(&[0x50, 0x4b, 0x01, 0x02]))
        {
            return Err("zip: bad central directory entry".into());
        }
        let field = |off: usize| u16_at(zip, pos + off).ok_or("zip: truncated");
        let (name_len, extra_len, comment_len) = (field(28)?, field(30)?, field(32)?);
        let name = zip
            .get(pos + 46..pos + 46 + name_len)
            .ok_or("zip: truncated")?;
        out.push(Entry {
            name: String::from_utf8_lossy(name).into_owned(),
            method: field(10)?,
            compressed: u32_at(zip, pos + 20).ok_or("zip: truncated")?,
            local_offset: u32_at(zip, pos + 42).ok_or("zip: truncated")?,
        });
        pos += 46 + name_len + extra_len + comment_len;
    }
    Ok(out)
}

/// Decompresses one entry.
pub fn read(zip: &[u8], entry: &Entry) -> Result<Vec<u8>, String> {
    let at = entry.local_offset;
    let header_len = 30
        + u16_at(zip, at + 26).ok_or("zip: truncated")?
        + u16_at(zip, at + 28).ok_or("zip: truncated")?;
    let data = zip
        .get(at + header_len..at + header_len + entry.compressed)
        .ok_or("zip: truncated entry")?;
    match entry.method {
        0 => Ok(data.to_vec()),
        8 => {
            let mut out = Vec::new();
            DeflateDecoder::new(data)
                .read_to_end(&mut out)
                .map_err(|e| e.to_string())?;
            Ok(out)
        }
        m => Err(format!("zip: unsupported compression method {m}")),
    }
}
