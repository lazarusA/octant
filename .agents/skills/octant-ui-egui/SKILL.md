---
name: octant-ui-egui
description: >-
  Immediate-mode GUI development in Octant using egui and eframe. Covers AppAction
  event dispatching, dimension sliders, colormap palettes, and animation timelines.
  Use when modifying src/ui/ or src/app/ui.rs.
---

# Octant egui UI Skill

This skill guides development of the user interface in Octant using `egui` and `eframe`.

## Key Patterns

### 1. Modular Subsystem Architecture (`src/ui/`, `src/export/`)
UI and export files are decomposed into focused submodules (< 250 lines per file):
- **Colorbar Subsystem (`src/ui/colorbar/`)**:
  - `ticks.rs`: Scientific tick generation (`generate_colorbar_ticks`), label formatting, and `ColorbarTick`.
  - `handles.rs`: Interactive range inputs (`draw_end_range_inputs`) and clamp triangles (`draw_clip_triangles`).
  - `mod.rs`: Coordinator overlay widget `show_colorbar_overlay`.
- **Variables Panel Subsystem (`src/ui/variables_panel/`)**:
  - `info.rs`: Dataset metadata, summary cards, and chunk shape breakdown (`show_variable_info`).
  - `dimension_slider.rs`: Sliders, range configuration, download sizes, element limits, and slice builders (`show_dimension_sliders`).
  - `mod.rs`: Coordinator widget `show_variable_controls`.
- **Hover Tooltip Subsystem (`src/ui/hover/`)**:
  - `callout.rs`: Leader lines and anchor callout cards.
  - `camera.rs`: 3D view frustum, ray projection, and AABB intersection.
  - `format.rs`: Multi-dimensional coordinate label formatting and dimension enrichment.
  - `raycast_sphere.rs`, `raycast_surface.rs`, `raycast_volume.rs`: Analytical and voxel raymarching hit-tests.
  - `sample_1d.rs`, `sample_2d.rs`: Nearest-neighbor and bilinear data sampling.
- **Export Engine (`src/export/`)**:
  - `raster.rs`: PNG, JPEG, WebP encoding, zero-allocation row-by-row `crop_rgba_buffer`, and Display P3 chunk injection.
  - `vector.rs`: SVG and ISO 32000 PDF document generators.
  - `clipboard.rs`: Platform-native file manager reveal (`open -R`, `xdg-open`, `explorer.exe`).

### 2. `AppAction` Event Dispatching (`src/app/actions.rs`)
- Do not mutate complex state deep inside nested UI widget closures.
- Emit an `AppAction` (e.g. `AppAction::SelectVariable(name)`, `AppAction::TogglePlayback`, `AppAction::SetColormap(map)`).
- Handle mutations centrally in `OctantApp::apply_action` or `update` to keep data flow unidirectional and debuggable.

### 3. Zero-Allocation UI Invariants & Performance
- **Tuple Salts**: Pass tuple literals directly into ID salts to avoid heap strings:
  ```rust
  // Good: Zero heap allocations
  egui::ComboBox::from_id_salt(("spatial_role", dim_idx))
  ui.make_persistent_id(("var_info_header", &var_info.name))

  // Avoid: Allocates a String every single frame
  egui::ComboBox::from_id_salt(format!("spatial_role_{}", dim_idx))
  ```
- **Canvas Ticks (`src/ui/axes.rs`)**:
  - Never allocate `Vec<TickMark>` or `String`s per frame.
  - Use `[TickMark; 7]` stack arrays with an internal fixed stack buffer (`[u8; 32]`) and in-place `write!` cursor formatting.
- **Dimension & Coordinate Checks (`src/utils/coordinates.rs`)**:
  - Use zero-allocation ASCII search (`is_spatial_x_name`, `is_spatial_y_name`, `is_spatial_z_name`, `is_animated_time_name`) without lowercasing or cloning `String`s.
- **Borrow FontId**: Pass `&FontId` to helper rendering functions instead of cloning `FontId`.

