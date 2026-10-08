//! One plotted layer: its source, data, renderers and style.

use super::{
    ColorStyle, CompositeStyle, LayerData, LayerRenderers, LoadState, Source, VariableSelection,
};

#[derive(Default)]
pub struct Layer {
    pub source: Source,
    pub data: LayerData,
    pub renderers: LayerRenderers,
    pub color: ColorStyle,
    pub composite: CompositeStyle,
    pub load: LoadState,
}

impl Layer {
    pub fn selection(&self) -> &VariableSelection {
        self.source.selection()
    }

    pub fn selection_mut(&mut self) -> &mut VariableSelection {
        self.source.selection_mut()
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
