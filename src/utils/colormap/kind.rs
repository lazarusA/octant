//! Perceptual classification of colormaps, used for grouping and filtering.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum ColormapKind {
    #[default]
    Sequential = 0,
    Diverging = 1,
    Cyclic = 2,
    Categorical = 3,
    /// Rainbow, multi-sequential, isoluminant and other special-purpose maps.
    Other = 4,
}

impl ColormapKind {
    pub const ALL: [ColormapKind; 5] = [
        ColormapKind::Sequential,
        ColormapKind::Diverging,
        ColormapKind::Cyclic,
        ColormapKind::Categorical,
        ColormapKind::Other,
    ];

    /// Inverse of `kind as u8`; `None` for bytes no kind encodes to.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Sequential),
            1 => Some(Self::Diverging),
            2 => Some(Self::Cyclic),
            3 => Some(Self::Categorical),
            4 => Some(Self::Other),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Sequential => "Sequential",
            Self::Diverging => "Diverging",
            Self::Cyclic => "Cyclic",
            Self::Categorical => "Categorical",
            Self::Other => "Other",
        }
    }

    /// Categorical palettes are resampled with hard steps instead of blending.
    pub fn is_discrete(self) -> bool {
        self == Self::Categorical
    }
}
