//! The share link of what is plotted: the base layer's store, variable,
//! roles, dimension values, plot type, colors, view or camera, and its
//! overlays.

use std::fmt::Write;

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

use super::{KEYS, SCHEMES};
use crate::app::actions::{plot_type_id, value_text};
use crate::app::layers::LevelChoice;
use crate::app::multiscale::at_base;
use crate::app::{AnimationRole, OctantApp, SpatialRole, VariableSelection};
use crate::plots::PlotType;

/// Escaped in a key or value: what ends or splits one; `/` and `:` stay
/// readable.
const ESCAPED: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'&')
    .add(b'+')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'`');

/// The deployed web app, the base of links made on the desktop.
const WEB_APP: &str = "https://ndvis.github.io/octant/";

impl OctantApp {
    /// Whether the plot has a share link: something plotted from a store a
    /// link can open (never a local one). Allocates nothing, for the UI.
    pub fn can_share(&self) -> bool {
        let sel = self.plotted();
        sel.variable_info().is_some() && SCHEMES.iter().any(|p| sel.store_target.starts_with(p))
    }

    /// The share link of the plot; `None` when `can_share` is false.
    pub fn share_link(&self) -> Option<String> {
        if !self.can_share() {
            return None;
        }
        Some(format!("{}?{}", base(), self.share_query()?))
    }

    /// A multiscale plot names level 0 with level-0 indices: its coarser
    /// levels are no variables to select, and the opened link picks the
    /// level for its own screen: from the one shown (`auto:n`), or the one
    /// pinned (`n`).
    fn share_query(&self) -> Option<String> {
        let base = at_base(self.plotted());
        let sel = base.as_ref().unwrap_or(self.plotted());
        let var = sel.variable_info()?;
        let mut q = String::new();
        pair(&mut q, "src", &sel.store_target);
        pair(&mut q, "var", var.name.trim_matches('/'));
        if let Some(level) = self.level_value() {
            pair(&mut q, "multiscale_level", &level);
        }
        let roles = [SpatialRole::X, SpatialRole::Y, SpatialRole::Z];
        let axes: Vec<&str> = roles
            .iter()
            .filter_map(|role| {
                let i = sel.dim_config.iter().position(|c| c.spatial == *role)?;
                var.dimension_names.get(i).map(String::as_str)
            })
            .collect();
        pair(&mut q, "axes", &axes.join(","));
        self.dimension_pairs(sel, &mut q);
        pair(&mut q, "plot", plot_type_id(sel.plot_type));
        let style = &self.layers.base.color;
        let cmap = crate::utils::colormap::registry::key_of(style.colormap);
        pair(&mut q, "cmap", &cmap);
        self.composite_pairs(sel, &mut q);
        self.style_pairs(&mut q);
        if let Some(view) = self.view_value(sel) {
            pair(&mut q, "view", &view);
        }
        if let Some(camera) = self.camera_value(sel.plot_type) {
            pair(&mut q, "camera", &camera);
        }
        for overlay in self.layers.overlays() {
            let base = at_base(overlay.selection());
            let sel = base.as_ref().unwrap_or(overlay.selection());
            let name = sel.variable_info().map_or("", |v| v.name.as_str());
            pair(&mut q, "overlay", name.trim_matches('/'));
        }
        Some(q)
    }

    /// A 3D plot's camera as drawn, `pitch,yaw,distance[,tx,ty,tz]`
    /// (degrees, the yaw within [-180, 180); the target unless at the center,
    /// never for the sphere, which looks at its center). Written for every
    /// 3D plot, the default view too: nothing else moves the receiver's
    /// camera back from where it was.
    fn camera_value(&self, plot_type: PlotType) -> Option<String> {
        if !plot_type.is_3d() {
            return None;
        }
        let camera = self.nav.orbit_camera(plot_type, 1.0);
        // Rounded first, so a panning residue writes no `,0,0,0`.
        let target = Some(camera.target.map(|t| round(t, 2))).filter(|t| *t != [0.0; 3]);
        // Rounded first, so 179.96 degrees wraps to -180, never 180.
        let yaw = round(camera.rotation_y.to_degrees(), 1);
        let yaw = yaw - 360.0 * ((yaw + 180.0) / 360.0).floor();
        let mut text = format!(
            "{},{},{}",
            round(camera.rotation_x.to_degrees(), 1),
            round(yaw, 1),
            // Three decimals: the sphere comes as close as 1.001 (~6 km).
            round(camera.distance, 3)
        );
        for t in target.iter().flatten() {
            let _ = write!(text, ",{t}");
        }
        Some(text)
    }

