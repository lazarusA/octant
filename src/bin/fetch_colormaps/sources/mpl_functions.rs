//! Matplotlib's colormap generator functions (`_cm.py`): the `gfunc` table,
//! flag/prism and gist_heat/gist_yarg, by the names `_cm.py` uses.

use std::f64::consts::PI;

/// Generator functions referenced by name in `_cm.py`.
pub fn function(name: &str) -> Option<fn(f64) -> f64> {
    if let Some(n) = name
        .strip_prefix("gfunc[")
        .and_then(|s| s.strip_suffix(']'))
    {
        return n.parse().ok().and_then(gfunc);
    }
    Some(match name {
        "_flag_red" => |x| 0.75 * ((x * 31.5 + 0.25) * PI).sin() + 0.5,
        "_flag_green" => |x| (x * 31.5 * PI).sin(),
        "_flag_blue" => |x| 0.75 * ((x * 31.5 - 0.25) * PI).sin() + 0.5,
        "_prism_red" => |x| 0.75 * ((x * 20.9 + 0.25) * PI).sin() + 0.67,
        "_prism_green" => |x| 0.75 * ((x * 20.9 - 0.25) * PI).sin() + 0.33,
        "_prism_blue" => |x| -1.1 * ((x * 20.9) * PI).sin(),
        "_gist_heat_red" => |x| 1.5 * x,
        "_gist_heat_green" => |x| 2.0 * x - 1.0,
        "_gist_heat_blue" => |x| 4.0 * x - 3.0,
        "_gist_yarg" => |x| 1.0 - x,
        _ => return None,
    })
}

/// Matplotlib's `gfunc` table (`_g0` … `_g36`).
fn gfunc(n: u8) -> Option<fn(f64) -> f64> {
    Some(match n {
        0 => |_| 0.0,
        1 => |_| 0.5,
        2 => |_| 1.0,
        3 => |x| x,
        4 => |x| x.powi(2),
        5 => |x| x.powi(3),
        6 => |x| x.powi(4),
        7 => |x| x.sqrt(),
        8 => |x| x.sqrt().sqrt(),
        9 => |x| (x * PI / 2.0).sin(),
        10 => |x| (x * PI / 2.0).cos(),
        11 => |x| (x - 0.5).abs(),
        12 => |x| (2.0 * x - 1.0).powi(2),
        13 => |x| (x * PI).sin(),
        14 => |x| (x * PI).cos().abs(),
        15 => |x| (x * 2.0 * PI).sin(),
        16 => |x| (x * 2.0 * PI).cos(),
        17 => |x| (x * 2.0 * PI).sin().abs(),
        18 => |x| (x * 2.0 * PI).cos().abs(),
        19 => |x| (x * 4.0 * PI).sin().abs(),
        20 => |x| (x * 4.0 * PI).cos().abs(),
        21 => |x| 3.0 * x,
        22 => |x| 3.0 * x - 1.0,
        23 => |x| 3.0 * x - 2.0,
        24 => |x| (3.0 * x - 1.0).abs(),
        25 => |x| (3.0 * x - 2.0).abs(),
        26 => |x| (3.0 * x - 1.0) / 2.0,
        27 => |x| (3.0 * x - 2.0) / 2.0,
        28 => |x| ((3.0 * x - 1.0) / 2.0).abs(),
        29 => |x| ((3.0 * x - 2.0) / 2.0).abs(),
        30 => |x| x / 0.32 - 0.78125,
        31 => |x| 2.0 * x - 0.84,
        32 => |x| match x {
            x if x < 0.25 => 4.0 * x,
            x if x < 0.92 => -2.0 * x + 1.84,
            x => x / 0.08 - 11.5,
        },
        33 => |x| (2.0 * x - 0.5).abs(),
        34 => |x| 2.0 * x,
        35 => |x| 2.0 * x - 0.5,
        36 => |x| 2.0 * x - 1.0,
        _ => return None,
    })
}
