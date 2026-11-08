//! The header of a classic NetCDF file (CDF-1, CDF-2 64-bit offset, CDF-5 64-bit data), as
//! the NetCDF classic format specification lays it out: big-endian, lists of dimensions,
//! global attributes and variables, names and values padded to 4 bytes.

/// An external type and its size in bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NcType {
    Byte,
    Char,
    Short,
    Int,
    Float,
    Double,
    UByte,
    UShort,
    UInt,
    Int64,
    UInt64,
}

impl NcType {
    fn of(code: u32) -> Option<Self> {
        use NcType as T;
        Some(match code {
            1 => T::Byte,
            2 => T::Char,
            3 => T::Short,
            4 => T::Int,
            5 => T::Float,
            6 => T::Double,
            7 => T::UByte,
            8 => T::UShort,
            9 => T::UInt,
            10 => T::Int64,
            11 => T::UInt64,
            _ => return None,
        })
    }

    pub fn size(self) -> u64 {
        use NcType as T;
        match self {
            T::Byte | T::Char | T::UByte => 1,
            T::Short | T::UShort => 2,
            T::Int | T::Float | T::UInt => 4,
            T::Double | T::Int64 | T::UInt64 => 8,
        }
    }

    /// The type name netCDF-C gives it.
    pub fn name(self) -> &'static str {
        use NcType as T;
        match self {
            T::Byte => "int8",
            T::Char => "char",
            T::Short => "int16",
            T::Int => "int32",
            T::Float => "float32",
            T::Double => "float64",
            T::UByte => "uint8",
            T::UShort => "uint16",
            T::UInt => "uint32",
            T::Int64 => "int64",
            T::UInt64 => "uint64",
        }
    }
}

pub struct Attr {
    pub name: String,
    pub ty: NcType,
    /// Its values, big-endian.
    pub bytes: Vec<u8>,
}

pub struct Var {
    pub name: String,
    pub dimids: Vec<usize>,
    pub attrs: Vec<Attr>,
    pub ty: NcType,
    /// Where its data (or its first record's) starts.
    pub begin: u64,
}

pub struct Header {
    /// `None` while a writer streams records (the count is then read from the file size).
    pub numrecs: Option<u64>,
    /// Each dimension's name and length, 0 for the record dimension.
    pub dims: Vec<(String, u64)>,
    pub attrs: Vec<Attr>,
    pub vars: Vec<Var>,
}

/// Why the header could not be read: the bytes ran out (read more), or it is malformed.
pub enum Short {
    More,
    Bad(String),
}

const DIMENSION: u32 = 0x0A;
const VARIABLE: u32 = 0x0B;
const ATTRIBUTE: u32 = 0x0C;

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
    version: u8,
}

impl Cursor<'_> {
    fn take(&mut self, n: u64) -> Result<&[u8], Short> {
        let n = usize::try_from(n).map_err(|_| Short::Bad("Header field too large".into()))?;
        let end = self.at.checked_add(n).ok_or(Short::More)?;
        let bytes = self.bytes.get(self.at..end).ok_or(Short::More)?;
        self.at = end;
        Ok(bytes)
    }

    fn u32(&mut self) -> Result<u32, Short> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn u64(&mut self) -> Result<u64, Short> {
        let b = self.take(8)?;
        Ok(u64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    /// A count or length: 8 bytes in CDF-5, else 4.
    fn non_neg(&mut self) -> Result<u64, Short> {
        match self.version {
            5 => self.u64(),
            _ => self.u32().map(u64::from),
        }
    }

    /// `n` bytes and the padding up to the next multiple of 4.
    fn padded(&mut self, n: u64) -> Result<Vec<u8>, Short> {
        let bytes = self.take(n)?.to_vec();
        self.take(n.next_multiple_of(4) - n)?;
        Ok(bytes)
    }

    fn name(&mut self) -> Result<String, Short> {
        let n = self.non_neg()?;
        Ok(String::from_utf8_lossy(&self.padded(n)?).into_owned())
    }

    /// The element count of a list tagged `tag`; 0 when absent.
    fn list(&mut self, tag: u32) -> Result<u64, Short> {
        let found = self.u32()?;
        let n = self.non_neg()?;
        match (found, n) {
            (0, 0) => Ok(0),
            (t, n) if t == tag => Ok(n),
            _ => Err(Short::Bad(format!("Unexpected header tag {found:#x}"))),
        }
    }

    fn attrs(&mut self) -> Result<Vec<Attr>, Short> {
        (0..self.list(ATTRIBUTE)?)
            .map(|_| {
                let name = self.name()?;
                let code = self.u32()?;
                let ty = NcType::of(code).ok_or(Short::Bad(format!("Unknown type {code}")))?;
                let n = self.non_neg()?;
                let bytes = self.padded(n.saturating_mul(ty.size()))?;
                Ok(Attr { name, ty, bytes })
            })
            .collect()
    }

    /// A variable over the file's `dims` dimensions.
    fn var(&mut self, dims: usize) -> Result<Var, Short> {
        let name = self.name()?;
        let ndims = self.non_neg()?;
        let dimids = (0..ndims)
            .map(|_| {
                let id = self.non_neg()?;
                (usize::try_from(id).ok().filter(|&d| d < dims))
                    .ok_or_else(|| Short::Bad(format!("'{name}' has no dimension {id}")))
            })
            .collect::<Result<_, Short>>()?;
        let attrs = self.attrs()?;
        let code = self.u32()?;
        let ty = NcType::of(code).ok_or(Short::Bad(format!("Unknown type {code}")))?;
        // vsize is recomputed from the shape: it overflows for large variables.
        self.non_neg()?;
        let begin = match self.version {
            1 => u64::from(self.u32()?),
            _ => self.u64()?,
        };
        Ok(Var {
            name,
            dimids,
            attrs,
            ty,
            begin,
        })
    }
}

/// Whether `head` starts a classic NetCDF file.
pub fn is_classic(head: &[u8]) -> bool {
    matches!(head, [b'C', b'D', b'F', 1 | 2 | 5, ..])
}

/// The header at the start of `bytes`.
pub fn parse(bytes: &[u8]) -> Result<Header, Short> {
    let version = match bytes {
        [b'C', b'D', b'F', v @ (1 | 2 | 5), ..] => *v,
        [_, _, _, _, ..] => return Err(Short::Bad("Not a classic NetCDF file".into())),
        _ => return Err(Short::More),
    };
    let mut c = Cursor {
        bytes,
        at: 4,
        version,
    };
    let numrecs = c.non_neg()?;
    let streaming = numrecs == u64::from(u32::MAX) || numrecs == u64::MAX;
    let dims: Vec<(String, u64)> = (0..c.list(DIMENSION)?)
        .map(|_| Ok((c.name()?, c.non_neg()?)))
        .collect::<Result<_, Short>>()?;
    let attrs = c.attrs()?;
    let vars = (0..c.list(VARIABLE)?)
        .map(|_| c.var(dims.len()))
        .collect::<Result<_, Short>>()?;
    Ok(Header {
        numrecs: (!streaming).then_some(numrecs),
        dims,
        attrs,
        vars,
    })
}