    /// The level the plot reads: `n` when pinned, `auto:n` on Auto (the
    /// level shown, for the receiver to start from); `None` off a pyramid.
    fn level_value(&self) -> Option<String> {
        let plotted = self.plotted();
        let shown = plotted.variable_info()?.multiscale.as_ref()?.level;
        Some(match plotted.level {
            LevelChoice::Pinned(level) => level.to_string(),
            LevelChoice::Auto | LevelChoice::AutoFrom(_) => format!("auto:{shown}"),
        })
    }

    /// Each dimension's `name=value` as its slider holds it (a range while
    /// expanded, else its index; every dimension, as the receiver's defaults
    /// may differ), and the animated one as `anim` with the plot's `step`.
    fn dimension_pairs(&self, sel: &VariableSelection, q: &mut String) {
        let (Some(meta), Some(var)) = (sel.metadata.as_ref(), sel.variable_info()) else {
            return;
        };
        let store = sel.store_target.as_str();
        for (i, config) in sel.dim_config.iter().enumerate() {
            let Some(name) = var.dimension_names.get(i) else {
                continue;
            };
            let text = |idx, last| value_text(meta, var, store, i, (idx, last));
            let animated = config.animation == AnimationRole::Animated;
            if animated {
                pair(q, "anim", name);
                pair(q, "step", &text(self.playback.current_timestep, false));
            }
            let value = if !config.active {
                if animated {
                    continue; // its index is the step
                }
                text(config.index, false)
            } else if config.range.0 == config.range.1 {
                text(config.range.0, false)
            } else {
                format!(
                    "{}..{}",
                    text(config.range.0, false),
                    text(config.range.1, true)
                )
            };
            let key = if KEYS.contains(&name.as_str()) {
                format!("dim.{name}")
            } else {
                name.clone()
            };
            pair(q, &key, &value);
        }
    }

    /// The heatmap's visible window as `x0..x1,y0..y1` on `sel` (level 0):
    /// the cells it touches, outward; `None` unless zoomed or panned.
    fn view_value(&self, sel: &VariableSelection) -> Option<String> {
        let nav = &self.nav;
        let moved = nav.heatmap_zoom != 1.0 || nav.heatmap_pan != egui::Vec2::ZERO;
        if !moved || !self.heatmap_evenly_drawn() {
            return None;
        }
        let (meta, var) = (sel.metadata.as_ref()?, sel.variable_info()?);
        let store = sel.store_target.as_str();
        let uv = [nav.visible_uv.0, nav.visible_uv.1];
        let mut ends = Vec::with_capacity(2);
        for (axis, t) in self.plotted_axes()?.iter().zip(uv) {
            let (lo, hi) = axis.visible(t);
            // A coarse level's last cell may end past an odd level 0.
            let end = var.shape.get(axis.dim).map_or(1, |&n| n.max(1) as usize) - 1;
            let first = (lo.floor().max(0.0) as usize).min(end);
            let last = (hi.ceil() as usize).saturating_sub(1).clamp(first, end);
            let text = |idx, end| value_text(meta, var, store, axis.dim, (idx, end));
            ends.push(format!("{}..{}", text(first, false), text(last, true)));
        }
        Some(ends.join(","))
    }
}

/// Appends `key=value` (none for an empty value), escaped.
pub(super) fn pair(query: &mut String, key: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    let sep = if query.is_empty() { "" } else { "&" };
    let (key, value) = (
        utf8_percent_encode(key, ESCAPED),
        utf8_percent_encode(value, ESCAPED),
    );
    let _ = write!(query, "{sep}{key}={value}");
}

/// The page's own address on the web, else the deployed web app.
fn base() -> String {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let location = window.location();
        if let (Ok(origin), Ok(path)) = (location.origin(), location.pathname()) {
            return origin + &path;
        }
    }
    WEB_APP.to_string()
}

/// `v` with at most `places` decimals, without trailing zeros (`-0` as `0`).
fn round(v: f32, places: i32) -> f32 {
    let scale = 10f32.powi(places);
    let r = (v * scale).round() / scale;
    if r == 0.0 { 0.0 } else { r }
}
