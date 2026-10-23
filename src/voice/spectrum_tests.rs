use super::*;

#[test]
fn fft_matches_a_direct_transform() {
    let n = 16;
    let signal: Vec<f32> = (0..n).map(|i| ((i * 7 % 5) as f32) - 2.0).collect();
    let (mut re, mut im) = (signal.clone(), vec![0.0; n]);
    let twiddles = twiddles(re.len());
    fft(&mut re, &mut im, &twiddles);
    for k in 0..n {
        let (mut dr, mut di) = (0.0, 0.0);
        for (t, &x) in signal.iter().enumerate() {
            let a = -2.0 * PI * (k * t) as f32 / n as f32;
            dr += x * a.cos();
            di += x * a.sin();
        }
        assert!(
            (re[k] - dr).abs() < 1e-3 && (im[k] - di).abs() < 1e-3,
            "bin {k}"
        );
    }
}

/// The brightest band of a sine's columns.
fn loudest_band(rate: u32, freq: f32) -> usize {
    let mut spectrum = Spectrum::new(rate);
    let samples: Vec<f32> = (0..rate / 4)
        .map(|i| 0.5 * (2.0 * PI * freq * i as f32 / rate as f32).sin())
        .collect();
    let mut last = [0.0; BANDS];
    let mut columns = 0;
    spectrum.push(&samples, |c| {
        last = *c;
        columns += 1;
    });
    assert!(columns > 10, "a quarter second gives columns: {columns}");
    last.iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map_or(0, |(i, _)| i)
}

#[test]
fn a_higher_tone_lights_a_higher_band() {
    for rate in [16_000, 44_100, 48_000] {
        let (low, mid, high) = (
            loudest_band(rate, 200.0),
            loudest_band(rate, 1000.0),
            loudest_band(rate, 5000.0),
        );
        assert!(low < mid && mid < high, "{rate} Hz: {low} {mid} {high}");
    }
}

#[test]
fn silence_reads_zero_and_samples_carry_over() {
    let mut spectrum = Spectrum::new(48_000);
    let mut columns = 0;
    // 480 samples is one hop: no column until a whole window has arrived.
    for _ in 0..10 {
        spectrum.push(&[0.0; 480], |c| {
            assert!(c.iter().all(|&v| v == 0.0));
            columns += 1;
        });
    }
    assert_eq!(columns, (4800 - FFT) / 480 + 1);
}
