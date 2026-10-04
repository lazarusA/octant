//! ColorSchemes.jl data files (`loadcolorscheme(:name, [colors...], ...)`).
//! Used only by the Paintings family, whose palettes were created by the
//! ColorSchemes.jl authors and have no other source.

use super::text::hex_rgb;

/// Named unit-RGB palettes.
type Named = Vec<(String, Vec<[f64; 3]>)>;

/// The colors of each requested scheme, in the order of `names`.
pub fn schemes(text: &str, names: &[&str]) -> Result<Named, String> {
    names
        .iter()
        .map(|&name| {
            let colors =
                scheme(text, name).ok_or_else(|| format!("ColorSchemes.jl has no `{name}`"))?;
            Ok((name.to_string(), colors))
        })
        .collect()
}

fn scheme(text: &str, name: &str) -> Option<Vec<[f64; 3]>> {
    let start = text.split("loadcolorscheme(").skip(1).find(|block| {
        let head = block.trim_start().strip_prefix(':').unwrap_or_default();
        head.starts_with(name) && head[name.len()..].trim_start().starts_with(',')
    })?;
    let open = start.find('[')?;
    let body = &start[open + 1..open + start[open..].find(']')?];
    let colors = colors(body);
    (!colors.is_empty()).then_some(colors)
}

/// `RGB(r, g, b)`, `RGB{Float64}(...)`, `Colors.RGB{...}(...)` and `colorant"#rrggbb"`.
fn colors(body: &str) -> Vec<[f64; 3]> {
    let mut out = Vec::new();
    let mut rest = body;
    loop {
        let (rgb, hex) = (rest.find("RGB"), rest.find("colorant\"#"));
        let (at, parsed) = match (rgb, hex) {
            (Some(r), Some(h)) if h < r => (h, colorant(&rest[h..])),
            (Some(r), _) => (r, rgb_call(&rest[r..])),
            (None, Some(h)) => (h, colorant(&rest[h..])),
            (None, None) => break,
        };
        out.extend(parsed);
        rest = &rest[at + 3..];
    }
    out
}

fn rgb_call(s: &str) -> Option<[f64; 3]> {
    let open = s.find('(')?;
    let close = open + s[open..].find(')')?;
    let mut it = s[open + 1..close]
        .split(',')
        .map(|v| v.trim().parse::<f64>().ok());
    Some([it.next()??, it.next()??, it.next()??])
}

fn colorant(s: &str) -> Option<[f64; 3]> {
    hex_rgb(s.get(9..16)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_named_schemes() {
        let jl = "loadcolorscheme(:klimtx, [RGB(9, 9, 9)], \"general\")\n\
                  loadcolorscheme(:klimt, [\n    RGB(0.0, 0.5, 1.0),\n    RGB{Float64}(1.0,0.0,0.0),\n    colorant\"#FF8000\",\n], \"general\", \"The Kiss\")\n";
        let s = schemes(jl, &["klimt"]).unwrap_or_default();
        assert_eq!(s[0].1.len(), 3);
        assert_eq!(s[0].1[2], [1.0, 128.0 / 255.0, 0.0]);
        assert!(schemes(jl, &["missing"]).is_err());
    }
}
