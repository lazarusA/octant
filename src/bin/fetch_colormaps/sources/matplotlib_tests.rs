//! Tests for the Matplotlib colormap evaluator.

use super::*;

#[test]
fn segment_lut_matches_matplotlib_gray_and_steps() {
    let Ok(gray) = segment_lut(&[[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]) else {
        panic!("gray")
    };
    assert_eq!(gray[0], 0.0);
    assert!((gray[128] - 128.0 / 255.0).abs() < 1e-12);
    assert_eq!(gray[255], 1.0);
    // A discontinuity at 0.5 jumps from y0 (left side) to y1 (right side).
    let Ok(step) = segment_lut(&[[0.0, 0.0, 0.0], [0.5, 0.2, 0.8], [1.0, 1.0, 1.0]]) else {
        panic!("step")
    };
    assert!(step[127] < 0.2 && step[128] > 0.8);
}

#[test]
fn segment_lut_rejects_invalid_tables() {
    assert!(segment_lut(&[]).is_err());
    assert!(segment_lut(&[[0.0, 0.5, 0.5]]).is_err(), "one row");
    assert!(
        segment_lut(&[
            [0.0, 0.0, 0.0],
            [0.7, 0.5, 0.5],
            [0.3, 0.2, 0.2],
            [1.0, 1.0, 1.0]
        ])
        .is_err(),
        "unsorted x"
    );
    assert!(
        segment_lut(&[[0.1, 0.0, 0.0], [1.0, 1.0, 1.0]]).is_err(),
        "x starts above 0"
    );
    assert!(
        segment_lut(&[[0.0, 0.0, 0.0], [0.9, 1.0, 1.0]]).is_err(),
        "x ends below 1"
    );
}

#[test]
fn from_list_needs_two_colors() {
    let one = Value::Seq(vec![Value::Seq(vec![Value::Num(1.0); 3])]);
    assert!(spec(&one, "").is_err());
}

#[test]
fn evaluates_datad_entries() {
    let py = "def _g3(x): return x\n\
              def _g23(x): return 3 * x - 2\n\
              def _g28(x): return np.abs((3 * x - 1) / 2)\n\
              gfunc = {i: globals()[f\"_g{i}\"] for i in range(37)}\n\
              _gray_data = {'red': ((0., 0, 0), (1., 1, 1)), 'green': ((0., 0, 0), (1., 1, 1)), 'blue': ((0., 0, 0), (1., 1, 1))}\n\
              _ocean_data = {'red': gfunc[23], 'green': gfunc[28], 'blue': gfunc[3]}\n\
              _t_data = ((1.0, 0.0, 0.0), (0.0, 0.0, 1.0))\n\
              datad = {'gray': _gray_data, 'ocean': _ocean_data, 't': {'listed': _t_data}}\n";
    let Ok(datad) = Datad::parse(py) else {
        panic!("datad")
    };
    let Ok(gray) = datad.map("gray") else {
        panic!("gray")
    };
    assert_eq!(gray[255], [1.0, 1.0, 1.0]);
    let Ok(ocean) = datad.map("ocean") else {
        panic!("ocean")
    };
    assert_eq!(ocean[0], [0.0, 0.5, 0.0]);
    let Ok(t) = datad.map("t") else {
        panic!("listed")
    };
    assert_eq!(t.len(), 2);
}

#[test]
fn ported_functions_must_match_upstream() {
    // `_g3` redefined upstream: the port no longer applies.
    let py = "def _g3(x): return x ** 2\n\
              gfunc = {i: globals()[f\"_g{i}\"] for i in range(37)}\n\
              _d = {'red': gfunc[3], 'green': gfunc[3], 'blue': gfunc[3]}\n\
              datad = {'d': _d}\n";
    let Ok(datad) = Datad::parse(py) else {
        panic!("datad")
    };
    assert!(datad.map("d").is_err());
}

#[test]
fn cubehelix_with_arguments_is_rejected() {
    let call = |args| Value::Call {
        name: "cubehelix".into(),
        args,
        kwargs: Vec::new(),
    };
    assert!(spec(&call(vec![Value::Num(0.5)]), "").is_err());
    let source = CUBEHELIX_SOURCE.join("\n");
    assert_eq!(spec(&call(Vec::new()), &source).map(|s| s.len()), Ok(N));
    assert!(
        spec(&call(Vec::new()), "").is_err(),
        "helper changed upstream"
    );
}
