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

/// `cargo test --lib whisper_cpu_speed -- --ignored --nocapture`:
/// tiny (already downloaded by `whisper_transcribes`) on the CPU, full
/// precision against quantized, over the sample.
#[test]
#[ignore = "needs Whisper tiny in the data folder"]
fn whisper_cpu_speed() {
    use crate::voice::stt::WindowModel;
    let dir = eframe::storage_dir("Octant")
        .expect("data folder")
        .join("models/whisper/tiny");
    let read = |f: &str| std::fs::read(dir.join(f)).expect("model file");
    let audio = jfk();
    // The whole sample, and its first 3.5 s ("And so my fellow Americans").
    let clips = [&audio[..], &audio[..56_000]];
    for quantize in [false, true] {
        let mut w = super::Whisper::build(
            &read("config.json"),
            &read("tokenizer.json"),
            read(super::models::WEIGHTS),
            candle_core::Device::Cpu,
            quantize,
        )
        .expect("model");
        for clip in clips {
            let started = Instant::now();
            let text = w.transcribe(clip).expect("text");
            let secs = clip.len() as f32 / 16_000.0;
            println!(
                "quantized {quantize}: {secs:.1} s of speech in {:?}: {text}",
                started.elapsed()
            );
        }
    }
}

/// `cargo test --lib whisper_prompt -- --ignored --nocapture`: with tiny in
/// the data folder, prompts round-trip through the real vocabulary and a
/// prompted transcription still hears the sample.
#[test]
#[ignore = "needs Whisper tiny in the data folder"]
fn whisper_prompt_encodes_and_transcribes() {
    use crate::voice::stt::WindowModel;
    let dir = eframe::storage_dir("Octant")
        .expect("data folder")
        .join("models/whisper/tiny");
    let read = |f: &str| std::fs::read(dir.join(f)).expect("model file");
    let tokens = super::tokens::Tokens::parse(&read("tokenizer.json")).expect("vocabulary");
    for text in [
        " t2m, sst, valid_time, lat, lon",
        " 气温, Breite, широта",
        " ask not",
    ] {
        let ids = tokens.encode(text);
        assert_eq!(tokens.decode(&ids), text.trim(), "{ids:?}");
    }
    // GPT-2's own pieces: " the" is one token.
    assert_eq!(tokens.encode(" the").len(), 1);
    let mut w = super::Whisper::from_bytes(
        &read("config.json"),
        &read("tokenizer.json"),
        read(super::models::WEIGHTS),
    )
    .expect("model");
    w.set_prompt("t2m, sst, country, Americans");
    let text = w.transcribe(&jfk()).expect("text").to_lowercase();
    println!("prompted: {text}");
    assert!(text.contains("ask not what your country"), "{text}");
}
