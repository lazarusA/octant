//! A message as the browser carries it: an array of the JSON text and its
//! binary parts, whose buffers are transferred, not copied.

use std::sync::Arc;

use js_sys::{Array, Float32Array, Float64Array, Uint8Array};
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::{JsCast, JsValue};

/// Something messages are posted to: a worker, or a worker's own scope.
pub trait Post {
    fn post(&self, message: &JsValue, transfer: &JsValue) -> Result<(), JsValue>;
}

impl Post for web_sys::Worker {
    fn post(&self, message: &JsValue, transfer: &JsValue) -> Result<(), JsValue> {
        self.post_message_with_transfer(message, transfer)
    }
}

impl Post for web_sys::DedicatedWorkerGlobalScope {
    fn post(&self, message: &JsValue, transfer: &JsValue) -> Result<(), JsValue> {
        self.post_message_with_transfer(message, transfer)
    }
}

/// A binary part of a message.
pub enum Part<'a> {
    F32(&'a [f32]),
    F64(&'a [f64]),
    Bytes(&'a [u8]),
}

impl Part<'_> {
    /// A copy outside wasm memory, whose buffer the message then transfers.
    fn array(&self) -> (JsValue, JsValue) {
        let (array, buffer): (JsValue, _) = match self {
            Self::F32(v) => {
                let a = Float32Array::from(*v);
                let b = a.buffer();
                (a.into(), b)
            }
            Self::F64(v) => {
                let a = Float64Array::from(*v);
                let b = a.buffer();
                (a.into(), b)
            }
            Self::Bytes(v) => {
                let a = Uint8Array::from(*v);
                let b = a.buffer();
                (a.into(), b)
            }
        };
        (array, buffer.into())
    }
}

/// Posts `message` with `parts` to `target`; whether it was posted (a
/// failure is logged).
pub fn post<M: Serialize>(target: &impl Post, message: &M, parts: &[Part<'_>]) -> bool {
    let text = match serde_json::to_string(message) {
        Ok(text) => text,
        Err(e) => {
            log::error!("[data worker] Unencodable message: {e}");
            return false;
        }
    };
    let framed = Array::of1(&JsValue::from_str(&text));
    let transfer = Array::new();
    for part in parts {
        let (array, buffer) = part.array();
        framed.push(&array);
        transfer.push(&buffer);
    }
    let posted = target.post(&framed, &transfer);
    if let Err(e) = &posted {
        log::error!("[data worker] Failed to post a message: {e:?}");
    }
    posted.is_ok()
}

/// The message in `data` and its parts, or `None` for anything else.
pub fn receive<M: DeserializeOwned>(data: &JsValue) -> Option<(M, Array)> {
    let framed = data.dyn_ref::<Array>()?;
    let text = framed.get(0).as_string()?;
    let message = serde_json::from_str(&text)
        .inspect_err(|e| log::error!("[data worker] Undecodable message: {e}"))
        .ok()?;
    Some((message, framed.slice(1, framed.length())))
}

/// Part `i` of `parts` as `f32`s, copied once, straight into the shared
/// slice blocks hold.
pub fn f32s(parts: &Array, i: u32) -> Option<Arc<[f32]>> {
    let array = parts.get(i).dyn_into::<Float32Array>().ok()?;
    let len = array.length() as usize;
    let mut values: Arc<[f32]> = std::iter::repeat_n(0.0, len).collect();
    array.copy_to(Arc::get_mut(&mut values)?);
    Some(values)
}

/// Part `i` of `parts` as `f64`s.
pub fn f64s(parts: &Array, i: u32) -> Option<Vec<f64>> {
    Some(parts.get(i).dyn_into::<Float64Array>().ok()?.to_vec())
}

/// Part `i` of `parts` as bytes.
pub fn bytes(parts: &Array, i: u32) -> Option<Vec<u8>> {
    Some(parts.get(i).dyn_into::<Uint8Array>().ok()?.to_vec())
}
