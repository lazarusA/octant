//! Colormap tables stored in Python modules: listed float tables, colorcet's
//! `# cmap_def` definitions, seaborn's palette dict, and cmyt's modules.

use super::matplotlib;
use super::mpl_functions::require_lines;
use super::pylit::{Value, assignment};
use super::text::hex_rgb;
use std::collections::HashMap;

/// Named unit-RGB tables.
type Named = Vec<(String, Vec<[f64; 3]>)>;

/// `var = [[r, g, b], ...]` (or a tuple of tuples) in 0–1 floats.
pub fn table(text: &str, var: &str) -> Result<Vec<[f64; 3]>, String> {
    matplotlib::listed(&assignment(text, var)?)
}

/// Every `name = [  # cmap_def` table in colorcet's `__init__.py`, each parsed
/// from its own line (no rescan of the module per definition).
pub fn colorcet_defs(text: &str) -> Result<Named, String> {
    let mut out = Vec::new();
    let mut start = 0;
    for line in text.split_inclusive('\n') {
        let rest = &text[start..];
        start += line.len();
        if !line.contains("# cmap_def") {
            continue;
        }
        let name = line.split(['=', ':']).next().unwrap_or_default().trim();
        out.push((name.to_string(), table(rest, name)?));
    }
    Ok(out)
}

/// colorcet's `aliases = {'name': ['alias', ...]}` as name → first alias.
pub fn colorcet_aliases(text: &str) -> Result<HashMap<String, String>, String> {
    let Value::Dict(items) = assignment(text, "aliases")? else {
        return Err("aliases is not a dict".into());
    };
    Ok(items
        .iter()
        .filter_map(|(k, v)| Some((k.str()?.to_string(), v.seq()?.first()?.str()?.to_string())))
        .collect())
}

/// seaborn `SEABORN_PALETTES = dict(name=["#hex", ...], ...)`.
pub fn seaborn_palettes(text: &str) -> Result<Named, String> {
    let Value::Call { kwargs, .. } = assignment(text, "SEABORN_PALETTES")? else {
        return Err("SEABORN_PALETTES is not a dict(...) call".into());
    };
    Ok(kwargs
        .iter()
        .map(|(name, v)| {
            let colors = v
                .seq()
                .unwrap_or_default()
                .iter()
                .filter_map(|c| hex_rgb(c.str()?))
                .collect();
            (name.clone(), colors)
        })
        .collect())
}

/// A cmyt module: `luts = np.transpose(([r...], [g...], [b...]))` or segment `data = {...}`.
pub fn cmyt(text: &str) -> Result<Vec<[f64; 3]>, String> {
    if let Ok(Value::Call { args, .. }) = assignment(text, "luts") {
        return cmyt_luts(&args);
    }
    if text.contains("_kamae_red") {
        return kamae(text);
    }
    matplotlib::spec(&assignment(text, "data")?, text)
}

/// The three equally long numeric channels of cmyt's `luts`, as RGB rows.
fn cmyt_luts(args: &[Value]) -> Result<Vec<[f64; 3]>, String> {
    let channels = args
        .first()
        .and_then(Value::seq)
        .ok_or("luts has no channels")?;
    let [r, g, b] = channels else {
        return Err(format!("luts has {} channels, expected 3", channels.len()));
    };
    let ch = |v: &Value| -> Result<Vec<f64>, String> {
        v.seq()
            .ok_or("luts channel is not a list")?
            .iter()
            .map(|n| {
                n.num()
                    .ok_or_else(|| format!("luts: non-numeric entry {n:?}"))
            })
            .collect()
    };
    let (r, g, b) = (ch(r)?, ch(g)?, ch(b)?);
    if r.len() != g.len() || r.len() != b.len() {
        return Err(format!(
            "luts channels differ in length ({}, {}, {})",
            r.len(),
            g.len(),
            b.len()
        ));
    }
    Ok(r.iter()
        .zip(&g)
        .zip(&b)
        .map(|((r, g), b)| [*r, *g, *b])
        .collect())
}

