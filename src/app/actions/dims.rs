//! Dimension actions on the staged selection, as the Dimensions panel sets
//! them: an index or a range by coordinate value, a role. The staged
//! animated dimension is the plot's step, moved as the playback bar moves it.

use super::axes::{Axis, axis_of};
use super::coord_match::{Dim, index_of};
use super::resolve::{Resolution, among};
use super::{DimRole, Done, Failure, NameKind, Outcome, resolved};
use crate::app::{AnimationRole, OctantApp, SpatialRole};
use crate::data::VariableInfo;

impl OctantApp {
    /// Puts the staged variable's dimension settings in place.
    fn init_staged_dims(&mut self) -> Result<(), Failure> {
        let var = self.selected_variable_info().ok_or(Failure::NoDataset)?;
        if self.selected.dim_config.len() != var.shape.len() {
            let var = var.clone();
            crate::ui::variables_panel::init_variable_dimension_defaults(self, &var);
        }
        Ok(())
    }

    /// The staged variable's dimension `dim` names (its time dimension
    /// without one), and its name.
    fn staged_dim(&mut self, dim: Option<&str>) -> Result<(usize, String), Failure> {
        self.init_staged_dims()?;
        let var = self.selected_variable_info().ok_or(Failure::NoDataset)?;
        let i = self.dim_of(var, dim)?;
        let name = var.dimension_names.get(i).cloned().unwrap_or_default();
        Ok((i, name))
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

    /// The staged dimension `i` as the hover resolves it.
    fn staged_dim_info(&self, i: usize) -> Result<Dim<'_>, Failure> {
        let meta = self.selected.metadata.as_ref().ok_or(Failure::NoDataset)?;
        let var = self.selected.variable_info().ok_or(Failure::NoDataset)?;
        Ok(Dim {
            meta,
            var,
            store: &self.selected.store_target,
            name: var.dimension_names.get(i).map_or("", String::as_str),
            len: var.shape.get(i).copied().unwrap_or(0) as usize,
        })
    }

    /// The index `value` names along staged dimension `i`, once its
    /// coordinates are read (`await_coordinates`).
    fn index_for(&self, i: usize, value: &str, last: bool) -> Result<usize, Failure> {
        let dim = self.staged_dim_info(i)?;
        let coords = dim.meta.get_dim_coords(Some(&dim.var.name), dim.name);
        index_of(&dim, coords, value, last).ok_or_else(|| Failure::NoValue {
            dim: dim.name.to_string(),
            value: value.to_string(),
        })
    }

    /// Waits for the staged dimension `i`'s coordinates: asks for them and
    /// fails with `Reading` while they are read. A command asked again once
    /// they settled (`CommandQueue::retrying`) never asks twice: a read that
    /// failed leaves the dimension's positions.
    fn await_coordinates(&mut self, i: usize) -> Result<(), Failure> {
        let dim = self.staged_dim_info(i)?;
        if dim
            .meta
            .get_dim_coords(Some(&dim.var.name), dim.name)
            .is_some()
            || self.commands.retrying()
        {
            return Ok(());
        }
        self.request_variable_coordinates(self.selected.variable_idx);
        if self.coordinate_loader.pending_count() > 0 {
            return Err(Failure::Reading);
        }
        Ok(())
    }

    /// The coordinate at `idx` of staged dimension `i`, as the hover writes it.
    fn coord_text(&self, i: usize, idx: usize) -> String {
        self.staged_dim_info(i)
            .map_or_else(|_| idx.to_string(), |dim| dim.text(idx))
    }

    /// The step of the staged animated dimension: the plot's when it shows
    /// the staged variable (`request_step_or_load`, as the playback bar),
    /// else only the step the Dimensions panel's slider shows, as dragging
    /// it does.
    fn go_to_step(&mut self, step: usize) {
        if self.plotted().metadata.is_some() && !self.is_exploring_unplotted_variable() {
            self.request_step_or_load(step);
        } else {
            self.playback.current_timestep = step;
        }
    }

    pub(super) fn set_dim(&mut self, dim: Option<&str>, value: &str) -> Outcome {
        let (i, name) = self.staged_dim(dim)?;
        self.await_coordinates(i)?;
        let index = self.index_for(i, value, false)?;
        if self.selected.dim_config[i].animation == AnimationRole::Animated {
            self.go_to_step(index);
        } else {
            // A range shown (a plot axis) narrows to that one value.
            self.selected.set_index(i, index);
        }
        let value = self.coord_text(i, index);
        Ok(Done::DimSet { dim: name, value })
    }

    pub(super) fn set_dim_range(&mut self, dim: Option<&str>, from: &str, to: &str) -> Outcome {
        let (i, name) = self.staged_dim(dim)?;
        self.await_coordinates(i)?;
        let (a, b) = (
            self.index_for(i, from, false)?,
            self.index_for(i, to, true)?,
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
            dim: name,
            from: self.coord_text(i, range.0),
            to: self.coord_text(i, range.1),
        })
    }

    pub(super) fn set_dim_role(&mut self, dim: &str, role: DimRole) -> Outcome {
        let (i, dim) = self.staged_dim(Some(dim))?;
        let config = self.selected.dim_config[i];
        let (spatial, anim) = match role {
            DimRole::X => (SpatialRole::X, config.animation),
            DimRole::Y => (SpatialRole::Y, config.animation),
            DimRole::Z => (SpatialRole::Z, config.animation),
            DimRole::Animate => (config.spatial, AnimationRole::Animated),
            DimRole::None => (SpatialRole::None, AnimationRole::None),
        };
        crate::ui::variables_panel::dimension_slider::apply_role_change(i, spatial, anim, self);
        Ok(Done::RoleSet { dim, role })
    }
}
