//! Evaluates Matplotlib colormap definitions exactly like `matplotlib.colors`
//! does for a 256-entry lookup table: segment tables, generator functions,
//! color lists (`LinearSegmentedColormap.from_list`) and listed palettes.

use super::mpl_functions::{CUBEHELIX_SOURCE, function, require_lines};
use super::pylit::{Value, assignment};
use std::f64::consts::PI;

pub const N: usize = 256;

/// `datad` of a `_cm.py` module, parsed once and evaluated per map.
pub struct Datad<'a> {
    text: &'a str,
    dict: Value,
}

impl<'a> Datad<'a> {
    pub fn parse(text: &'a str) -> Result<Self, String> {
        Ok(Self {
            text,
            dict: assignment(text, "datad")?,
        })
    }

    /// Evaluates `datad[name]`: 256 samples, or the colors of a listed palette.
    pub fn map(&self, name: &str) -> Result<Vec<[f64; 3]>, String> {
        let entry = self
            .dict
            .get(name)
            .ok_or_else(|| format!("datad has no `{name}`"))?;
        if let Some(Value::Ident(var)) = entry.get("listed") {
            return listed(&assignment(self.text, var)?);
        }
        let Value::Ident(var) = entry else {
            return Err(format!("datad `{name}` is not a variable"));
        };
        spec(&assignment(self.text, var)?, self.text).map_err(|e| format!("datad `{name}`: {e}"))
    }
}

/// Evaluates a segment-data dict, a `cubehelix()` call, or a color list.
/// Generator functions must still be defined in `source` as they were ported.
pub fn spec(value: &Value, source: &str) -> Result<Vec<[f64; 3]>, String> {
    if let Value::Call { name, args, kwargs } = value
        && name == "cubehelix"
    {
        if !args.is_empty() || !kwargs.is_empty() {
            return Err(
                "cubehelix() is called with arguments; only the defaults are ported".into(),
            );
        }
        require_lines(source, "cubehelix", CUBEHELIX_SOURCE)?;
        return Ok(sample_fns(
            |x| cubehelix(x, -0.14861, 1.78277),
            |x| cubehelix(x, -0.29227, -0.90649),
            |x| cubehelix(x, 1.97294, 0.0),
        ));
    }
    if value.get("red").is_some() {
        let channel = |key: &str| -> Result<Vec<f64>, String> {
            let def = value.get(key).ok_or_else(|| format!("missing `{key}`"))?;
            channel_lut(def, source)
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
        segment_lut(&table(1))?,
        segment_lut(&table(2))?,
        segment_lut(&table(3))?,
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

fn channel_lut(value: &Value, source: &str) -> Result<Vec<f64>, String> {
    match value {
        Value::Seq(rows) => {
            let table: Result<Vec<[f64; 3]>, String> = rows
                .iter()
                .map(|r| rgb(r.seq().unwrap_or_default()))
                .collect();
            segment_lut(&table?)
        }
        Value::Ident(name) => {
            let f = function(name).ok_or_else(|| format!("unknown function `{name}`"))?;
            require_lines(source, name, f.source)?;
            Ok((0..N)
                .map(|i| (f.eval)(i as f64 / (N - 1) as f64).clamp(0.0, 1.0))
                .collect())
        }
        _ => Err("unsupported channel definition".into()),
    }
}

/// `matplotlib.colors._create_lookup_table` for `(x, y0, y1)` rows, gamma 1,
/// with the checks of `LinearSegmentedColormap`: at least two rows, `x` from 0
/// to 1 and never decreasing.
pub fn segment_lut(rows: &[[f64; 3]]) -> Result<Vec<f64>, String> {
    let (Some(first), Some(last)) = (rows.first(), rows.last()) else {
        return Err("segment table is empty".into());
    };
    if rows.len() < 2 {
        return Err("segment table needs at least two rows".into());
    }
    if first[0] != 0.0 || last[0] != 1.0 {
        return Err("segment table must start at x=0 and end at x=1".into());
    }
    if rows.windows(2).any(|w| w[1][0] < w[0][0]) {
        return Err("segment table x must be in increasing order".into());
    }
    let x: Vec<f64> = rows.iter().map(|r| r[0] * (N - 1) as f64).collect();
    let mut lut = vec![first[2]; N];
    for (i, v) in lut.iter_mut().enumerate().take(N - 1).skip(1) {
        let xi = i as f64;
        let ind = x.partition_point(|&xv| xv < xi).clamp(1, x.len() - 1);
        let distance = (xi - x[ind - 1]) / (x[ind] - x[ind - 1]);
        *v = distance * (rows[ind][1] - rows[ind - 1][2]) + rows[ind - 1][2];
    }
    lut[N - 1] = last[1];
    Ok(lut.iter().map(|v| v.clamp(0.0, 1.0)).collect())
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

#[cfg(test)]
#[path = "matplotlib_tests.rs"]
mod tests;
