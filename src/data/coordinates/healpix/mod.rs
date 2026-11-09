//! HEALPix (Hierarchical Equal Area isoLatitude Pixelation) discrete global grid math.
//!
//! Implements O(1) constant-time coordinate mappings (pix2ang, ang2pix), ring/nested conversions,
//! and cell polygon boundaries matching standard HEALPix formulation (Górski et al., 2005).

pub mod math;
pub mod scheme;

pub use math::{ang2pix_nest, ang2pix_ring, pix_boundaries, pix2ang_nest, pix2ang_ring, pix2ring};
pub use scheme::{morton_deinterleave2, morton_interleave2, nest2ring, ring2nest};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healpix_nside16_ring1() {
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
