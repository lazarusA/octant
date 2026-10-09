//! One plotted layer: its source, data, renderers and style.

use super::{
    Alignment, ColorStyle, CompositeStyle, LayerData, LayerId, LayerRenderers, LoadState, Source,
    VariableSelection,
};
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
}

impl Layer {
    /// A layer drawn from `source`, with default data, renderers and style.
    pub(super) fn new(id: LayerId, source: Source) -> Self {
        Self {
            id,
            source,
            data: LayerData::default(),
            renderers: LayerRenderers::default(),
            color: ColorStyle::default(),
            composite: CompositeStyle::default(),
            load: LoadState::default(),
            visible: true,
            alignment: Alignment::SameGrid,
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
        self.renderers.heatmap = None;
        self.renderers.sphere = None;
        self.renderers.surface = None;
        self.renderers.line = None;
    }

    /// Invalidates the 3D volume and its renderers.
    pub fn clear_3d(&mut self) {
        self.data.volume = None;
        self.renderers.volume = None;
        self.renderers.point_cloud = None;
    }

    /// The plotted variable's name with its units, or "Scalar Field".
    pub fn default_colorbar_label(&self) -> String {
        let Some(meta) = &self.selection().metadata else {
            return "Scalar Field".to_string();
        };
        meta.variables
            .get(self.selection().variable_idx)
            .map(|v| {
                if let Some(unit) = v.attributes.get("units").or(v.units.as_ref()) {
                    format!("{} ({})", v.name, unit)
                } else {
                    v.name.clone()
                }
            })
            .unwrap_or_else(|| "Scalar Field".to_string())
    }

    /// The custom colorbar label when set, else the default one.
    pub fn colorbar_label(&self) -> String {
        if let Some(custom) = &self.color.custom_label {
            custom.clone()
        } else {
            self.default_colorbar_label()
        }
    }
}
