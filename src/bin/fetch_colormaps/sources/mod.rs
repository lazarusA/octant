//! Converts each upstream [`Source`] into named, classified color stops.

mod json;
mod julia;
pub mod matplotlib;
mod mpl_functions;
pub mod pylit;
mod pylit_parser;
mod python;
pub mod text;
mod zip;

use crate::classify;
use crate::families::Source;
use crate::fetch::{Error, Fetcher};
use matplotlib::MplMap;
use octant::utils::colormap::ColormapKind;

/// A converted map: Octant name, kind and unit RGB stops.
pub type Converted = (String, ColormapKind, Vec<[f64; 3]>);

pub fn convert(fetcher: &mut Fetcher, source: &Source) -> Result<Vec<Converted>, Error> {
    match source {
        Source::MatplotlibCm { .. }
        | Source::PyTables { .. }
        | Source::Cmyt { .. }
        | Source::SeabornPalettes { .. }
        | Source::Colorcet { .. }
        | Source::Cmocean { .. } => python_source(fetcher, source),
        Source::CBytes { .. }
        | Source::Table { .. }
        | Source::CartoTs { .. }
        | Source::RList { .. }
        | Source::Yaml { .. }
        | Source::CssVars { .. }
        | Source::Krzywinski { .. }
        | Source::Hex { .. } => text_source(fetcher, source),
        Source::Cmasher { .. }
        | Source::CrameriZip { .. }
        | Source::ColorBrewer { .. }
        | Source::Tol { .. }
        | Source::Catppuccin { .. }
        | Source::ColorSchemesJl { .. } => structured_source(fetcher, source),
    }
}

/// One map from one upstream file.
fn one(name: &str, kind: ColormapKind, stops: Vec<[f64; 3]>) -> Vec<Converted> {
    vec![(name.to_string(), kind, stops)]
}

/// Sources stored as Python modules (Matplotlib, BIDS, seaborn, colorcet, cmocean, cmyt).
fn python_source(fetcher: &mut Fetcher, source: &Source) -> Result<Vec<Converted>, Error> {
    Ok(match *source {
        Source::MatplotlibCm { url, maps } => {
            let text = fetcher.text(url)?;
            maps.iter()
                .map(|&(mpl, name, kind)| {
                    let (MplMap::Sampled(stops) | MplMap::Listed(stops)) =
                        matplotlib::datad_map(text, mpl)?;
                    Ok((name.to_string(), kind, stops))
                })
                .collect::<Result<_, String>>()?
        }
        Source::PyTables { url, maps } => {
            let text = fetcher.text(url)?;
            maps.iter()
                .map(|&(var, name, kind)| Ok((name.to_string(), kind, python::table(text, var)?)))
                .collect::<Result<_, String>>()?
        }
        Source::Cmyt { base, names } => names
            .iter()
            .map(|n| {
                let stops = python::cmyt(fetcher.text(&format!("{base}{n}.py"))?)?;
                Ok((n.to_string(), ColormapKind::Sequential, stops))
            })
            .collect::<Result<_, Error>>()?,
        Source::SeabornPalettes { url } => {
            categorical(python::seaborn_palettes(fetcher.text(url)?)?)
        }
        Source::Colorcet { url } => colorcet(fetcher.text(url)?)?,
        Source::Cmocean { cm_py, rgb_base } => cmocean(fetcher, cm_py, rgb_base)?,
        _ => return Err("not a Python source".into()),
    })
}

/// Plain-text sources (tables, C arrays, TypeScript, R, YAML, CSS, published values).
fn text_source(fetcher: &mut Fetcher, source: &Source) -> Result<Vec<Converted>, Error> {
    Ok(match *source {
        Source::CBytes {
            url,
            array,
            name,
            kind,
        } => one(name, kind, text::c_byte_triples(fetcher.text(url)?, array)),
        Source::Table { url, name, kind } => {
            one(name, kind, text::numeric_table(fetcher.text(url)?))
        }
        Source::CartoTs { url } => text::carto_ts(fetcher.text(url)?)
            .into_iter()
            .map(|(name, colors, tags)| (name, classify::carto(&tags), colors))
            .collect(),
        Source::RList { url, list } => palettes(text::r_list(fetcher.text(url)?, list)),
        Source::Yaml { url } => palettes(text::yaml_hex_lists(fetcher.text(url)?)),
        Source::CssVars { url, prefix, name } => one(
            name,
            ColormapKind::Categorical,
            text::css_vars(fetcher.text(url)?, prefix),
        ),
        Source::Krzywinski { url, name } => one(
            name,
            ColormapKind::Categorical,
            text::krzywinski(fetcher.text(url)?),
        ),
        Source::Hex { name, kind, colors } => one(
            name,
            kind,
            colors.iter().filter_map(|h| text::hex_rgb(h)).collect(),
        ),
        _ => return Err("not a text source".into()),
    })
}

