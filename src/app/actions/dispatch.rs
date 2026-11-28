//! `OctantApp::dispatch`: an action run against the app as it is now,
//! through the UI's own methods.

use super::{AppAction, Done, Outcome};
use crate::app::OctantApp;

impl OctantApp {
    /// Runs `action` against the app as it is now.
    pub fn dispatch(&mut self, action: AppAction) -> Outcome {
        match action {
            AppAction::OpenSource { source } => self.open_by_name(&source),
            AppAction::OpenGallery => {
                self.open_sample(crate::app::sources::GALLERY_SAMPLE);
                Ok(Done::GalleryOpened)
            }
            AppAction::DescribeDataset { group } => self.describe_dataset(group.as_deref()),
            AppAction::Search { text } => self.search_variables(&text),
            AppAction::DescribeVariable { path } => self.describe_variable(path.as_deref()),
            AppAction::DescribeView => self.describe_view(),
            AppAction::SelectVariable { path } => self.select_by_name(&path),
            AppAction::SetLevel { level, auto } => self.set_level(level, auto),
            AppAction::Plot { plot_type } => self.plot_as(plot_type.as_deref()),
            AppAction::SetDim { dim, value } => self.set_dim(dim.as_deref(), &value),
            AppAction::SetRange { dim, from, to } => self.set_dim_range(dim.as_deref(), &from, &to),
            AppAction::SetRole { dim, role } => self.set_dim_role(&dim, role),
            AppAction::Play => self.play(true),
            AppAction::Pause => self.play(false),
            AppAction::Step { by } => self.step_by(by),
            AppAction::AddOverlay => self.add_staged_overlay(),
            AppAction::SetLayerVisible { layer, visible } => self.show_layer(&layer, visible),
            AppAction::RemoveOverlay { layer } => self.remove_layer(&layer),
            AppAction::SetColorRange { min, max, layer } => {
                self.set_color_range(layer.as_deref(), min, max)
            }
            AppAction::Export { format } => self.open_export(format.as_deref()),
            AppAction::OpenPlateWell { well, field } => {
                self.open_plate_well(well.as_deref(), field)
            }
            AppAction::SetColormap { name, layer } => self.set_colormap(&name, layer.as_deref()),
            AppAction::SetComposite { composite } => self.set_composite(&composite),
            AppAction::SetColorStyle { style } => self.set_color_style(&style),
            AppAction::SetView {
                x_from,
                x_to,
                y_from,
                y_to,
            } => self.set_view((&x_from, &x_to), (&y_from, &y_to)),
            AppAction::SetCamera {
                pitch,
                yaw,
                distance,
                target,
            } => self.set_camera(pitch, yaw, distance, target),
            AppAction::ResetView => {
                let plot_type = self.effective_canvas_plot_type();
                crate::app::controllers::controller_for(plot_type).reset_view(&mut self.nav);
                Ok(Done::ViewReset)
            }
        }
    }
}
