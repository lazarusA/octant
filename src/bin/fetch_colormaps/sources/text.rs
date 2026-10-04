//! Plain-text upstream formats: numeric tables, hex lists, C arrays, CSS
//! variables, R lists, YAML lists and CARTOColors `carto.ts`.

/// Parses `#rrggbb` (alpha suffixes ignored) to unit RGB.
pub fn hex_rgb(s: &str) -> Option<[f64; 3]> {
    let hex = s.trim().strip_prefix('#')?;
    let ch = |i: usize| -> Option<f64> {
        let v = u8::from_str_radix(hex.get(i..i + 2)?, 16).ok()?;
        Some(f64::from(v) / 255.0)
    };
    Some([ch(0)?, ch(2)?, ch(4)?])
}

/// `rgb(r,g,b)` with 0–255 components.
pub fn css_rgb(s: &str) -> Option<[f64; 3]> {
    let inner = s.trim().strip_prefix("rgb(")?.strip_suffix(')')?;
    let mut it = inner
        .split(',')
        .map(|v| v.trim().parse::<f64>().ok().map(|v| v / 255.0));
    Some([it.next()??, it.next()??, it.next()??])
}

/// Rows of numbers separated by commas or whitespace; the last three columns are
/// RGB. Non-numeric rows (headers, comments) are skipped. Values above 1 mean 0–255.
pub fn numeric_table(text: &str) -> Vec<[f64; 3]> {
    let rows: Vec<Vec<f64>> = text
        .lines()
        .filter_map(|line| {
            let vals: Result<Vec<f64>, _> = line
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|v| !v.is_empty())
                .map(str::parse::<f64>)
                .collect();
            vals.ok().filter(|v| v.len() >= 3)
        })
        .collect();
    let scale = if rows.iter().flatten().any(|&v| v > 1.0) {
        255.0
    } else {
        1.0
    };
    rows.iter()
        .map(|r| {
            let n = r.len();
            [r[n - 3] / scale, r[n - 2] / scale, r[n - 1] / scale]
        })
        .collect()
}

/// Krzywinski palette files: `color  N-main HEX r g b names...` rows, `-main` only.
pub fn krzywinski(text: &str) -> Vec<[f64; 3]> {
    text.lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            (fields.next()? == "color" && fields.next()?.ends_with("-main"))
                .then(|| hex_rgb(&format!("#{}", fields.next()?)))?
        })
        .collect()
}

/// `unsigned char <array>[256][3] = {{r,g,b},...};` in a C source.
pub fn c_byte_triples(text: &str, array: &str) -> Vec<[f64; 3]> {
    let Some(start) = text.find(array) else {
        return Vec::new();
    };
    let body = text[start..].split(';').next().unwrap_or_default();
    body.split('{')
        .filter_map(|chunk| {
            let nums: Vec<f64> = chunk
                .split(['}', ','])
                .filter_map(|v| v.trim().parse::<f64>().ok())
                .take(3)
                .collect();
            (nums.len() == 3).then(|| [nums[0] / 255.0, nums[1] / 255.0, nums[2] / 255.0])
        })
        .collect()
}

/// CSS custom properties `--<prefix>N: #hex;`, ordered by N.
pub fn css_vars(text: &str, prefix: &str) -> Vec<[f64; 3]> {
    let mut vars: Vec<(u32, [f64; 3])> = text
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("--")?.strip_prefix(prefix)?;
            let (n, value) = rest.split_once(':')?;
            Some((
                n.trim().parse().ok()?,
                hex_rgb(value.trim().trim_end_matches(';'))?,
            ))
        })
        .collect();
    vars.sort_by_key(|(n, _)| *n);
    vars.dedup_by_key(|(n, _)| *n);
    vars.into_iter().map(|(_, c)| c).collect()
}

/// Top-level entries of an R `<list> <- list(name = c("#..", ...), ...)`; for
/// `list(c(...), ...)` or `rbind(c(...), ...)` entries the first `c(...)` is used.
pub fn r_list(text: &str, list: &str) -> Vec<(String, Vec<[f64; 3]>)> {
    let Some(start) = text.find(&format!("{list} <- list(")) else {
        return Vec::new();
    };
    let body = &text[start + list.len() + 9..];
    let mut out = Vec::new();
    let (mut depth, mut item_start) = (0i32, 0usize);
    for (i, c) in body.char_indices() {
        match c {
            '(' => depth += 1,
            ')' if depth == 0 => {
                out.extend(r_entry(&body[item_start..i]));
                break;
            }
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.extend(r_entry(&body[item_start..i]));
                item_start = i + 1;
            }
            _ => {}
        }
    }
    out
}

fn r_entry(item: &str) -> Option<(String, Vec<[f64; 3]>)> {
    let (name, expr) = item.split_once('=')?;
    let name = name
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<String>();
    let c = expr.find("c(")?;
    let list = &expr[c + 2..c + 2 + expr[c + 2..].find(')')?];
    let colors: Vec<[f64; 3]> = list
        .split(',')
        .filter_map(|v| hex_rgb(v.trim().trim_matches(['"', '\''])))
        .collect();
    (!colors.is_empty()).then(|| (name.trim().to_string(), colors))
}

