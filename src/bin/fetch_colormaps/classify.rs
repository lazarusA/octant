//! Colormap kind classification: name rules per family plus a lightness heuristic.

use octant::utils::colormap::ColormapKind;

/// Crameri Scientific colour maps: `S` categorical palettes, `O` cyclic maps,
/// and the diverging and multi-sequential maps named in the user guide.
pub fn crameri(name: &str) -> ColormapKind {
    const DIVERGING: &[&str] = &[
        "bam", "berlin", "broc", "cork", "lisbon", "managua", "roma", "tofino", "vanimo", "vik",
    ];
    const MULTI: &[&str] = &["bukavu", "fes", "oleron"];
    if name.len() > 2 && name.ends_with('S') {
        ColormapKind::Categorical
    } else if name.len() > 2 && name.ends_with('O') {
        ColormapKind::Cyclic
    } else if DIVERGING.contains(&name) {
        ColormapKind::Diverging
    } else if MULTI.contains(&name) {
        ColormapKind::Other
    } else {
        ColormapKind::Sequential
    }
}

/// colorcet: the kind is the first word of the CET name.
pub fn colorcet(name: &str) -> ColormapKind {
    match name.split('_').next().unwrap_or_default() {
        "linear" => ColormapKind::Sequential,
        "diverging" => ColormapKind::Diverging,
        "cyclic" | "circle" => ColormapKind::Cyclic,
        "glasbey" => ColormapKind::Categorical,
        _ => ColormapKind::Other,
    }
}

pub fn cmocean(name: &str) -> ColormapKind {
    match name {
        "balance" | "curl" | "delta" | "diff" | "tarn" => ColormapKind::Diverging,
        "phase" => ColormapKind::Cyclic,
        "topo" => ColormapKind::Other,
        _ => ColormapKind::Sequential,
    }
}

/// CARTOColors tags: `diverging`, `qualitative`, otherwise sequential.
pub fn carto(tags: &[String]) -> ColormapKind {
    if tags.iter().any(|t| t == "diverging") {
        ColormapKind::Diverging
    } else if tags.iter().any(|t| t == "qualitative") {
        ColormapKind::Categorical
    } else {
        ColormapKind::Sequential
    }
}

/// Palettes: short ones are categorical; long ones are continuous and classified by lightness.
pub fn palette_kind(stops: &[[u8; 3]]) -> ColormapKind {
    if stops.len() < 32 {
        ColormapKind::Categorical
    } else {
        by_lightness(stops)
    }
}

/// Classifies a map from its CIE L* profile: cyclic if both ends match, diverging
/// if lightness peaks or dips in the middle, sequential otherwise.
pub fn by_lightness(stops: &[[u8; 3]]) -> ColormapKind {
    let l: Vec<f64> = stops.iter().map(|&c| lightness(c)).collect();
    let (Some(&first), Some(&last)) = (l.first(), l.last()) else {
        return ColormapKind::Sequential;
    };
    let ends_match = stops
        .first()
        .zip(stops.last())
        .is_some_and(|(a, b)| (0..3).all(|i| a[i].abs_diff(b[i]) < 24));
    if ends_match {
        return ColormapKind::Cyclic;
    }
    let n = l.len();
    let mid = &l[n / 4..(3 * n / 4).max(n / 4 + 1)];
    let peak = mid.iter().copied().fold(f64::MIN, f64::max);
    let dip = mid.iter().copied().fold(f64::MAX, f64::min);
    if peak > first.max(last) + 15.0 || dip < first.min(last) - 15.0 {
        return ColormapKind::Diverging;
    }
    ColormapKind::Sequential
}

/// CIE L* of an sRGB color.
fn lightness(c: [u8; 3]) -> f64 {
    let lin = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let y = 0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2]);
    if y > 216.0 / 24389.0 {
        116.0 * y.cbrt() - 16.0
    } else {
        y * 24389.0 / 27.0
    }
}
