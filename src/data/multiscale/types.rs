//! A multiscale stack of resolution levels, read the same way from every format.

use std::fmt;
use std::sync::Arc;

/// Where a stack's levels were described.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiscaleKind {
    /// OME-Zarr `multiscales` (`datasets` with per-axis scale and translation).
    Ome,
    /// The zarr-conventions `multiscales` `layout` used by GeoZarr pyramids.
    GeoZarr,
    /// A (Cloud Optimized) GeoTIFF's reduced-resolution IFDs.
    GeoTiffOverviews,
}

/// One resolution level: its variable, its shape, and where its cells lie
/// on the finest level's index grid.
///
/// Along dimension `d`, cell `i` of this level covers the level-0 index
/// interval `[offset[d] + i * factor[d], offset[d] + (i + 1) * factor[d])`,
/// where level-0 cell `j` covers `[j, j + 1)`.
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleLevel {
    /// The level's variable (`VariableInfo::name`).
    pub path: String,
    pub shape: Vec<u64>,
    /// Level-0 cells per cell of this level, per dimension (`1.0` where the
    /// dimension is not downsampled: time, channel, band, ...).
    pub factor: Vec<f64>,
    /// The level-0 index where this level's cell 0 starts, per dimension.
    pub offset: Vec<f64>,
}

/// The levels of one variable, finest (level 0, the largest) first.
#[derive(Debug, Clone, PartialEq)]
pub struct Multiscale {
    /// The name the stack is shown under.
    pub name: String,
    /// Dimension names, shared by every level.
    pub dims: Vec<String>,
    pub levels: Vec<ScaleLevel>,
    pub kind: MultiscaleKind,
}

/// Why levels do not form a multiscale stack.
#[derive(Debug, Clone, PartialEq)]
pub enum MultiscaleError {
    /// Fewer than two levels.
    TooFewLevels,
    /// A level's shape, factor or offset does not have one entry per dimension.
    Rank { level: usize },
    /// Level 0 is not the identity (factor 1, offset 0).
    BaseNotIdentity,
    /// A factor is not finite or below 1.
    BadFactor { level: usize, dim: usize },
    /// A factor is smaller than the previous level's.
    NotCoarsening { level: usize, dim: usize },
    /// A dimension grows, or keeps factor 1 with another length.
    Shape { level: usize, dim: usize },
}

impl fmt::Display for MultiscaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewLevels => write!(f, "fewer than two levels"),
            Self::Rank { level } => write!(f, "level {level} has the wrong rank"),
            Self::BaseNotIdentity => write!(f, "level 0 is not the identity"),
            Self::BadFactor { level, dim } => {
                write!(
                    f,
                    "level {level} has an invalid factor along dimension {dim}"
                )
            }
            Self::NotCoarsening { level, dim } => {
                write!(
                    f,
                    "level {level} is finer than level {} along dimension {dim}",
                    level - 1
                )
            }
            Self::Shape { level, dim } => {
                write!(
                    f,
                    "level {level} has an inconsistent length along dimension {dim}"
                )
            }
        }
    }
}

/// Tolerance for factors read from floating-point metadata.
const EPS: f64 = 1e-6;

impl Multiscale {
    /// A stack of `levels`, finest first, after checking that they are one:
    /// at least two levels, one entry per dimension everywhere, level 0 the
    /// identity, factors at least 1 and never shrinking, and no dimension
    /// longer than at level 0 (nor of another length where its factor is 1).
    pub fn new(
        name: String,
        dims: Vec<String>,
        levels: Vec<ScaleLevel>,
        kind: MultiscaleKind,
    ) -> Result<Self, MultiscaleError> {
        if levels.len() < 2 {
            return Err(MultiscaleError::TooFewLevels);
        }
        let rank = dims.len();
        for (l, level) in levels.iter().enumerate() {
            let ranks = [level.shape.len(), level.factor.len(), level.offset.len()];
            if ranks.iter().any(|&r| r != rank) {
                return Err(MultiscaleError::Rank { level: l });
            }
        }
        let base = &levels[0];
        let identity = base.factor.iter().all(|f| (f - 1.0).abs() < EPS)
            && base.offset.iter().all(|o| o.abs() < EPS);
        if !identity {
            return Err(MultiscaleError::BaseNotIdentity);
        }
        for (l, pair) in levels.windows(2).enumerate() {
            check_step(l + 1, &pair[0], &pair[1], base)?;
        }
        Ok(Self {
            name,
            dims,
            levels,
            kind,
        })
    }

    pub fn len(&self) -> usize {
        self.levels.len()
    }

    /// Never true: a stack has at least two levels.
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    pub fn level(&self, level: usize) -> Option<&ScaleLevel> {
        self.levels.get(level)
    }

    /// The finest level (level 0).
    pub fn base(&self) -> &ScaleLevel {
        &self.levels[0]
    }

    /// The level whose variable is `path`.
    pub fn level_of(&self, path: &str) -> Option<usize> {
        self.levels.iter().position(|l| l.path == path)
    }

    /// Index of dimension `name`, matched ASCII case-insensitively.
    pub fn dim_index(&self, name: &str) -> Option<usize> {
        self.dims.iter().position(|d| d.eq_ignore_ascii_case(name))
    }
}

/// Checks level `l` (`level`) against the level before it (`prev`).
fn check_step(
    l: usize,
    prev: &ScaleLevel,
    level: &ScaleLevel,
    base: &ScaleLevel,
) -> Result<(), MultiscaleError> {
    for d in 0..level.factor.len() {
        let f = level.factor[d];
        if !f.is_finite() || f < 1.0 - EPS || !level.offset[d].is_finite() {
            return Err(MultiscaleError::BadFactor { level: l, dim: d });
        }
        if f < prev.factor[d] - EPS {
            return Err(MultiscaleError::NotCoarsening { level: l, dim: d });
        }
        let len = level.shape[d];
        let grows = len == 0 || len > base.shape[d];
        let resized = (f - 1.0).abs() < EPS && len != base.shape[d];
        if grows || resized {
            return Err(MultiscaleError::Shape { level: l, dim: d });
        }
    }
    Ok(())
}

/// A variable's place in a multiscale stack.
#[derive(Debug, Clone, PartialEq)]
pub struct LevelRef {
    pub set: Arc<Multiscale>,
    pub level: usize,
}

impl LevelRef {
    pub fn info(&self) -> Option<&ScaleLevel> {
        self.set.level(self.level)
    }

    /// Whether this is the finest level, the one a stack is listed under.
    pub fn is_base(&self) -> bool {
        self.level == 0
    }
}
