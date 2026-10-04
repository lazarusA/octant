//! Evaluates Matplotlib colormap definitions exactly like `matplotlib.colors`
//! does for a 256-entry lookup table: segment tables, generator functions,
//! color lists (`LinearSegmentedColormap.from_list`) and listed palettes.

use super::pylit::{Value, assignment};
use std::f64::consts::PI;

pub const N: usize = 256;

/// A Matplotlib map: 256 interpolated samples, or the colors of a listed palette.
pub enum MplMap {
    Sampled(Vec<[f64; 3]>),
    Listed(Vec<[f64; 3]>),
}

/// Resolves `datad[name]` from `_cm.py` and evaluates it.
pub fn datad_map(text: &str, name: &str) -> Result<MplMap, String> {
    let datad = assignment(text, "datad")?;
    let entry = datad
        .get(name)
        .ok_or_else(|| format!("datad has no `{name}`"))?;
    if let Some(Value::Ident(var)) = entry.get("listed") {
        return listed(&assignment(text, var)?).map(MplMap::Listed);
    }
    let Value::Ident(var) = entry else {
        return Err(format!("datad `{name}` is not a variable"));
    };
    spec(&assignment(text, var)?).map(MplMap::Sampled)
}

/// Evaluates a segment-data dict, a `cubehelix()` call, or a color list.
pub fn spec(value: &Value) -> Result<Vec<[f64; 3]>, String> {
    if let Value::Call { name, .. } = value
        && name == "cubehelix"
    {
        return Ok(sample_fns(
            |x| cubehelix(x, -0.14861, 1.78277),
            |x| cubehelix(x, -0.29227, -0.90649),
            |x| cubehelix(x, 1.97294, 0.0),
        ));
    }
    if value.get("red").is_some() {
        let channel = |key: &str| -> Result<Vec<f64>, String> {
            channel_lut(value.get(key).ok_or_else(|| format!("missing `{key}`"))?)
        };
        let (r, g, b) = (channel("red")?, channel("green")?, channel("blue")?);
        return Ok((0..N).map(|i| [r[i], g[i], b[i]]).collect());
    }
    from_list(value)
}

/// `LinearSegmentedColormap.from_list`: evenly spaced colors or `(x, color)` pairs.
fn from_list(value: &Value) -> Result<Vec<[f64; 3]>, String> {
    let items = value.seq().ok_or("expected a color list")?;
    let n = items.len().max(2) as f64 - 1.0;
    let mut rows = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let parts = item.seq().ok_or("expected a color")?;
        let (x, color) = match parts {
            [Value::Num(x), Value::Seq(c)] => (*x, c.as_slice()),
            _ => (i as f64 / n, parts),
        };
        let c = rgb(color)?;
        rows.push([x, c[0], c[1], c[2]]);
    }
    let table = |k: usize| rows.iter().map(|r| [r[0], r[k], r[k]]).collect::<Vec<_>>();
    let (r, g, b) = (
        segment_lut(&table(1)),
        segment_lut(&table(2)),
        segment_lut(&table(3)),
    );
    Ok((0..N).map(|i| [r[i], g[i], b[i]]).collect())
}

/// Colors of a `ListedColormap`.
pub fn listed(value: &Value) -> Result<Vec<[f64; 3]>, String> {
    value
        .seq()
        .ok_or("expected a color table")?
        .iter()
        .map(|c| rgb(c.seq().unwrap_or_default()))
        .collect()
}

fn rgb(parts: &[Value]) -> Result<[f64; 3], String> {
    match parts {
        [r, g, b, ..] => Ok([
            r.num().ok_or("bad r")?,
            g.num().ok_or("bad g")?,
            b.num().ok_or("bad b")?,
        ]),
        _ => Err("expected (r, g, b)".into()),
    }
}

fn channel_lut(value: &Value) -> Result<Vec<f64>, String> {
    match value {
        Value::Seq(rows) => {
            let table: Result<Vec<[f64; 3]>, String> = rows
                .iter()
                .map(|r| rgb(r.seq().unwrap_or_default()))
                .collect();
            Ok(segment_lut(&table?))
        }
        Value::Ident(name) => {
            let f = function(name).ok_or_else(|| format!("unknown function `{name}`"))?;
            Ok((0..N)
                .map(|i| f(i as f64 / (N - 1) as f64).clamp(0.0, 1.0))
                .collect())
        }
        _ => Err("unsupported channel definition".into()),
    }
}

