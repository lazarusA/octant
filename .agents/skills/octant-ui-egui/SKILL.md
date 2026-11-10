---
name: octant-ui-egui
description: >-
  Immediate-mode GUI development in Octant using egui and eframe. Covers the AppAction
  layer for voice commands, dimension sliders, colormap palettes, canvas interactions,
  plot controllers, and animation timelines. Use when modifying src/ui/, src/app/canvas/,
  src/app/controllers/, src/app/state/, or src/app/ui.rs.
---

# Octant egui UI Skill

This skill guides development of the user interface in Octant using `egui` and `eframe`.

**Reference**: `ui.md` (every `src/ui/` module), `app.md` (`src/app/`: state, actions, share links, controllers, canvas, pipeline, layers, block loading), `i18n.md` (translations and fonts). Update them with the code they describe.

## Key Patterns

### 1. Modular Subsystem Architecture (`src/ui/`, `src/export/`, `src/app/`)
UI and application files are decomposed into focused submodules (< 250 lines per file):
- **Canvas Engine (`src/app/canvas/`)**:
  - `viewport.rs`: Viewport dimensions, rect allocation, and aspect-ratio uniform calculations (`compute_viewport_uniforms`).
  - `interactions.rs`: User drag, pan, zoom, click, orbit input events (`handle_canvas_interactions`).
  - `axes.rs`: Coordinate grid lines, ticks, labels, and boundary rulers (`draw_canvas_axes`).
  - `overlays.rs`: Interactive canvas crop bounding box, reticles, hover indicator pins (`draw_canvas_overlays`).
  - `mod.rs`: Central viewport entry point `render_canvas(app, ui, canvas_rect)`.
- **Plot Controllers Subsystem (`src/app/controllers/`)**:
  - `traits.rs`: `PlotController`, a plot's canvas interaction (`reset_view`, `handle_drag`, `handle_pan`, `handle_zoom`, `handle_scroll`), what it draws (`draws_coastlines`, `draws_categories`) and the resolution it reads (`resolution_demand`).
  - Controllers: `planar.rs` `HeatmapController` and `LineController`, `orbit.rs` `Orbit3DController` (surface, sphere, volume, point cloud).
  - `mod.rs`: `controller_for(plot_type) -> &'static dyn PlotController` singleton dispatcher.
- **Modular App State (`src/app/state/`)**:
  - `navigation.rs` (`app.nav`): 2D pan/zoom, 3D camera orbit/zoom, `view_interacting`.
  - `playback.rs` (`app.playback`): Animation timer, FPS, looping, `current_timestep`.
  - `plot_configs.rs` (`app.plot_configs`): Plot options, `plot_transparency`, DVR raymarching parameters, line settings.
  - `ui_layout.rs` (`app.layout`): Layout flags (`panel_positions`, `plot_as_overlay`, `reveal_layers_menu`, modal dialogs).
- **Export Lifecycle (`src/app/export_lifecycle.rs`)**:
  - `process_pending_export`: Processes in-flight raster/vector figure captures, ensuring UI overlays and tooltips are suppressed during capture passes.
- **About Modal (`src/ui/about/`)**:
  - `types.rs`: `AboutTab` variants and tab constants.
  - `overview.rs`: Architecture overview and subsystem capabilities.
  - `icons.rs`: Icon gallery preview grid.
  - `mod.rs`: Coordinator modal window `show_about_window`.
- **Dataset Catalog (`src/ui/catalog/`)**:
  - `header.rs`: Modal heading and zero-allocation count badge.
  - `filters.rs`: Search query text edit and category filter chips.
  - `card.rs`: Responsive desktop/mobile store cards.
  - `list.rs`: Scrollable results list and ASCII substring filtering.
  - `mod.rs`: Modal coordinator `show_catalog_window`.
- **Color Picker (`src/ui/color_picker/`)**:
  - `shape.rs`: Geometric triangle, circle, and polygon painter (`ColorShape`).
  - `popup.rs`: Hex/RGB sliders and popup dropdown.
  - `widget.rs`: Clickable color swatch.
  - `mod.rs`: Coordinator `show` / `show_at`.
- **Colorbar Subsystem (`src/ui/colorbar/`)**:
  - `ticks.rs`: Scientific tick generation (`generate_colorbar_ticks`), label formatting, and `ColorbarTick`.
  - `handles.rs`: Interactive range inputs (`draw_end_range_inputs`) and clamp triangles (`draw_clip_triangles`).
  - `mod.rs`: Coordinator overlay widget `show_colorbar_overlay`.
- **Crop Overlay (`src/ui/crop_overlay/`)**:
  - `handles.rs`: Draggable corners, edges, and directional mouse cursors.
  - `toolbar.rs`: Floating action bar (confirm, reset, cancel).
  - `mod.rs`: Canvas bounding box coordinator `show_crop_overlay`.
- **Variables Overlay (`src/ui/variables_overlay/`)**:
  - `search.rs`: Search input bar and clear trigger.
  - `item.rs`: Variable rows with stack-formatted badges and units.
  - `tree.rs`: Recursive hierarchy group folding.
  - `mod.rs`: Floating overlay coordinator `show_variables_overlay`.
- **Variables Panel Subsystem (`src/ui/variables_panel/`)**:
  - `info.rs`: Dataset metadata, summary cards, and chunk shape breakdown (`show_variable_info`).
  - `dimension_slider/`: `defaults.rs` (`defaults_clamp.rs`, `defaults_dggs.rs`, `defaults_spatial.rs`), `composite.rs`, `coord_label.rs`, `double_slider.rs`, `metrics.rs`, `roles.rs`, `role_row.rs`, `slider_row.rs`, `mod.rs`.
  - `mod.rs`: Coordinator widget `show_variable_controls`.
