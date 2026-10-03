//! Polygon helpers shared by the drawing primitives.

use egui::Pos2;

/// Drop consecutive duplicate points. Pixel snapping can merge neighbors at
/// small sizes, and egui's path tessellator emits NaN for zero-length segments.
pub(super) fn dedup(mut pts: Vec<Pos2>) -> Vec<Pos2> {
    pts.dedup_by(|b, a| near(*a, *b));
    pts
}

/// Points closer than a hundredth of a physical pixel at any common scale.
pub(super) fn near(a: Pos2, b: Pos2) -> bool {
    (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3
}

/// True when every turn of the polygon has the same orientation.
pub(super) fn is_convex(pts: &[Pos2]) -> bool {
    let n = pts.len();
    if n < 4 {
        return true;
    }
    let mut sign = 0.0_f32;
    for i in 0..n {
        let (a, b, c) = (pts[i], pts[(i + 1) % n], pts[(i + 2) % n]);
        let cross = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
        if cross.abs() <= 1e-3 {
            continue;
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if cross.signum() != sign {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn convexity_detects_folder_tab_step() {
        let square = [
            pos2(0.0, 0.0),
            pos2(4.0, 0.0),
            pos2(4.0, 4.0),
            pos2(0.0, 4.0),
        ];
        assert!(is_convex(&square));
        let folder = [
            pos2(3.0, 4.5),
            pos2(9.0, 4.5),
            pos2(11.0, 7.0),
            pos2(21.0, 7.0),
            pos2(21.0, 19.5),
            pos2(3.0, 19.5),
        ];
        assert!(!is_convex(&folder));
    }

    #[test]
    fn dedup_merges_snapped_neighbors() {
        let pts = vec![pos2(1.0, 1.0), pos2(1.0, 1.0 + 1e-5), pos2(2.0, 1.0)];
        assert_eq!(dedup(pts).len(), 2);
    }
}
