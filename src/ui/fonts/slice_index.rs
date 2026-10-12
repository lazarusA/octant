//! Which slice of a CJK font holds each character: written by
//! `tools/cjk_slices` (which includes this file by path, so it depends on
//! nothing in Octant) as `fonts/cjk/<sc|jp|kr>/index.bin`, read by the web
//! build to fetch only the slices a dataset's text needs.
//!
//! Layout, little-endian: `OCJK`, version `u8`, slice count `u16`, range
//! count `u32`, then each range of consecutive codepoints in one slice as
//! first `u32`, last `u32`, slice `u16`, sorted by codepoint.

const MAGIC: &[u8; 4] = b"OCJK";
const VERSION: u8 = 1;
const HEADER: usize = 11;
const RANGE: usize = 10;

/// File name of the index in each font's slice folder.
pub const INDEX_FILE: &str = "index.bin";

/// File name of slice `slice`.
pub fn slice_file(slice: u16) -> String {
    format!("{slice:03}.otf")
}

/// Ranges of codepoints and the slice holding them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SliceIndex {
    slices: u16,
    /// `(first, last, slice)`, sorted, not overlapping.
    ranges: Vec<(u32, u32, u16)>,
}

impl SliceIndex {
    /// The index of `slices` slices, each listed as the codepoints it holds.
    pub fn from_slices(slices: &[Vec<u32>]) -> Self {
        let mut assigned: Vec<(u32, u16)> = slices
            .iter()
            .enumerate()
            .flat_map(|(slice, codepoints)| {
                let slice = u16::try_from(slice).unwrap_or(u16::MAX);
                codepoints.iter().map(move |&c| (c, slice))
            })
            .collect();
        assigned.sort_unstable();
        let mut ranges: Vec<(u32, u32, u16)> = Vec::new();
        for (c, slice) in assigned {
            match ranges.last_mut() {
                Some((_, last, s)) if *s == slice && last.checked_add(1) == Some(c) => *last = c,
                _ => ranges.push((c, c, slice)),
            }
        }
        Self {
            slices: u16::try_from(slices.len()).unwrap_or(u16::MAX),
            ranges,
        }
    }

    /// Number of slices.
    pub fn slice_count(&self) -> u16 {
        self.slices
    }

    /// The slice holding `c`, `None` when the font lacks it.
    pub fn slice_of(&self, c: char) -> Option<u16> {
        let c = u32::from(c);
        let at = self.ranges.partition_point(|&(_, last, _)| last < c);
        self.ranges
            .get(at)
            .filter(|&&(first, _, _)| first <= c)
            .map(|&(_, _, slice)| slice)
    }

    /// The index as bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER + self.ranges.len() * RANGE);
        out.extend_from_slice(MAGIC);
        out.push(VERSION);
        out.extend_from_slice(&self.slices.to_le_bytes());
        let count = u32::try_from(self.ranges.len()).unwrap_or(u32::MAX);
        out.extend_from_slice(&count.to_le_bytes());
        for &(first, last, slice) in &self.ranges {
            out.extend_from_slice(&first.to_le_bytes());
            out.extend_from_slice(&last.to_le_bytes());
            out.extend_from_slice(&slice.to_le_bytes());
        }
        out
    }

    /// Reads an index written by [`Self::encode`]; `None` when malformed.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.get(..4)? != MAGIC || *bytes.get(4)? != VERSION {
            return None;
        }
        let slices = u16::from_le_bytes(bytes.get(5..7)?.try_into().ok()?);
        let count = u32::from_le_bytes(bytes.get(7..11)?.try_into().ok()?) as usize;
        let body = bytes.get(HEADER..)?;
        if body.len() != count.checked_mul(RANGE)? {
            return None;
        }
        let ranges = body
            .as_chunks::<RANGE>()
            .0
            .iter()
            .map(|r| {
                let first = u32::from_le_bytes(r.get(0..4)?.try_into().ok()?);
                let last = u32::from_le_bytes(r.get(4..8)?.try_into().ok()?);
                let slice = u16::from_le_bytes(r.get(8..10)?.try_into().ok()?);
                (first <= last && slice < slices).then_some((first, last, slice))
            })
            .collect::<Option<Vec<_>>>()?;
        let sorted = ranges.windows(2).all(|w| w[0].1 < w[1].0);
        sorted.then_some(Self { slices, ranges })
    }
}
