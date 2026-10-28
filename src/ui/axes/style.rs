//! Which ticks and grid lines the axes draw (Settings > View).

/// Where minor ticks go between the labelled ones.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinorPlacement {
    /// Round subdivisions of the labelled step.
    Round,
    /// Each data cell's center, where the hover snaps, while cells are wide
    /// enough; round subdivisions otherwise.
    #[default]
    Cells,
}

/// The axes' tick marks and grid lines, major and minor, each on or off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxisStyle {
    pub major_ticks: bool,
    pub major_grid: bool,
    pub minor_ticks: bool,
    pub minor_grid: bool,
    pub minor_placement: MinorPlacement,
}

impl Default for AxisStyle {
    fn default() -> Self {
        Self {
            major_ticks: true,
            major_grid: false,
            minor_ticks: true,
            minor_grid: false,
            minor_placement: MinorPlacement::Cells,
        }
    }
}

impl AxisStyle {
    /// Whether anything is drawn at the majors.
    pub fn majors(&self) -> bool {
        self.major_ticks || self.major_grid
    }

    /// Where minors go, when anything is drawn at them.
    pub fn minors(&self) -> Option<MinorPlacement> {
        (self.minor_ticks || self.minor_grid).then_some(self.minor_placement)
    }
}
