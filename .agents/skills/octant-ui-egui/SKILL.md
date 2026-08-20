---
name: octant-ui-egui
description: >-
  Immediate-mode GUI development in Octant using egui and eframe. Covers AppAction
  event dispatching, dimension sliders, colormap palettes, and animation timelines.
  Use when modifying src/ui/ or src/app/ui.rs.
---

# Octant egui UI Skill

This skill guides development of the user interface in Octant using `egui` 0.36 and `eframe`.

## Key Patterns

### 1. `AppAction` Event Dispatching (`src/app/actions.rs`)
- Do not mutate complex state deep inside nested UI widget closures.
- Emit an `AppAction` (e.g. `AppAction::SelectVariable(name)`, `AppAction::TogglePlayback`, `AppAction::SetColormap(map)`).
- Handle mutations centrally in `OctantApp::apply_action` or `update` to keep data flow unidirectional and debuggable.

### 2. UI Layout Hierarchy
- **Top Panel**: Menu bar, dataset load/open dialog, store selector, preset catalog.
- **Side Panel (Left)**: Variables inspector, dimension axis mapping (X, Y, Z, Time, Elevation), slice sliders.
- **Central Panel**: WGPU canvas viewport, dynamic aspect ratio framing, hover tooltips with raw scalar values.
- **Bottom Panel**: Animation timeline, playback speed slider, loop toggle, step forward/backward buttons.

### 3. Smooth Animations & Timers
- Track elapsed delta time (`ctx.input(|i| i.stable_dt)`).
- Request continuous repaints only when playing animations or waiting for background prefetch (`ctx.request_repaint()`).
