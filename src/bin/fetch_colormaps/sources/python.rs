//! Colormap tables stored in Python modules: listed float tables, colorcet's
//! `# cmap_def` definitions, seaborn's palette dict, and cmyt's modules.

use super::matplotlib;
use super::pylit::{Value, assignment};
use super::text::hex_rgb;
use std::collections::HashMap;

/// Named unit-RGB tables.
type Named = Vec<(String, Vec<[f64; 3]>)>;

/// `var = [[r, g, b], ...]` (or a tuple of tuples) in 0–1 floats.
pub fn table(text: &str, var: &str) -> Result<Vec<[f64; 3]>, String> {
    matplotlib::listed(&assignment(text, var)?)
}

/// Every `name = [  # cmap_def` table in colorcet's `__init__.py`.
pub fn colorcet_defs(text: &str) -> Result<Named, String> {
    text.lines()
        .filter(|l| l.contains("# cmap_def"))
        .filter_map(|l| {
            l.split(['=', ':'])
                .next()
                .map(|name| name.trim().to_string())
        })
        .map(|name| table(text, &name).map(|t| (name, t)))
        .collect()
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
        let channels = args
            .first()
            .and_then(Value::seq)
            .ok_or("luts has no channels")?;
        let ch = |k: usize| -> Vec<f64> {
            channels
                .get(k)
                .and_then(Value::seq)
                .unwrap_or_default()
                .iter()
                .filter_map(Value::num)
                .collect()
        };
        let (r, g, b) = (ch(0), ch(1), ch(2));
        return Ok(r
            .iter()
            .zip(&g)
            .zip(&b)
            .map(|((r, g), b)| [*r, *g, *b])
            .collect());
    }
    if text.contains("_kamae_red") {
        return kamae(text);
    }
    matplotlib::spec(&assignment(text, "data")?)
}

/// cmyt `pastel`: Tune Kamae's closed-form channels sampled at 255 points and
/// used as a segment table. The upstream formulas are checked to be unchanged.
fn kamae(text: &str) -> Result<Vec<[f64; 3]>, String> {
    const FORMULAS: [&str; 3] = [
        "113.9 * np.sin(7.64 * (_vs**1.705) + 0.701)",
        "70.0 * np.sin(8.7 * (_vs**1.26) - 2.418) + 151.7 * _vs**0.5 + 70.0",
        "99.72 * np.exp(-77.24 * (_vs - 0.742) ** 2.0)",
    ];
    if let Some(missing) = FORMULAS.iter().find(|f| !text.contains(**f)) {
        return Err(format!(
            "cmyt pastel formulas changed upstream (missing `{missing}`)"
        ));
    }
    let vs: Vec<f64> = (0..255).map(|i| f64::from(i) / 254.0).collect();
    let channel = |f: fn(f64) -> f64| -> Vec<f64> {
        let rows: Vec<[f64; 3]> = vs.iter().map(|&v| [v, f(v), f(v)]).collect();
        matplotlib::segment_lut(&rows)
    };
    let r = channel(|v| {
        (113.9 * (7.64 * v.powf(1.705) + 0.701).sin() - 916.1 * (v + 1.755).powf(1.862)
            + 3587.9 * v
            + 2563.4)
            .min(255.0)
            / 255.0
    });
    let g = channel(|v| {
        (70.0 * (8.7 * v.powf(1.26) - 2.418).sin() + 151.7 * v.powf(0.5) + 70.0).min(255.0) / 255.0
    });
    let b = channel(|v| {
        (194.5 * v.powf(2.88)
            + 99.72 * (-77.24 * (v - 0.742).powi(2)).exp()
            + 45.40 * v.powf(0.089)
            + 10.0)
            .min(255.0)
            / 255.0
    });
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
}
