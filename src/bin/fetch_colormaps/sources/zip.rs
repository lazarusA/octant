//! Minimal ZIP reader (stored and deflate entries) for the Crameri Zenodo archive.
//! Local headers and the CRC-32 of every extracted entry are verified.

use flate2::read::DeflateDecoder;
use std::io::Read;

const LOCAL_HEADER: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
const CENTRAL_HEADER: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
const END_OF_CENTRAL_DIR: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];

fn u16_at(b: &[u8], i: usize) -> Option<usize> {
    Some(usize::from(u16::from_le_bytes([
        *b.get(i)?,
        *b.get(i + 1)?,
    ])))
}

fn u32_raw(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
        *b.get(i)?,
        *b.get(i + 1)?,
        *b.get(i + 2)?,
        *b.get(i + 3)?,
    ]))
}

fn u32_at(b: &[u8], i: usize) -> Option<usize> {
    usize::try_from(u32_raw(b, i)?).ok()
}

/// One central-directory entry.
pub struct Entry {
    pub name: String,
    method: usize,
    crc32: u32,
    compressed: usize,
    local_offset: usize,
}

/// Lists the archive's entries from its central directory.
pub fn entries(zip: &[u8]) -> Result<Vec<Entry>, String> {
    let eocd = (0..zip.len().saturating_sub(21))
        .rev()
        .find(|&i| zip[i..].starts_with(&END_OF_CENTRAL_DIR))
        .ok_or("zip: end of central directory not found")?;
    let count = u16_at(zip, eocd + 10).ok_or("zip: truncated")?;
    let mut pos = u32_at(zip, eocd + 16).ok_or("zip: truncated")?;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        if !zip
            .get(pos..)
            .is_some_and(|b| b.starts_with(&CENTRAL_HEADER))
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
            crc32: u32_raw(zip, pos + 16).ok_or("zip: truncated")?,
            compressed: u32_at(zip, pos + 20).ok_or("zip: truncated")?,
            local_offset: u32_at(zip, pos + 42).ok_or("zip: truncated")?,
        });
        pos += 46 + name_len + extra_len + comment_len;
    }
    Ok(out)
}

/// Decompresses one entry and checks it against its CRC-32.
pub fn read(zip: &[u8], entry: &Entry) -> Result<Vec<u8>, String> {
    let at = entry.local_offset;
    if !zip.get(at..).is_some_and(|b| b.starts_with(&LOCAL_HEADER)) {
        return Err(format!("zip: `{}` has no local file header", entry.name));
    }
    let header_len = 30
        + u16_at(zip, at + 26).ok_or("zip: truncated")?
        + u16_at(zip, at + 28).ok_or("zip: truncated")?;
    let data = zip
        .get(at + header_len..at + header_len + entry.compressed)
        .ok_or("zip: truncated entry")?;
    let out = match entry.method {
        0 => data.to_vec(),
        8 => {
            let mut out = Vec::new();
            DeflateDecoder::new(data)
                .read_to_end(&mut out)
                .map_err(|e| e.to_string())?;
            out
        }
        m => return Err(format!("zip: unsupported compression method {m}")),
    };
    let crc = crc32fast::hash(&out);
    if crc != entry.crc32 {
        return Err(format!(
            "zip: `{}` CRC-32 mismatch (expected {:08x}, got {crc:08x})",
            entry.name, entry.crc32
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A one-entry stored archive holding `data` under `name`, with `crc`.
    fn archive(name: &str, data: &[u8], crc: u32) -> Vec<u8> {
        let (n, len) = (name.len() as u16, data.len() as u32);
        let mut z = LOCAL_HEADER.to_vec();
        z.extend([0; 10]);
        z.extend(crc.to_le_bytes());
        z.extend(len.to_le_bytes());
        z.extend(len.to_le_bytes());
        z.extend(n.to_le_bytes());
        z.extend([0; 2]);
        z.extend(name.as_bytes());
        z.extend(data);
        let central = z.len() as u32;
        z.extend(CENTRAL_HEADER);
        z.extend([0; 12]);
        z.extend(crc.to_le_bytes());
        z.extend(len.to_le_bytes());
        z.extend(len.to_le_bytes());
        z.extend(n.to_le_bytes());
        z.extend([0; 12]);
        z.extend(0u32.to_le_bytes());
        z.extend(name.as_bytes());
        let size = z.len() as u32 - central;
        z.extend(END_OF_CENTRAL_DIR);
        z.extend([0, 0, 0, 0, 1, 0, 1, 0]);
        z.extend(size.to_le_bytes());
        z.extend(central.to_le_bytes());
        z.extend([0; 2]);
        z
    }

    #[test]
    fn reads_and_verifies_entries() {
        let data = b"0 0 0\n1 1 1\n";
        let good = archive("a/a.txt", data, crc32fast::hash(data));
        let Ok(entries) = entries(&good) else {
            panic!("entries")
        };
        assert_eq!(entries[0].name, "a/a.txt");
        assert_eq!(read(&good, &entries[0]).as_deref(), Ok(&data[..]));

        let bad_crc = archive("a/a.txt", data, 0);
        let Ok(entries) = super::entries(&bad_crc) else {
            panic!("entries")
        };
        assert!(read(&bad_crc, &entries[0]).is_err());

        let mut bad_header = good.clone();
        bad_header[2] = 0;
        assert!(read(&bad_header, &entries[0]).is_err());
    }
}
