//! What the plotted view's arriving block tells the user: that its step holds
//! no values, or that its variable's chunks are so large every step loads
//! slowly.

use crate::app::OctantApp;
use crate::data::{DataSourceKind, OctantBlock, VariableInfo};
use crate::i18n::{NoticesKey as N, filled, tr};
use crate::ui::toast::Severity;

/// Stored chunks past this many bytes, decoded, make every step slow: a fetch
/// downloads and decodes whole chunks whatever window it reads.
pub(crate) const LARGE_CHUNK_BYTES: usize = 64 * 1024 * 1024;

/// Whether a fetch from a source of `kind` at `uri` downloads whole stored
/// chunks: remote Zarr, Icechunk and GeoTIFF (tiles), not the browser's
/// local files and folders, which register as those kinds but read from
/// disk. Readers without chunks (NetCDF, GRIB) read what they need;
/// procedural data is generated.
fn fetches_whole_chunks(kind: &DataSourceKind, uri: &str) -> bool {
    let remote = matches!(
        kind,
        DataSourceKind::RemoteZarr | DataSourceKind::RemoteIcechunk | DataSourceKind::RemoteGeoTiff
    );
    remote && !uri.starts_with(crate::data::backends::local::SCHEME)
}

/// Bytes of one stored chunk of `var`, decoded, when its source of `kind`
/// at `uri` fetches whole chunks ([`fetches_whole_chunks`]); 0 otherwise.
/// An array stored as one chunk counts whole (the worst case), except from
/// Icechunk (an `icechunk+` address opened as Zarr included), whose browser
/// inspection reports that shape when it cannot read the array's metadata.
pub(crate) fn stored_chunk_bytes(var: &VariableInfo, kind: &DataSourceKind, uri: &str) -> usize {
    // A Zarr chunk may run past the array's edge; only the array's part matters.
    let clamped = || {
        var.chunk_shape
            .iter()
            .zip(&var.shape)
            .map(|(&c, &s)| c.min(s))
    };
    let icechunk = *kind == DataSourceKind::RemoteIcechunk
        || crate::utils::remote::strip_scheme(uri, "icechunk+").is_some();
    let unknown = icechunk && clamped().eq(var.shape.iter().copied());
    if !fetches_whole_chunks(kind, uri) || var.chunk_shape.len() != var.shape.len() || unknown {
        return 0;
    }
    let dtype = crate::utils::data_type_bytes(&var.data_type);
    clamped()
        .try_fold(dtype, |acc, n| acc.checked_mul(n))
        .and_then(|b| usize::try_from(b).ok())
        .unwrap_or(0) // a size past usize is corrupt metadata, not a chunk
}

/// Whether `values` hold nothing but NaN or one repeated value.
pub(crate) fn holds_no_data<'a>(values: impl IntoIterator<Item = &'a f32>) -> bool {
    let mut finite = values.into_iter().filter(|v| v.is_finite());
    let first = finite.next();
    first.is_none_or(|f| finite.all(|v| v == f))
}

impl OctantApp {
    /// Tells the user when the base layer's `block`, fetched or from the
    /// cache, holds no data at the shown step outside the span its variable's
    /// attributes state: a missing chunk reads as the fill value, NaN or (a
    /// Zarr `fill_value` no attribute masks) one constant. Inside the span, a
    /// masked region is data. Once per run of steps outside the span, however
    /// far apart; only a step inside ends it (here or `stage_current_step`).
    /// Not while playing, which steps on by itself.
    pub(super) fn notice_empty_block(&mut self, block: &OctantBlock) {
        if self.playback.is_playing {
            return;
        }
        if !self.plotted_step_outside_coverage() {
            self.playback.empty_step_warned = None;
            return;
        }
        if !self.shown_step_holds_no_data(block) {
            return;
        }
        let plotted = self.plotted_variable();
        if self.playback.empty_step_warned.as_ref() == Some(&plotted) {
            return;
        }
        self.playback.empty_step_warned = Some(plotted);
        let detail = filled(tr(N::NoDataAtStepDetail), &[&block.variable_name]);
        self.notify(Severity::Warning, tr(N::NoDataAtStep), detail);
    }

    /// Whether the shown step's slab of `block` (which may hold several steps,
    /// a chunk's or a volume ring's) holds no data; the whole block when it
    /// does not name the animated dimension.
    fn shown_step_holds_no_data(&self, block: &OctantBlock) -> bool {
        let selection = self.layers.base.selection();
        let anim = selection
            .variable_info()
            .zip(selection.animated_dim())
            .and_then(|(v, d)| v.dimension_names.get(d));
        let Some(pos) = anim.and_then(|n| block.dimension_names.iter().position(|b| b == n)) else {
            return holds_no_data(block.values.iter());
        };
        let (Some(&origin), Some(&n), Some(&stride)) = (
            block.origin.get(pos),
            block.shape.get(pos),
            block.strides.get(pos),
        ) else {
            return holds_no_data(block.values.iter());
        };
        let step = self.playback.current_timestep;
        let Some(k) = step.checked_sub(origin).filter(|&k| k < n) else {
            return false;
        };
        // Row-major: the step's slab is one run of `stride` values in every
        // `stride * n` of the block.
        let stride = stride.max(1);
        let runs = block.values.chunks(stride * n);
        holds_no_data(runs.flat_map(|run| run.get(k * stride..(k + 1) * stride).unwrap_or(&[])))
    }

    /// The plotted view's store and variable, which a new plot changes.
    pub(super) fn plotted_variable(&self) -> (String, Option<String>) {
        let s = self.layers.base.selection();
        let name = s.variable_info().map(|v| v.name.clone());
        (s.store_target.clone(), name)
    }

    /// Tells the user, once a new plot arrives, when its variable's stored
    /// chunks pass [`LARGE_CHUNK_BYTES`].
    pub(crate) fn notice_large_chunks(&mut self) {
        let selection = self.layers.base.selection();
        let Some(var) = selection.variable_info() else {
            return;
        };
        let kind = selection.store_kind.to_data_source_kind();
        let bytes = stored_chunk_bytes(var, &kind, &selection.store_target);
        if bytes <= LARGE_CHUNK_BYTES {
            return;
        }
        let size = crate::utils::units::ByteSize(bytes as u64);
        let detail = filled(tr(N::LargeChunksDetail), &[&var.name, &size]);
        self.notify(Severity::Info, tr(N::LargeChunks), detail);
    }
}
