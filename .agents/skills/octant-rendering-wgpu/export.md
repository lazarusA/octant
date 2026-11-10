# Export (`src/export/`)

Reference for this skill, moved from `AGENTS.md` (which keeps the rules most tasks need); update it with the code it describes.

- **Export Engine (`src/export/`)**:
  - Submodules: `raster.rs` (PNG, JPEG, WebP, Display P3 chunk injection), `vector.rs` (SVG, PDF), `clipboard.rs` (native file manager reveal & clipboard), `download.rs` (web only: `save_exported_file` hands the file to the browser as a download, since the web has no filesystem).
