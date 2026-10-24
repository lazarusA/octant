//! Dimension actions on the staged selection, as the Dimensions panel sets
//! them: an index or a range by coordinate value, a role. The staged
//! animated dimension is the plot's step, moved as the playback bar moves it.

use super::axes::{Axis, axis_of};
use super::coord_match::{Dim, index_of};
use super::resolve::{Resolution, among};
use super::{DimRole, Done, Failure, NameKind, Outcome, resolved};
use crate::app::{AnimationRole, OctantApp, SpatialRole};
use crate::data::{CoordValues, VariableInfo};

impl OctantApp {
    /// The staged variable, its dimension settings in place.
    fn staged_variable(&mut self) -> Result<VariableInfo, Failure> {
        let var = self
            .selected_variable_info()
            .cloned()
            .ok_or(Failure::NoDataset)?;
        if self.selected.dim_config.len() != var.shape.len() {
            crate::ui::variables_panel::init_variable_dimension_defaults(self, &var);
        }
        Ok(var)
    }

    /// The dimension of `var` `dim` names, by name or role word; without
    /// one, its time dimension.
    fn dim_of(&self, var: &VariableInfo, dim: Option<&str>) -> Result<usize, Failure> {
        let names: Vec<&str> = var.dimension_names.iter().map(String::as_str).collect();
        let spoken = dim.unwrap_or("time");
        let found = among(&names, spoken);
        if let Resolution::Resolved(i) = found {
            return Ok(i);
        }
        if let Some(i) = self.by_axis(var, spoken) {
            return Ok(i);
        }
        resolved(found, NameKind::Dimension, spoken)
    }

    /// The dimension of `var` an axis word names ("time", "latitude"), as
    /// `axis_of` reads each dimension, the animated one first; for time
    /// without a dimension that reads as time, the animated one.
    fn by_axis(&self, var: &VariableInfo, spoken: &str) -> Option<usize> {
        let axis = Axis::of_word(spoken)?;
        let meta = self.selected.metadata.as_ref()?;
        let config = &self.selected.dim_config;
        let is_axis = |i: usize| axis_of(meta, var, i, config.get(i)) == Some(axis);
        let animated = self.selected.animated_dim;
        animated
            .filter(|&a| is_axis(a))
            .or_else(|| (0..var.dimension_names.len()).find(|&i| is_axis(i)))
            .or(animated.filter(|_| axis == Axis::Time))
    }

    /// The index `value` names along dimension `i`, once its coordinates
    /// are read.
    fn index_for(
        &mut self,
        var: &VariableInfo,
        i: usize,
        value: &str,
        last: bool,
    ) -> Result<usize, Failure> {
        let name = var.dimension_names.get(i).cloned().unwrap_or_default();
        let coords = self.dim_coordinates(var, &name)?;
        let meta = self.selected.metadata.as_ref().ok_or(Failure::NoDataset)?;
        let dim = Dim {
            meta,
            var,
            store: &self.selected.store_target,
            name: &name,
            len: var.shape.get(i).copied().unwrap_or(0) as usize,
        };
        index_of(&dim, coords.as_ref(), value, last).ok_or(Failure::NoValue {
            dim: name.clone(),
            value: value.to_string(),
        })
    }

    /// Dimension `name`'s coordinates; while they are read, the command
    /// waits.
    fn dim_coordinates(
        &mut self,
        var: &VariableInfo,
        name: &str,
    ) -> Result<Option<CoordValues>, Failure> {
        let meta = self.selected.metadata.as_ref().ok_or(Failure::NoDataset)?;
        if let Some(coords) = meta.get_dim_coords(Some(&var.name), name) {
            return Ok(Some(coords.clone()));
        }
        self.request_variable_coordinates(self.selected.variable_idx);
        if self.coordinate_loader.pending_count() > 0 {
            return Err(Failure::Reading);
        }
        Ok(None)
    }

    /// The coordinate at `idx` of dimension `i`, as the hover writes it.
    fn coord_text(&self, var: &VariableInfo, i: usize, idx: usize) -> String {
        let Some(meta) = self.selected.metadata.as_ref() else {
            return idx.to_string();
        };
        let name = var.dimension_names.get(i).map_or("", String::as_str);
        let len = var.shape.get(i).copied().unwrap_or(0) as usize;
        let store = &self.selected.store_target;
        Dim {
            meta,
            var,
            store,
            name,
            len,
        }
        .text(idx)
    }

    /// The plot's step (`request_step_or_load`, as the playback bar).
    fn go_to_step(&mut self, step: usize) {
        if self.plotted().metadata.is_some() {
            self.request_step_or_load(step);
        } else {
            self.playback.current_timestep = step;
        }
    }

    pub(super) fn set_dim(&mut self, dim: Option<&str>, value: &str) -> Outcome {
        let var = self.staged_variable()?;
        let i = self.dim_of(&var, dim)?;
        let index = self.index_for(&var, i, value, false)?;
        let name = var.dimension_names.get(i).cloned().unwrap_or_default();
        if self.selected.dim_config[i].animation == AnimationRole::Animated {
            self.go_to_step(index);
        } else {
            // A range shown (a plot axis) narrows to that one value.
            self.selected.set_index(i, index);
        }
        let value = self.coord_text(&var, i, index);
        Ok(Done::DimSet { dim: name, value })
    }

    pub(super) fn set_dim_range(&mut self, dim: Option<&str>, from: &str, to: &str) -> Outcome {
        let var = self.staged_variable()?;
        let i = self.dim_of(&var, dim)?;
        let (a, b) = (
            self.index_for(&var, i, from, false)?,
            self.index_for(&var, i, to, true)?,
        );
        let range = (a.min(b), a.max(b));
        self.selected.dim_config[i].active = true;
        if self.selected.dim_config[i].animation == AnimationRole::Animated {
            let step = self.playback.current_timestep.clamp(range.0, range.1);
            self.selected.set_range(i, range, step);
            self.go_to_step(step);
        } else {
            self.selected.set_range(i, range, range.0);
        }
        Ok(Done::RangeSet {
            dim: var.dimension_names.get(i).cloned().unwrap_or_default(),
            from: self.coord_text(&var, i, range.0),
            to: self.coord_text(&var, i, range.1),
        })
    }

    pub(super) fn set_dim_role(&mut self, dim: &str, role: DimRole) -> Outcome {
        let var = self.staged_variable()?;
        let i = self.dim_of(&var, Some(dim))?;
        let config = self.selected.dim_config[i];
        let (spatial, anim) = match role {
            DimRole::X => (SpatialRole::X, config.animation),
            DimRole::Y => (SpatialRole::Y, config.animation),
            DimRole::Z => (SpatialRole::Z, config.animation),
            DimRole::Animate => (config.spatial, AnimationRole::Animated),
            DimRole::None => (SpatialRole::None, AnimationRole::None),
        };
        crate::ui::variables_panel::dimension_slider::apply_role_change(i, spatial, anim, self);
        let dim = var.dimension_names.get(i).cloned().unwrap_or_default();
        Ok(Done::RoleSet { dim, role })
    }
}