- **Hover Tooltip Subsystem (`src/ui/hover/`)**:
  - `card/`: the hover card and its leader line (`leader.rs`), laid out and painted (`layout.rs`, `place.rs`, `paint.rs`).
  - `camera.rs`: 3D view frustum, ray projection, and AABB intersection.
  - `format.rs`: Multi-dimensional coordinate label formatting and dimension enrichment.
  - `raycast_sphere.rs`, `raycast_surface.rs`, `raycast_volume.rs`: Analytical and voxel raymarching hit-tests.
  - `sample_1d.rs`, `sample_2d.rs`: Nearest-neighbor and bilinear data sampling.
- **Export Engine (`src/export/`)**:
  - `raster.rs`: PNG, JPEG, WebP encoding, zero-allocation row-by-row `crop_rgba_buffer`, and Display P3 chunk injection.
  - `vector.rs`: SVG and ISO 32000 PDF document generators.
  - `clipboard.rs`: Platform-native file manager reveal (`open -R`, `xdg-open`, `explorer.exe`).

### 2. Event Dispatching & Controller Coordination
- **Action layer (`src/app/actions/`)**:
  - Voice and typed commands become `AppAction`s (`SelectVariable { path }`, `Plot { plot_type }`, `Play`, `SetColormap { name }`, ...), names resolved against the loaded dataset, run by `OctantApp::dispatch`, which calls the same methods the UI calls. Panels keep calling those methods directly.
- **Plot-Specific Interactions (`src/app/controllers/`)**:
  - Delegate plot-specific pointer interaction to `controller_for(plot_type)`.

### 3. Zero-Allocation UI Invariants & Performance
- **Tuple Salts**: Pass tuple literals directly into ID salts to avoid heap strings:
  ```rust
  // Good: Zero heap allocations
  egui::ComboBox::from_id_salt(("spatial_role", dim_idx))
  ui.make_persistent_id(("var_info_header", &var_info.name))

  // Avoid: Allocates a String every single frame
  egui::ComboBox::from_id_salt(format!("spatial_role_{}", dim_idx))
  ```
- **Canvas Ticks (`src/ui/axes/`, `src/app/canvas/axes.rs`)**:
  - Never allocate `Vec<TickMark>` or `String`s per frame.
  - Use stack arrays with an internal fixed stack buffer (`[u8; 32]`) and in-place `write!` cursor formatting.
- **Dimension & Coordinate Checks (`src/data/coordinates/naming.rs`)**:
  - Use zero-allocation name checks (`is_lat_name`, `is_lon_name`, `is_spatial_x_name`, `is_spatial_y_name`, `is_spatial_z_name`, `is_animated_time_name`) without lowercasing or cloning `String`s.
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
- Request continuous repaints only when playing animations (`app.playback.is_playing`) or waiting for background prefetch (`ctx.request_repaint()`).

### 7. Canvas Paint Dispatch & Viewport Math
- In `src/app/ui.rs`, canvas rendering delegates to `crate::app::canvas::render_canvas(self, ui, canvas_rect)`, which coordinates viewport allocation, pointer interactions, background rendering via `paint_active_plot(...)` in `src/app/pipeline/paint.rs`, axes, overlays, and colorbars.
- Use `crate::utils::apply_zoom_pan_at_point(...)` from `src/utils/math.rs` for cursor-centered zoom and pan offsets.
- Dynamic 2D aspect ratios are resolved using `self.compute_aspect_scale(canvas_rect.size())` and `self.active_data_dimensions_2d()`.

### 8. Figure Export & Clean Capture Architecture
- **Export Standards (`src/export/`)**:
  - PNG with Display P3 color primaries (`cHRM`) and Gamma 2.2 (`gAMA`) chunks without conflicting `sRGB` chunks.
  - ISO 32000 PDF with strictly 20-byte cross-reference (`xref`) table entries (`{:010} 00000 n \r\n`).
  - Expand `~` to `$HOME/Downloads` via `resolve_export_path`.
- **Capture Cleanliness**:
  - Suppress interactive overlays (crop handles, grid lines, hover reticles) during the single screenshot pass (`if app.pending_export.is_none()`).
  - Managed centrally in `src/app/export_lifecycle.rs` (`self.process_pending_export(&ctx)`).
- **Flash Overlay**:
  - Flash timers start *after* screenshot readback finishes to prevent capturing flash luminance into exported files.

### 9. Theme Awareness & Pro-Grade Overlay UI Patterns
- **Full Theme Adaptation**:
  - All custom canvas overlays (crop guidelines, bounding boxes, toast popups, floating toolbars) must dynamically adapt to `ui.visuals().dark_mode`.
- **Pro-Grade Overlay Handles**:
  - Use geometric bracket shapes (L-shaped corner brackets with $16$ px arms, $3$ px thickness) and thin bars ($28\times 4$ px) along edges.
  - Provide responsive cursor feedback (`ResizeNorthWest`, `ResizeNorthEast`, `ResizeVertical`, `ResizeHorizontal`, `Grab`).
- **Responsive Layout Accounting**:
  - Rows that would widen their panel stack instead (`src/ui/flow_row/`): measure with its cached widths, never per frame.

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
