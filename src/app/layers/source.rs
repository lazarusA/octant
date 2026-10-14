//! What a layer is drawn from.

use std::sync::Arc;

use super::VariableSelection;
use crate::data::ome::OmeLabelInfo;

/// The data behind a layer. Every source carries the selection of the
/// dimensions it is shown on.
#[derive(Debug, Clone)]
pub enum Source {
    /// A dataset variable read from its store.
    Variable(VariableSelection),
    /// An OME-Zarr label image (segmentation): one color per label id.
    Labels(VariableSelection, LabelStyle),
}

/// How a label image is drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelStyle {
    /// The image's label colors and properties.
    pub info: Arc<OmeLabelInfo>,
    /// Only each label's boundary, else the whole label.
    pub outline: bool,
}

impl LabelStyle {
    pub fn new(info: Arc<OmeLabelInfo>) -> Self {
        Self {
            info,
            outline: false,
        }
    }
}

impl Default for Source {
    fn default() -> Self {
        Self::Variable(VariableSelection::default())
    }
}

impl Source {
    pub fn selection(&self) -> &VariableSelection {
        match self {
            Self::Variable(selection) | Self::Labels(selection, _) => selection,
        }
    }

    pub fn selection_mut(&mut self) -> &mut VariableSelection {
        match self {
            Self::Variable(selection) | Self::Labels(selection, _) => selection,
        }
    }

    /// The label style, for a label image.
    pub fn labels(&self) -> Option<&LabelStyle> {
        match self {
            Self::Labels(_, style) => Some(style),
            Self::Variable(_) => None,
        }
    }

    pub fn labels_mut(&mut self) -> Option<&mut LabelStyle> {
        match self {
            Self::Labels(_, style) => Some(style),
            Self::Variable(_) => None,
        }
    }

    /// The source for `selection`: a label image when its variable is one.
    pub fn for_selection(selection: VariableSelection) -> Self {
        let info = label_info_of(&selection);
        Self::with_labels(selection, info)
    }

    fn with_labels(selection: VariableSelection, info: Option<Arc<OmeLabelInfo>>) -> Self {
        match info {
            Some(info) => Self::Labels(selection, LabelStyle::new(info)),
            None => Self::Variable(selection),
        }
    }

    /// Turns into a label source when the selected variable became a label
    /// image, and back, keeping the selection (and a label style already set).
    /// Whether it changed between drawing labels and values.
    pub fn follow_variable(&mut self) -> bool {
        let was_labels = self.labels().is_some();
        let info = label_info_of(self.selection());
        *self = match (std::mem::take(self), info) {
            (Self::Labels(selection, style), Some(info)) if Arc::ptr_eq(&style.info, &info) => {
                Self::Labels(selection, style)
            }
            (Self::Variable(selection) | Self::Labels(selection, _), info) => {
                Self::with_labels(selection, info)
            }
        };
        was_labels != self.labels().is_some()
    }
}

/// The label info of the selection's variable, for an OME-Zarr label image
/// of integer ids.
fn label_info_of(selection: &VariableSelection) -> Option<Arc<OmeLabelInfo>> {
    let metadata = selection.metadata.as_ref()?;
    metadata
        .variables
        .get(selection.variable_idx)?
        .label_info()
        .cloned()
}