### 4. Float Sorting & Safe Comparisons
- Always sort floats with `f32::total_cmp` (`ticks.sort_by(|a, b| a.t_pos.total_cmp(&b.t_pos))`) to prevent panics when encountering `NaN` or unnormalized coordinates.

### 5. UI Layout Hierarchy
- **Top Panel**: Menu bar, dataset load/open dialog, store selector, preset catalog.
- **Side Panel (Left)**: Variables inspector, dimension axis mapping (X, Y, Z, Time, Elevation), slice sliders.
- **Central Panel**: WGPU canvas viewport, dynamic aspect ratio framing, hover tooltips with raw scalar values.
- **Bottom Panel**: Animation timeline, playback speed slider, loop toggle, step forward/backward buttons.

### 6. Smooth Animations & Timers
- Track elapsed delta time (`ctx.input(|i| i.stable_dt)`).
- Request continuous repaints only when playing animations or waiting for background prefetch (`ctx.request_repaint()`).

### 7. Canvas Paint Dispatch & Viewport Math
- In `src/app/ui.rs`, canvas rendering delegates to `self.paint_active_plot(...)` in `src/app/pipeline/paint.rs`.
- Use `crate::utils::apply_zoom_pan_at_point(...)` from `src/utils/math.rs` for cursor-centered zoom and pan offsets.
- Dynamic 2D aspect ratios are resolved using `self.compute_aspect_scale(canvas_rect.size())` and `self.active_data_dimensions_2d()`.

### 8. Figure Export & Clean Capture Architecture
- **Export Standards (`src/export/`)**:
  - PNG with Display P3 color primaries (`cHRM`) and Gamma 2.2 (`gAMA`) chunks without conflicting `sRGB` chunks.
  - ISO 32000 PDF with strictly 20-byte cross-reference (`xref`) table entries (`{:010} 00000 n \r\n`).
  - Expand `~` to `$HOME/Downloads` via `resolve_export_path`.
- **Capture Cleanliness**:
  - Suppress interactive overlays (crop handles, grid lines, hover reticles) during the single screenshot pass (`if self.pending_export.is_none()`).
- **Flash Overlay**:
  - Flash timers start *after* screenshot readback finishes to prevent capturing flash luminance into exported files.

### 9. Theme Awareness & Pro-Grade Overlay UI Patterns
- **Full Theme Adaptation**:
  - All custom canvas overlays (crop guidelines, bounding boxes, toast popups, floating toolbars) must dynamically adapt to `ui.visuals().dark_mode`.
- **Pro-Grade Overlay Handles**:
  - Use geometric bracket shapes (L-shaped corner brackets with $16$ px arms, $3$ px thickness) and thin bars ($28\times 4$ px) along edges.
  - Provide responsive cursor feedback (`ResizeNorthWest`, `ResizeNorthEast`, `ResizeVertical`, `ResizeHorizontal`, `Grab`).
- **Responsive Layout Accounting**:
  - When calculating dynamic widget space (e.g. `slider_w`), subtract all trailing badges, buttons, and menus in `right_elements_w`.

### 10. Native Procedural Vector Icons (`src/ui/icons/`)
- **Forbid Raw Emojis and Unicode Glyphs**: Never use font-dependent emojis or unicode symbols in UI labels, buttons, toasts, status text, or logs.
- **Use `crate::ui::icons::Icon`**:
  ```rust
  use crate::ui::icons::{Icon, UiIconExt};

  // Icon only
  ui.icon(Icon::DropTray, 14.0);
  ui.icon_colored(Icon::Warning, 14.0, warning_color);

  // Buttons with vector icons
  ui.icon_button(Icon::Save, "Save Figure");
  ui.icon_button(Icon::Cross, ""); // Pure icon button

  // Labels with vector icons
  ui.icon_label(Icon::VariableDoc, "Temperature");

  // Direct GPU Painter drawing
  Icon::DropTray.paint(ui.painter(), rect, stroke_color, ui.visuals().dark_mode);
  ```
- **Extending Icons**: Add new variants to `Icon` in `src/ui/icons/mod.rs` and implement procedural painting in `nav.rs`, `playback.rs`, `plots.rs`, `status.rs`, or `store.rs`.
