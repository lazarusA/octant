//! JSON upstream formats: ColorBrewer, Paul Tol's `colors.json`, Catppuccin's
//! `palette.json` and CMasher's `.jscm` files.
//! Objects are read in file order (`serde_json` with `preserve_order`).

use super::text::{css_rgb, hex_rgb};
use octant::utils::colormap::ColormapKind;
use serde_json::Value;

type Palette = (String, ColormapKind, Vec<[f64; 3]>);

/// ColorBrewer `colorbrewer.json`: the largest class of each scheme, kind from `type`.
pub fn colorbrewer(text: &str) -> Result<Vec<Palette>, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let schemes = root.as_object().ok_or("expected an object")?;
    Ok(schemes
        .iter()
        .filter_map(|(name, scheme)| {
            let obj = scheme.as_object()?;
            let (_, colors) = obj
                .iter()
                .filter_map(|(k, v)| Some((k.parse::<u32>().ok()?, v.as_array()?)))
                .max_by_key(|(n, _)| *n)?;
            let colors = colors.iter().filter_map(|c| css_rgb(c.as_str()?)).collect();
            let kind = match obj.get("type")?.as_str()? {
                "div" => ColormapKind::Diverging,
                "qual" => ColormapKind::Categorical,
                _ => ColormapKind::Sequential,
            };
            Some((name.clone(), kind, colors))
        })
        .collect())
}

/// Paul Tol `colors.json`: color sets (in file order), colormaps and both rainbows.
pub fn tol(text: &str) -> Result<Vec<Palette>, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    let sets = root
        .get("colorsets")
        .and_then(Value::as_object)
        .ok_or("no colorsets")?;
    for (name, set) in sets {
        let colors = set
            .as_object()
            .ok_or_else(|| format!("colorset `{name}` is not an object"))?
            .values()
            .filter_map(|c| hex_rgb(c.as_str()?))
            .collect();
        out.push((name.clone(), ColormapKind::Categorical, colors));
    }
    let maps = root
        .get("colormaps")
        .and_then(Value::as_object)
        .ok_or("no colormaps")?;
    for (name, map) in maps {
        let kind = match name.as_str() {
            "sunset" | "nightfall" | "BuRd" | "PRGn" => ColormapKind::Diverging,
            _ => ColormapKind::Sequential,
        };
        out.push((name.clone(), kind, hexes(map.get("colors"))));
    }
    out.push((
        "rainbow".into(),
        ColormapKind::Other,
        hexes(root.pointer("/rainbow_linear/colors")),
    ));
    out.push((
        "rainbow_discrete".into(),
        ColormapKind::Categorical,
        tol_discrete_rainbow(&root),
    ));
    Ok(out)
}

fn hexes(v: Option<&Value>) -> Vec<[f64; 3]> {
    v.and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|c| hex_rgb(c.as_str()?)).collect())
        .unwrap_or_default()
}

/// The largest discrete rainbow: `colors` picked by the longest `indexes` entry.
fn tol_discrete_rainbow(root: &Value) -> Vec<[f64; 3]> {
    let colors = hexes(root.pointer("/rainbow_discrete/colors"));
    let longest = root
        .pointer("/rainbow_discrete/indexes")
        .and_then(Value::as_array)
        .and_then(|sets| {
            sets.iter()
                .filter_map(Value::as_array)
                .max_by_key(|s| s.len())
        });
    longest
        .map(|idx| {
            idx.iter()
                .filter_map(|i| colors.get(usize::try_from(i.as_u64()?).ok()?).copied())
                .collect()
        })
        .unwrap_or(colors)
}

/// Catppuccin `palette.json`: the accent colors of each flavor, in palette order.
pub fn catppuccin(text: &str) -> Result<Vec<Palette>, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let mut flavors: Vec<(u64, Palette)> = Vec::new();
    for (key, flavor) in root.as_object().ok_or("expected an object")? {
        let (Some(order), Some(colors)) = (
            flavor.get("order").and_then(Value::as_u64),
            flavor.get("colors"),
        ) else {
            continue;
        };
        let mut accents: Vec<(u64, [f64; 3])> = colors
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(_, c)| c.get("accent").and_then(Value::as_bool) == Some(true))
            .filter_map(|(_, c)| {
                Some((c.get("order")?.as_u64()?, hex_rgb(c.get("hex")?.as_str()?)?))
            })
            .collect();
        accents.sort_by_key(|(o, _)| *o);
        let colors = accents.into_iter().map(|(_, c)| c).collect();
        flavors.push((order, (key.clone(), ColormapKind::Categorical, colors)));
    }
    flavors.sort_by_key(|(o, _)| *o);
    Ok(flavors.into_iter().map(|(_, p)| p).collect())
}

