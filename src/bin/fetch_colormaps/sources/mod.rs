//! Converts each upstream [`Source`] into named, classified color stops.

mod json;
mod julia;
mod matplotlib;
mod mpl_functions;
mod packages;
mod pylit;
mod pylit_parser;
mod python;
mod text;
mod zip;

use crate::classify;
use crate::families::Source;
use crate::fetch::{Error, Fetcher};
use octant::utils::colormap::ColormapKind;

/// A converted map: Octant name, kind and 8-bit sRGB stops.
pub type Converted = (String, ColormapKind, Vec<[u8; 3]>);

/// A map before quantization: name, kind and unit RGB stops.
type Unit = (String, ColormapKind, Vec<[f64; 3]>);

pub fn convert(fetcher: &mut Fetcher, source: &Source) -> Result<Vec<Converted>, Error> {
    let maps = match source {
        // Palette collections are classified from their quantized colors.
        Source::RList { url, list } => return Ok(palettes(text::r_list(fetcher.text(url)?, list))),
        Source::Yaml { url } => return Ok(palettes(text::yaml_hex_lists(fetcher.text(url)?))),
        Source::MatplotlibCm { .. }
        | Source::PyTables { .. }
        | Source::Cmyt { .. }
        | Source::SeabornPalettes { .. }
        | Source::Colorcet { .. }
        | Source::Cmocean { .. } => python_source(fetcher, source)?,
        Source::CBytes { .. }
        | Source::Table { .. }
        | Source::CartoTs { .. }
        | Source::CssVars { .. }
        | Source::Krzywinski { .. }
        | Source::Hex { .. } => text_source(fetcher, source)?,
        Source::Cmasher { .. }
        | Source::CrameriZip { .. }
        | Source::ColorBrewer { .. }
        | Source::Tol { .. }
        | Source::Catppuccin { .. }
        | Source::ColorSchemesJl { .. } => structured_source(fetcher, source)?,
    };
    Ok(maps
        .into_iter()
        .map(|(name, kind, stops)| (name, kind, to_rgb8(&stops)))
        .collect())
}

/// One map from one upstream file.
fn one(name: &str, kind: ColormapKind, stops: Vec<[f64; 3]>) -> Vec<Unit> {
    vec![(name.to_string(), kind, stops)]
}

/// Sources stored as Python modules (Matplotlib, BIDS, seaborn, colorcet, cmocean, cmyt).
fn python_source(fetcher: &mut Fetcher, source: &Source) -> Result<Vec<Unit>, Error> {
    Ok(match *source {
        Source::MatplotlibCm { ref url, maps } => {
            let datad = matplotlib::Datad::parse(fetcher.text(url)?)?;
            maps.iter()
                .map(|&(mpl, name, kind)| Ok((name.to_string(), kind, datad.map(mpl)?)))
                .collect::<Result<_, String>>()?
        }
        Source::PyTables { ref url, maps } => {
            let text = fetcher.text(url)?;
            maps.iter()
                .map(|&(var, name, kind)| Ok((name.to_string(), kind, python::table(text, var)?)))
                .collect::<Result<_, String>>()?
        }
        Source::Cmyt { base, names } => names
            .iter()
            .map(|n| {
                let stops = python::cmyt(fetcher.pinned_text(&format!("{base}{n}.py"))?)
                    .map_err(|e| format!("cmyt `{n}`: {e}"))?;
                Ok((n.to_string(), ColormapKind::Sequential, stops))
            })
            .collect::<Result<_, Error>>()?,
        Source::SeabornPalettes { ref url } => {
            categorical(python::seaborn_palettes(fetcher.text(url)?)?)
        }
        Source::Colorcet { ref url } => packages::colorcet(fetcher.text(url)?)?,
        Source::Cmocean {
            ref cm_py,
            rgb_base,
        } => packages::cmocean(fetcher, cm_py, rgb_base)?,
        _ => return Err("not a Python source".into()),
    })
}

/// Plain-text sources (tables, C arrays, TypeScript, CSS, published values).
fn text_source(fetcher: &mut Fetcher, source: &Source) -> Result<Vec<Unit>, Error> {
    Ok(match *source {
        Source::CBytes {
            ref url,
            array,
            name,
            kind,
        } => one(name, kind, text::c_byte_triples(fetcher.text(url)?, array)),
        Source::Table {
            ref url,
            name,
            kind,
        } => one(name, kind, text::numeric_table(fetcher.text(url)?)),
        Source::CartoTs { ref url } => text::carto_ts(fetcher.text(url)?)
            .into_iter()
            .map(|(name, colors, tags)| (name, classify::carto(&tags), colors))
            .collect(),
        Source::CssVars {
            ref url,
            prefix,
            name,
        } => one(
            name,
            ColormapKind::Categorical,
            text::css_vars(fetcher.text(url)?, prefix),
        ),
        Source::Krzywinski { ref url, name } => one(
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
fn structured_source(fetcher: &mut Fetcher, source: &Source) -> Result<Vec<Unit>, Error> {
    Ok(match *source {
        Source::Cmasher {
            ref tree_api,
            raw_base,
        } => packages::cmasher(fetcher, tree_api, raw_base)?,
        Source::CrameriZip { ref url, cache } => {
            packages::crameri(&fetcher.cached_bytes(url, cache)?)?
        }
        Source::ColorBrewer { ref url } => json::colorbrewer(fetcher.text(url)?)?,
        Source::Tol { ref url } => json::tol(fetcher.text(url)?)?,
        Source::Catppuccin { ref url } => json::catppuccin(fetcher.text(url)?)?,
        Source::ColorSchemesJl { ref url, names } => {
            categorical(julia::schemes(fetcher.text(url)?, names)?)
        }
        _ => return Err("not a structured source".into()),
    })
}

fn categorical(maps: Vec<(String, Vec<[f64; 3]>)>) -> Vec<Unit> {
    maps.into_iter()
        .map(|(n, c)| (n, ColormapKind::Categorical, c))
        .collect()
}

/// Palette collections: categorical when short, otherwise classified by lightness.
fn palettes(maps: Vec<(String, Vec<[f64; 3]>)>) -> Vec<Converted> {
    maps.into_iter()
        .map(|(n, c)| {
            let stops = to_rgb8(&c);
            (n, classify::palette_kind(&stops), stops)
        })
        .collect()
}

/// Quantizes unit RGB stops to 8 bits per channel.
fn to_rgb8(colors: &[[f64; 3]]) -> Vec<[u8; 3]> {
    colors
        .iter()
        .map(|c| c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8))
        .collect()
}
