//! HEALPix (Hierarchical Equal Area isoLatitude Pixelation) discrete global grid math.
//!
//! Implements O(1) constant-time coordinate mappings (pix2ang, ang2pix), ring/nested conversions,
//! and cell polygon boundaries matching SpeedyWeather, cuHPX, and Healpy standards.

use std::f32::consts::{FRAC_PI_2, PI};

/// Pixel ordering scheme for HEALPix grids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HealpixOrder {
    /// Standard ring ordering (sorted from north pole to south pole along iso-latitude rings).
    #[default]
    Ring,
    /// Hierarchical nested ordering (quadtree on each of the 12 base diamond faces).
    Nested,
}

impl HealpixOrder {
    pub fn from_str_case_insensitive(s: &str) -> Self {
        if s.eq_ignore_ascii_case("nested") {
            Self::Nested
        } else {
            Self::Ring
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ring => "ring",
            Self::Nested => "nested",
        }
    }
}

/// Returns the total number of pixels `npix = 12 * nside^2` for a given `nside`.
#[inline]
pub fn nside_to_npix(nside: usize) -> usize {
    12 * nside * nside
}

/// Inverts `npix` to find `nside` if `npix = 12 * nside^2`, returning `None` if invalid.
pub fn npix_to_nside(npix: usize) -> Option<usize> {
    if npix == 0 || !npix.is_multiple_of(12) {
        return None;
    }
    let nside_sq = npix / 12;
    let nside = (nside_sq as f64).sqrt().round() as usize;
    if nside > 0 && nside * nside == nside_sq {
        Some(nside)
    } else {
        None
    }
}

/// Converts a pixel index in RING scheme to spherical coordinates `(lon_rad, lat_rad)`.
///
/// Longitude is in `[0, 2π)` and latitude is in `[-π/2, π/2]`.
pub fn pix2ang_ring(nside: usize, pix: usize) -> (f32, f32) {
    let nside = nside.max(1);
    let npix = nside_to_npix(nside);
    let p = pix.min(npix.saturating_sub(1));
    let ncap = 2 * nside * (nside.saturating_sub(1));

    if p < ncap {
        // North Polar Cap (rings 1 .. nside - 1)
        let r = (((1.0 + (1.0 + 2.0 * p as f64).sqrt()) * 0.5).floor() as usize).max(1);
        let p_ring_start = 2 * r * (r - 1);
        let i = p.saturating_sub(p_ring_start);

        let z = 1.0 - (r as f32 * r as f32) / (3.0 * nside as f32 * nside as f32);
        let lat = z.clamp(-1.0, 1.0).asin();
        let lon = ((i as f32 + 0.5) * PI / (2.0 * r as f32)).rem_euclid(2.0 * PI);
        (lon, lat)
    } else if p < npix.saturating_sub(ncap) {
        // Equatorial Belt (rings nside .. 3 * nside)
        let p_eq = p - ncap;
        let r_eq = p_eq / (4 * nside);
        let r = nside + r_eq;
        let i = p_eq % (4 * nside);

        let z = 2.0 * (2.0 * nside as f32 - r as f32) / (3.0 * nside as f32);
        let lat = z.clamp(-1.0, 1.0).asin();

        let shift = if (r - nside).is_multiple_of(2) {
            0.5
        } else {
            0.0
        };
        let lon = ((i as f32 + shift) * PI / (2.0 * nside as f32)).rem_euclid(2.0 * PI);
        (lon, lat)
    } else {
        // South Polar Cap (rings 3 * nside + 1 .. 4 * nside - 1)
        let p_south = (npix - 1).saturating_sub(p);
        let r_south = (((1.0 + (1.0 + 2.0 * p_south as f64).sqrt()) * 0.5).floor() as usize).max(1);
        let _r = 4 * nside - r_south;
        let p_ring_start = npix - 2 * r_south * (r_south + 1);
        let i = p.saturating_sub(p_ring_start);

        let z = -(1.0 - (r_south as f32 * r_south as f32) / (3.0 * nside as f32 * nside as f32));
        let lat = z.clamp(-1.0, 1.0).asin();
        let lon = ((i as f32 + 0.5) * PI / (2.0 * r_south as f32)).rem_euclid(2.0 * PI);
        (lon, lat)
    }
}

