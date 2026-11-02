//! One plotted layer: its source, data, renderers and style.

use super::{
    Alignment, ColorStyle, ColorbarPlacement, CompositeStyle, LayerData, LayerId, LayerRenderers,
    LoadState, Slot, Source, VariableSelection,
};
use crate::data::MatrixData;
use crate::plots::VolumeEncoding;

pub struct Layer {
    /// Given by the `LayerStack`; read with `id()`.
    id: LayerId,
    pub source: Source,
    pub data: LayerData,
    pub renderers: LayerRenderers,
    pub color: ColorStyle,
    pub composite: CompositeStyle,
    pub load: LoadState,
    /// Drawn on the canvas (the layer list's eye toggle).
    pub visible: bool,
    /// How the layer lines up with the base layer; the base is `SameGrid`.
    pub alignment: Alignment,
    /// Where the layer's colorbar sits on the canvas.
    pub colorbar: ColorbarPlacement,
}

impl Layer {
    /// A layer drawn from `source`, with default data, renderers and style.
    /// Its colorbar starts in `slot`.
    pub(super) fn new(id: LayerId, source: Source, slot: Slot) -> Self {
        Self {
            id,
            source,
            data: LayerData::default(),
            renderers: LayerRenderers::default(),
            color: ColorStyle {
                alpha_key: id.key(),
                ..ColorStyle::default()
            },
            composite: CompositeStyle::default(),
            load: LoadState::default(),
            visible: true,
            alignment: Alignment::SameGrid,
            colorbar: ColorbarPlacement::at(slot),
        }
    }

    pub fn id(&self) -> LayerId {
        self.id
    }

    /// Whether the layer draws on the canvas: visible, and lined up with the
    /// base layer.
    pub fn is_drawn(&self) -> bool {
        self.visible && self.alignment.is_drawn()
    }

    pub fn selection(&self) -> &VariableSelection {
        self.source.selection()
    }

    pub fn selection_mut(&mut self) -> &mut VariableSelection {
        self.source.selection_mut()
    }

    /// The label style, when the layer shows an OME-Zarr label image.
    pub fn labels(&self) -> Option<&super::LabelStyle> {
        self.source.labels()
    }

    /// How this layer's pyramid aggregates cells: `chosen`, except for label
    /// images, whose ids are never averaged (one id per cell).
    pub fn pyramid_op(&self, chosen: crate::data::AggregationOp) -> crate::data::AggregationOp {
        if self.labels().is_some() {
            crate::data::AggregationOp::Nearest
        } else {
            chosen
        }
    }

    /// How volume values reach the GPU: packed RGB in composite mode.
    pub fn volume_encoding(&self) -> VolumeEncoding {
        if self.composite.enabled {
            VolumeEncoding::PackedRgb
        } else {
            VolumeEncoding::Scalar
        }
    }

    /// Invalidates the 2D data and its renderers.
    pub fn clear_2d(&mut self) {
        self.data.matrix = None;
        self.data.touch_matrix();
        self.renderers.heatmap = None;
        self.renderers.labels = None;
        self.renderers.channels = None;
        self.renderers.detail = None;
        self.renderers.detail_channels = None;
        self.data.detail.clear();
        self.renderers.sphere = None;
        self.renderers.surface = None;
        self.data.mesh = None;
        self.renderers.line = None;
    }

    /// How this layer's surface and sphere grid thins its 2D data, for the
    /// bake of a composite the GPU blends.
    pub fn mesh_thinning(&self) -> crate::plots::Thinning {
        let (Some(mesh), Some(full)) = (&self.data.mesh, &self.data.matrix) else {
            return crate::plots::Thinning::NONE;
        };
        let size = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        crate::plots::Thinning {
            stride: size(mesh.stride),
            width: size(mesh.matrix.width),
            height: size(mesh.matrix.height),
            full_width: size(full.width),
            full_height: size(full.height),
        }
    }

    /// The grid this layer's surface and sphere draw for its 2D data
    /// `full`, and its stride over `full` (1 when it is `full`).
    pub fn mesh_view<'a>(&'a self, full: &'a MatrixData) -> (&'a MatrixData, usize) {
        match &self.data.mesh {
            Some(mesh) => (&mesh.matrix, mesh.stride),
            None => (full, 1),
        }
    }

    /// Invalidates the 3D volume and its renderers.
    pub fn clear_3d(&mut self) {
        self.data.volume = None;
        self.data.channel_volume = None;
        self.data.points = None;
        self.data.touch_volume();
        self.renderers.volume = None;
        self.renderers.point_cloud = None;
    }

    /// The plotted variable's name with its units, or "Scalar Field",
    /// written into `buf` (per-frame UI code formats without allocating).
    pub fn write_default_label<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        let var = self.selection().variable_info();
        let Some(var) = var else {
            return crate::i18n::tr(crate::i18n::PlotKey::ScalarField);
        };
        match var.attributes.get("units").or(var.units.as_ref()) {
            Some(unit) => crate::utils::stack_str(buf, format_args!("{} ({unit})", var.name)),
            None => crate::utils::stack_str(buf, format_args!("{}", var.name)),
        }
    }

    /// The plotted variable's name with its units, or "Scalar Field".
    pub fn default_colorbar_label(&self) -> String {
        let mut buf = [0u8; LABEL_BUF];
        self.write_default_label(&mut buf).to_string()
    }

    /// The custom colorbar label when set, else the default one.
    pub fn colorbar_label(&self) -> String {
        match &self.color.custom_label {
            Some(custom) => custom.clone(),
            None => self.default_colorbar_label(),
        }
    }
}

/// Room for a formatted colorbar label (longer ones are cut at a char
/// boundary).
pub const LABEL_BUF: usize = 192;
