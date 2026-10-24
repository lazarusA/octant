//! The actions on the staged selection and the plot: selecting a variable
//! by name, plotting it (as a plot type), and what playback needs.

use super::{Done, Failure, NameKind, Outcome, resolved};
use crate::app::OctantApp;
use crate::app::layers::LevelChoice;

impl OctantApp {
    pub(super) fn select_by_name(&mut self, path: &str) -> Outcome {
        if self.selected.metadata.is_none() {
            return Err(Failure::NoDataset);
        }
        let idx = resolved(self.vocabulary().variable(path), NameKind::Variable, path)?;
        // Selecting the staged variable again would reset its dimensions.
        if idx != self.selected.variable_idx {
            crate::ui::variables_overlay::apply_selection(self, idx, LevelChoice::Auto);
        }
        Ok(Done::Selected(self.staged_name()))
    }

    pub(super) fn plot_as(&mut self, plot_type: Option<&str>) -> Outcome {
        if self.selected_variable_info().is_none() {
            return Err(Failure::NoDataset);
        }
        if let Some(text) = plot_type {
            let plot_type = resolved(self.vocabulary().plot_type(text), NameKind::PlotType, text)?;
            let options = crate::ui::plot_type::options::plot_options(self);
            if let Some(blocked) = options
                .iter()
                .find(|o| o.plot_type == plot_type)
                .and_then(|o| o.blocked)
            {
                let mut buf = [0; 256];
                let reason =
                    crate::ui::plot_type::options::blocked_text(&mut buf, blocked).to_string();
                return Err(Failure::Blocked { plot_type, reason });
            }
            self.switch_plot_type(plot_type);
        }
        self.plot_from_panel();
        Ok(Done::Plotted {
            variable: self.staged_name(),
            plot_type: self.selected.plot_type,
        })
    }

    fn staged_name(&self) -> String {
        self.selected_variable_info()
            .map(|v| v.display_name().to_string())
            .unwrap_or_default()
    }

    /// Whether the plot has an animated dimension to play or step along.
    pub(super) fn base_animates(&self) -> Result<(), Failure> {
        match self.layers.base.selection().animated_dim {
            Some(_) if self.plotted().metadata.is_some() => Ok(()),
            _ => Err(Failure::NothingToAnimate),
        }
    }
}
