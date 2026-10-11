//! # subset_fonts
//!
//! Regenerates the bundled Chinese, Japanese and Korean fonts in
//! `assets/fonts/`: reads Noto Sans SC, JP and KR from the noto-cjk project
//! (pinned to a commit and verified by MD5 in `sources.rs`) and cuts each one
//! down to the characters Octant's translations use (`*-ui.otf`) and to its
//! language's own name (`*-names.otf`). Fails when a translation uses a
//! character its font lacks. Run it after changing a CJK translation:
//!
//! ```sh
//! cargo run --features font-tools --bin subset_fonts
//! ```

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
mod download;
#[cfg(not(target_arch = "wasm32"))]
mod sources;
#[cfg(not(target_arch = "wasm32"))]
mod subset;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run() -> Result<(), download::Error> {
    use octant::ui::fonts::CjkFont;
    use std::path::Path;

    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    std::fs::create_dir_all(&out)?;
    let downloader = download::Downloader::new()?;
    let mut copyright = String::new();
    for font in CjkFont::ALL {
        let source = sources::font(font);
        let bytes = downloader.cached(&source.remote, source.file_name)?;
        if copyright.is_empty() {
            copyright = subset::copyright(&bytes)?;
        }
        let ui_chars = font.required_chars();
        let names: std::collections::BTreeSet<char> =
            font.language().display_name().chars().collect();
        let ui = subset::subset(&bytes, &ui_chars)?;
        let names_font = subset::subset(&bytes, &names)?;
        std::fs::write(out.join(font.ui_file()), &ui)?;
        std::fs::write(out.join(font.names_file()), &names_font)?;
        println!(
            "{}: {} characters, {} bytes; names {} bytes",
            source.file_name,
            ui_chars.len(),
            ui.len(),
            names_font.len()
        );
    }
    let license = downloader.cached(&sources::LICENSE, "noto-cjk-LICENSE")?;
    let license = String::from_utf8(license)?;
    std::fs::write(
        out.join("LICENSE.md"),
        sources::license_notice(&copyright, &license),
    )?;
    println!("wrote {}", out.display());
    Ok(())
}