/// CMasher `.jscm`: concatenated 8-bit hex colors, kind from `usage-hints`.
pub fn jscm(text: &str) -> Result<(ColormapKind, Vec<[f64; 3]>), String> {
    let root: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let hex = root
        .get("colors")
        .and_then(Value::as_str)
        .ok_or("no colors")?;
    let colors = hex
        .as_bytes()
        .as_chunks::<6>()
        .0
        .iter()
        .filter_map(|c| hex_rgb(&format!("#{}", std::str::from_utf8(c).ok()?)))
        .collect();
    let hints: Vec<&str> = root
        .get("usage-hints")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let kind = if hints.contains(&"diverging") {
        ColormapKind::Diverging
    } else if hints.contains(&"cyclic") {
        ColormapKind::Cyclic
    } else if hints.contains(&"qualitative") {
        ColormapKind::Categorical
    } else if hints.contains(&"sequential") {
        ColormapKind::Sequential
    } else {
        ColormapKind::Other
    };
    Ok((kind, colors))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brewer_tol_catppuccin_jscm() {
        let brewer = r#"{"Blues": {"3": ["rgb(0,0,0)", "rgb(1,1,1)", "rgb(2,2,2)"], "4": ["rgb(0,0,0)", "rgb(1,1,1)", "rgb(2,2,2)", "rgb(255,255,255)"], "type": "seq"}}"#;
        let b = colorbrewer(brewer).unwrap_or_default();
        assert_eq!((b[0].1, b[0].2.len()), (ColormapKind::Sequential, 4));

        let tol_json = r##"{"colorsets": {"bright": {"blue": "#0000FF", "red": "#FF0000"}},
            "colormaps": {"sunset": {"colors": ["#000000", "#FFFFFF"]}},
            "rainbow_linear": {"colors": ["#000000", "#FFFFFF"]},
            "rainbow_discrete": {"colors": ["#000000", "#FFFFFF", "#FF0000"], "indexes": [[0], [0, 2]]}}"##;
        let t = tol(tol_json).unwrap_or_default();
        assert_eq!(
            t[0].2,
            vec![[0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
            "file order kept"
        );
        assert_eq!(t[3].2, vec![[0.0; 3], [1.0, 0.0, 0.0]]);

        let cat = r##"{"version": "1", "mocha": {"order": 3, "colors": {"b": {"order": 1, "hex": "#FFFFFF", "accent": true}, "a": {"order": 0, "hex": "#000000", "accent": true}, "base": {"order": 2, "hex": "#111111", "accent": false}}}}"##;
        assert_eq!(
            catppuccin(cat).unwrap_or_default()[0].2,
            vec![[0.0; 3], [1.0; 3]]
        );

        let j = r#"{"usage-hints": ["cyclic"], "colors": "000000ffffff"}"#;
        assert_eq!(
            jscm(j),
            Ok((ColormapKind::Cyclic, vec![[0.0; 3], [1.0; 3]]))
        );
    }

    #[test]
    fn names_and_colors_follow_file_order() {
        let pair = r#"["rgb(0,0,0)", "rgb(255,255,255)"]"#;
        let brewer = format!(
            r#"{{"YlGn": {{"2": {pair}, "type": "seq"}}, "Accent": {{"2": {pair}, "type": "qual"}}, "PuOr": {{"2": {pair}, "type": "div"}}}}"#
        );
        let names: Vec<String> = colorbrewer(&brewer)
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.0)
            .collect();
        assert_eq!(names, ["YlGn", "Accent", "PuOr"]);

        let tol_json = r##"{"colorsets": {"vibrant": {"orange": "#FF0000", "blue": "#0000FF", "cyan": "#00FF00"}, "bright": {"blue": "#0000FF"}},
            "colormaps": {"YlOrBr": {"colors": ["#000000", "#FFFFFF"]}, "BuRd": {"colors": ["#000000", "#FFFFFF"]}},
            "rainbow_linear": {"colors": ["#000000", "#FFFFFF"]},
            "rainbow_discrete": {"colors": ["#000000", "#FFFFFF"], "indexes": [[0, 1]]}}"##;
        let t = tol(tol_json).unwrap_or_default();
        let names: Vec<&str> = t.iter().map(|p| p.0.as_str()).collect();
        assert_eq!(
            names,
            [
                "vibrant",
                "bright",
                "YlOrBr",
                "BuRd",
                "rainbow",
                "rainbow_discrete"
            ]
        );
        assert_eq!(
            t[0].2,
            vec![[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]]
        );
    }
}
