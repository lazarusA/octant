//! The slicer's state (Settings > View, dragged on the canvas through
//! `ui::slice_gizmo`) and the one reading of it every consumer shares: the
//! gizmo, the renderers' cut (`uniform`) and statistics over the cut
//! (`index_ranges`, `plane_indices`, `height_values`).
//!
//! Positions are fractions (0..1) of the plotted window along each drawn
//! axis (u, v, w), in drawn order (north up, `flipped_dims` applied): cell
//! `i` of `n` spans `[i/n, (i+1)/n)`. What each axis is depends on the plot
//! (`SliceAxis::of`). Fractions of the plotted window hold on any level of
//! a multiscale stack; index ranges read on the plotted level reach level 0
//! through `multiscale::to_base`.

use std::ops::Range;

use crate::plots::PlotType;
use crate::plots::slice::SliceUniform;
use crate::utils::height::HeightScale;

/// What the slicer does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SliceMode {
    #[default]
    Off,
    /// Keeps the part of the plot inside a box.
    Box,
    /// Shows the data on axis-aligned planes (volumes and point clouds).
    Planes,
}

impl SliceMode {
    /// Whether `plot_type` offers this mode.
    pub fn offered_for(self, plot_type: PlotType) -> bool {
        match self {
            Self::Off => true,
            Self::Box => plot_type != PlotType::Line,
            Self::Planes => matches!(plot_type, PlotType::Volume | PlotType::PointCloud),
        }
    }
}

/// What a slicer axis cuts on a plot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceAxis {
    /// One of the plot's x, y or z data dimensions, cut by cells.
    Cells,
    /// The drawn heights of a surface or sphere: a value range
    /// (`Slicer::height_values`).
    Height,
    /// Nothing (the heatmap's third axis).
    None,
}

impl SliceAxis {
    /// The axes u, v, w of `plot_type`.
    pub fn of(plot_type: PlotType) -> [Self; 3] {
        match plot_type {
            PlotType::Volume | PlotType::PointCloud => [Self::Cells; 3],
            PlotType::Surface | PlotType::Sphere => [Self::Cells, Self::Cells, Self::Height],
            _ => [Self::Cells, Self::Cells, Self::None],
        }
    }
}

/// The slicer. Session only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slicer {
    pub mode: SliceMode,
    /// The box mode's kept range per axis.
    pub lo: [f32; 3],
    pub hi: [f32; 3],
    /// The planes mode's plane across each axis, where shown.
    pub planes: [Option<f32>; 3],
    /// Whether the gizmo's guides show (the cut stays when hidden).
    pub guides: bool,
    /// Whether the box cuts the coastlines too (off: they stay whole over
    /// a cut heatmap, surface or sphere).
    pub cut_coastlines: bool,
}

impl Default for Slicer {
    fn default() -> Self {
        Self {
            mode: SliceMode::Off,
            lo: [0.0; 3],
            hi: [1.0; 3],
            planes: [Some(0.5), Some(0.5), None],
            guides: true,
            cut_coastlines: false,
        }
    }
}

impl Slicer {
    /// Whether the plot is cut: a mode is on, its guides shown or not.
    pub fn cutting(&self) -> bool {
        self.mode != SliceMode::Off
    }

    /// Whether the gizmo shows: a mode is on with its guides shown.
    pub fn guides_shown(&self) -> bool {
        self.cutting() && self.guides
    }

    /// Switches mode, starting it over (nothing cut, the planes across x
    /// and y through the middle), the guides and coastline choice as they
    /// were.
    pub fn set_mode(&mut self, mode: SliceMode) {
        *self = Self {
            mode,
            guides: self.guides,
            cut_coastlines: self.cut_coastlines,
            ..Self::default()
        };
    }

    /// `cut` for the coastlines: as is when they are cut, else nothing.
    pub fn for_coastlines(&self, cut: SliceUniform) -> SliceUniform {
        if self.cut_coastlines {
            cut
        } else {
            SliceUniform::OFF
        }
    }

    /// What the cut is, for caches of what it changes.
    pub fn hash_into(&self, h: &mut impl std::hash::Hasher) {
        use std::hash::Hash;
        self.mode.hash(h);
        self.lo.map(f32::to_bits).hash(h);
        self.hi.map(f32::to_bits).hash(h);
        self.planes.map(|p| p.map(f32::to_bits)).hash(h);
    }

    /// Starts over on a new plot type, whose axes mean other things; off
    /// when it does not offer the mode.
    pub fn adapt_to(&mut self, plot_type: PlotType) {
        let mode = if self.mode.offered_for(plot_type) {
            self.mode
        } else {
            SliceMode::Off
        };
        self.set_mode(mode);
    }

    /// The cut as the renderers read it (`SliceUniform`), in drawn order.
    pub fn uniform(&self) -> SliceUniform {
        self.uniform_in([false; 3], None)
    }