/// Converts spherical coordinates `(lon_rad, lat_rad)` to the corresponding pixel index in RING scheme.
pub fn ang2pix_ring(nside: usize, lon_rad: f32, lat_rad: f32) -> usize {
    let nside = nside.max(1);
    let nside_f = nside as f64;
    let npix = nside_to_npix(nside);

    let z = lat_rad.sin().clamp(-1.0, 1.0) as f64;
    let za = z.abs();
    let lon = (lon_rad.rem_euclid(2.0 * PI)) as f64;
    let tt = lon / (0.5 * std::f64::consts::PI); // in [0, 4)

    if za <= 2.0 / 3.0 {
        // Equatorial Belt
        let temp1 = nside_f * (0.5 + tt);
        let temp2 = nside_f * (0.75 * z);
        let jp = (temp1 - temp2).floor() as i64;
        let jm = (temp1 + temp2).floor() as i64;
        let ir = nside as i64 + 1 + jp - jm; // ring number in [1, 4*nside-1]
        let kshift = 1 - (ir & 1);
        let mut ip =
            ((jp + jm - nside as i64 + kshift + 1) / 2).rem_euclid(4 * nside as i64) as usize;
        if ip >= 4 * nside {
            ip = 4 * nside - 1;
        }

        let ncap = 2 * nside * (nside.saturating_sub(1));
        let ring_idx = (ir - 1).max(0) as usize;
        ncap + ring_idx * (4 * nside) + ip
    } else {
        // Polar Caps
        let tp = tt - tt.floor();
        let tmp = nside_f * (3.0 * (1.0 - za)).max(0.0).sqrt();
        let jp = (tp * tmp).floor() as usize;
        let jm = ((1.0 - tp) * tmp).floor() as usize;
        let ir = jp + jm + 1;
        let mut ip = (tt * ir as f64).floor() as usize;
        if ip >= 4 * ir {
            ip -= 4 * ir;
        }

        if z > 0.0 {
            2 * ir * (ir - 1) + ip
        } else {
            npix - 2 * ir * (ir + 1) + ip
        }
    }
}

/// Returns the 1-based latitude ring index `(1 ..= 4 * nside - 1)` and 0-based pixel index in ring.
pub fn pix2ring(nside: usize, pix: usize) -> (usize, usize) {
    let nside = nside.max(1);
    let npix = nside_to_npix(nside);
    let p = pix.min(npix.saturating_sub(1));
    let ncap = 2 * nside * (nside.saturating_sub(1));

    if p < ncap {
        let r = (((1.0 + (1.0 + 2.0 * p as f64).sqrt()) * 0.5).floor() as usize).max(1);
        let p_ring_start = 2 * r * (r - 1);
        (r, p.saturating_sub(p_ring_start))
    } else if p < npix.saturating_sub(ncap) {
        let p_eq = p - ncap;
        let r_eq = p_eq / (4 * nside);
        let r = nside + r_eq;
        (r, p_eq % (4 * nside))
    } else {
        let p_south = (npix - 1).saturating_sub(p);
        let r_south = (((1.0 + (1.0 + 2.0 * p_south as f64).sqrt()) * 0.5).floor() as usize).max(1);
        let r = 4 * nside - r_south;
        let p_ring_start = npix - 2 * r_south * (r_south + 1);
        (r, p.saturating_sub(p_ring_start))
    }
}

/// Converts a pixel index from NESTED scheme to spherical coordinates `(lon_rad, lat_rad)`.
pub fn pix2ang_nest(nside: usize, pix_nest: usize) -> (f32, f32) {
    let p_ring = nest2ring(nside, pix_nest);
    pix2ang_ring(nside, p_ring)
}

/// Converts spherical coordinates `(lon_rad, lat_rad)` to the corresponding pixel index in NESTED scheme.
pub fn ang2pix_nest(nside: usize, lon_rad: f32, lat_rad: f32) -> usize {
    let p_ring = ang2pix_ring(nside, lon_rad, lat_rad);
    ring2nest(nside, p_ring)
}

