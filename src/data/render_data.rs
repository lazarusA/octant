//! Unified plotted data container for visualization renderers.

use super::matrix_data::MatrixData;
use super::volume_data::VolumeData;

/// Unified enum encapsulating any plotted data payload resident in memory.
#[derive(Debug, Clone)]
pub enum RenderData {
    Matrix(MatrixData),
    Volume(VolumeData),
}

impl RenderData {
    pub fn as_matrix(&self) -> Option<&MatrixData> {
        match self {
            Self::Matrix(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_volume(&self) -> Option<&VolumeData> {
        match self {
            Self::Volume(v) => Some(v),
            _ => None,
        }
    }

    pub fn min_val(&self) -> f32 {
        match self {
            Self::Matrix(m) => m.min_val,
            Self::Volume(v) => v.min_val,
        }
    }

    pub fn max_val(&self) -> f32 {
        match self {
            Self::Matrix(m) => m.max_val,
            Self::Volume(v) => v.max_val,
        }
    }
}

impl From<MatrixData> for RenderData {
    fn from(data: MatrixData) -> Self {
        Self::Matrix(data)
    }
}

impl From<VolumeData> for RenderData {
    fn from(data: VolumeData) -> Self {
        Self::Volume(data)
    }
}
