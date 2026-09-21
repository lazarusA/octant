//! Dimension configuration and role metadata.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialRole {
    None,
    X,
    Y,
    Z,
    Grid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationRole {
    None,
    Animated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimConfig {
    pub spatial: SpatialRole,
    pub animation: AnimationRole,
    pub active: bool, // true = range (expanded), false = single index (collapsed)
    pub index: usize, // selected single step / index
    pub range: (usize, usize), // selected (start, end) range
}

impl Default for DimConfig {
    fn default() -> Self {
        Self {
            spatial: SpatialRole::None,
            animation: AnimationRole::None,
            active: false,
            index: 0,
            range: (0, 0),
        }
    }
}

impl DimConfig {
    pub fn new(
        spatial: SpatialRole,
        animation: AnimationRole,
        active: bool,
        index: usize,
        range: (usize, usize),
    ) -> Self {
        Self {
            spatial,
            animation,
            active,
            index,
            range,
        }
    }

    pub fn grid_dim(configs: &[DimConfig]) -> Option<usize> {
        configs.iter().position(|c| c.spatial == SpatialRole::Grid)
    }

    pub fn x_dim(configs: &[DimConfig]) -> Option<usize> {
        configs
            .iter()
            .position(|c| c.spatial == SpatialRole::X || c.spatial == SpatialRole::Grid)
    }

    pub fn y_dim(configs: &[DimConfig]) -> Option<usize> {
        configs.iter().position(|c| c.spatial == SpatialRole::Y)
    }

    pub fn z_dim(configs: &[DimConfig]) -> Option<usize> {
        configs.iter().position(|c| c.spatial == SpatialRole::Z)
    }

    pub fn animated_dim(configs: &[DimConfig]) -> Option<usize> {
        configs
            .iter()
            .position(|c| c.animation == AnimationRole::Animated)
    }

    pub fn spatial_dims(configs: &[DimConfig]) -> Vec<usize> {
        let mut list = Vec::new();
        if let Some(i) = Self::x_dim(configs) {
            list.push(i);
        }
        if let Some(i) = Self::y_dim(configs) {
            list.push(i);
        }
        if let Some(i) = Self::z_dim(configs) {
            list.push(i);
        }
        list
    }
}