    /// The cut for a renderer whose axes in `flip` run the other way
    /// (fraction `f` read as `1 - f`), with the box's w range replaced by
    /// `heights` (a surface's or sphere's normalized heights,
    /// `height_range`) when given.
    pub fn uniform_in(&self, flip: [bool; 3], heights: Option<(f32, f32)>) -> SliceUniform {
        let turn = |k: usize, f: f32| if flip[k] { 1.0 - f } else { f };
        match self.mode {
            SliceMode::Off => SliceUniform::OFF,
            SliceMode::Box => {
                let (mut lo, mut hi) = ([0.0; 3], [0.0; 3]);
                for k in 0..3 {
                    let (a, b) = (turn(k, self.lo[k]), turn(k, self.hi[k]));
                    (lo[k], hi[k]) = (a.min(b), a.max(b));
                }
                if let Some((h_lo, h_hi)) = heights {
                    (lo[2], hi[2]) = (h_lo, h_hi);
                }
                SliceUniform {
                    lo: [lo[0], lo[1], lo[2], SliceUniform::BOX],
                    hi: [hi[0], hi[1], hi[2], 0.0],
                }
            }
            SliceMode::Planes => {
                let at = |k: usize| self.planes[k].map_or(-1.0, |f| turn(k, f));
                SliceUniform {
                    lo: [at(0), at(1), at(2), SliceUniform::PLANES],
                    hi: [1.0, 1.0, 1.0, 0.0],
                }
            }
        }
    }

    /// Whether the box keeps the point at fractions `fx`, `fy` along u and v
    /// (the CPU twin of `slice_box_keeps`; always without a box).
    pub fn keeps_uv(&self, fx: f32, fy: f32) -> bool {
        self.mode != SliceMode::Box
            || ((self.lo[0]..=self.hi[0]).contains(&fx) && (self.lo[1]..=self.hi[1]).contains(&fy))
    }

    /// Whether the box keeps cell `(x, y)` of a `w` x `h` grid, by its center.
    pub fn keeps_cell(&self, (x, y): (usize, usize), (w, h): (usize, usize)) -> bool {
        let center = |i: usize, n: usize| (i as f32 + 0.5) / n.max(1) as f32;
        self.keeps_uv(center(x, w), center(y, h))
    }

    /// The cells the box keeps along each axis of `cells` cells (whole
    /// axes when not cutting): those whose centers lie in `[lo, hi]`, as the
    /// shaders keep them. With `flipped` an axis's range is in stored order
    /// (reversed).
    pub fn index_ranges(&self, cells: [usize; 3], flipped: [bool; 3]) -> [Range<usize>; 3] {
        std::array::from_fn(|k| {
            let n = cells[k];
            if self.mode != SliceMode::Box {
                return 0..n;
            }
            // Cell i's center (i + 0.5) / n.
            let first = (self.lo[k] * n as f32 - 0.5).ceil().max(0.0) as usize;
            let end = ((self.hi[k] * n as f32 - 0.5).floor() + 1.0).max(0.0) as usize;
            let kept = first.min(n)..end.clamp(first.min(n), n);
            if flipped[k] {
                n - kept.end..n - kept.start
            } else {
                kept
            }
        })
    }

    /// The cell each shown plane lies in along its axis of `cells` cells,
    /// in stored order where `flipped`.
    pub fn plane_indices(&self, cells: [usize; 3], flipped: [bool; 3]) -> [Option<usize>; 3] {
        std::array::from_fn(|k| {
            let at = self.planes[k].filter(|_| self.mode == SliceMode::Planes)?;
            let n = cells[k].max(1);
            let i = ((at * n as f32) as usize).min(n - 1);
            Some(if flipped[k] { n - 1 - i } else { i })
        })
    }

    /// The normalized heights (`HeightScale::normalized`) the box keeps
    /// along a height axis (`SliceAxis::Height`): its w range over the drawn
    /// heights, `scale`'s bounds with the zero plane; open at the box's ends
    /// so nothing drawn is lost to rounding there, and wholly open on a plot
    /// drawn without `relief` (a flat globe or surface, whose gizmo has no
    /// height handles to undo a cut with).
    pub fn height_range(&self, scale: &HeightScale, relief: bool) -> (f32, f32) {
        if !relief {
            return (f32::MIN, f32::MAX);
        }
        let (low, high) = scale.bounds();
        let (low, high) = (low.min(0.0), high.max(0.0));
        let at = |w: f32| low + (high - low) * w;
        let lo = if self.lo[2] <= 0.0 {
            f32::MIN
        } else {
            at(self.lo[2])
        };
        let hi = if self.hi[2] >= 1.0 {
            f32::MAX
        } else {
            at(self.hi[2])
        };
        (lo, hi)
    }

    /// Whether the box keeps normalized height `h` (the CPU twin of the
    /// surface's and sphere's height test; always without a box).
    pub fn keeps_height(&self, h: f32, scale: &HeightScale, relief: bool) -> bool {
        let (lo, hi) = self.height_range(scale, relief);
        self.mode != SliceMode::Box || (lo..=hi).contains(&h)
    }

    /// The data values the box keeps along a height axis (`SliceAxis::Height`,
    /// through `height_range`); `None` for composites, whose heights are
    /// colors, and the open ends at the data's extremes.
    pub fn height_values(&self, scale: &HeightScale) -> Option<(f32, f32)> {
        let (lo, hi) = self.height_range(scale, true);
        let (bl, bh) = scale.bounds();
        let at = |h: f32| scale.value_at(h.clamp(bl.min(0.0), bh.max(0.0)));
        let (a, b) = (at(lo)?, at(hi)?);
        Some((a.min(b), a.max(b)))
    }
}
