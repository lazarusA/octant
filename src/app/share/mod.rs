//! Share links: a remote store, a variable, its dimensions, the plot type
//! and colors as a readable query (`?src=https://x/a.zarr&var=t2m&time=2020-01-15
//! &lat=30..60&plot=heatmap&cmap=cmocean:thermal&range=250..310`), replayed
//! as `AppAction`s through the command queue (`Origin::Link`), so a link
//! fails as a command does. Keys apply in a fixed order whatever their
//! order: `src`, `var`, `axes` (x, y[, z] dimensions), `anim`, the
//! dimensions (any other key, `dim.<name>` for one named like a key; `a..b`
//! a range, since ISO times hold `:`), `plot`, `cmap`, `range`, `overlay`
//! (repeatable). Values are what the actions take: ISO dates, numbers in
//! the coordinate's units, labels.

use percent_encoding::percent_decode_str;

use super::actions::{AppAction, DimRole};

mod build;
#[cfg(test)]
mod tests;

/// Keys that are no dimension; `dim.<key>` names a dimension called so.
const KEYS: [&str; 8] = [
    "src", "var", "axes", "anim", "plot", "cmap", "range", "overlay",
];

/// Stores a link may open: remote ones, never a local path (a link is
/// untrusted text).
const SCHEMES: [&str; 5] = ["http://", "https://", "s3://", "gs://", "procedural://"];

/// What the share link in `text` (a whole URL, or its query) does, in the
/// order it applies; `None` when `text` holds no `src` with a remote store.
pub fn parse_share(text: &str) -> Option<Vec<AppAction>> {
    let text = text.trim();
    let query = text.split_once('?').map_or(text, |(_, q)| q);
    let query = query.split_once('#').map_or(query, |(q, _)| q);
    let pairs: Vec<(String, String)> = query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .map(|(k, v)| (decode(k), decode(v)))
        .filter(|(_, v)| !v.is_empty())
        .collect();
    let last = |key: &str| pairs.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v);
    let src = last("src").filter(|s| SCHEMES.iter().any(|p| s.starts_with(p)))?;
    let mut actions = vec![AppAction::OpenSource {
        source: src.clone(),
    }];
    let Some(var) = last("var") else {
        return Some(actions);
    };
    actions.push(AppAction::SelectVariable { path: var.clone() });
    let axes = last("axes").map_or("", |a| a.as_str()).split(',');
    for (dim, role) in axes.zip([DimRole::X, DimRole::Y, DimRole::Z]) {
        if !dim.is_empty() {
            actions.push(AppAction::SetRole {
                dim: dim.into(),
                role,
            });
        }
    }
    if let Some(dim) = last("anim") {
        let role = DimRole::Animate;
        actions.push(AppAction::SetRole {
            dim: dim.clone(),
            role,
        });
    }
    actions.extend(dimension_actions(&pairs));
    let plot_type = last("plot").cloned();
    actions.push(AppAction::Plot { plot_type });
    if let Some(name) = last("cmap") {
        let (name, layer) = (name.clone(), None);
        actions.push(AppAction::SetColormap { name, layer });
    }
    if let Some((min, max)) = last("range").and_then(|r| numbers(r)) {
        actions.push(AppAction::SetColorRange {
            min,
            max,
            layer: None,
        });
    }
    for (_, path) in pairs.iter().filter(|(k, _)| k == "overlay") {
        actions.push(AppAction::SelectVariable { path: path.clone() });
        actions.push(AppAction::AddOverlay);
    }
    Some(actions)
}

/// `SetDim`/`SetRange` for each dimension key, the last of a repeated one.
fn dimension_actions(pairs: &[(String, String)]) -> Vec<AppAction> {
    let mut actions = Vec::new();
    for (i, (key, value)) in pairs.iter().enumerate() {
        let dim = match key.strip_prefix("dim.") {
            Some(dim) => dim,
            None if KEYS.contains(&key.as_str()) => continue,
            None => key,
        };
        if pairs[i + 1..].iter().any(|(k, _)| k == key) {
            continue;
        }
        let dim = Some(dim.to_string());
        actions.push(match value.split_once("..") {
            Some((from, to)) => AppAction::SetRange {
                dim,
                from: from.into(),
                to: to.into(),
            },
            None => AppAction::SetDim {
                dim,
                value: value.clone(),
            },
        });
    }
    actions
}

/// `a..b` as two finite numbers.
fn numbers(range: &str) -> Option<(f32, f32)> {
    let (a, b) = range.split_once("..")?;
    let (a, b) = (a.trim().parse::<f32>().ok()?, b.trim().parse::<f32>().ok()?);
    (a.is_finite() && b.is_finite()).then_some((a, b))
}

/// A query component: `+` is a space, `%XX` a byte.
fn decode(text: &str) -> String {
    percent_decode_str(&text.replace('+', " "))
        .decode_utf8_lossy()
        .into_owned()
}
