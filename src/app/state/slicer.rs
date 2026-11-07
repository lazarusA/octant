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
use crate::utils::height::HeightScale;

/// What the slicer does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
    /// The plot's x, y or z data dimension (cells).
    Dim(usize),
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
            PlotType::Volume | PlotType::PointCloud => [Self::Dim(0), Self::Dim(1), Self::Dim(2)],
            PlotType::Surface | PlotType::Sphere => [Self::Dim(0), Self::Dim(1), Self::Height],
            _ => [Self::Dim(0), Self::Dim(1), Self::None],
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
}

impl Default for Slicer {
    fn default() -> Self {
        Self {
            mode: SliceMode::Off,
            lo: [0.0; 3],
            hi: [1.0; 3],
            planes: [Some(0.5), Some(0.5), None],
            guides: true,
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
    /// and y through the middle), the guides as they were.
    pub fn set_mode(&mut self, mode: SliceMode) {
        *self = Self {
            mode,
            guides: self.guides,
            ..Self::default()
        };
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

    /// The cut as the renderers read it: the box's `lo` (w the mode: 0 off,
    /// 1 box, 2 planes) and `hi`, or the planes' positions (-1 none) in `lo`.
    pub fn uniform(&self) -> [[f32; 4]; 2] {
        match self.mode {
            SliceMode::Off => [[0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 0.0]],
            SliceMode::Box => [
                [self.lo[0], self.lo[1], self.lo[2], 1.0],
                [self.hi[0], self.hi[1], self.hi[2], 0.0],
            ],
            SliceMode::Planes => {
                let at = |k: usize| self.planes[k].unwrap_or(-1.0);
                [[at(0), at(1), at(2), 2.0], [1.0, 1.0, 1.0, 0.0]]
            }
        }
    }

    /// The cells the box keeps along each axis of `cells` cells (whole
    /// axes when not cutting): those whose centers lie in `[lo, hi)`. With
    /// `flipped` an axis's range is in stored order (reversed).
    pub fn index_ranges(&self, cells: [usize; 3], flipped: [bool; 3]) -> [Range<usize>; 3] {
        std::array::from_fn(|k| {
            let n = cells[k];
            if self.mode != SliceMode::Box {
                return 0..n;
            }
            let first = |f: f32| ((f * n as f32 - 0.5).ceil().max(0.0) as usize).min(n);
            let kept = first(self.lo[k])..first(self.hi[k]);
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

    /// The data values the box keeps along a height axis (`SliceAxis::Height`),
    /// the w range read through the drawn heights `span` (their lowest and
    /// highest, as `HeightScale::bounds` with the zero plane) and `scale`;
    /// `None` for composites, whose heights are colors.
    pub fn height_values(&self, span: (f32, f32), scale: &HeightScale) -> Option<(f32, f32)> {
        let at = |w: f32| scale.value_at(span.0 + (span.1 - span.0) * w);
        let (a, b) = (at(self.lo[2])?, at(self.hi[2])?);
        Some((a.min(b), a.max(b)))
    }
}
