//! In-memory synthetic TIFF builder for self-contained unit testing.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub enum TiffType {
    Byte = 1,
    Ascii = 2,
    Short = 3,
    Long = 4,
    Double = 12,
}

/// Helper for constructing valid TIFF byte buffers in memory.
#[derive(Default)]
pub struct SyntheticTiffBuilder {
    entries: BTreeMap<u16, (TiffType, Vec<u8>)>,
    raw_data: Vec<u8>,
}

impl SyntheticTiffBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        let mut builder = Self::default();
        builder.add_short(256, width as u16);
        builder.add_short(257, height as u16);
        builder.add_short(258, 8);
        builder.add_short(259, 1);
        builder.add_short(262, 1);
        builder.add_short(277, 1);
        builder.add_short(284, 1);
        builder.add_short(339, 1);
        builder
    }

    pub fn samples(mut self, count: u16, bits_per_sample: u16) -> Self {
        self.add_short(277, count);
        let bits = vec![bits_per_sample; count as usize];
        self.add_short_vec(258, &bits);
        self
    }

    pub fn sample_format(mut self, format: u16, samples: usize) -> Self {
        let formats = vec![format; samples];
        self.add_short_vec(339, &formats);
        self
    }

    pub fn photometric(mut self, photo: u16) -> Self {
        self.add_short(262, photo);
        self
    }

    /// Drops tag `tag`, for files that omit a required one.
    pub fn without(mut self, tag: u16) -> Self {
        self.entries.remove(&tag);
        self
    }

    pub fn compression(mut self, comp: u16) -> Self {
        self.add_short(259, comp);
        self
    }

    pub fn predictor(mut self, pred: u16) -> Self {
        self.add_short(317, pred);
        self
    }

    pub fn colormap(mut self, reds: &[u16], greens: &[u16], blues: &[u16]) -> Self {
        let mut cmap = Vec::with_capacity(reds.len() + greens.len() + blues.len());
        cmap.extend_from_slice(reds);
        cmap.extend_from_slice(greens);
        cmap.extend_from_slice(blues);
        self.add_short_vec(320, &cmap);
        self
    }

    pub fn tiled(mut self, tile_w: u32, tile_h: u32, tile_data: &[u8], num_tiles: usize) -> Self {
        self.add_short(322, tile_w as u16);
        self.add_short(323, tile_h as u16);
        let bytes_per_tile = (tile_data.len() / num_tiles.max(1)) as u32;
        let counts = vec![bytes_per_tile; num_tiles];
        self.add_long_vec(325, &counts);
        self.raw_data = tile_data.to_vec();
        self
    }

    pub fn striped_data(mut self, data: &[u8], rows_per_strip: u32) -> Self {
        self.add_long(278, rows_per_strip);
        self.add_long_vec(279, &[data.len() as u32]);
        self.raw_data = data.to_vec();
        self
    }

    pub fn geo_keys(
        mut self,
        scale: [f64; 3],
        tiepoint: [f64; 6],
        keys: &[u16],
        cit: Option<&str>,
    ) -> Self {
        self.add_double_vec(33550, &scale);
        self.add_double_vec(33922, &tiepoint);
        self.add_short_vec(34735, keys);
        if let Some(c) = cit {
            self.add_ascii(34737, c);
        }
        self
    }

    pub fn add_short(&mut self, tag: u16, val: u16) {
        self.entries
            .insert(tag, (TiffType::Short, val.to_le_bytes().to_vec()));
    }

    pub fn add_long(&mut self, tag: u16, val: u32) {
        self.entries
            .insert(tag, (TiffType::Long, val.to_le_bytes().to_vec()));
    }

    pub fn add_short_vec(&mut self, tag: u16, vals: &[u16]) {
        self.entries.insert(
            tag,
            (
                TiffType::Short,
                vals.iter().flat_map(|v| v.to_le_bytes()).collect(),
            ),
        );
    }

    pub fn add_long_vec(&mut self, tag: u16, vals: &[u32]) {
        self.entries.insert(
            tag,
            (
                TiffType::Long,
                vals.iter().flat_map(|v| v.to_le_bytes()).collect(),
            ),
        );
    }

    pub fn add_double_vec(&mut self, tag: u16, vals: &[f64]) {
        self.entries.insert(
            tag,
            (
                TiffType::Double,
                vals.iter().flat_map(|v| v.to_le_bytes()).collect(),
            ),
        );
    }

    pub fn add_ascii(&mut self, tag: u16, text: &str) {
        let mut buf = text.as_bytes().to_vec();
        buf.push(0);
        self.entries.insert(tag, (TiffType::Ascii, buf));
    }

    pub fn build(mut self) -> Vec<u8> {
        let is_tiled = self.entries.contains_key(&322);
        let num_entries = self.entries.len() + 1;
        let ifd_offset = 8u32;
        let extra_offset = ifd_offset + 2 + (num_entries as u32 * 12) + 4;
        let offset_tag = if is_tiled { 324 } else { 273 };
        let offset_count = self
            .entries
            .get(&325)
            .map(|(_, b)| b.len() / 4)
            .unwrap_or(1) as u32;

        self.entries.insert(
            offset_tag,
            (TiffType::Long, vec![0u8; (offset_count as usize) * 4]),
        );
        let (mut ifd_bytes, mut extra_data) = (vec![], vec![]);
        ifd_bytes.extend_from_slice(&(num_entries as u16).to_le_bytes());

        for (&tag, (dtype, bytes)) in &self.entries {
            encode_ifd_entry(
                tag,
                *dtype,
                bytes,
                &mut ifd_bytes,
                &mut extra_data,
                extra_offset,
            );
        }
        ifd_bytes.extend_from_slice(&0u32.to_le_bytes());

        let data_start = ifd_offset + ifd_bytes.len() as u32 + extra_data.len() as u32;
        let corrected = fixup_offsets(
            &ifd_bytes,
            &mut extra_data,
            offset_tag,
            data_start,
            offset_count,
            self.raw_data.len(),
        );

        let mut final_out = Vec::with_capacity(data_start as usize + self.raw_data.len());
        final_out.extend_from_slice(b"II\x2a\x00");
        final_out.extend_from_slice(&ifd_offset.to_le_bytes());
        final_out.extend_from_slice(&corrected);
        final_out.extend_from_slice(&extra_data);
        final_out.extend_from_slice(&self.raw_data);
        final_out
    }
}