/// Every line of cmyt `pastel.py` the port below depends on.
const KAMAE_SOURCE: &[&str] = &[
    "_vs = np.linspace(0, 1, 255)",
    "255,",
    "113.9 * np.sin(7.64 * (_vs**1.705) + 0.701)",
    "- 916.1 * (_vs + 1.755) ** 1.862",
    "+ 3587.9 * _vs",
    "+ 2563.4,",
    "/ 255.0",
    "np.minimum(255, 70.0 * np.sin(8.7 * (_vs**1.26) - 2.418) + 151.7 * _vs**0.5 + 70.0)",
    "194.5 * _vs**2.88",
    "+ 99.72 * np.exp(-77.24 * (_vs - 0.742) ** 2.0)",
    "+ 45.40 * _vs**0.089",
    "+ 10.0,",
    "\"red\": np.transpose([_vs, _kamae_red, _kamae_red]),",
    "\"green\": np.transpose([_vs, _kamae_grn, _kamae_grn]),",
    "\"blue\": np.transpose([_vs, _kamae_blu, _kamae_blu]),",
];

/// cmyt `pastel`: Tune Kamae's closed-form channels sampled at 255 points and
/// used as a segment table. Every upstream formula line is checked to be unchanged.
fn kamae(text: &str) -> Result<Vec<[f64; 3]>, String> {
    require_lines(text, "cmyt pastel", KAMAE_SOURCE)?;
    let vs: Vec<f64> = (0..255).map(|i| f64::from(i) / 254.0).collect();
    let channel = |f: fn(f64) -> f64| -> Result<Vec<f64>, String> {
        let rows: Vec<[f64; 3]> = vs.iter().map(|&v| [v, f(v), f(v)]).collect();
        matplotlib::segment_lut(&rows)
    };
    let r = channel(|v| {
        (113.9 * (7.64 * v.powf(1.705) + 0.701).sin() - 916.1 * (v + 1.755).powf(1.862)
            + 3587.9 * v
            + 2563.4)
            .min(255.0)
            / 255.0
    })?;
    let g = channel(|v| {
        (70.0 * (8.7 * v.powf(1.26) - 2.418).sin() + 151.7 * v.powf(0.5) + 70.0).min(255.0) / 255.0
    })?;
    let b = channel(|v| {
        (194.5 * v.powf(2.88)
            + 99.72 * (-77.24 * (v - 0.742).powi(2)).exp()
            + 45.40 * v.powf(0.089)
            + 10.0)
            .min(255.0)
            / 255.0
    })?;
    Ok((0..matplotlib::N).map(|i| [r[i], g[i], b[i]]).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colorcet_definitions_and_aliases() {
        let py = "aliases = {\n    'linear_kryw_0_100_c71': ['fire'],\n}\n\
                  linear_kryw_0_100_c71 = [  # cmap_def\n[0, 0, 0],\n[1, 1, 1],\n]\n";
        let defs = colorcet_defs(py).unwrap_or_default();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].1[1], [1.0, 1.0, 1.0]);
        let aliases = colorcet_aliases(py).unwrap_or_default();
        assert_eq!(
            aliases.get("linear_kryw_0_100_c71").map(String::as_str),
            Some("fire")
        );
    }

    #[test]
    fn cmyt_luts_are_transposed() {
        let py = "luts = np.transpose(\n    (\n        [0.0, 1.0],\n        [0.5, 0.5],\n        [1.0, 0.0],\n    )\n)\n";
        assert_eq!(cmyt(py), Ok(vec![[0.0, 0.5, 1.0], [1.0, 0.5, 0.0]]));
    }

    #[test]
    fn cmyt_luts_reject_bad_channels() {
        let uneven = "luts = np.transpose(([0.0, 1.0], [0.5], [1.0, 0.0]))\n";
        assert!(cmyt(uneven).is_err());
        let text = "luts = np.transpose(([0.0, 'x'], [0.5, 0.5], [1.0, 0.0]))\n";
        assert!(cmyt(text).is_err());
        let two = "luts = np.transpose(([0.0, 1.0], [0.5, 0.5]))\n";
        assert!(cmyt(two).is_err());
    }

    #[test]
    fn kamae_requires_unchanged_formulas() {
        let source = KAMAE_SOURCE.join("\n");
        assert_eq!(kamae(&source).map(|s| s.len()), Ok(matplotlib::N));
        let changed = source.replace("+ 2563.4,", "+ 2563.5,");
        assert!(kamae(&changed).is_err());
    }
}
