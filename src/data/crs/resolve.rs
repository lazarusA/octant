//! `CrsRef` to a proj string `proj4rs` reads: WKT without an EPSG code
//! through `proj4wkt` (desktop only), EPSG
//! codes through `crs-definitions` on the desktop and spatialreference.org
//! on the web (fetched once per code, the frame woken when it arrives).

use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};

use super::CrsRef;

/// A CRS `proj4rs` understands.
#[derive(Debug, Clone, PartialEq)]
pub struct Crs {
    /// Longitude/latitude in degrees (`+proj=longlat`).
    pub geographic: bool,
    /// The unit of its x/y (`m`, `us-ft`, ...; `degrees` when geographic).
    pub unit: &'static str,
}

/// What a `CrsRef` resolves to.
#[derive(Debug, Clone, PartialEq)]
pub enum Resolution {
    Ready(Arc<Crs>),
    /// On the web: its EPSG definition is on its way.
    Pending,
    Unsupported(Arc<str>),
}

static CACHE: Mutex<Option<HashMap<CrsRef, Resolution>>> = Mutex::new(None);
#[cfg(target_arch = "wasm32")]
static WAKE: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

/// What wakes the UI when a fetched definition arrives (a repaint).
#[cfg(target_arch = "wasm32")]
pub fn set_wake(wake: impl Fn() + Send + Sync + 'static) {
    let _ = WAKE.set(Box::new(wake));
}

/// What `crs` resolves to; computed once per reference.
pub fn resolve(crs: &CrsRef) -> Resolution {
    if let Some(known) = cached(crs) {
        return known;
    }
    let resolution = match crs {
        CrsRef::Geographic => from_proj4("+proj=longlat +datum=WGS84 +no_defs"),
        CrsRef::Wkt(wkt) => from_wkt(wkt),
        CrsRef::Epsg(code) => epsg(*code),
    };
    store(crs.clone(), resolution.clone());
    resolution
}

fn cached(crs: &CrsRef) -> Option<Resolution> {
    let cache = CACHE.lock().unwrap_or_else(|p| p.into_inner());
    cache.as_ref()?.get(crs).cloned()
}

fn store(crs: CrsRef, resolution: Resolution) {
    // Once per CRS: the cache answers every later ask.
    if let Resolution::Unsupported(reason) = &resolution {
        log::warn!("CRS {crs:?} not supported: {reason}");
    }
    let mut cache = CACHE.lock().unwrap_or_else(|p| p.into_inner());
    cache.get_or_insert_default().insert(crs, resolution);
}

/// `proj4` read by `proj4rs`.
fn from_proj4(proj4: &str) -> Resolution {
    match proj4rs::proj::Proj::from_proj_string(proj4) {
        Ok(proj) => Resolution::Ready(Arc::new(Crs {
            geographic: proj.is_latlong(),
            unit: if proj.is_latlong() {
                "degrees"
            } else {
                proj.units()
            },
        })),
        Err(e) => Resolution::Unsupported(e.to_string().into()),
    }
}

/// A WKT without an EPSG code (`discover` reads those as `Epsg`).
#[cfg(not(target_arch = "wasm32"))]
fn from_wkt(wkt: &str) -> Resolution {
    match proj4wkt::wkt_to_projstring(wkt) {
        Ok(proj4) => from_proj4(&proj4),
        Err(e) => Resolution::Unsupported(e.to_string().into()),
    }
}

/// shortcut: `proj4wkt` exports a wasm `main` of its own, so the web reads
/// only WKT carrying an EPSG code; add it once it no longer does.
#[cfg(target_arch = "wasm32")]
fn from_wkt(_wkt: &str) -> Resolution {
    Resolution::Unsupported("WKT without an EPSG code".into())
}

fn unknown(code: u32) -> Resolution {
    Resolution::Unsupported(format!("EPSG:{code} not found").into())
}

#[cfg(not(target_arch = "wasm32"))]
fn epsg(code: u32) -> Resolution {
    let def = u16::try_from(code)
        .ok()
        .and_then(crs_definitions::from_code);
    def.map_or_else(|| unknown(code), |d| from_proj4(d.proj4))
}

/// Pending until spatialreference.org answers; the answer replaces it.
#[cfg(target_arch = "wasm32")]
fn epsg(code: u32) -> Resolution {
    wasm_bindgen_futures::spawn_local(async move {
        let url = format!("https://spatialreference.org/ref/epsg/{code}/proj4.txt");
        let answer = crate::data::backends::http::fetch::fetch_url_bytes(&url).await;
        let resolution = match answer.map(|b| String::from_utf8_lossy(&b).trim().to_string()) {
            Ok(proj4) if proj4.starts_with('+') => from_proj4(&proj4),
            Ok(_) | Err(_) => unknown(code),
        };
        store(CrsRef::Epsg(code), resolution);
        if let Some(wake) = WAKE.get() {
            wake();
        }
    });
    Resolution::Pending
}
