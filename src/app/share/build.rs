//! The share link of what is plotted: the base layer's store, variable,
//! roles, dimension values, plot type and colors, and its overlays.

use std::fmt::Write;

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

use super::{KEYS, SCHEMES};
use crate::app::actions::{plot_type_id, value_text};
use crate::app::multiscale::at_base;
use crate::app::{AnimationRole, OctantApp, SpatialRole, VariableSelection};

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
const WEB_APP: &str = "https://lazarusa.github.io/octant/";

impl OctantApp {
    /// Whether the plot has a share link: something plotted from a store a
    /// link can open (never a local one). Allocates nothing, for the UI.
    pub fn can_share(&self) -> bool {
        let sel = self.plotted();
        sel.metadata.is_some() && SCHEMES.iter().any(|p| sel.store_target.starts_with(p))
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
    /// level for its own screen.
    fn share_query(&self) -> Option<String> {
        let base = at_base(self.plotted());
        let sel = base.as_ref().unwrap_or(self.plotted());
        let var = sel.variable_info()?;
        let mut q = String::new();
        pair(&mut q, "src", &sel.store_target);
        pair(&mut q, "var", var.name.trim_matches('/'));
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
        if style.lock_bounds {
            pair(
                &mut q,
                "range",
                &format!("{}..{}", style.range_min, style.range_max),
            );
        }
        for overlay in self.layers.overlays() {
            let base = at_base(overlay.selection());
            let sel = base.as_ref().unwrap_or(overlay.selection());
            let name = sel.variable_info().map_or("", |v| v.name.as_str());
            pair(&mut q, "overlay", name.trim_matches('/'));
        }
        Some(q)
    }

    /// Each dimension's `name=value` (the animated one's step, a range
    /// narrower than the dimension, a pinned index) and `anim`.
    fn dimension_pairs(&self, sel: &VariableSelection, q: &mut String) {
        let (Some(meta), Some(var)) = (sel.metadata.as_ref(), sel.variable_info()) else {
            return;
        };
        let store = sel.store_target.as_str();
        for (i, config) in sel.dim_config.iter().enumerate() {
            let Some(name) = var.dimension_names.get(i) else {
                continue;
            };
            let len = var.shape.get(i).copied().unwrap_or(0) as usize;
            let text = |idx, last| value_text(meta, var, store, i, (idx, last));
            let value = if config.animation == AnimationRole::Animated {
                pair(q, "anim", name);
                text(self.playback.current_timestep, false)
            } else if !config.active || config.range.0 == config.range.1 {
                text(config.index, false)
            } else if config.range != (0, len.saturating_sub(1)) {
                format!(
                    "{}..{}",
                    text(config.range.0, false),
                    text(config.range.1, true)
                )
            } else {
                continue;
            };
            let key = if KEYS.contains(&name.as_str()) {
                format!("dim.{name}")
            } else {
                name.clone()
            };
            pair(q, &key, &value);
        }
    }
}

/// Appends `key=value` (none for an empty value), escaped.
fn pair(query: &mut String, key: &str, value: &str) {
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
