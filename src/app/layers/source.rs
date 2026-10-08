//! What a layer is drawn from.

use super::VariableSelection;

/// The data behind a layer. Every source carries the selection of the
/// dimensions it is shown on.
#[derive(Debug, Clone)]
pub enum Source {
    /// A dataset variable read from its store.
    Variable(VariableSelection),
}

impl Default for Source {
    fn default() -> Self {
        Self::Variable(VariableSelection::default())
    }
}

impl Source {
    pub fn selection(&self) -> &VariableSelection {
        match self {
            Self::Variable(selection) => selection,
        }
    }

    pub fn selection_mut(&mut self) -> &mut VariableSelection {
        match self {
            Self::Variable(selection) => selection,
        }
    }
}
