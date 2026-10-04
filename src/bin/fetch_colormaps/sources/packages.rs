//! Sources spread over several upstream files or names: colorcet's module,
//! cmocean's per-map tables, CMasher's `.jscm` files and Crameri's archive.

use super::{Unit, json, pylit, python, text, zip};
use crate::classify;
use crate::families::Remote;
use crate::fetch::{Error, Fetcher};

pub fn colorcet(init: &str) -> Result<Vec<Unit>, Error> {
    let aliases = python::colorcet_aliases(init)?;
    Ok(python::colorcet_defs(init)?
        .into_iter()
        .map(|(raw, stops)| {
            let kind = classify::colorcet(&raw);
            (aliases.get(&raw).cloned().unwrap_or(raw), kind, stops)
        })
        .collect())
}

pub fn cmocean(fetcher: &mut Fetcher, cm_py: &Remote, rgb_base: &str) -> Result<Vec<Unit>, Error> {
    let names: Vec<String> = pylit::assignment(fetcher.text(cm_py)?, "cmapnames")?
        .seq()
        .ok_or("cmocean: `cmapnames` is not a list")?
        .iter()
        .map(|v| {
            v.str()
                .map(str::to_string)
                .ok_or_else(|| format!("cmocean: `cmapnames` entry {v:?} is not a string"))
        })
        .collect::<Result<_, _>>()?;
    if names.is_empty() {
        return Err("cmocean: `cmapnames` is empty".into());
    }
    names
        .into_iter()
        .map(|n| {
            let url = format!("{rgb_base}{n}-rgb.txt");
            let stops = text::numeric_table(fetcher.pinned_text(&url)?);
            let kind = classify::cmocean(&n);
            Ok((n, kind, stops))
        })
        .collect()
}

pub fn cmasher(
    fetcher: &mut Fetcher,
    tree_api: &Remote,
    raw_base: &str,
) -> Result<Vec<Unit>, Error> {
    const DIR: &str = "src/cmasher/colormaps/";
    let tree: serde_json::Value = serde_json::from_str(fetcher.text(tree_api)?)?;
    let paths: Vec<String> = tree["tree"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| e["path"].as_str())
        .filter(|p| p.starts_with(DIR) && p.ends_with(".jscm"))
        .map(str::to_string)
        .collect();
    if paths.is_empty() {
        return Err(format!("CMasher: no `{DIR}*.jscm` files in the tree").into());
    }
    paths
        .iter()
        .map(|path| {
            let name = path
                .rsplit('/')
                .next()
                .unwrap_or_default()
                .trim_end_matches(".jscm")
                .to_string();
            let url = format!("{raw_base}{path}");
            let (kind, stops) =
                json::jscm(fetcher.pinned_text(&url)?).map_err(|e| format!("{url}: {e}"))?;
            Ok((name, kind, stops))
        })
        .collect()
}

pub fn crameri(zip_bytes: &[u8]) -> Result<Vec<Unit>, Error> {
    let entries = zip::entries(zip_bytes)?;
    let mut out = Vec::new();
    for entry in &entries {
        let parts: Vec<&str> = entry.name.split('/').collect();
        let name = match parts.as_slice() {
            [dir, file] if *file == format!("{dir}.txt") => *dir,
            [_, "CategoricalPalettes", file] if file.ends_with("S.txt") => {
                file.trim_end_matches(".txt")
            }
            _ => continue,
        };
        let text = String::from_utf8(zip::read(zip_bytes, entry)?)
            .map_err(|e| format!("{}: {e}", entry.name))?;
        out.push((
            name.to_string(),
            classify::crameri(name),
            text::numeric_table(&text),
        ));
    }
    out.sort_by_key(|a| a.0.to_lowercase());
    Ok(out)
}
