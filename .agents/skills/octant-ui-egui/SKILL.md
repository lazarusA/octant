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
