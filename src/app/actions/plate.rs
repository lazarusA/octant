//! A plate's wells and fields, as the plate browser picks and opens them:
//! a well by its name (`B3`), a field by the 1-based number its button
//! shows, once the well's fields have been read.

use super::resolve::among;
use super::{Done, Failure, NameKind, Outcome, resolved};
use crate::app::OctantApp;
use crate::app::state::plate::WellFields;
use crate::data::ome::OmePlate;

/// A well's name as the plate writes it: its row and column (`B3`).
fn well_name(plate: &OmePlate, index: usize) -> String {
    let Some(well) = plate.wells.get(index) else {
        return String::new();
    };
    let row = plate.rows.get(well.row).map_or("", String::as_str);
    let column = plate.columns.get(well.column).map_or("", String::as_str);
    format!("{row}{column}")
}

/// The names of every well of `plate`.
pub(super) fn well_names(plate: &OmePlate) -> Vec<String> {
    (0..plate.wells.len())
        .map(|i| well_name(plate, i))
        .collect()
}

impl OctantApp {
    /// Picks well `well` (the picked one without), and opens its field
    /// number `field` when given.
    pub(super) fn open_plate_well(&mut self, well: Option<&str>, field: Option<usize>) -> Outcome {
        let no_well = || Failure::NotFound {
            kind: NameKind::Well,
            text: well.unwrap_or_default().to_string(),
            closest: Vec::new(),
        };
        let plate = self.plate.as_mut().ok_or_else(no_well)?;
        let index = match well {
            Some(name) => {
                let names = well_names(&plate.plate);
                let names: Vec<&str> = names.iter().map(String::as_str).collect();
                resolved(among(&names, name), NameKind::Well, name)?
            }
            None => plate.well.ok_or_else(no_well)?,
        };
        plate.well = Some(index);
        let name = well_name(&plate.plate, index);
        let Some(number) = field else {
            return Ok(Done::WellPicked(name));
        };
        let path = plate.plate.wells[index].path.clone();
        let fields = match plate.well_fields(&path) {
            WellFields::Ready(fields) => std::sync::Arc::clone(fields),
            WellFields::Loading => return Err(Failure::Reading),
            WellFields::Failed => return Err(Failure::Refused),
        };
        let acquisition = plate.acquisition;
        let found = number
            .checked_sub(1)
            .and_then(|n| fields.fields_of(acquisition).nth(n));
        let Some((_, field_of)) = found else {
            let value = number.to_string();
            return Err(Failure::NoValue { dim: name, value });
        };
        self.open_plate_field(index, &field_of.path);
        self.commands.await_dataset(&self.selected.store_target);
        Ok(Done::Opening(format!("{name} {number}")))
    }
}
