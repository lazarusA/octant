use crate::voice::stt::{Prep, SpeechToText, SttEvent, WhisperSize, Windowed};
use std::time::{Duration, Instant};

/// whisper.cpp's sample: 11 s of John F. Kennedy at 16 kHz, 16-bit mono.
const JFK: &str = "https://raw.githubusercontent.com/ggml-org/whisper.cpp/v1.7.6/samples/jfk.wav";

fn jfk() -> Vec<f32> {
    let wav = reqwest::blocking::get(JFK)
        .and_then(|r| r.bytes())
        .expect("sample");
    let data = wav
        .windows(4)
        .position(|w| w == b"data")
        .expect("data chunk")
        + 8;
    wav[data..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&b| f32::from(i16::from_le_bytes(b)) / 32768.0)
        .collect()
}

/// `cargo test --lib whisper_transcribes -- --ignored --nocapture`: downloads
/// Whisper tiny (151 MB, kept for the app) and transcribes the sample.
#[test]
#[ignore = "downloads Whisper tiny and a sample"]
fn whisper_transcribes_speech() {
    let mut stt = Windowed::spawn(|prep: &Prep| super::prepare(WhisperSize::Tiny, prep));
    let deadline = Instant::now() + Duration::from_secs(600);
    let mut text = String::new();
    let mut sent: Option<Instant> = None;
    while Instant::now() < deadline && !text.contains("ask not what your country") {
        match stt.poll() {
            Some(SttEvent::Ready) if sent.is_none() => {
                stt.push(&jfk());
                stt.finish();
                sent = Some(Instant::now());
            }
            Some(SttEvent::Final(t)) => {
                let took = sent.map(|s| s.elapsed());
                println!("final after {took:?}: {t}");
                text.push_str(&t.to_lowercase());
                text.push(' ');
            }
            Some(SttEvent::Failed(e)) => panic!("{e}"),
            Some(_) => {}
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    }
    assert!(text.contains("ask not what your country"), "{text}");
}
