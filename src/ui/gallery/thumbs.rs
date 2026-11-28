//! Entry thumbnails: fetched from the docs site the first time their card
//! shows, decoded off the UI thread, kept as textures for the session. Each
//! is tried once: offline, cards keep their placeholder; the gallery's
//! Reload forgets the failures.

use crate::gallery::GalleryEntry;
use egui::{ColorImage, Context, Id, TextureHandle, TextureOptions};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Wide enough for a card of three columns; a quarter of an 800 px picture.
const WIDTH: u32 = 400;

enum Thumb {
    Loading,
    Decoded(ColorImage),
    Ready(TextureHandle),
    Failed,
}

/// The thumbnails of one egui context, by entry key.
type Thumbs = Arc<Mutex<HashMap<String, Thumb>>>;

fn thumbs(ctx: &Context) -> Thumbs {
    let id = Id::unique("gallery_thumbs");
    ctx.data_mut(|d| d.get_temp_mut_or_default::<Thumbs>(id).clone())
}

/// The texture of `entry`'s thumbnail once it arrived; the first call starts
/// its fetch.
pub fn texture(ctx: &Context, entry: &GalleryEntry) -> Option<TextureHandle> {
    // Tests draw cards without the network.
    if cfg!(test) {
        return None;
    }
    let path = entry.thumbnail.as_deref()?;
    let thumbs = thumbs(ctx);
    let mut map = thumbs.lock().unwrap_or_else(|p| p.into_inner());
    match map.get_mut(entry.key.as_str()) {
        None => {
            map.insert(entry.key.clone(), Thumb::Loading);
            drop(map);
            let url = format!("{}{path}", crate::gallery::remote::SITE);
            fetch(ctx, &thumbs, entry.key.clone(), url);
            None
        }
        Some(slot @ Thumb::Decoded(_)) => {
            let Thumb::Decoded(image) = std::mem::replace(slot, Thumb::Loading) else {
                return None;
            };
            let name = format!("gallery-{}", entry.key);
            let texture = ctx.load_texture(name, image, TextureOptions::LINEAR);
            *slot = Thumb::Ready(texture.clone());
            Some(texture)
        }
        Some(Thumb::Ready(texture)) => Some(texture.clone()),
        Some(Thumb::Loading | Thumb::Failed) => None,
    }
}

/// Forgets the thumbnails that failed, so their cards fetch them again.
pub fn forget_failed(ctx: &Context) {
    let thumbs = thumbs(ctx);
    let mut map = thumbs.lock().unwrap_or_else(|p| p.into_inner());
    map.retain(|_, t| !matches!(t, Thumb::Failed));
}

fn fetch(ctx: &Context, thumbs: &Thumbs, key: String, url: String) {
    let (ctx, thumbs) = (ctx.clone(), Arc::clone(thumbs));
    let done = move |url: &str, bytes: Result<Vec<u8>, String>| {
        let thumb = match bytes.and_then(|b| decode(&b)) {
            Ok(image) => Thumb::Decoded(image),
            Err(e) => {
                log::debug!("thumbnail {url}: {e}");
                Thumb::Failed
            }
        };
        let mut map = thumbs.lock().unwrap_or_else(|p| p.into_inner());
        map.insert(key, thumb);
        ctx.request_repaint();
    };
    #[cfg(not(target_arch = "wasm32"))]
    crate::utils::executor::TaskExecutor::spawn_background(move || {
        let rt = crate::utils::executor::get_shared_tokio_rt();
        let bytes = rt.block_on(crate::data::backends::http::fetch_url_bytes(&url));
        done(&url, bytes);
    });
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let bytes = crate::data::backends::http::fetch_url_bytes(&url).await;
        done(&url, bytes);
    });
}

/// The picture in `bytes`, at most [`WIDTH`] wide.
fn decode(bytes: &[u8]) -> Result<ColorImage, String> {
    let image = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let image = if image.width() > WIDTH {
        image.thumbnail(WIDTH, u32::MAX)
    } else {
        image
    };
    let rgba = image.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Ok(ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()))
}

#[cfg(test)]
mod tests {
    /// A WebP picture decodes to at most [`super::WIDTH`] wide, its shape kept.
    #[test]
    fn a_wide_picture_shrinks_to_the_card() {
        let mut bytes = Vec::new();
        let picture = image::RgbaImage::from_pixel(800, 450, image::Rgba([10, 20, 30, 255]));
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut bytes);
        let encoded = picture.write_with_encoder(encoder);
        assert!(encoded.is_ok(), "{encoded:?}");
        let decoded = super::decode(&bytes).map(|i| i.size);
        assert_eq!(decoded, Ok([400, 225]));
        assert!(super::decode(b"not a picture").is_err());
    }
}