const JRLL: [isize; 12] = [2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
const JPLL: [isize; 12] = [1, 3, 5, 7, 0, 2, 4, 6, 1, 3, 5, 7];

/// Converts a pixel index from NESTED scheme to RING scheme.
pub fn nest2ring(nside: usize, pix_nest: usize) -> usize {
    let npix = nside_to_npix(nside);
    if nside <= 1 {
        return pix_nest.min(npix.saturating_sub(1));
    }
    let p_nest = pix_nest.min(npix.saturating_sub(1));
    let nside_sq = nside * nside;
    let face = (p_nest / nside_sq).min(11);
    let in_face = p_nest % nside_sq;

    let mut ix = 0usize;
    let mut iy = 0usize;
    for b in 0..16 {
        ix |= ((in_face >> (2 * b)) & 1) << b;
        iy |= ((in_face >> (2 * b + 1)) & 1) << b;
    }

    let nside_i = nside as isize;
    let nl4 = 4 * nside_i;
    let ncap = 2 * nside * (nside - 1);

    let jr = JRLL[face] * nside_i - ix as isize - iy as isize - 1;

    if jr < nside_i {
        let nr = jr;
        let mut ip = (JPLL[face] * nr + ix as isize - iy as isize + 1) / 2;
        if ip > 4 * nr {
            ip -= 4 * nr;
        }
        if ip < 1 {
            ip += 4 * nr;
        }
        let p_start = 2 * (nr as usize) * ((nr - 1) as usize);
        p_start + (ip as usize - 1)
    } else if jr <= 3 * nside_i {
        let kshift = (jr - nside_i) & 1;
        let mut ip = (JPLL[face] * nside_i + ix as isize - iy as isize + 1 + kshift) / 2;
        if ip > nl4 {
            ip -= nl4;
        }
        if ip < 1 {
            ip += nl4;
        }
        let p_start = ncap + ((jr - nside_i) as usize) * (4 * nside);
        p_start + (ip as usize - 1)
    } else {
        let nr = 4 * nside_i - jr;
        let mut ip = (JPLL[face] * nr + ix as isize - iy as isize + 1) / 2;
        if ip > 4 * nr {
            ip -= 4 * nr;
        }
        if ip < 1 {
            ip += 4 * nr;
        }
        let p_start = npix - 2 * (nr as usize) * ((nr + 1) as usize);
        p_start + (ip as usize - 1)
    }
}

/// Converts a pixel index from RING scheme to NESTED scheme.
pub fn ring2nest(nside: usize, pix_ring: usize) -> usize {
    let npix = nside_to_npix(nside);
    if nside <= 1 {
        return pix_ring.min(npix.saturating_sub(1));
    }
    let p_ring = pix_ring.min(npix.saturating_sub(1));
    let nside_i = nside as isize;
    let nl4 = 4 * nside_i;
    let ncap = 2 * nside * (nside - 1);

    let (jr, ip) = if p_ring < ncap {
        let r = (((1.0 + (1.0 + 2.0 * p_ring as f64).sqrt()) * 0.5).floor() as isize).max(1);
        let p_start = 2 * (r as usize) * ((r - 1) as usize);
        (r, (p_ring - p_start + 1) as isize)
    } else if p_ring < npix - ncap {
        let p_eq = p_ring - ncap;
        let r_eq = (p_eq / (4 * nside)) as isize;
        let r = nside_i + r_eq;
        (r, (p_eq % (4 * nside) + 1) as isize)
    } else {
        let p_south = (npix - 1).saturating_sub(p_ring);
        let r_south = (((1.0 + (1.0 + 2.0 * p_south as f64).sqrt()) * 0.5).floor() as isize).max(1);
        let r = 4 * nside_i - r_south;
        let p_start = npix - 2 * (r_south as usize) * ((r_south + 1) as usize);
        (r, (p_ring - p_start + 1) as isize)
    };

    for face in 0..12 {
        let (ix_isize, iy_isize) = if jr < nside_i {
            if face >= 4 {
                continue;
            }
            let nr = jr;
            let jpll = JPLL[face];
            let diff = 2 * ip - jpll * nr - 1;
            let sum = 2 * nside_i - 1 - nr;
            ((sum + diff) / 2, (sum - diff) / 2)
        } else if jr <= 3 * nside_i {
            let kshift = (jr - nside_i) & 1;
            let jpll = JPLL[face];
            let jrll = JRLL[face];
            let mut found = None;
            for &wrap in &[0, nl4, -nl4] {
                let ip_adj = ip + wrap;
                let diff = 2 * ip_adj - jpll * nside_i - 1 - kshift;
                let sum = jrll * nside_i - 1 - jr;
                let ix = (sum + diff) / 2;
                let iy = (sum - diff) / 2;
                if ix >= 0 && ix < nside_i && iy >= 0 && iy < nside_i {
                    found = Some((ix, iy));
                    break;
                }
            }
            if let Some(coords) = found {
                coords
            } else {
                continue;
            }
        } else {
            if face < 8 {
                continue;
            }
            let nr = 4 * nside_i - jr;
            let jpll = JPLL[face];
            let diff = 2 * ip - jpll * nr - 1;
            let sum = nr - 1;
            ((sum + diff) / 2, (sum - diff) / 2)
        };

        if ix_isize >= 0 && ix_isize < nside_i && iy_isize >= 0 && iy_isize < nside_i {
            let ix = ix_isize as usize;
            let iy = iy_isize as usize;
            let mut in_face = 0usize;
            for b in 0..16 {
                in_face |= ((ix >> b) & 1) << (2 * b);
                in_face |= ((iy >> b) & 1) << (2 * b + 1);
            }
            return face * (nside * nside) + in_face;
        }
    }

    0
}

/// Computes the 4 spherical corner vertices `[(lon, lat); 4]` of a HEALPix cell.
pub fn pix_boundaries(nside: usize, pix: usize) -> [(f32, f32); 4] {
    let (center_lon, center_lat) = pix2ang_ring(nside, pix);
    let d_lat = (PI / (4.0 * nside as f32)).min(FRAC_PI_2 * 0.5);
    let cos_lat = center_lat.cos().abs().max(0.1);
    let d_lon = (PI / (2.0 * nside as f32 * cos_lat)).min(PI);

    [
        (
            center_lon,
            (center_lat + d_lat).clamp(-FRAC_PI_2, FRAC_PI_2),
        ), // North
        ((center_lon + d_lon).rem_euclid(2.0 * PI), center_lat), // East
        (
            center_lon,
            (center_lat - d_lat).clamp(-FRAC_PI_2, FRAC_PI_2),
        ), // South
        ((center_lon - d_lon).rem_euclid(2.0 * PI), center_lat), // West
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speedy_weather_nside16_ring1() {
        let nside = 16;
        let npix = nside_to_npix(nside);
        assert_eq!(npix, 3072);

        let expected_lat_deg = 87.07582;
        let expected_lons = [45.0f32, 135.0, 225.0, 315.0];

        for (i, &expected_lon) in expected_lons.iter().enumerate() {
            let (lon_rad, lat_rad) = pix2ang_ring(nside, i);
            let lon_deg = lon_rad.to_degrees();
            let lat_deg = lat_rad.to_degrees();
            let (ring, in_ring) = pix2ring(nside, i);

            assert_eq!(ring, 1);
            assert_eq!(in_ring, i);
            assert!((lat_deg - expected_lat_deg).abs() < 0.001);
            assert!((lon_deg - expected_lon).abs() < 0.001);

            let round_trip_p = ang2pix_ring(nside, lon_rad, lat_rad);
            assert_eq!(round_trip_p, i);
        }
    }

    #[test]
    fn test_npix_to_nside_validation() {
        assert_eq!(npix_to_nside(12), Some(1));
        assert_eq!(npix_to_nside(48), Some(2));
        assert_eq!(npix_to_nside(192), Some(4));
        assert_eq!(npix_to_nside(768), Some(8));
        assert_eq!(npix_to_nside(3072), Some(16));
        assert_eq!(npix_to_nside(12288), Some(32));
        assert_eq!(npix_to_nside(500), None);
    }

    #[test]
    fn test_round_trip_all_pixels_nside4() {
        let nside = 4;
        let npix = nside_to_npix(nside);
        assert_eq!(npix, 192);

        for p in 0..npix {
            let (lon_rad, lat_rad) = pix2ang_ring(nside, p);
            let recovered_p = ang2pix_ring(nside, lon_rad, lat_rad);
            assert_eq!(
                recovered_p, p,
                "Mismatch at p={p} (lon={lon_rad}, lat={lat_rad})"
            );
        }
    }

    #[test]
    fn test_nested_round_trip_all_pixels() {
        for &nside in &[1, 2, 4, 8, 16] {
            let npix = nside_to_npix(nside);
            for p in 0..npix {
                let p_ring = nest2ring(nside, p);
                let p_nest = ring2nest(nside, p_ring);
                assert_eq!(
                    p_nest, p,
                    "nest2ring / ring2nest mismatch at nside={nside}, p={p}"
                );
            }
        }
    }
}
