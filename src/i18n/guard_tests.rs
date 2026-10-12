//! Guards against new untranslated UI text: scans the UI sources for English
//! string literals handed straight to egui text calls or toasts, which should
//! go through `tr` instead.

use std::path::{Path, PathBuf};

/// Calls whose first string literal is text shown to the user.
const TEXT_CALLS: &[&str] = &[
    ".label(",
    ".small(",
    ".heading(",
    ".button(",
    ".on_hover_text(",
    ".on_disabled_hover_text(",
    ".hover(",
    ".tooltip(",
    ".text(",
    ".hint_text(",
    "checkbox(",
    "selectable_label(",
    "selectable_value(",
    "radio_value(",
    "toggle_value(",
    "collapsing(",
    "menu_button(",
    "close_button(",
    "icon_button(",
    "hyperlink_to(",
    "CollapsingHeader::new(",
    "Window::new(",
    "ToolbarButton::new(",
    "RichText::new(",
    "notify(",
    "Notice::new(",
    "report(",
    "section(",
];

/// Literals that read as words but are the same in every language.
const ALLOWED: &[&str] = &[
    "Octant",
    "@lazarusA",
    "github.com/lazarusA/octant",
    "github.com/lazarusA/octant/issues",
    "Zarr",
    "Icechunk",
    "NetCDF",
    "GeoTIFF",
    "HEALPix",
    "CMYK",
    "Oklab",
    "sRGB",
    "CIE Lab",
    "B-spline",
    "Catmull-Rom",
    "Symlog",
];

/// Source trees holding UI text.
const ROOTS: &[&str] = &["src/ui", "src/app", "src/export", "src/utils/units"];

#[test]
fn ui_text_goes_through_translations() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for root in ROOTS {
        collect_sources(&manifest.join(root), &mut files);
    }
    let mut found = Vec::new();
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if let Some(literal) = untranslated_literal(line, lines.get(i + 1).copied()) {
                let path = file.strip_prefix(manifest).unwrap_or(file);
                found.push(format!("{}:{}: \"{literal}\"", path.display(), i + 1));
            }
        }
    }
    assert!(
        found.is_empty(),
        "UI text not routed through i18n::tr (add a key, or ALLOWED if it is the same in every language):\n{}",
        found.join("\n")
    );
}

fn collect_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_sources(&path, out);
        } else if is_checked_source(&path) {
            out.push(path);
        }
    }
}

/// Rust sources that ship UI text: not tests, test helpers or review sheets.
fn is_checked_source(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    name.ends_with(".rs")
        && !name.ends_with("tests.rs")
        && !name.ends_with("sheet.rs")
        && !name.starts_with("test_")
}

/// The English literal `line` passes to a text call, if any. A call whose
/// arguments start on the next line is read from `next`.
fn untranslated_literal<'a>(line: &'a str, next: Option<&'a str>) -> Option<&'a str> {
    let code = line.trim_start();
    if code.starts_with("//") || code.contains("log::") {
        return None;
    }
    for call in TEXT_CALLS {
        let Some(at) = line.find(call) else {
            continue;
        };
        let rest = &line[at + call.len()..];
        let args = if rest.trim().is_empty() { next? } else { rest };
        if let Some(literal) = first_literal(args).filter(|l| reads_as_words(l)) {
            return Some(literal);
        }
    }
    None
}

/// The first string literal in `text`, when the call's first argument is one
/// (or follows a leading icon or value argument on the same line).
fn first_literal(text: &str) -> Option<&str> {
    let start = text.find('"')? + 1;
    let len = text[start..].find('"')?;
    Some(&text[start..start + len])
}

/// Whether `literal` holds a word (three or more letters, mostly lowercase)
/// that a translator would translate.
fn reads_as_words(literal: &str) -> bool {
    if ALLOWED.contains(&literal) || literal.contains("://") || literal.contains('_') {
        return false;
    }
    literal
        .split(|c: char| !c.is_ascii_alphabetic())
        .any(|word| word.len() >= 3 && word.chars().filter(char::is_ascii_lowercase).count() >= 2)
}

#[test]
fn guard_flags_english_and_skips_translated_text() {
    assert_eq!(
        untranslated_literal(r#"    ui.label("Hello world");"#, None),
        Some("Hello world")
    );
    assert_eq!(
        untranslated_literal("    ui.label(", Some(r#"        "Next line","#)),
        Some("Next line")
    );
    assert_eq!(
        untranslated_literal("    ui.label(tr(CommonKey::Close));", None),
        None
    );
    assert_eq!(untranslated_literal(r#"    ui.label("R:");"#, None), None);
    assert_eq!(untranslated_literal(r#"    ui.label("Zarr");"#, None), None);
    assert_eq!(
        untranslated_literal(r#"    // ui.label("Hello world");"#, None),
        None
    );
}

/// Widgets that show and read numbers; each must take `.localized()`
/// (`crate::ui::number_input`) so a typed decimal comma parses.
const NUMBER_WIDGETS: &[&str] = &["DragValue::new(", "Slider::new("];

#[test]
fn number_widgets_read_localized_input() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_sources(&manifest.join("src/ui"), &mut files);
    collect_sources(&manifest.join("src/app"), &mut files);
    let mut found = Vec::new();
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") || !NUMBER_WIDGETS.iter().any(|w| line.contains(w)) {
                continue;
            }
            if !statement_from(&lines[i..]).contains(".localized()") {
                let path = file.strip_prefix(manifest).unwrap_or(file);
                found.push(format!("{}:{}", path.display(), i + 1));
            }
        }
    }
    assert!(
        found.is_empty(),
        "number widgets without .localized() (a typed decimal comma would not parse):\n{}",
        found.join("\n")
    );
}

/// The lines of the statement starting at `lines[0]`, up to the first one
/// ending it with `;`, at most twelve.
fn statement_from(lines: &[&str]) -> String {
    let mut statement = String::new();
    for line in lines.iter().take(12) {
        statement.push_str(line);
        if line.trim_end().ends_with(';') {
            break;
        }
    }
    statement
}
