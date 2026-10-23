//! What the app and the browser's voice worker say to each other, as text
//! (audio goes as a transferred `Float32Array`): commands to the worker and
//! [`SttEvent`]s back, each a one-letter kind and its payload.

use super::{SttEvent, WhisperSize};

/// The worker loads `size` and starts transcribing.
pub fn load(size: WhisperSize) -> String {
    format!("load:{}", size.name())
}

/// The speaker stopped.
pub const FINISH: &str = "finish";

/// The worker is listening: its WASM starts after its script, so messages
/// sent before this one would find no handler.
pub const HELLO: &str = "hello";

/// The model a `load` command names.
pub fn loaded(command: &str) -> Option<WhisperSize> {
    let name = command.strip_prefix("load:")?;
    WhisperSize::ALL.into_iter().find(|s| s.name() == name)
}

pub fn encode(event: &SttEvent) -> String {
    match event {
        SttEvent::Partial(text) => format!("p{text}"),
        SttEvent::Final(text) => format!("f{text}"),
        SttEvent::Failed(e) => format!("e{e}"),
        SttEvent::Ready => "r".to_string(),
        SttEvent::Preparing(Some(done)) => format!("d{done}"),
        SttEvent::Preparing(None) => "l".to_string(),
    }
}

pub fn decode(message: &str) -> Option<SttEvent> {
    let mut chars = message.chars();
    let kind = chars.next()?;
    let rest = chars.as_str();
    Some(match kind {
        'p' => SttEvent::Partial(rest.to_string()),
        'f' => SttEvent::Final(rest.to_string()),
        'e' => SttEvent::Failed(rest.to_string()),
        'r' => SttEvent::Ready,
        'd' => SttEvent::Preparing(Some(rest.parse().ok()?)),
        'l' => SttEvent::Preparing(None),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_survive_the_trip() {
        for event in [
            SttEvent::Partial("plot the 气温".into()),
            SttEvent::Final(String::new()),
            SttEvent::Failed("no network".into()),
            SttEvent::Ready,
            SttEvent::Preparing(Some(0.25)),
            SttEvent::Preparing(None),
        ] {
            assert_eq!(decode(&encode(&event)), Some(event));
        }
        assert_eq!(decode(""), None);
        assert_eq!(decode("x"), None);
    }

    #[test]
    fn load_names_the_model() {
        for size in WhisperSize::ALL {
            assert_eq!(loaded(&load(size)), Some(size));
        }
        assert_eq!(loaded("load:huge"), None);
        assert_eq!(loaded(FINISH), None);
    }
}
