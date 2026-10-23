use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, StreamConfig};
use std::sync::{Arc, Mutex};

pub struct Capture {
    _stream: cpal::Stream,
    samples: Arc<Mutex<Vec<f32>>>,
    rate: u32,
}

impl Capture {
    /// Starts recording the default input device; the error names why not.
    pub fn start() -> Result<Self, String> {
        let device = cpal::default_host()
            .default_input_device()
            .ok_or_else(|| "no input device".to_string())?;
        let supported = device.default_input_config().map_err(|e| e.to_string())?;
        let config = supported.config();
        let samples = Arc::new(Mutex::new(Vec::new()));
        let stream = match supported.sample_format() {
            SampleFormat::F32 => build::<f32>(&device, config, &samples),
            SampleFormat::I16 => build::<i16>(&device, config, &samples),
            SampleFormat::I32 => build::<i32>(&device, config, &samples),
            SampleFormat::U16 => build::<u16>(&device, config, &samples),
            other => return Err(format!("unsupported sample format {other}")),
        }
        .map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?;
        Ok(Self {
            _stream: stream,
            samples,
            rate: config.sample_rate,
        })
    }

    /// Samples per second; 0 until known (the web's microphone).
    pub fn sample_rate(&self) -> u32 {
        self.rate
    }

    /// The microphone will never open; the desktop knows at `start`.
    pub fn failed(&self) -> bool {
        false
    }

    /// Moves the samples recorded since the last call into `out`.
    pub fn drain(&self, out: &mut Vec<f32>) {
        if let Ok(mut pending) = self.samples.lock() {
            out.append(&mut pending);
        }
    }
}

/// An input stream mixing each frame of `T` samples down to one `f32`.
fn build<T>(
    device: &cpal::Device,
    config: StreamConfig,
    samples: &Arc<Mutex<Vec<f32>>>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channels = usize::from(config.channels.max(1));
    let samples = Arc::clone(samples);
    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            if let Ok(mut pending) = samples.lock() {
                let mono = data.chunks(channels).map(|frame| {
                    frame.iter().map(|&s| s.to_sample::<f32>()).sum::<f32>() / channels as f32
                });
                super::keep_recent(&mut pending, mono);
            }
        },
        |err| log::warn!("microphone stream: {err}"),
        None,
    )
}