/// `matplotlib.colors._create_lookup_table` for `(x, y0, y1)` rows, gamma 1.
pub fn segment_lut(rows: &[[f64; 3]]) -> Vec<f64> {
    let (Some(first), Some(last)) = (rows.first(), rows.last()) else {
        return vec![0.0; N];
    };
    let x: Vec<f64> = rows.iter().map(|r| r[0] * (N - 1) as f64).collect();
    let mut lut = vec![first[2]; N];
    for (i, v) in lut.iter_mut().enumerate().take(N - 1).skip(1) {
        let xi = i as f64;
        let ind = x.partition_point(|&xv| xv < xi).clamp(1, x.len() - 1);
        let distance = (xi - x[ind - 1]) / (x[ind] - x[ind - 1]);
        *v = distance * (rows[ind][1] - rows[ind - 1][2]) + rows[ind - 1][2];
    }
    lut[N - 1] = last[1];
    lut.iter().map(|v| v.clamp(0.0, 1.0)).collect()
}

fn sample_fns(
    r: impl Fn(f64) -> f64,
    g: impl Fn(f64) -> f64,
    b: impl Fn(f64) -> f64,
) -> Vec<[f64; 3]> {
    (0..N)
        .map(|i| {
            let x = i as f64 / (N - 1) as f64;
            [
                r(x).clamp(0.0, 1.0),
                g(x).clamp(0.0, 1.0),
                b(x).clamp(0.0, 1.0),
            ]
        })
        .collect()
}

/// `_ch_helper` with Matplotlib's defaults gamma=1, s=0.5, r=-1.5, h=1.
fn cubehelix(x: f64, p0: f64, p1: f64) -> f64 {
    let a = x * (1.0 - x) / 2.0;
    let phi = 2.0 * PI * (0.5 / 3.0 - 1.5 * x);
    x + a * (p0 * phi.cos() + p1 * phi.sin())
}

/// Generator functions referenced by name in `_cm.py`.
fn function(name: &str) -> Option<fn(f64) -> f64> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_lut_matches_matplotlib_gray_and_steps() {
        let gray = segment_lut(&[[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]);
        assert_eq!(gray[0], 0.0);
        assert!((gray[128] - 128.0 / 255.0).abs() < 1e-12);
        assert_eq!(gray[255], 1.0);
        // A discontinuity at 0.5 jumps from y0 (left side) to y1 (right side).
        let step = segment_lut(&[[0.0, 0.0, 0.0], [0.5, 0.2, 0.8], [1.0, 1.0, 1.0]]);
        assert!(step[127] < 0.2 && step[128] > 0.8);
    }

    #[test]
    fn evaluates_datad_entries() {
        let py = "_gray_data = {'red': ((0., 0, 0), (1., 1, 1)), 'green': ((0., 0, 0), (1., 1, 1)), 'blue': ((0., 0, 0), (1., 1, 1))}\n\
                  _ocean_data = {'red': gfunc[23], 'green': gfunc[28], 'blue': gfunc[3]}\n\
                  _t_data = ((1.0, 0.0, 0.0), (0.0, 0.0, 1.0))\n\
                  datad = {'gray': _gray_data, 'ocean': _ocean_data, 't': {'listed': _t_data}}\n";
        let Ok(MplMap::Sampled(gray)) = datad_map(py, "gray") else {
            panic!("gray")
        };
        assert_eq!(gray[255], [1.0, 1.0, 1.0]);
        let Ok(MplMap::Sampled(ocean)) = datad_map(py, "ocean") else {
            panic!("ocean")
        };
        assert_eq!(ocean[0], [0.0, 0.5, 0.0]);
        let Ok(MplMap::Listed(t)) = datad_map(py, "t") else {
            panic!("listed")
        };
        assert_eq!(t.len(), 2);
    }
}