fn encode_ifd_entry(
    tag: u16,
    dtype: TiffType,
    bytes: &[u8],
    ifd: &mut Vec<u8>,
    extra: &mut Vec<u8>,
    extra_base: u32,
) {
    let count = match dtype {
        TiffType::Byte | TiffType::Ascii => bytes.len() as u32,
        TiffType::Short => (bytes.len() / 2) as u32,
        TiffType::Long => (bytes.len() / 4) as u32,
        TiffType::Double => (bytes.len() / 8) as u32,
    };
    ifd.extend_from_slice(&tag.to_le_bytes());
    ifd.extend_from_slice(&(dtype as u16).to_le_bytes());
    ifd.extend_from_slice(&count.to_le_bytes());
    if bytes.len() <= 4 {
        let mut val = [0u8; 4];
        val[..bytes.len()].copy_from_slice(bytes);
        ifd.extend_from_slice(&val);
    } else {
        let val_offset = extra_base + extra.len() as u32;
        ifd.extend_from_slice(&val_offset.to_le_bytes());
        extra.extend_from_slice(bytes);
    }
}

fn fixup_offsets(
    ifd_bytes: &[u8],
    extra_data: &mut [u8],
    offset_tag: u16,
    data_start: u32,
    count: u32,
    total_len: usize,
) -> Vec<u8> {
    let mut ifd = ifd_bytes.to_vec();
    let num_entries = u16::from_le_bytes([ifd[0], ifd[1]]) as usize;
    for i in 0..num_entries {
        let pos = 2 + i * 12;
        if u16::from_le_bytes([ifd[pos], ifd[pos + 1]]) == offset_tag {
            if count == 1 {
                ifd[pos + 8..pos + 12].copy_from_slice(&data_start.to_le_bytes());
            } else {
                let extra_pos =
                    u32::from_le_bytes(ifd[pos + 8..pos + 12].try_into().unwrap_or_default())
                        as usize
                        - (8 + ifd.len());
                let tile_sz = (total_len / count as usize) as u32;
                for t in 0..count {
                    let cur_off = data_start + t * tile_sz;
                    let byte_pos = extra_pos + (t as usize * 4);
                    if byte_pos + 4 <= extra_data.len() {
                        extra_data[byte_pos..byte_pos + 4].copy_from_slice(&cur_off.to_le_bytes());
                    }
                }
            }
        }
    }
    ifd
}