/// JSON, archive and ColorSchemes.jl sources.
fn structured_source(fetcher: &mut Fetcher, source: &Source) -> Result<Vec<Converted>, Error> {
    Ok(match *source {
        Source::Cmasher { tree_api, raw_base } => cmasher(fetcher, tree_api, raw_base)?,
        Source::CrameriZip { url, cache } => crameri(&fetcher.cached_bytes(url, cache)?)?,
        Source::ColorBrewer { url } => json::colorbrewer(fetcher.text(url)?)?,
        Source::Tol { url } => json::tol(fetcher.text(url)?)?,
        Source::Catppuccin { url } => json::catppuccin(fetcher.text(url)?)?,
        Source::ColorSchemesJl { url, names } => {
            categorical(julia::schemes(fetcher.text(url)?, names)?)
        }
        _ => return Err("not a structured source".into()),
    })
}

fn categorical(maps: Vec<(String, Vec<[f64; 3]>)>) -> Vec<Converted> {
    maps.into_iter()
        .map(|(n, c)| (n, ColormapKind::Categorical, c))
        .collect()
}

/// Palette collections: categorical when short, otherwise classified by lightness.
fn palettes(maps: Vec<(String, Vec<[f64; 3]>)>) -> Vec<Converted> {
    maps.into_iter()
        .map(|(n, c)| (n, classify::palette_kind(&to_rgb8(&c)), c))
        .collect()
}

/// Quantizes unit RGB stops to 8 bits per channel.
pub fn to_rgb8(colors: &[[f64; 3]]) -> Vec<[u8; 3]> {
    colors
        .iter()
        .map(|c| c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8))
        .collect()
}

fn colorcet(init: &str) -> Result<Vec<Converted>, Error> {
    let aliases = python::colorcet_aliases(init)?;
    Ok(python::colorcet_defs(init)?
        .into_iter()
        .map(|(raw, stops)| {
            let kind = classify::colorcet(&raw);
            (aliases.get(&raw).cloned().unwrap_or(raw), kind, stops)
        })
        .collect())
}

fn cmocean(fetcher: &mut Fetcher, cm_py: &str, rgb_base: &str) -> Result<Vec<Converted>, Error> {
    let names: Vec<String> = pylit::assignment(fetcher.text(cm_py)?, "cmapnames")?
        .seq()
        .unwrap_or_default()
        .iter()
        .filter_map(|v| v.str().map(str::to_string))
        .collect();
    names
        .into_iter()
        .map(|n| {
            let stops = text::numeric_table(fetcher.text(&format!("{rgb_base}{n}-rgb.txt"))?);
            Ok((n.clone(), classify::cmocean(&n), stops))
        })
        .collect()
}

fn cmasher(fetcher: &mut Fetcher, tree_api: &str, raw_base: &str) -> Result<Vec<Converted>, Error> {
    let tree: serde_json::Value = serde_json::from_str(fetcher.text(tree_api)?)?;
    let paths: Vec<String> = tree["tree"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| e["path"].as_str())
        .filter(|p| p.starts_with("src/cmasher/colormaps/") && p.ends_with(".jscm"))
        .map(str::to_string)
        .collect();
    paths
        .iter()
        .map(|path| {
            let name = path
                .rsplit('/')
                .next()
                .unwrap_or_default()
                .trim_end_matches(".jscm")
                .to_string();
            let (kind, stops) = json::jscm(fetcher.text(&format!("{raw_base}{path}"))?)?;
            Ok((name, kind, stops))
        })
        .collect()
}

fn crameri(zip_bytes: &[u8]) -> Result<Vec<Converted>, Error> {
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
        let text = String::from_utf8(zip::read(zip_bytes, entry)?)?;
        out.push((
            name.to_string(),
            classify::crameri(name),
            text::numeric_table(&text),
        ));
    }
    out.sort_by_key(|a| a.0.to_lowercase());
    Ok(out)
}
