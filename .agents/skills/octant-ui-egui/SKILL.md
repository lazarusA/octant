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

### 1. `AppAction` Event Dispatching (`src/app/actions.rs`)
- Do not mutate complex state deep inside nested UI widget closures.
- Emit an `AppAction` (e.g. `AppAction::SelectVariable(name)`, `AppAction::TogglePlayback`, `AppAction::SetColormap(map)`).
- Handle mutations centrally in `OctantApp::apply_action` or `update` to keep data flow unidirectional and debuggable.

### 2. Zero-Allocation UI Salts & Performance (`perf-collect-once`)
- Avoid allocating heap strings via `format!(...)` inside per-frame UI closures (running 60+ FPS).
- Pass tuple literals directly into ID salts:
  ```rust
  // Good: Zero heap allocations
  egui::ComboBox::from_id_salt(("spatial_role", dim_idx))
  ui.make_persistent_id(("var_info_header", &var_info.name))

  // Avoid: Allocates a String every single frame
  egui::ComboBox::from_id_salt(format!("spatial_role_{}", dim_idx))
  ```
- Use `&str` references directly for combo box and button labels instead of `.to_string()`.

### 3. Float Sorting & Safe Comparisons (`num-nan-inf-checks`)
- Always sort floats with `f32::total_cmp` (`ticks.sort_by(|a, b| a.t_pos.total_cmp(&b.t_pos))`) to prevent panics when encountering `NaN` or unnormalized coordinates.

### 4. UI Layout Hierarchy
- **Top Panel**: Menu bar, dataset load/open dialog, store selector, preset catalog.
- **Side Panel (Left)**: Variables inspector, dimension axis mapping (X, Y, Z, Time, Elevation), slice sliders.
- **Central Panel**: WGPU canvas viewport, dynamic aspect ratio framing, hover tooltips with raw scalar values.
- **Bottom Panel**: Animation timeline, playback speed slider, loop toggle, step forward/backward buttons.

### 5. Smooth Animations & Timers
- Track elapsed delta time (`ctx.input(|i| i.stable_dt)`).
- Request continuous repaints only when playing animations or waiting for background prefetch (`ctx.request_repaint()`).

### 6. Canvas Paint Dispatch & Viewport Math
- In `src/app/ui.rs`, canvas rendering delegates to `self.paint_active_plot(ui, canvas_rect, plot_rect, gpu_pan, gpu_zoom, gpu_aspect_scale)` in `src/app/pipeline.rs`.
- Use `crate::utils::apply_zoom_pan_at_point(old_zoom, old_pan, mouse_pos, center, scroll, min_zoom, max_zoom)` from `src/utils/math.rs` for cursor-centered zoom and pan offsets.
- Dynamic 2D aspect ratios are resolved using `self.compute_aspect_scale(canvas_rect.size())` and `self.active_data_dimensions_2d()`.

### 7. Figure Export, Canvas Screenshot & Clean Capture Architecture
- **Export Formats & Standards** (`src/export/mod.rs`):
  - Supported formats: PNG, JPEG, WebP, SVG, PDF, and System Clipboard (`arboard`).
  - **Color Accuracy & Gamut**:
    - Always inject Display P3 color primaries (`cHRM`) and Gamma 2.2 (`gAMA`) chunks in PNG outputs without conflicting `sRGB` chunks to preserve wide-gamut contrast across macOS Preview / Safari / Photoshop.
  - **ISO 32000 PDF Specification Compliance**:
    - Must include a Root `/Type /Catalog` object referencing `/Type /Pages`.
    - Every entry in the cross-reference (`xref`) table must be strictly 20 bytes long (`{:010} 00000 n \r\n`).
  - Default export directory: `~/Downloads` (always expand tilde `~` to `$HOME/Downloads` via `resolve_export_path`).
- **Clean Single-Frame Viewport Capture (`src/app/ui.rs`)**:
  - Screenshot readback is triggered via `ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()))`.
  - The resulting image is delivered via `egui::Event::Screenshot { image, .. }` in `ctx.input(...)`.
  - **Capture Cleanliness**: Interactive UI overlays (crop handles, rule-of-thirds grid lines, dashed borders, hover reticle tooltips) must be suppressed *only* during the single GPU render pass of the screenshot request (`if self.pending_export.is_none()`), ensuring pure scientific visualization data without UI widgets or handles.
- **ROI Crop & Buffers**:
  - `crop_rgba_buffer(&rgba, full_w, full_h, crop_rect)` slices raw RGBA buffers based on physical pixel bounds converted via `ctx.pixels_per_point()`.
- **Camera Flash Shutter Effect**:
  - Flash timers (`export_flash_timer`) must start **after** screenshot readback finishes so flash luminance is never captured into saved image files.
  - Flash overlays should be rendered on the topmost visual layer (after crop overlay) and confined to the active ROI box with continuous `ctx.request_repaint()` during decay ($\sim 300\text{ms}$).
- **Toast Notifications & File Manager Reveal**:
  - Display floating non-blocking cards (`show_export_toast`) with an action button invoking `reveal_in_file_manager(path)` (`open -R` on macOS, `explorer /select` on Windows, `xdg-open` on Linux).

### 8. Theme Awareness & Pro-Grade Overlay UI Patterns
- **Full Theme Adaptation**:
  - All custom canvas overlays (such as crop guidelines, bounding boxes, toast popups, and floating toolbars) must dynamically adapt to `ui.visuals().dark_mode` or `ui.style().visuals`.
  - Adapt background fills, outer mask alpha (`from_black_alpha(150)` in dark mode vs `from_black_alpha(80)` in light mode), grid strokes, and border accent colors (`0, 190, 255` dark vs `0, 125, 220` light).
  - Floating toolbars must use `egui::Frame::window(ui.style())` with `corner_radius(6.0)` to match the current theme palette.
- **Pro-Grade Overlay Handles & Interaction**:
  - Use geometric bracket shapes (e.g. L-shaped corner brackets with $16$ px arms, $3$ px thickness) and elongated thin pills/bars ($28\times 4$ px) along edges rather than circular dots.
  - Render handles using clean loops over coordinate offsets $(dx, dy)$ to avoid code duplication.
  - Provide responsive cursor changes (`ResizeNorthWest`, `ResizeNorthEast`, `ResizeVertical`, `ResizeHorizontal`, `Grab`).
- **Responsive Layout & Slider Width Accounting**:
  - When calculating dynamically expanding widget space in horizontal bars (e.g., timeline slider `slider_w`), all trailing elements (badges, FPS buttons, Save/Export buttons, overflow menus) must be included in `right_elements_w`:
    ```rust
    let right_elements_w = ... + (if show_export { export_w + spacing } else { 0.0 }) + ...;
    let slider_w = (ui.available_width() - right_elements_w - spacing * 2.0).max(min_slider_w);
    ```
- **Encapsulated UI Helper Pattern**:
  - Extract complex multi-step frame lifecycle actions (e.g. screenshot polling, buffer cropping, format dispatch) into private helper methods (`self.process_pending_export(&ctx)`) on `OctantApp` rather than inlining large blocks into the main `ui()` loop.
