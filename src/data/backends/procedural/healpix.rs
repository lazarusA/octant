//! Synthetic HEALPix Nside=16 block generation.

use std::collections::HashMap;

use crate::data::blocks::BlockStoreError;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;

pub fn slice_healpix_block(request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
    let nside = 16;
    let npix = 3072;
    let angles: Vec<(f32, f32)> = (0..npix)
        .map(|p| crate::data::coordinates::healpix::pix2ang_ring(nside, p))
        .collect();

    let lons_f64: Vec<f64> = angles
        .iter()
        .map(|&(lon, _)| lon.to_degrees() as f64)
        .collect();
    let lats_f64: Vec<f64> = angles
        .iter()
        .map(|&(_, lat)| lat.to_degrees() as f64)
        .collect();

    let mut coords_map = HashMap::new();
    coords_map.insert("lon".to_string(), lons_f64);
    coords_map.insert("lat".to_string(), lats_f64);

    let t_idx = request
        .selections
        .first()
        .map(|s| s.bounds().0)
        .unwrap_or(0);
    let layer_idx = if request.selections.len() >= 3 {
        request.selections.get(1).map(|s| s.bounds().0).unwrap_or(0)
    } else {
        0
    };

    let values: Vec<f32> = match request.variable.as_str() {
        "lat" => angles.iter().map(|&(_, lat)| lat.to_degrees()).collect(),
        "lon" => angles.iter().map(|&(lon, _)| lon.to_degrees()).collect(),
        "ring" => (0..npix)
            .map(|p| crate::data::coordinates::healpix::pix2ring(nside, p).0 as f32)
            .collect(),
        "mslp" => angles
            .iter()
            .map(|&(lon_rad, lat_rad)| {
                let phase = t_idx as f32 * 0.3;
                1013.25 + 25.0 * (4.0 * lon_rad - phase).cos() * lat_rad.cos().powi(2)
                    - 15.0 * lat_rad.sin()
            })
            .collect(),
        _ => angles
            .iter()
            .map(|&(lon_rad, lat_rad)| {
                let phase = t_idx as f32 * 0.3;
                let alt_decay = layer_idx as f32 * 6.5;
                285.0 + 35.0 * lat_rad.cos() - 15.0 * lat_rad.sin().powi(2)
                    + 18.0
                        * (4.0 * lon_rad - phase).cos()
                        * lat_rad.cos().powi(2)
                        * (2.0 * lat_rad).sin()
                    - alt_decay
            })
            .collect(),
    };

    Ok(OctantBlock::new(
        request.variable.clone(),
        vec![npix],
        vec!["cell".to_string()],
        vec![0],
        values,
        coords_map,
        HashMap::new(),
    ))
}
