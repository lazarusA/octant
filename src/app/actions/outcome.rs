//! What actions did, or why they did nothing: data, worded by the front
//! ends (`answer.rs` for voice).

use super::DimRole;
use super::ask::DimSummary;
use super::resolve::Resolution;
use crate::plots::PlotType;

/// What an action did.
#[derive(Debug, Clone, PartialEq)]
pub enum Done {
    Opening(String),
    CatalogOpened,
    Selected(String),
    /// A dataset or group: its name, its variables counted, the first
    /// entries (groups, then variables).
    DatasetShown {
        name: String,
        variables: usize,
        entries: Vec<String>,
    },
    /// The Variables panel searched for `text`.
    Searching {
        text: String,
        matches: usize,
    },
    VariableShown {
        name: String,
        long_name: Option<String>,
        units: Option<String>,
        dims: Vec<DimSummary>,
    },
    /// What is plotted: the step of its animated dimension, its colormap,
    /// its overlays, and the variable staged when another one is.
    ViewShown {
        variable: String,
        plot_type: PlotType,
        step: Option<(String, String)>,
        colormap: String,
        overlays: usize,
        staged: Option<String>,
    },
    /// A dimension and the coordinate it moved to, as the hover writes it.
    DimSet {
        dim: String,
        value: String,
    },
    RangeSet {
        dim: String,
        from: String,
        to: String,
    },
    RoleSet {
        dim: String,
        role: DimRole,
    },
    Plotted {
        variable: String,
        plot_type: PlotType,
    },
    Playing,
    Paused,
    Stepped(i32),
    Colormap(String),
    ViewReset,
}

/// What a name named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameKind {
    Source,
    Group,
    Dimension,
    Variable,
    PlotType,
    Colormap,
}

/// Why an action did nothing.
#[derive(Debug, Clone, PartialEq)]
pub enum Failure {
    NoDataset,
    NotFound {
        kind: NameKind,
        text: String,
        closest: Vec<String>,
    },
    Ambiguous {
        kind: NameKind,
        text: String,
        matches: Vec<String>,
    },
    /// The staged variable cannot be drawn as this plot type.
    Blocked {
        plot_type: PlotType,
        reason: String,
    },
    NothingToAnimate,
    NotUnderstood(String),
    /// The app refused and said why (a desktop-only catalog entry).
    Refused,
    /// No coordinate of `dim` matches `value`.
    NoValue {
        dim: String,
        value: String,
    },
    /// The coordinates are being read: the command waits for them.
    CoordinatesPending,
}

pub type Outcome = Result<Done, Failure>;

/// `resolution` of `text` as a value or the failure it answers.
pub(super) fn resolved<T>(
    resolution: Resolution<T>,
    kind: NameKind,
    text: &str,
) -> Result<T, Failure> {
    let text = text.to_string();
    match resolution {
        Resolution::Resolved(value) => Ok(value),
        Resolution::Ambiguous(matches) => Err(Failure::Ambiguous {
            kind,
            text,
            matches,
        }),
        Resolution::NotFound(closest) => Err(Failure::NotFound {
            kind,
            text,
            closest,
        }),
    }
}