/// YAML `Name:` keys followed by `- '#hex'` items.
pub fn yaml_hex_lists(text: &str) -> Vec<(String, Vec<[f64; 3]>)> {
    let mut out: Vec<(String, Vec<[f64; 3]>)> = Vec::new();
    for line in text.lines() {
        if let Some(item) = line.trim().strip_prefix('-') {
            if let (Some(last), Some(c)) = (
                out.last_mut(),
                hex_rgb(item.trim().trim_matches(['"', '\''])),
            ) {
                last.1.push(c);
            }
        } else if let Some(key) = line.strip_suffix(':').filter(|k| !k.starts_with(' ')) {
            out.push((key.trim().to_string(), Vec::new()));
        }
    }
    out
}

/// CARTOColors `export const Name = { 2: [...], ..., tags: [...] };`: largest class and tags.
pub fn carto_ts(text: &str) -> Vec<(String, Vec<[f64; 3]>, Vec<String>)> {
    let quoted = |s: &str| {
        s.split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .collect::<Vec<_>>()
    };
    text.split("export const ")
        .skip(1)
        .filter_map(|block| {
            let (name, body) = block.split_once('=')?;
            let body = body.split("};").next()?;
            let mut best: Option<(u32, Vec<String>)> = None;
            let mut tags = Vec::new();
            for part in body.split(']').filter(|p| p.contains('[')) {
                let (head, list) = part.split_once('[')?;
                let key = head
                    .rsplit([',', '{', '\n'])
                    .next()?
                    .trim()
                    .trim_end_matches(':')
                    .trim();
                if key == "tags" {
                    tags = quoted(list);
                } else if let Ok(n) = key.parse::<u32>()
                    && best.as_ref().is_none_or(|(b, _)| n > *b)
                {
                    best = Some((n, quoted(list)));
                }
            }
            let colors: Vec<[f64; 3]> = best?.1.iter().filter_map(|h| hex_rgb(h)).collect();
            (colors.len() >= 2).then(|| (name.trim().to_string(), colors, tags))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_and_byte_values() {
        let csv = "scalar,RGB_r,RGB_g,RGB_b\n0.0,0,0,0\n1.0,255,128,0\n";
        assert_eq!(numeric_table(csv)[1], [1.0, 128.0 / 255.0, 0.0]);
        assert_eq!(numeric_table("0.5 0.25 1\n")[0], [0.5, 0.25, 1.0]);
        let c = "float x[2][3] = {{0.1,0.2,0.3}};\nunsigned char t_bytes[2][3] = {{255,0,0},{0,0,255}};";
        assert_eq!(
            c_byte_triples(c, "t_bytes"),
            vec![[1.0, 0.0, 0.0], [0.0, 0.0, 1.0]]
        );
    }

    #[test]
    fn palettes_from_text_formats() {
        let mk = "# header\ncolor  1-main 000000   0 0 0 black\ncolor  2-alt  AA0DB4 1 2 3 x\ncolor  2-main FFFFFF 1 1 1 white\n";
        assert_eq!(krzywinski(mk), vec![[0.0; 3], [1.0; 3]]);
        let css = ":root {\n  --nord1: #ffffff;\n  --nord0: #000000;\n}\n.x { --nord0: #000000; }";
        assert_eq!(css_vars(css, "nord"), vec![[0.0; 3], [1.0; 3]]);
        let r = "x <- list(\n  A = c(\"#000000\", \"#FFFFFF\"),\n  B=list(c('#ff0000'), c(1), colorblind=TRUE),\n  C = rbind(c('#00ff00'),c(1))\n)\n";
        let l = r_list(r, "x");
        assert_eq!(
            l.iter()
                .map(|(n, c)| (n.as_str(), c.len()))
                .collect::<Vec<_>>(),
            vec![("A", 2), ("B", 1), ("C", 1)]
        );
        let yaml = "A:\n- '#000000'\n- '#FFFFFF'\nB:\n- '#FF0000'\n";
        assert_eq!(yaml_hex_lists(yaml)[0].1.len(), 2);
        assert_eq!(css_rgb("rgb(255,0,0)"), Some([1.0, 0.0, 0.0]));
    }

    #[test]
    fn carto_takes_largest_class_and_tags() {
        let ts = "export const Burg = {\n  2: [\"#000000\", \"#ffffff\"],\n  3: [\"#000000\", \"#808080\", \"#ffffff\"],\n  tags: [\"quantitative\"],\n};\n";
        let s = carto_ts(ts);
        assert_eq!(s[0].0, "Burg");
        assert_eq!(s[0].1.len(), 3);
        assert_eq!(s[0].2, vec!["quantitative"]);
    }
}
